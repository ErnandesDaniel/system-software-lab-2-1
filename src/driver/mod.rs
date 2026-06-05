pub mod nasm;
pub mod stub;

use std::fs;
use std::path::Path;

use crate::ast;
use crate::codegen;
use crate::error::CompilerError;
use crate::ir::diagram::CfgMermaidGenerator;
use crate::ir::validator::IrValidator;
use crate::ir_generator::IrGenerator;
use crate::mermaid::MermaidGenerator;
use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;
use crate::CodeGenTarget;

pub struct CompilerDriver;

impl CompilerDriver {
    pub fn compile(args: &crate::cli::Args) -> crate::Result<()> {
        let source = Self::read_source(&args.source_path)?;
        let ast = Self::parse(&source)?;
        Self::create_output_dir(&args.output_dir)?;
        Self::generate_ast_diagrams(&ast, &args.output_dir);
        Self::run_semantic_analysis(&ast)?;
        let ir_program = Self::generate_ir(&ast);
        IrValidator::validate(&ir_program)?;
        Self::generate_cfg_diagrams(&ir_program, &args.output_dir);

        match args.target {
            CodeGenTarget::NASM => Self::generate_nasm(&ir_program, &args.output_dir),
            CodeGenTarget::JVM => Self::generate_jvm(&ir_program, &args.output_dir)?,
        }
        Ok(())
    }

    fn read_source(path: &str) -> crate::Result<String> {
        fs::read_to_string(path).map_err(|e| CompilerError::Io(format!("Failed to read file '{path}': {e}")))
    }

    fn parse(source: &str) -> crate::Result<ast::Program> {
        let mut parser = Parser::new(source);
        parser
            .parse()
            .map_err(|e| CompilerError::Parse(format!("Parse error: {e}")))
    }

    fn create_output_dir(path: &str) -> crate::Result<()> {
        let _ = fs::remove_dir_all(path);
        fs::create_dir_all(path)
            .map_err(|e| CompilerError::Io(format!("Failed to create output directory '{path}': {e}")))
    }

    fn generate_ast_diagrams(ast: &ast::Program, output_dir: &str) {
        for item in &ast.items {
            if let ast::SourceItem::FuncDefinition(func) = item {
                let path = Path::new(output_dir).join(format!("{}-ast.mmd", func.signature.name.name));
                let mut gen = MermaidGenerator::new();
                let diagram = gen.generate_function(func);
                if let Err(e) = fs::write(&path, &diagram) {
                    eprintln!("Failed to write AST diagram: {e}");
                }
            }
        }
    }

    fn run_semantic_analysis(ast: &ast::Program) -> crate::Result<()> {
        let mut analyzer = SemanticsAnalyzer::new();
        analyzer.analyze(ast).map_err(|e| {
            eprintln!("Semantic error: {e}");
            e
        })
    }

    fn generate_ir(ast: &ast::Program) -> crate::ir::IrProgram {
        let mut ir_gen = IrGenerator::new();
        ir_gen.generate(ast)
    }

    fn generate_cfg_diagrams(ir: &crate::ir::IrProgram, output_dir: &str) {
        for func in &ir.functions {
            let path = Path::new(output_dir).join(format!("{}-cfg.mmd", func.name));
            let mut gen = CfgMermaidGenerator::new();
            let diagram = gen.generate_function_only(func);
            if let Err(e) = fs::write(&path, &diagram) {
                eprintln!("Failed to write CFG diagram: {e}");
            }
        }
    }

    fn generate_jvm(ir: &crate::ir::IrProgram, output_dir: &str) -> crate::Result<()> {
        use crate::ir::types::IrType;
        use std::process::Command;

        let mut gen = codegen::JvmGenerator::new();
        let classes = gen.generate_program(ir);

        // Write function class files first (needed for javac compilation)
        let _stub_bytes = classes
            .iter()
            .find(|(name, _)| name == "RuntimeStub")
            .map(|(_, bytes)| bytes.clone());
        for (class_name, class_bytes) in &classes {
            if class_name == "RuntimeStub" {
                continue;
            }
            let path = Path::new(output_dir).join(format!("{class_name}.class"));
            if let Err(e) = fs::write(&path, class_bytes) {
                return Err(CompilerError::Io(format!(
                    "Failed to write class file '{class_name}': {e}"
                )));
            }
        }

        let mut global_info: Vec<(String, String, usize, usize)> = Vec::new();
        let mut scalar_inits = String::new();
        for g in &ir.globals {
            let desc = match &g.ty {
                IrType::String => "[B".to_string(),
                IrType::Array(_, _n) => {
                    if gen.is_global_uses_object_array(&g.name) {
                        "[Ljava/lang/Object;".to_string()
                    } else {
                        gen.get_global_jvm_descriptor(&g.name, &g.ty)
                    }
                }
                _ => gen.get_global_jvm_descriptor(&g.name, &g.ty),
            };
            let outer = if let IrType::Array(_, n) = &g.ty { *n } else { 0 };
            let inner = if gen.is_global_uses_object_array(&g.name) {
                gen.get_global_object_array_inner_size(&g.name)
            } else {
                0
            };
            global_info.push((g.name.clone(), desc, outer, inner));
            if outer == 0 {
                if let Some(ref init_val) = g.initializer {
                    let val_str = match init_val {
                        crate::ir::types::Constant::Int(n) => n.to_string(),
                        crate::ir::types::Constant::Bool(true) => "1".to_string(),
                        crate::ir::types::Constant::Bool(false) => "0".to_string(),
                        _ => continue,
                    };
                    scalar_inits.push_str(&format!("        {} = {};\n", g.name, val_str));
                }
            }
        }
        let has_coroutines = ir.functions.iter().any(|f| f.is_coroutine);

        Self::generate_jvm_stub(output_dir, &global_info, &scalar_inits);

        if has_coroutines {
            // Use internal RuntimeStub.class (has coroutine methods)
            for (class_name, class_bytes) in &classes {
                if class_name == "RuntimeStub" {
                    let path = Path::new(output_dir).join("RuntimeStub.class");
                    let _ = fs::write(&path, class_bytes);
                    break;
                }
            }
        } else {
            let stub_output = Command::new("javac")
                .current_dir(output_dir)
                .arg("RuntimeStub.java")
                .output()
                .map_err(|e| CompilerError::Codegen(format!("Failed to run javac for RuntimeStub.java: {e}")))?;

            if !stub_output.status.success() {
                return Err(CompilerError::Codegen(format!(
                    "javac (RuntimeStub.java) failed:\n{}",
                    String::from_utf8_lossy(&stub_output.stderr)
                )));
            }
        }

        let runner_output = Command::new("javac")
            .current_dir(output_dir)
            .arg("MainRunner.java")
            .output()
            .map_err(|e| CompilerError::Codegen(format!("Failed to run javac for MainRunner.java: {e}")))?;

        if !runner_output.status.success() {
            return Err(CompilerError::Codegen(format!(
                "javac (MainRunner.java) failed:\n{}",
                String::from_utf8_lossy(&runner_output.stderr)
            )));
        }

        for (class_name, class_bytes) in classes {
            if class_name == "RuntimeStub" {
                continue;
            }
            let path = Path::new(output_dir).join(format!("{class_name}.class"));
            if let Err(e) = fs::write(&path, &class_bytes) {
                return Err(CompilerError::Io(format!(
                    "Failed to write class file '{class_name}': {e}"
                )));
            }
        }

        Ok(())
    }
}

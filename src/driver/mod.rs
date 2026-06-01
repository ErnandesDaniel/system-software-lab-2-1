pub mod nasm;
pub mod stub;

use std::fs;
use std::path::Path;

use crate::ast;
use crate::codegen;
use crate::ir::diagram::CfgMermaidGenerator;
use crate::ir::validator::IrValidator;
use crate::ir_generator::IrGenerator;
use crate::mermaid::MermaidGenerator;
use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;
use crate::CodeGenTarget;

pub struct CompilerDriver;

impl CompilerDriver {
    pub fn compile(args: &crate::cli::Args) {
        let source = Self::read_source(&args.source_path);
        let ast = Self::parse(&source);
        Self::create_output_dir(&args.output_dir);
        Self::generate_ast_diagrams(&ast, &args.output_dir);
        Self::run_semantic_analysis(&ast);
        let ir_program = Self::generate_ir(&ast);
        if let Err(errors) = IrValidator::validate(&ir_program) {
            for err in errors {
                eprintln!("IR validation error: {err}");
            }
        }
        Self::generate_cfg_diagrams(&ir_program, &args.output_dir);

        match args.target {
            CodeGenTarget::NASM => Self::generate_nasm(&ir_program, &args.output_dir),
            CodeGenTarget::JVM => Self::generate_jvm(&ir_program, &args.output_dir),
        }
    }

    fn read_source(path: &str) -> String {
        match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read file: {e}");
                std::process::exit(1);
            }
        }
    }

    fn parse(source: &str) -> ast::Program {
        let mut parser = Parser::new(source);
        match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Parse error: {e}");
                std::process::exit(1);
            }
        }
    }

    fn create_output_dir(path: &str) {
        let _ = fs::remove_dir_all(path);
        if let Err(e) = fs::create_dir_all(path) {
            eprintln!("Failed to create output directory: {e}");
            std::process::exit(1);
        }
    }

    fn generate_ast_diagrams(ast: &ast::Program, output_dir: &str) {
        for item in &ast.items {
            if let ast::SourceItem::FuncDefinition(func) = item {
                let path = Path::new(output_dir).join(format!("{}-ast.mmd", func.signature.name.name));
                let mut gen = MermaidGenerator::new();
                let diagram = gen.generate_function(func);

                if let Err(e) = fs::write(&path, &diagram) {
                    eprintln!("Failed to write AST: {e}");
                }
            }
        }
    }

    fn run_semantic_analysis(ast: &ast::Program) {
        let mut analyzer = SemanticsAnalyzer::new();
        if let Err(errors) = analyzer.analyze(ast) {
            for err in errors {
                eprintln!("Semantic error: {err}");
            }
            std::process::exit(1);
        }
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
                eprintln!("Failed to write CFG: {e}");
            }
        }
    }

    fn generate_jvm(ir: &crate::ir::IrProgram, output_dir: &str) {
        use std::process::Command;

        let mut gen = codegen::JvmGenerator::new();
        let classes = gen.generate_program(ir);

        for (class_name, class_bytes) in classes {
            let path = Path::new(output_dir).join(format!("{class_name}.class"));
            if let Err(e) = fs::write(&path, &class_bytes) {
                eprintln!("Failed to write class file: {e}");
            }
        }

        use crate::ir::types::IrType;
        let mut global_info: Vec<(String, String, usize, usize)> = Vec::new();
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
        }
        Self::generate_jvm_stub(output_dir, &global_info);

        let stub_file = "RuntimeStub.java";
        let stub_output = Command::new("javac")
            .current_dir(output_dir)
            .arg(stub_file)
            .output()
            .expect("Failed to run javac");

        if !stub_output.status.success() {
            eprintln!("javac (RuntimeStub.java) failed:");
            eprintln!("{}", String::from_utf8_lossy(&stub_output.stderr));
            std::process::exit(1);
        }

        let runner_file = "MainRunner.java";
        let runner_output = Command::new("javac")
            .current_dir(output_dir)
            .arg(runner_file)
            .output()
            .expect("Failed to run javac");

        if !runner_output.status.success() {
            eprintln!("javac (MainRunner.java) failed:");
            eprintln!("{}", String::from_utf8_lossy(&runner_output.stderr));
            std::process::exit(1);
        }
    }
}

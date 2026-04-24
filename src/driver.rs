use std::fs;
use std::path::Path;

use crate::ast;
use crate::codegen::{self, LlvmGenerator};
use crate::CodeGenTarget;
use crate::ir::cfg::CfgMermaidGenerator;
use crate::ir_generator::IrGenerator;
use crate::mermaid::MermaidGenerator;
use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;

pub struct CompilerDriver;

impl CompilerDriver {
    pub fn new() -> Self {
        Self
    }

    pub fn compile(&self, args: &crate::cli::Args) {
        let source = self.read_source(&args.source_path);
        
        // Parse
        let ast = self.parse(&source);
        
        // Create output directory
        self.create_output_dir(&args.output_dir);
        
        // Generate AST diagrams
        self.generate_ast_diagrams(&ast, &args.output_dir);
        
        // Semantic analysis
        self.run_semantic_analysis(&ast);
        
        // Generate IR
        let ir_program = self.generate_ir(&ast);
        
        // Generate CFG diagrams
        self.generate_cfg_diagrams(&ir_program, &args.output_dir);
        
        // Generate code
        match args.target {
            CodeGenTarget::NASM => self.generate_nasm(&ir_program, &args.output_dir),
            CodeGenTarget::LLVM => self.generate_llvm(&ir_program, &args.output_dir),
            CodeGenTarget::WASM => self.generate_wasm(&ir_program, &args.output_dir),
        }
    }

    fn read_source(&self, path: &str) -> String {
        match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Failed to read file: {}", e);
                std::process::exit(1);
            }
        }
    }

    fn parse(&self, source: &str) -> ast::Program {
        let mut parser = Parser::new(source);
        match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                eprintln!("Parse error: {}", e);
                std::process::exit(1);
            }
        }
    }

    fn create_output_dir(&self, path: &str) {
        if let Err(e) = fs::create_dir_all(path) {
            eprintln!("Failed to create output directory: {}", e);
            std::process::exit(1);
        }
    }

    fn generate_ast_diagrams(&self, ast: &ast::Program, output_dir: &str) {
        for item in &ast.items {
            if let ast::SourceItem::FuncDefinition(func) = item {
                let path = Path::new(output_dir)
                    .join(format!("{}-ast.mmd", func.signature.name.name));
                let mut gen = MermaidGenerator::new();
                let diagram = gen.generate_function(func);
                
                if let Err(e) = fs::write(&path, &diagram) {
                    eprintln!("Failed to write AST: {}", e);
                } else {
                    println!("Function AST written to: {}", path.display());
                }
            }
        }
    }

    fn run_semantic_analysis(&self, ast: &ast::Program) {
        let mut analyzer = SemanticsAnalyzer::new();
        if let Err(errors) = analyzer.analyze(ast) {
            for err in errors {
                eprintln!("Semantic error: {}", err);
            }
            std::process::exit(1);
        }
    }

    fn generate_ir(&self, ast: &ast::Program) -> crate::ir::IrProgram {
        let mut ir_gen = IrGenerator::new();
        ir_gen.generate(ast)
    }

    fn generate_cfg_diagrams(&self, ir: &crate::ir::IrProgram, output_dir: &str) {
        for func in &ir.functions {
            let path = Path::new(output_dir).join(format!("{}-cfg.mmd", func.name));
            let mut gen = CfgMermaidGenerator::new();
            let diagram = gen.generate_function_only(func);
            
            if let Err(e) = fs::write(&path, &diagram) {
                eprintln!("Failed to write CFG: {}", e);
            } else {
                println!("Function CFG written to: {}", path.display());
            }
        }
    }

    fn generate_nasm(&self, ir: &crate::ir::IrProgram, output_dir: &str) {
        use std::process::Command;

        // Generate assembly files
        for func in &ir.functions {
            let mut gen = codegen::AsmGenerator::new();
            let asm = gen.generate_single_function(func);
            let path = Path::new(output_dir).join(format!("{}.asm", func.name));
            
            if let Err(e) = fs::write(&path, &asm) {
                eprintln!("Failed to write assembly: {}", e);
            }
        }

        // Assemble
        let mut obj_files = Vec::new();
        for func in &ir.functions {
            let asm_path = Path::new(output_dir).join(format!("{}.asm", func.name));
            let obj_path = Path::new(output_dir).join(format!("{}.obj", func.name));

            let output = Command::new("nasm")
                .args(["-f", "win64", "-o"])
                .arg(obj_path.to_str().unwrap())
                .arg(asm_path.to_str().unwrap())
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        obj_files.push(obj_path);
                    } else {
                        eprintln!("NASM failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to run NASM: {}", e),
            }
        }

        // Link
        if !obj_files.is_empty() {
            let exe_path = Path::new(output_dir).join("program.exe");
            let mut args: Vec<String> = obj_files
                .iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            args.push("-o".to_string());
            args.push(exe_path.to_string_lossy().to_string());

            match Command::new("clang").args(&args).output() {
                Ok(out) => {
                    if out.status.success() {
                        println!("Successfully built: {}", exe_path.display());
                    } else {
                        eprintln!("Link failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to run Clang: {}", e),
            }
        }
    }

    fn generate_llvm(&self, ir: &crate::ir::IrProgram, output_dir: &str) {
        use std::process::Command;

        let mut gen = LlvmGenerator::new();
        let llvm_ir = gen.generate_program(ir);

        // Write LLVM IR
        let ll_path = Path::new(output_dir).join("program.ll");
        if let Err(e) = fs::write(&ll_path, &llvm_ir) {
            eprintln!("Failed to write LLVM IR: {}", e);
            return;
        }
        println!("LLVM IR written to: {}", ll_path.display());

        // Compile
        let obj_path = Path::new(output_dir).join("program.obj");
        let compile_result = Command::new("clang")
            .args(["-c", "-o"])
            .arg(obj_path.to_str().unwrap())
            .arg(ll_path.to_str().unwrap())
            .output();

        match compile_result {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!("Clang failed: {}", String::from_utf8_lossy(&out.stderr));
                    return;
                }
                println!("Object file created: {}", obj_path.display());
            }
            Err(e) => {
                eprintln!("Failed to run Clang: {}", e);
                return;
            }
        }

        // Link
        let exe_path = Path::new(output_dir).join("program.exe");
        match Command::new("clang")
            .arg(obj_path.to_str().unwrap())
            .arg("-o")
            .arg(exe_path.to_str().unwrap())
            .output()
        {
            Ok(out) => {
                if out.status.success() {
                    println!("Successfully built: {}", exe_path.display());
                } else {
                    eprintln!("Link failed: {}", String::from_utf8_lossy(&out.stderr));
                }
            }
            Err(e) => eprintln!("Failed to run Clang: {}", e),
        }
    }

    fn generate_wasm(&self, ir: &crate::ir::IrProgram, output_dir: &str) {
        use std::process::Command;

        let mut gen = LlvmGenerator::new();
        let llvm_ir = gen.generate_program(ir);

        // Write LLVM IR
        let ll_path = Path::new(output_dir).join("program.ll");
        if let Err(e) = fs::write(&ll_path, &llvm_ir) {
            eprintln!("Failed to write LLVM IR: {}", e);
            return;
        }
        println!("LLVM IR written to: {}", ll_path.display());

        // Compile to WebAssembly using clang with wasm target
        let wasm_path = Path::new(output_dir).join("program.wasm");
        
        // Just allow all undefined symbols since stdlib functions will be provided by JS
        let compile_result = Command::new("clang")
            .args([
                "--target=wasm32",
                "-nostdlib",
                "-Wl,--no-entry",
                "-Wl,--export-all",
                "-Wl,--allow-undefined",
                "-o",
                wasm_path.to_str().unwrap(),
                ll_path.to_str().unwrap()
            ])
            .output();

        match compile_result {
            Ok(out) => {
                if !out.status.success() {
                    eprintln!("Clang compile failed: {}", String::from_utf8_lossy(&out.stderr));
                    return;
                }
                println!("WebAssembly module created: {}", wasm_path.display());
            }
            Err(e) => {
                eprintln!("Failed to run Clang: {}", e);
            }
        }
    }
}

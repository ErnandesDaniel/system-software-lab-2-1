mod ast;
mod cfg_mermaid;
mod codegen;
mod ir;
mod ir_generator;
mod lexer;
mod lexer_iter;
mod mermaid;
mod parser;
mod semantics;
mod tests;

use std::env;
use std::fs;
use std::process::Command;

use crate::semantics::analysis::SemanticsAnalyzer;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: {} <source_file> -o <output_dir> [--ast <ast_file>] [--cfg <cfg_file>]",
            args[0]
        );
        eprintln!("Options:");
        eprintln!("  -o, --output <dir>    Output directory (required)");
        eprintln!("  --ast <file>          Save AST to Mermaid file");
        eprintln!("  --cfg <file>         Save CFG to Mermaid file");
        std::process::exit(1);
    }

    let source_path = &args[1];
    let source = match fs::read_to_string(source_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to read file: {}", e);
            std::process::exit(1);
        }
    };

    let mut output_dir: Option<String> = None;
    let mut ast_file: Option<String> = None;
    let mut cfg_file: Option<String> = None;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_dir = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: -o requires an argument");
                    std::process::exit(1);
                }
            }
            "--ast" => {
                if i + 1 < args.len() {
                    ast_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --ast requires an argument");
                    std::process::exit(1);
                }
            }
            "--cfg" => {
                if i + 1 < args.len() {
                    cfg_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: --cfg requires an argument");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    let output_dir = match output_dir {
        Some(d) => d,
        None => {
            eprintln!("Error: -o <output_dir> is required");
            std::process::exit(1);
        }
    };

    // Parse
    let mut parser = parser::Parser::new(&source);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Save AST to file if requested
    if let Some(ref path) = ast_file {
        let mut mermaid_gen = mermaid::MermaidGenerator::new();
        let ast_diagram = mermaid_gen.generate(&ast);
        match fs::write(path, ast_diagram) {
            Ok(_) => println!("AST written to: {}", path),
            Err(e) => eprintln!("Failed to write AST: {}", e),
        }
    }

    // Semantic analysis
    let mut sem_analyzer = SemanticsAnalyzer::new();
    if let Err(errors) = sem_analyzer.analyze(&ast) {
        for err in errors {
            eprintln!("Semantic error: {}", err);
        }
        std::process::exit(1);
    }

    // Generate IR
    let mut ir_gen = ir_generator::IrGenerator::new();
    let ir_program = ir_gen.generate(&ast);

    // Save CFG to file if requested - generate separate CFG for each function
    if let Some(ref path) = cfg_file {
        let cfg_dir = std::path::Path::new(path)
            .parent()
            .unwrap_or(std::path::Path::new("."));
        if let Err(e) = fs::create_dir_all(cfg_dir) {
            eprintln!("Failed to create CFG directory: {}", e);
        }

        let mut cfg_gen = cfg_mermaid::CfgMermaidGenerator::new();

        // Generate combined CFG first
        let cfg_diagram = cfg_gen.generate(&ir_program);
        match fs::write(path, &cfg_diagram) {
            Ok(_) => println!("CFG written to: {}", path),
            Err(e) => eprintln!("Failed to write CFG: {}", e),
        }

        // Also generate separate CFG files for each function
        for func in &ir_program.functions {
            let func_cfg_path = cfg_dir.join(format!("{}.mmd", func.name));
            let mut func_cfg_gen = cfg_mermaid::CfgMermaidGenerator::new();
            let func_cfg_diagram = func_cfg_gen.generate_function_only(func);
            match fs::write(&func_cfg_path, &func_cfg_diagram) {
                Ok(_) => println!("Function CFG written to: {}", func_cfg_path.display()),
                Err(e) => eprintln!("Failed to write function CFG: {}", e),
            }
        }
    }

    // Generate Assembly - generate separate files for each function
    let mut asm_gen = codegen::AsmGenerator::new();
    let _assembly = asm_gen.generate(&ir_program);

    // Create output directory if needed
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Failed to create output directory: {}", e);
    }

    // Generate separate assembly files for each function in output dir
    for func in &ir_program.functions {
        let mut func_asm_gen = codegen::AsmGenerator::new();
        let func_asm = func_asm_gen.generate_single_function(&func);
        let func_asm_path = std::path::Path::new(&output_dir).join(format!("{}.asm", func.name));
        if let Err(e) = fs::write(&func_asm_path, &func_asm) {
            eprintln!("Failed to write function assembly: {}", e);
        }
        println!("Function assembly written to: {}", func_asm_path.display());
    }

    // Assemble and link all .asm files to .exe
    let exe_path = std::path::Path::new(&output_dir).join("program.exe");

    // Assemble each function
    let mut obj_files: Vec<std::path::PathBuf> = Vec::new();
    for func in &ir_program.functions {
        let asm_path = std::path::Path::new(&output_dir).join(format!("{}.asm", func.name));
        let obj_path = std::path::Path::new(&output_dir).join(format!("{}.obj", func.name));

        let output = Command::new("nasm")
            .args([
                "-f",
                "win64",
                "-o",
                obj_path.to_str().unwrap(),
                asm_path.to_str().unwrap(),
            ])
            .output();

        match output {
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("NASM assembly failed: {}", stderr);
                    std::process::exit(1);
                }
            }
            Err(e) => {
                eprintln!("Failed to run NASM: {}", e);
                std::process::exit(1);
            }
        }

        obj_files.push(obj_path);
    }

    // Link all .obj files
    let mut link_args: Vec<String> = Vec::new();
    for obj in &obj_files {
        link_args.push(obj.to_string_lossy().to_string());
    }
    link_args.push("-o".to_string());
    link_args.push(exe_path.to_string_lossy().to_string());

    let output = Command::new("gcc").args(&link_args).output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("GCC linking failed: {}", stderr);
                std::process::exit(1);
            }
        }
        Err(e) => {
            eprintln!("Failed to run GCC: {}", e);
            std::process::exit(1);
        }
    }

    println!("Successfully built: {}", exe_path.display());
}

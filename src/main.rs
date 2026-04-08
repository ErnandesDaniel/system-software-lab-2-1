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

    // Save CFG to file if requested
    if let Some(ref path) = cfg_file {
        let mut cfg_gen = cfg_mermaid::CfgMermaidGenerator::new();
        let cfg_diagram = cfg_gen.generate(&ir_program);
        match fs::write(path, cfg_diagram) {
            Ok(_) => println!("CFG written to: {}", path),
            Err(e) => eprintln!("Failed to write CFG: {}", e),
        }
    }

    // Generate Assembly
    let mut asm_gen = codegen::AsmGenerator::new();
    let assembly = asm_gen.generate(&ir_program);

    // Create output directory if needed
    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        std::process::exit(1);
    }

    // Write assembly file
    let asm_path = std::path::Path::new(&output_dir).join("program.asm");
    if let Err(e) = fs::write(&asm_path, &assembly) {
        eprintln!("Failed to write assembly: {}", e);
        std::process::exit(1);
    }
    println!("Assembly written to: {}", asm_path.display());

    // Assemble and link to .exe
    let exe_path = std::path::Path::new(&output_dir).join("program.exe");
    match assemble_and_link(&asm_path, &exe_path) {
        Ok(_) => println!("Successfully built: {}", exe_path.display()),
        Err(e) => {
            eprintln!("Assembly/linking failed: {}", e);
            std::process::exit(1);
        }
    }
}

fn assemble_and_link(asm_file: &std::path::Path, exe_file: &std::path::Path) -> Result<(), String> {
    let dir = asm_file.parent().unwrap_or(std::path::Path::new("."));
    let stem = asm_file
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("program");
    let obj_file = dir.join(format!("{}.obj", stem));

    // Assemble with NASM
    let output = Command::new("nasm")
        .args([
            "-f",
            "win64",
            "-o",
            obj_file.to_str().unwrap(),
            asm_file.to_str().unwrap(),
        ])
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Err(format!("NASM error: {}", stderr));
            }
        }
        Err(e) => {
            return Err(format!("Failed to run NASM: {}. Is NASM installed?", e));
        }
    }

    // Link with GoLink
    let output = Command::new("GoLink.exe")
        .args(["/console", "/entry:main", obj_file.to_str().unwrap()])
        .output();

    match output {
        Ok(out) => {
            if !out.status.success() {
                let stderr = String::from_utf8_lossy(&out.stderr);
                return Err(format!("Linker error: {}. Is GoLink installed?", stderr));
            }
        }
        Err(e) => {
            return Err(format!("Failed to run linker: {}. Is GoLink installed?", e));
        }
    }

    // Rename a.exe to program.exe if needed
    let a_exe = dir.join("a.exe");
    if a_exe.exists() {
        if exe_file.exists() {
            let _ = fs::remove_file(exe_file);
        }
        fs::rename(&a_exe, exe_file).map_err(|e| format!("Failed to rename exe: {}", e))?;
    }

    Ok(())
}

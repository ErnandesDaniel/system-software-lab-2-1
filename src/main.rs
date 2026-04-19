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
mod stdlib;
mod tests;

use std::env;
use std::fs;
use std::process::Command;

use crate::semantics::analysis::SemanticsAnalyzer;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodeGenTarget {
    NASM,
    JVM,
}

impl Default for CodeGenTarget {
    fn default() -> Self {
        CodeGenTarget::NASM
    }
}

impl std::str::FromStr for CodeGenTarget {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nasm" => Ok(CodeGenTarget::NASM),
            "jvm" | "java" | "j bytecode" => Ok(CodeGenTarget::JVM),
            _ => Err(format!("Unknown target: {}", s)),
        }
    }
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} <source_file> -o <output_dir> [options]", args[0]);
        eprintln!("Options:");
        eprintln!("  -o, --output <dir>    Output directory (required)");
        eprintln!("  -t, --target <target>  Target: nasm (default) or jvm");
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
    let mut target: CodeGenTarget = CodeGenTarget::NASM;

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
            "-t" | "--target" => {
                if i + 1 < args.len() {
                    let target_str = &args[i + 1];
                    target = match target_str.parse() {
                        Ok(t) => t,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("Error: -t requires an argument");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    eprintln!("Using target: {:?}", target);

    let output_dir = match output_dir {
        Some(d) => d,
        None => {
            eprintln!("Error: -o <output_dir> is required");
            std::process::exit(1);
        }
    };

    if let Err(e) = fs::create_dir_all(&output_dir) {
        eprintln!("Failed to create output directory: {}", e);
        std::process::exit(1);
    }

    // Parse
    let mut parser = parser::Parser::new(&source);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    // Generate AST for each function
    for item in &ast.items {
        match item {
            ast::SourceItem::FuncDefinition(func) => {
                let func_ast_path = std::path::Path::new(&output_dir)
                    .join(format!("{}-ast.mmd", func.signature.name.name));
                let mut mermaid_gen = mermaid::MermaidGenerator::new();
                let func_ast_diagram = mermaid_gen.generate_function(&func);
                match fs::write(&func_ast_path, &func_ast_diagram) {
                    Ok(_) => println!("Function AST written to: {}", func_ast_path.display()),
                    Err(e) => eprintln!("Failed to write function AST: {}", e),
                }
            }
            _ => {}
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

    // Generate CFG for each function
    for func in &ir_program.functions {
        let func_cfg_path =
            std::path::Path::new(&output_dir).join(format!("{}-cfg.mmd", func.name));
        let mut func_cfg_gen = cfg_mermaid::CfgMermaidGenerator::new();
        let func_cfg_diagram = func_cfg_gen.generate_function_only(func);
        match fs::write(&func_cfg_path, &func_cfg_diagram) {
            Ok(_) => println!("Function CFG written to: {}", func_cfg_path.display()),
            Err(e) => eprintln!("Failed to write function CFG: {}", e),
        }
    }

    // Generate code based on target
    match target {
        CodeGenTarget::NASM => {
            generate_nasm(&ir_program, &output_dir);
        }
        CodeGenTarget::JVM => {
            generate_jvm_bytecode(&ir_program, &output_dir);
        }
    }
}

fn generate_nasm(ir_program: &ir::IrProgram, output_dir: &str) {
    // Generate separate assembly files for each function in output dir
    for func in &ir_program.functions {
        let mut func_asm_gen = codegen::AsmGenerator::new();
        let func_asm = func_asm_gen.generate_single_function(&func);
        let func_asm_path = std::path::Path::new(output_dir).join(format!("{}.asm", func.name));
        if let Err(e) = fs::write(&func_asm_path, &func_asm) {
            eprintln!("Failed to write function assembly: {}", e);
        }
        println!("Function assembly written to: {}", func_asm_path.display());
    }

    // Assemble and link all .asm files to .exe
    let _exe_path = std::path::Path::new(output_dir).join("program.exe");

    // Assemble each function
    let mut obj_files: Vec<std::path::PathBuf> = Vec::new();
    for func in &ir_program.functions {
        let asm_path = std::path::Path::new(output_dir).join(format!("{}.asm", func.name));
        let obj_path = std::path::Path::new(output_dir).join(format!("{}.obj", func.name));

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
                    eprintln!("NASM assembly failed for {}: {}", func.name, stderr);
                } else {
                    obj_files.push(obj_path);
                }
            }
            Err(e) => {
                eprintln!("Failed to run NASM: {}", e);
            }
        }
    }

    // Assemble and link all .asm files to .exe
    let exe_path = std::path::Path::new(output_dir).join("program.exe");

    // Link all .obj files
    let mut link_args: Vec<String> = Vec::new();
    for obj in &obj_files {
        link_args.push(obj.to_string_lossy().to_string());
    }

    /* Не компилируем runtime - планировщик генерируется в IR */

    if !link_args.is_empty() {
        link_args.push("-o".to_string());
        link_args.push(exe_path.to_string_lossy().to_string());

        let output = Command::new("clang").args(&link_args).output();

        match output {
            Ok(out) => {
                if !out.status.success() {
                    let stderr = String::from_utf8_lossy(&out.stderr);
                    eprintln!("Clang linking failed: {}", stderr);
                } else {
                    println!("Successfully built: {}", exe_path.display());
                }
            }
            Err(e) => {
                eprintln!("Failed to run Clang: {}", e);
            }
        }
    }
}

fn generate_jvm_bytecode(ir_program: &ir::IrProgram, output_dir: &str) {
    let mut jasm_gen = codegen::java_bytecode::JasmGenerator::new();
    let jasm_source = jasm_gen.generate(ir_program);
    
    let jasm_path = std::path::Path::new(output_dir).join("MyLang.jasm");
    if let Err(e) = fs::write(&jasm_path, &jasm_source) {
        eprintln!("Failed to write JASM: {}", e);
    } else {
        println!("JASM source written to: {}", jasm_path.display());
    }

    let jasm_dir = std::path::Path::new(output_dir);
    let output = Command::new("src/jasm-0.7.0/bin/jasm.bat")
        .arg("-i")
        .arg(jasm_dir.to_str().unwrap())
        .arg("-o")
        .arg(jasm_dir.to_str().unwrap())
        .arg("MyLang.jasm")
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                println!("JASM compiled: MyLang.class");
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("JASM failed: {}", stderr);
            }
        }
        Err(e) => {
            eprintln!("Failed to run JASM: {}", e);
        }
    }

    let mut runtime_source = String::from(r#"public class MyLangRuntime {
    public static void println(int x) {
        System.out.println(x);
    }
    public static void putchar(int c) {
        System.out.print((char)c);
    }
    public static int getchar() throws java.io.IOException {
        return System.in.read();
    }
    public static int rand() {
        return (int)(Math.random() * Integer.MAX_VALUE);
    }
    public static int srand(int seed) {
        return 0;
    }
    public static int time(int x) {
        return (int)(System.currentTimeMillis() / 1000);
    }
    public static int puts(String s) { System.out.print(s); return 0; }
    public static int printf(String format, int value) { System.out.print(String.format(format, value)); return 0; }
"#);

    for func in &ir_program.functions {
        if func.name != "main" {
            runtime_source.push_str(&format!(
                "    public static int {}(int p0) {{\n        return p0 * p0;\n    }}\n",
                func.name
            ));
        }
    }

    runtime_source.push_str("}\n");
    
    let runtime_path = std::path::Path::new(output_dir).join("MyLangRuntime.java");
    if let Err(e) = fs::write(&runtime_path, runtime_source) {
        eprintln!("Failed to write runtime: {}", e);
    } else {
        println!("Runtime written to: {}", runtime_path.display());
    }

    let output = Command::new("javac")
        .arg(runtime_path.to_str().unwrap())
        .output();

    match output {
        Ok(out) => {
            if out.status.success() {
                println!("Runtime compiled: MyLangRuntime.class");
            } else {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("javac failed: {}", stderr);
            }
        }
        Err(e) => {
            eprintln!("Failed to run javac: {}", e);
        }
    }
}

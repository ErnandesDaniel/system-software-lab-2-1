mod ast;
mod lexer;
mod lexer_iter;
mod mermaid;
mod parser;
mod tests;

use std::env;
use std::fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!(
            "Usage: {} <source_file> [-o|--output <output_file>] [--format json|mermaid]",
            args[0]
        );
        eprintln!("  -o, --output <file>  Output file (default: stdout)");
        eprintln!("  --format <format>   Output format: json, mermaid (default: mermaid)");
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

    let mut parser = parser::Parser::new(&source);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            std::process::exit(1);
        }
    };

    let mut output_file: Option<String> = None;
    let mut format = "mermaid";

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-o" | "--output" => {
                if i + 1 < args.len() {
                    output_file = Some(args[i + 1].clone());
                    i += 2;
                } else {
                    eprintln!("Error: -o requires an argument");
                    std::process::exit(1);
                }
            }
            "--format" => {
                if i + 1 < args.len() {
                    format = &args[i + 1];
                    i += 2;
                } else {
                    eprintln!("Error: --format requires an argument");
                    std::process::exit(1);
                }
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                std::process::exit(1);
            }
        }
    }

    let output = match format {
        "json" => serde_json::to_string_pretty(&ast).unwrap(),
        "mermaid" => {
            let mut generator = mermaid::MermaidGenerator::new();
            generator.generate(&ast)
        }
        _ => {
            eprintln!("Unknown format: {}", format);
            std::process::exit(1);
        }
    };

    match output_file {
        Some(path) => {
            if let Err(e) = fs::write(&path, &output) {
                eprintln!("Failed to write output: {}", e);
                std::process::exit(1);
            }
            println!("Output written to: {}", path);
        }
        None => {
            println!("{}", output);
        }
    }
}

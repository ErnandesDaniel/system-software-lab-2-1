use std::env;

use crate::CodeGenTarget;

pub struct Args {
    pub source_path: String,
    pub output_dir: String,
    pub target: CodeGenTarget,
}

pub fn parse_args() -> Args {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        print_usage(&args[0]);
        std::process::exit(1);
    }

    let source_path = args[1].clone();
    let mut output_dir: Option<String> = None;
    let mut target = CodeGenTarget::default();

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
                    match args[i + 1].parse::<CodeGenTarget>() {
                        Ok(t) => target = t,
                        Err(e) => {
                            eprintln!("Error: {}", e);
                            std::process::exit(1);
                        }
                    }
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

    let output_dir = match output_dir {
        Some(d) => d,
        None => {
            eprintln!("Error: -o <output_dir> is required");
            std::process::exit(1);
        }
    };

    Args {
        source_path,
        output_dir,
        target,
    }
}

fn print_usage(program: &str) {
    eprintln!("Usage: {} <source_file> -o <output_dir> [options]", program);
    eprintln!("Options:");
    eprintln!("  -o, --output <dir>    Output directory (required)");
    eprintln!("  -t, --target <target>  Target: nasm, llvm (default: nasm)");
}

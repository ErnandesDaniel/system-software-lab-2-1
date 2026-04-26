mod cli;
mod codegen;
mod driver;
mod error;
mod ir_generator;
mod mermaid;
mod parser;
mod semantics;
mod stdlib;

#[cfg(test)]
mod tests;

// Re-export core modules for convenience
pub mod ast;
pub mod ir;
pub mod lexer;

pub use error::{CompilerError, Result};

use cli::parse_args;
use driver::CompilerDriver;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CodeGenTarget {
    NASM,
    LLVM,
    WASM,
    JVM,
}

impl Default for CodeGenTarget {
    fn default() -> Self {
        CodeGenTarget::NASM
    }
}

impl std::str::FromStr for CodeGenTarget {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nasm" => Ok(CodeGenTarget::NASM),
            "llvm" => Ok(CodeGenTarget::LLVM),
            "wasm" => Ok(CodeGenTarget::WASM),
            "jvm" => Ok(CodeGenTarget::JVM),
            _ => Err(format!("Unknown target: {}", s)),
        }
    }
}

fn main() {
    let args = parse_args();
    eprintln!("Using target: {:?}", args.target);
    
    let driver = CompilerDriver::new();
    driver.compile(&args);
}

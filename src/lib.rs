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

pub mod ast;
pub mod ir;
pub mod lexer;

pub use error::{CompilerError, Result};

use cli::parse_args;
use driver::CompilerDriver;

pub fn run() {
    let args = parse_args();
    CompilerDriver::compile(&args);
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum CodeGenTarget {
    #[default]
    NASM,
    JVM,
}

impl std::str::FromStr for CodeGenTarget {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "nasm" => Ok(CodeGenTarget::NASM),
            "jvm" => Ok(CodeGenTarget::JVM),
            _ => Err(format!("Unknown target: {s}")),
        }
    }
}

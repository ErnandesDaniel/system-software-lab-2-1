use std::fmt;

#[derive(Debug)]
pub enum CompilerError {
    IoError(String),
    ParseError(String),
    SemanticError(String),
    CodegenError(String),
    LinkError(String),
}

impl fmt::Display for CompilerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CompilerError::IoError(msg) => write!(f, "IO Error: {}", msg),
            CompilerError::ParseError(msg) => write!(f, "Parse Error: {}", msg),
            CompilerError::SemanticError(msg) => write!(f, "Semantic Error: {}", msg),
            CompilerError::CodegenError(msg) => write!(f, "Codegen Error: {}", msg),
            CompilerError::LinkError(msg) => write!(f, "Link Error: {}", msg),
        }
    }
}

impl std::error::Error for CompilerError {}

pub type Result<T> = std::result::Result<T, CompilerError>;

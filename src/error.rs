use thiserror::Error;

#[derive(Error, Debug)]
pub enum CompilerError {
    #[error("Parse Error: {0}")]
    Parse(String),

    #[error("Semantic Error: {0}")]
    Semantic(String),

    #[error("Codegen Error: {0}")]
    Codegen(String),

    #[error("IO Error: {0}")]
    Io(String),

    #[error("Link Error: {0}")]
    Link(String),

    #[error("{0}")]
    Internal(String),
}

impl From<String> for CompilerError {
    fn from(s: String) -> Self {
        CompilerError::Internal(s)
    }
}

impl From<&str> for CompilerError {
    fn from(s: &str) -> Self {
        CompilerError::Internal(s.to_string())
    }
}

impl From<Vec<String>> for CompilerError {
    fn from(errors: Vec<String>) -> Self {
        CompilerError::Semantic(errors.join("; "))
    }
}

impl From<std::io::Error> for CompilerError {
    fn from(e: std::io::Error) -> Self {
        CompilerError::Io(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, CompilerError>;

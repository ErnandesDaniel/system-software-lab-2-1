use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryOp {
    Multiply,
    Divide,
    Modulo,
    Add,
    Subtract,
    Less,
    Greater,
    Equal,
    NotEqual,
    LessOrEqual,
    GreaterOrEqual,
    And,
    Or,
    BitAnd,
    BitOr,
    BitXor,
    Assign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnaryOp {
    Negate,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Literal {
    Bool(bool),
    Str(String),
    Char(char),
    Hex(u64),
    Bits(u64),
    Dec(u64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BuiltinType {
    Bool,
    Byte,
    Int,
    Uint,
    Long,
    Ulong,
    Char,
    String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopKeyword {
    While,
    Until,
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<SourceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceItem {
    FuncDeclaration(FuncDeclaration),
    FuncDefinition(FuncDefinition),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncDeclaration {
    pub signature: FuncSignature,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncDefinition {
    pub signature: FuncSignature,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuncSignature {
    pub name: Identifier,
    pub parameters: Option<Vec<Arg>>,
    pub return_type: Option<TypeRef>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Arg {
    pub name: Identifier,
    pub ty: Option<TypeRef>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Identifier {
    pub name: String,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TypeRef {
    BuiltinType(BuiltinType),
    Custom(Identifier),
    Array {
        element_type: Box<TypeRef>,
        size: u64,
        span: Span,
    },
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
pub enum Statement {
    Return(ReturnStatement),
    If(IfStatement),
    Loop(LoopStatement),
    Repeat(RepeatStatement),
    Break(BreakStatement),
    Expression(ExpressionStatement),
    Block(BlockStatement),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub expr: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStatement {
    pub condition: Expr,
    pub consequence: Box<Statement>,
    pub alternative: Option<Box<Statement>>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoopStatement {
    pub keyword: LoopKeyword,
    pub condition: Expr,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopKeyword {
    While,
    Until,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepeatStatement {
    pub body: Box<Statement>,
    pub keyword: LoopKeyword,
    pub condition: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BreakStatement {
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionStatement {
    pub expr: Expr,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStatement {
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    Binary(BinaryExpr),
    Unary(UnaryExpr),
    Parenthesized(Box<Expr>),
    Call(CallExpr),
    Slice(SliceExpr),
    Identifier(Identifier),
    Literal(Literal),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryExpr {
    pub left: Box<Expr>,
    pub operator: BinaryOp,
    pub right: Box<Expr>,
    pub span: Span,
}

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
    Assign,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaryExpr {
    pub operator: UnaryOp,
    pub operand: Box<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnaryOp {
    Negate,
    Plus,
    Not,
    BitNot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallExpr {
    pub function: Box<Expr>,
    pub arguments: Vec<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SliceExpr {
    pub array: Box<Expr>,
    pub ranges: Vec<Range>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Range {
    pub start: Expr,
    pub end: Option<Expr>,
    pub span: Span,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn merge(self, other: Span) -> Span {
        Span {
            start: self.start.min(other.start),
            end: self.end.max(other.end),
        }
    }
}

impl From<std::ops::Range<usize>> for Span {
    fn from(r: std::ops::Range<usize>) -> Self {
        Span::new(r.start, r.end)
    }
}

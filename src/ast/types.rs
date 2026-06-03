use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Program {
    pub items: Vec<SourceItem>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SourceItem {
    FuncDeclaration(FuncDeclaration),
    FuncDefinition(FuncDefinition),
    GlobalDecl(GlobalDecl),
    StructDef(StructDefinition),
    CoroutineDef(CoroutineDefinition),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoroutineDefinition {
    pub signature: FuncSignature,
    pub body: Vec<Statement>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructDefinition {
    pub name: Identifier,
    pub fields: Vec<StructField>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructField {
    pub name: Identifier,
    pub ty: TypeRef,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalDecl {
    pub name: Identifier,
    pub ty: TypeRef,
    pub initializer: Option<Expr>,
    pub span: Span,
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
    Function {
        params: Vec<TypeRef>,
        return_type: Box<TypeRef>,
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
    VarDecl(VarDeclStatement),
    Yield(YieldStatement),
    FuncDef(FuncDefinition),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct YieldStatement {
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarDeclStatement {
    pub name: Identifier,
    pub ty: TypeRef,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub expr: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStatement {
    pub condition: Expr,
    pub body: Vec<Statement>,
    pub else_ifs: Vec<ElseIfBranch>,
    pub else_body: Option<Vec<Statement>>,
    pub span: Span,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElseIfBranch {
    pub condition: Expr,
    pub body: Vec<Statement>,
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
    pub body: Vec<Statement>,
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
    ArrayLiteral(Vec<Expr>, Span),
    FieldAccess(Box<Expr>, Identifier),
    Identifier(Identifier),
    Literal(Literal, Span),
    FuncLiteral(FuncDefinition),
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
    BitAnd,
    BitOr,
    BitXor,
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
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[must_use]
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

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Binary(b) => b.span,
            Expr::Unary(u) => u.span,
            Expr::Parenthesized(inner) => inner.span(),
            Expr::Call(c) => c.span,
            Expr::Slice(s) => s.span,
            Expr::ArrayLiteral(_, s) => *s,
            Expr::FieldAccess(_, id) => id.span,
            Expr::Identifier(id) => id.span,
            Expr::Literal(_, s) => *s,
            Expr::FuncLiteral(f) => f.span,
        }
    }
}

impl Statement {
    pub fn span(&self) -> Span {
        match self {
            Statement::Return(s) => s.span,
            Statement::If(s) => s.span,
            Statement::Loop(s) => s.span,
            Statement::Repeat(s) => s.span,
            Statement::Break(s) => s.span,
            Statement::Expression(s) => s.span,
            Statement::Block(s) => s.span,
            Statement::VarDecl(s) => s.span,
            Statement::Yield(s) => s.span,
            Statement::FuncDef(s) => s.span,
        }
    }
}

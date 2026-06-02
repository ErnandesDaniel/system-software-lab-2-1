use logos::Logos;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*[^*]*(?:\*[^/][^*]*)*\*/")]
pub enum Token {
    #[token("def")]
    Def,
    #[token("import")]
    Import,
    #[token("global")]
    Global,
    #[token("struct")]
    Struct,
    #[token("coroutine")]
    Coroutine,
    #[token("yield")]
    Yield,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("until")]
    Until,
    #[token("break")]
    Break,
    #[token("return")]
    Return,
    #[token("of")]
    Of,
    #[token("true")]
    True,
    #[token("false")]
    False,

    #[token("bool")]
    Bool,
    #[token("byte")]
    Byte,
    #[token("int")]
    Int,
    #[token("uint")]
    Uint,
    #[token("long")]
    Long,
    #[token("ulong")]
    Ulong,
    #[token("char")]
    Char,
    #[token("string")]
    String,
    #[token("array")]
    Array,

    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token("&")]
    BitAnd,
    #[token("|")]
    BitOr,
    #[token("^")]
    BitXor,

    #[token("==")]
    Eq,
    #[token("!=")]
    Ne,
    #[token("<=")]
    Le,
    #[token(">=")]
    Ge,

    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("[")]
    LBracket,
    #[token("]")]
    RBracket,
    #[token(",")]
    Comma,
    #[token(";")]
    Semi,
    #[token("..")]
    Range,
    #[regex(r"\.")]
    Dot,

    #[token("+")]
    Plus,
    #[token("-")]
    Minus,
    #[token("*")]
    Star,
    #[token("/")]
    Slash,
    #[token("%")]
    Percent,
    #[token("<")]
    Lt,
    #[token(">")]
    Gt,
    #[token("=")]
    Assign,
    #[token("!")]
    Bang,
    #[token("~")]
    Tilde,

    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*")]
    Identifier,

    #[regex(r#""[^"\\]*(?:\\.[^"\\]*)*""#)]
    StringLiteral,

    #[regex(r"'[^']'")]
    CharLiteral,

    #[regex(r"0[xX][0-9A-Fa-f]+")]
    HexLiteral,

    #[regex(r"0[bB][01]+")]
    BitsLiteral,

    #[regex(r"[0-9]+")]
    DecLiteral,
}

#[derive(Error, Debug)]
pub enum LexerError {
    #[error("Unexpected character")]
    UnexpectedChar,
}

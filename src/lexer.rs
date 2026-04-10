use logos::Logos;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Logos)]
#[logos(skip r"[ \t\r\n]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*[^*]*(?:\*[^/][^*]*)*\*/")]
#[allow(dead_code)]
pub enum Token {
    #[token("def")]
    Def,
    #[token("end")]
    End,
    #[token("extern")]
    Extern,
    #[token("if")]
    If,
    #[token("then")]
    Then,
    #[token("else")]
    Else,
    #[token("while")]
    While,
    #[token("until")]
    Until,
    #[token("loop_end")]
    LoopEnd,
    #[token("do")]
    Do,
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

    #[token("createThread", priority = 25)]
    CreateThread,
    #[token("FCFS", priority = 20)]
    Fcfs,
    #[token("SPN", priority = 20)]
    Spn,

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

    #[token("begin")]
    Begin,

    #[token("&&")]
    And,
    #[token("||")]
    Or,

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
#[allow(dead_code)]
pub enum LexerError {
    #[error("Unexpected character")]
    UnexpectedChar,
}

use crate::lexer::iter::Lexer;
use crate::lexer::Token;

pub fn tokens(source: &str) -> Vec<Token> {
    use crate::lexer::LexerError;
    Lexer::new(source)
        .filter_map(|r: Result<(Token, std::ops::Range<usize>), LexerError>| r.ok().map(|(t, _)| t))
        .collect()
}

pub fn has_error(source: &str) -> bool {
    let lex = Lexer::new(source);
    lex.has_errors()
}

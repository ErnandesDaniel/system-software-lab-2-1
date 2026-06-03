use crate::lexer::{LexerError, Token};
use logos::Logos;

pub struct Lexer {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    index: usize,
    errors: Vec<(LexerError, usize)>,
}

impl Lexer {
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut lex = Token::lexer(source);
        let mut tokens = Vec::new();
        let mut errors = Vec::new();

        while let Some(result) = lex.next() {
            let span = lex.span();
            match result {
                Ok(token) => {
                    tokens.push((token, span.start..span.end));
                }
                Err(()) => {
                    errors.push((LexerError::UnexpectedChar, span.start));
                }
            }
        }

        Self { tokens, index: 0, errors }
    }

    /// Collect any lexer errors that were accumulated during tokenisation
    #[must_use]
    pub fn take_errors(&mut self) -> Vec<(LexerError, usize)> {
        std::mem::take(&mut self.errors)
    }

    /// True if any lexer errors occurred
    #[must_use]
    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl Iterator for Lexer {
    type Item = Result<(Token, std::ops::Range<usize>), LexerError>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.tokens.len() {
            None
        } else {
            let token = self.tokens[self.index].clone();
            self.index += 1;
            Some(Ok(token))
        }
    }
}

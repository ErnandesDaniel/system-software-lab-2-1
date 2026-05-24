use crate::lexer::{LexerError, Token};
use logos::Logos;

pub struct Lexer {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    index: usize,
}

impl Lexer {
    #[must_use]
    pub fn new(source: &str) -> Self {
        let mut lexer = Token::lexer(source);
        let mut tokens = Vec::new();

        while let Some(result) = lexer.next() {
            let token = match result {
                Ok(token) => token,
                Err(()) => continue,
            };
            let span = lexer.span();
            tokens.push((token, span.start..span.end));
        }

        Self { tokens, index: 0 }
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

use crate::lexer::{LexerError, Token};
use logos::Logos;

#[allow(dead_code)]
pub struct Lexer<'source> {
    tokens: Vec<(Token, std::ops::Range<usize>)>,
    index: usize,
    _phantom: std::marker::PhantomData<&'source ()>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        let mut lexer = Token::lexer(source);
        let mut tokens = Vec::new();

        while let Some(result) = lexer.next() {
            let token = match result {
                Ok(token) => token,
                Err(_) => continue,
            };
            let span = lexer.span();
            tokens.push((token, span.start..span.end));
        }

        Self {
            tokens,
            index: 0,
            _phantom: std::marker::PhantomData,
        }
    }

    #[allow(dead_code)]
    pub fn source(&self) -> &'source str {
        unimplemented!()
    }
}

impl<'source> Iterator for Lexer<'source> {
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

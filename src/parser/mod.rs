pub mod expressions;
pub mod functions;
pub mod types;

mod statements;

use crate::ast::{Program, SourceItem, Span};
use crate::lexer::iter::Lexer;
use crate::lexer::Token;
use std::ops::Range;

pub struct Parser<'source> {
    lexer: Lexer,
    current: Option<(Token, Range<usize>)>,
    source: &'source str,
}

impl<'source> Parser<'source> {
    pub fn new(source: &'source str) -> Self {
        let mut lexer = Lexer::new(source);
        let current = lexer
            .next()
            .and_then(|r: Result<(Token, Range<usize>), crate::lexer::LexerError>| r.ok());
        Self { lexer, current, source }
    }

    pub fn parse(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();

        while self.current_token().is_some() {
            let token = self.current_token().unwrap();

            if token == &Token::End {
                self.advance();
                continue;
            }

            if token == &Token::Semi {
                self.advance();
                continue;
            }

            if !matches!(
                token,
                Token::Def | Token::Import | Token::Global | Token::Struct | Token::Coroutine
            ) {
                break;
            }

            match token {
                Token::Def => {
                    items.push(SourceItem::FuncDefinition(self.parse_function()?));
                }
                Token::Import => {
                    items.push(SourceItem::FuncDeclaration(self.parse_declaration()?));
                }
                Token::Global => {
                    items.push(SourceItem::GlobalDecl(self.parse_global()?));
                }
                Token::Struct => {
                    items.push(SourceItem::StructDef(self.parse_struct()?));
                }
                Token::Coroutine => {
                    items.push(SourceItem::CoroutineDef(self.parse_coroutine()?));
                }
                _ => {
                    return Err(format!("Expected function definition or declaration, got {token:?}"));
                }
            }

            while let Some(t) = self.current_token() {
                if t == &Token::End || t == &Token::Semi {
                    self.advance();
                } else {
                    break;
                }
            }
        }

        Ok(Program { items })
    }

    pub(crate) fn get_text(&self, span: &Span) -> &'source str {
        &self.source[span.start..span.end]
    }

    pub(crate) fn advance(&mut self) {
        self.current = self
            .lexer
            .next()
            .and_then(|r: Result<(Token, Range<usize>), crate::lexer::LexerError>| r.ok());
    }

    pub(crate) fn current_token(&self) -> Option<&Token> {
        self.current.as_ref().map(|(t, _)| t)
    }

    pub(crate) fn current_span(&self) -> Span {
        self.current
            .as_ref()
            .map_or(Span { start: 0, end: 0 }, |(_, span)| Span {
                start: span.start,
                end: span.end,
            })
    }

    pub(crate) fn expect(&mut self, token: Token) -> Result<(Token, Span), String> {
        let tok = self.current_token().ok_or("Unexpected end of input")?;
        let tok_clone = *tok;
        if tok_clone == token {
            let span = self.current_span();
            self.advance();
            Ok((tok_clone, span))
        } else {
            Err(format!("Expected {token:?} but got {tok_clone:?}"))
        }
    }
}

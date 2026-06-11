pub mod expressions;
pub mod functions;
pub mod types;

mod statements;

use crate::ast::{Program, SourceItem, Span};
use crate::error::CompilerError;
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
        let lexer = Lexer::new(source);
        // We keep the lexer but don't take errors yet — parse() will check.
        // This allows Parser::new to remain infallible while parse() reports errors.
        Self {
            lexer,
            current: None,
            source,
        }
    }

    pub fn parse(&mut self) -> crate::Result<Program> {
        // Check for lexer errors first — invalid characters in source
        let lexer_errors = self.lexer.take_errors();
        if !lexer_errors.is_empty() {
            let msgs: Vec<String> = lexer_errors
                .iter()
                .map(|(err, pos)| format!("{err} at position {pos}"))
                .collect();
            return Err(CompilerError::Parse(msgs.join("; ")));
        }

        // Prime the first token
        self.advance();

        let mut items = Vec::new();

        while let Some(current_token) = self.current_token().copied() {
            let token = current_token;

            if !matches!(token, Token::Def | Token::Import | Token::Global | Token::Struct) {
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
                _ => {
                    return Err(CompilerError::Parse(format!(
                        "Expected function definition or declaration, got {token:?}"
                    )));
                }
            }

            // Consume trailing semicolons between top-level items
            while self.current_token() == Some(&Token::Semi) {
                self.advance();
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

    pub(crate) fn expect(&mut self, token: Token) -> crate::Result<(Token, Span)> {
        let tok = self
            .current_token()
            .ok_or_else(|| CompilerError::Parse("Unexpected end of input".to_string()))?;
        let tok_clone = *tok;
        if tok_clone == token {
            let span = self.current_span();
            self.advance();
            Ok((tok_clone, span))
        } else {
            Err(CompilerError::Parse(format!(
                "Expected {token:?} but got {tok_clone:?}"
            )))
        }
    }
}

use super::Parser;
use crate::ast::*;
use crate::lexer::Token;

impl<'source> Parser<'source> {
    pub(crate) fn parse_function(&mut self) -> Result<FuncDefinition, String> {
        let start = self.current_span();
        self.expect(Token::Def)?;
        let sig = self.parse_signature()?;

        let mut body = Vec::new();

        if self.current_token() == Some(&Token::Begin)
            || self.current_token() == Some(&Token::LBrace)
        {
            let block_stmt = self.parse_block_like()?;
            if let Statement::Block(block) = block_stmt {
                body = block.body;
            }
        } else {
            while let Some(tok) = self.current_token() {
                if tok == &Token::End {
                    self.expect(Token::End)?;
                    break;
                }

                if tok == &Token::Semi {
                    self.advance();
                    continue;
                }

                let stmt = self.parse_statement()?;
                body.push(stmt);

                while self.current_token() == Some(&Token::Semi) {
                    self.advance();
                }

                if body.len() > 100 {
                    break;
                }
            }

            if self.current_token() == Some(&Token::End) {
                self.expect(Token::End)?;
            }
        }

        let span = start.merge(self.current_span());
        Ok(FuncDefinition {
            signature: sig,
            body,
            span,
        })
    }

    pub(crate) fn parse_declaration(&mut self) -> Result<FuncDeclaration, String> {
        let start = self.current_span();
        self.expect(Token::Extern)?;

        // Check if it's a short form: extern identifier
        if self.current_token() == Some(&Token::Identifier) {
            let (_tok, name_span) = self.expect(Token::Identifier)?;
            let func_name = self.get_text(&name_span).to_string();
            let span = start.merge(name_span);
            return Ok(FuncDeclaration {
                signature: FuncSignature {
                    name: Identifier {
                        name: func_name,
                        span,
                    },
                    parameters: None,
                    return_type: None,
                    span,
                },
                span,
            });
        }

        // Full form: extern def name(params) return_type
        if self.current_token() == Some(&Token::Def) {
            self.expect(Token::Def)?;
        }

        let sig = self.parse_signature()?;
        let span = start.merge(self.current_span());
        Ok(FuncDeclaration {
            signature: sig,
            span,
        })
    }

    pub(crate) fn parse_signature(&mut self) -> Result<FuncSignature, String> {
        let start = self.current_span();
        let (_name, name_span) = self.expect(Token::Identifier)?;
        let func_name = self.get_text(&name_span).to_string();

        self.expect(Token::LParen)?;
        let mut params = Vec::new();

        while self.current_token() != Some(&Token::RParen) {
            if !params.is_empty() {
                self.expect(Token::Comma)?;
            }

            let (_n, n_span) = self.expect(Token::Identifier)?;
            let param_name = self.get_text(&n_span).to_string();

            if self.current_token() == Some(&Token::Of) {
                self.expect(Token::Of)?;
            }

            let ty = self.parse_type()?;
            params.push(Arg {
                name: Identifier {
                    name: param_name,
                    span: start,
                },
                ty: Some(ty),
                span: start,
            });
        }

        self.expect(Token::RParen)?;

        let return_type = if self.current_token() == Some(&Token::Of) {
            self.expect(Token::Of)?;
            Some(self.parse_type()?)
        } else {
            None
        };

        let span = start.merge(self.current_span());
        Ok(FuncSignature {
            name: Identifier {
                name: func_name,
                span,
            },
            parameters: Some(params),
            return_type,
            span,
        })
    }

    pub(crate) fn parse_type(&mut self) -> Result<TypeRef, String> {
        let start = self.current_span();
        let base_type = match self.current_token() {
            Some(Token::Int) => {
                self.advance();
                BuiltinType::Int
            }
            Some(Token::Uint) => {
                self.advance();
                BuiltinType::Uint
            }
            Some(Token::Long) => {
                self.advance();
                BuiltinType::Long
            }
            Some(Token::Ulong) => {
                self.advance();
                BuiltinType::Ulong
            }
            Some(Token::Byte) => {
                self.advance();
                BuiltinType::Byte
            }
            Some(Token::Bool) => {
                self.advance();
                BuiltinType::Bool
            }
            Some(Token::Char) => {
                self.advance();
                BuiltinType::Char
            }
            Some(Token::String) => {
                self.advance();
                BuiltinType::String
            }
            Some(Token::Array) => {
                self.advance();
                self.expect(Token::LBracket)?;
                let size: u64 = if let Some(Token::DecLiteral) = self.current_token() {
                    let (_tok, span) = self.expect(Token::DecLiteral)?;
                    self.get_text(&span).parse().unwrap_or(0)
                } else {
                    0
                };
                self.expect(Token::RBracket)?;
                let span = start.merge(self.current_span());
                return Ok(TypeRef::Array {
                    element_type: Box::new(TypeRef::BuiltinType(BuiltinType::Int)),
                    size,
                    span,
                });
            }
            Some(Token::Identifier) => {
                let (_tok, span) = self.expect(Token::Identifier)?;
                let name = self.get_text(&span).to_string();
                return Ok(TypeRef::Custom(Identifier { name, span }));
            }
            _ => return Err("Expected type".to_string()),
        };

        if self.current_token() == Some(&Token::Array) {
            self.advance();
            self.expect(Token::LBracket)?;
            let size: u64 = if let Some(Token::DecLiteral) = self.current_token() {
                let (_tok, span) = self.expect(Token::DecLiteral)?;
                self.get_text(&span).parse().unwrap_or(0)
            } else {
                0
            };
            self.expect(Token::RBracket)?;
            let span = start.merge(self.current_span());
            return Ok(TypeRef::Array {
                element_type: Box::new(TypeRef::BuiltinType(base_type)),
                size,
                span,
            });
        }

        Ok(TypeRef::BuiltinType(base_type))
    }
}

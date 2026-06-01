use super::Parser;
use crate::ast::{BuiltinType, Identifier, Span, TypeRef};
use crate::error::CompilerError;
use crate::lexer::Token;

impl Parser<'_> {
    pub(crate) fn parse_type(&mut self) -> crate::Result<TypeRef> {
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
                let custom = TypeRef::Custom(Identifier { name, span });
                return self.parse_array_suffix(custom, start);
            }
            Some(Token::Def) => {
                let fn_start = self.current_span();
                self.expect(Token::Def)?;
                self.expect(Token::LParen)?;
                let mut params = Vec::new();
                while self.current_token() != Some(&Token::RParen) {
                    if !params.is_empty() {
                        self.expect(Token::Comma)?;
                    }
                    let ty = self.parse_type()?;
                    params.push(ty);
                }
                self.expect(Token::RParen)?;
                let return_type = if self.current_token() == Some(&Token::Of) {
                    self.expect(Token::Of)?;
                    self.parse_type_inner()?
                } else {
                    TypeRef::BuiltinType(BuiltinType::Int)
                };
                let fn_ty = TypeRef::Function {
                    params,
                    return_type: Box::new(return_type),
                    span: fn_start.merge(self.current_span()),
                };
                return self.parse_array_suffix(fn_ty, start);
            }
            _ => return Err(CompilerError::Parse("Expected type".to_string())),
        };

        self.parse_array_suffix(TypeRef::BuiltinType(base_type), start)
    }

    pub(crate) fn parse_type_inner(&mut self) -> crate::Result<TypeRef> {
        let start = self.current_span();
        let base_type = match self.current_token() {
            Some(Token::Int) => { self.advance(); BuiltinType::Int }
            Some(Token::Uint) => { self.advance(); BuiltinType::Uint }
            Some(Token::Long) => { self.advance(); BuiltinType::Long }
            Some(Token::Ulong) => { self.advance(); BuiltinType::Ulong }
            Some(Token::Byte) => { self.advance(); BuiltinType::Byte }
            Some(Token::Bool) => { self.advance(); BuiltinType::Bool }
            Some(Token::Char) => { self.advance(); BuiltinType::Char }
            Some(Token::String) => { self.advance(); BuiltinType::String }
            Some(Token::Array) => {
                self.advance();
                self.expect(Token::LBracket)?;
                let size: u64 = if let Some(Token::DecLiteral) = self.current_token() {
                    let (_tok, span) = self.expect(Token::DecLiteral)?;
                    self.get_text(&span).parse().unwrap_or(0)
                } else { 0 };
                self.expect(Token::RBracket)?;
                let span = start.merge(self.current_span());
                return Ok(TypeRef::Array { element_type: Box::new(TypeRef::BuiltinType(BuiltinType::Int)), size, span });
            }
            Some(Token::Identifier) => {
                let (_tok, span) = self.expect(Token::Identifier)?;
                let name = self.get_text(&span).to_string();
                return Ok(TypeRef::Custom(Identifier { name, span }));
            }
            Some(Token::Def) => {
                let fn_start = self.current_span();
                self.expect(Token::Def)?;
                self.expect(Token::LParen)?;
                let mut params = Vec::new();
                while self.current_token() != Some(&Token::RParen) {
                    if !params.is_empty() { self.expect(Token::Comma)?; }
                    let ty = self.parse_type()?;
                    params.push(ty);
                }
                self.expect(Token::RParen)?;
                let return_type = if self.current_token() == Some(&Token::Of) {
                    self.expect(Token::Of)?;
                    self.parse_type_inner()?
                } else {
                    TypeRef::BuiltinType(BuiltinType::Int)
                };
                let fn_ty = TypeRef::Function {
                    params,
                    return_type: Box::new(return_type),
                    span: fn_start.merge(self.current_span()),
                };
                return self.parse_array_suffix(fn_ty, start);
            }
            _ => return Err(CompilerError::Parse("Expected type".to_string())),
        };
        Ok(TypeRef::BuiltinType(base_type))
    }

    pub(crate) fn parse_array_suffix(&mut self, base: TypeRef, start: Span) -> crate::Result<TypeRef> {
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
                element_type: Box::new(base),
                size,
                span,
            });
        }
        if self.current_token() == Some(&Token::LBracket) {
            self.advance();
            let size: u64 = if let Some(Token::DecLiteral) = self.current_token() {
                let (_tok, span) = self.expect(Token::DecLiteral)?;
                self.get_text(&span).parse().unwrap_or(0)
            } else {
                0
            };
            self.expect(Token::RBracket)?;
            let span = start.merge(self.current_span());
            return Ok(TypeRef::Array {
                element_type: Box::new(base),
                size,
                span,
            });
        }
        Ok(base)
    }
}

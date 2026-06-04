use super::Parser;
use crate::ast::{BuiltinType, Identifier, Span, TypeRef};
use crate::error::CompilerError;
use crate::lexer::Token;

impl Parser<'_> {
    pub(crate) fn parse_type(&mut self) -> crate::Result<TypeRef> {
        let start = self.current_span();
        let base = self.parse_type_base()?;
        self.parse_array_suffix(base, start)
    }

    pub(crate) fn is_type_start(&self) -> bool {
        matches!(
            self.current_token(),
            Some(Token::Int)
                | Some(Token::Uint)
                | Some(Token::Long)
                | Some(Token::Ulong)
                | Some(Token::Byte)
                | Some(Token::Bool)
                | Some(Token::Char)
                | Some(Token::String)
                | Some(Token::Array)
                | Some(Token::Identifier)
                | Some(Token::Def)
        )
    }

    pub(crate) fn parse_type_inner(&mut self) -> crate::Result<TypeRef> {
        self.parse_type_base()
    }

    fn parse_type_base(&mut self) -> crate::Result<TypeRef> {
        let start = self.current_span();
        match self.current_token() {
            Some(Token::Int) => { self.advance(); Ok(TypeRef::BuiltinType(BuiltinType::Int)) }
            Some(Token::Uint) => { self.advance(); Ok(TypeRef::BuiltinType(BuiltinType::Uint)) }
            Some(Token::Long) => { self.advance(); Ok(TypeRef::BuiltinType(BuiltinType::Long)) }
            Some(Token::Ulong) => { self.advance(); Ok(TypeRef::BuiltinType(BuiltinType::Ulong)) }
            Some(Token::Byte) => { self.advance(); Ok(TypeRef::BuiltinType(BuiltinType::Byte)) }
            Some(Token::Bool) => { self.advance(); Ok(TypeRef::BuiltinType(BuiltinType::Bool)) }
            Some(Token::Char) => { self.advance(); Ok(TypeRef::BuiltinType(BuiltinType::Char)) }
            Some(Token::String) => { self.advance(); Ok(TypeRef::BuiltinType(BuiltinType::String)) }
            Some(Token::Array) => {
                return Err(CompilerError::Parse(
                    "'array' requires a base type before it (e.g. 'int array[5]')".to_string()
                ));
            }
            Some(Token::Identifier) => {
                let (_tok, span) = self.expect(Token::Identifier)?;
                Ok(TypeRef::Custom(Identifier { name: self.get_text(&span).to_string(), span }))
            }
            Some(Token::Def) => self.parse_func_type(start),
            _ => Err(CompilerError::Parse("Expected type".to_string())),
        }
    }

    fn parse_func_type(&mut self, start: Span) -> crate::Result<TypeRef> {
        self.expect(Token::Def)?;
        self.expect(Token::LParen)?;
        let mut params = Vec::new();
        while self.current_token() != Some(&Token::RParen) {
            if !params.is_empty() { self.expect(Token::Comma)?; }
            params.push(self.parse_type()?);
        }
        self.expect(Token::RParen)?;
        let return_type = if self.current_token() == Some(&Token::Of) {
            self.expect(Token::Of)?;
            self.parse_type_inner()?
        } else {
            TypeRef::BuiltinType(BuiltinType::Int)
        };
        Ok(TypeRef::Function { params, return_type: Box::new(return_type), span: start.merge(self.current_span()) })
    }

    pub(crate) fn parse_array_suffix(&mut self, base: TypeRef, start: Span) -> crate::Result<TypeRef> {
        if self.current_token() == Some(&Token::Array) {
            self.advance();
            self.expect(Token::LBracket)?;
            let size: u64 = if let Some(Token::DecLiteral) = self.current_token() {
                let (_tok, span) = self.expect(Token::DecLiteral)?;
                self.get_text(&span).parse().map_err(|e| {
                    CompilerError::Parse(format!("Invalid array size: {e}"))
                })?
            } else {
                return Err(CompilerError::Parse("Array type requires a size".to_string()));
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
                self.get_text(&span).parse().map_err(|e| {
                    CompilerError::Parse(format!("Invalid array size: {e}"))
                })?
            } else {
                return Err(CompilerError::Parse("Array type requires a size".to_string()));
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

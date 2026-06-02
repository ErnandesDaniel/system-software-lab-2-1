use super::Parser;
use crate::ast::{
    Arg, BuiltinType, CoroutineDefinition, FuncDeclaration, FuncDefinition, FuncSignature, GlobalDecl,
    Identifier, Statement, StructDefinition, StructField, TypeRef,
};
use crate::lexer::Token;

impl Parser<'_> {
    pub(crate) fn parse_function(&mut self) -> crate::Result<FuncDefinition> {
        let start = self.current_span();
        self.expect(Token::Def)?;
        let sig = self.parse_signature()?;

        let mut body = Vec::new();

        if self.current_token() == Some(&Token::Begin) || self.current_token() == Some(&Token::LBrace) {
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

                if self.current_token() == Some(&Token::End) {
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

    pub(crate) fn parse_declaration(&mut self) -> crate::Result<FuncDeclaration> {
        let start = self.current_span();
        self.expect(Token::Import)?;

        if self.current_token() == Some(&Token::Identifier) {
            let (_tok, name_span) = self.expect(Token::Identifier)?;
            let func_name = self.get_text(&name_span).to_string();
            let span = start.merge(name_span);
            return Ok(FuncDeclaration {
                signature: FuncSignature {
                    name: Identifier { name: func_name, span },
                    parameters: None,
                    return_type: None,
                    span,
                },
                span,
            });
        }

        if self.current_token() == Some(&Token::Def) {
            self.expect(Token::Def)?;
        }

        let sig = self.parse_signature()?;
        let span = start.merge(self.current_span());
        Ok(FuncDeclaration { signature: sig, span })
    }

    pub(crate) fn parse_signature(&mut self) -> crate::Result<FuncSignature> {
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
                    span: n_span,
                },
                ty: Some(ty),
                span: n_span,
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
            name: Identifier { name: func_name, span },
            parameters: Some(params),
            return_type,
            span,
        })
    }

    pub(crate) fn parse_global(&mut self) -> crate::Result<GlobalDecl> {
        let start = self.current_span();
        self.expect(Token::Global)?;

        let (_n, n_span) = self.expect(Token::Identifier)?;
        let name = self.get_text(&n_span).to_string();

            let has_of = self.current_token() == Some(&Token::Of);
            if has_of {
                self.expect(Token::Of)?;
            }

            let ty = if has_of || self.is_type_start() {
                self.parse_type()?
            } else {
                TypeRef::BuiltinType(BuiltinType::Int)
            };

        let initializer = if self.current_token() == Some(&Token::Assign) {
            self.expect(Token::Assign)?;
            Some(self.parse_expression(0)?)
        } else {
            None
        };

        let span = start.merge(self.current_span());
        Ok(GlobalDecl {
            name: Identifier { name, span: n_span },
            ty,
            initializer,
            span,
        })
    }

    pub(crate) fn parse_struct(&mut self) -> crate::Result<StructDefinition> {
        let start = self.current_span();
        self.expect(Token::Struct)?;

        let (_n, n_span) = self.expect(Token::Identifier)?;
        let name = self.get_text(&n_span).to_string();

        self.expect(Token::LBrace)?;
        let mut fields = Vec::new();

        while self.current_token() != Some(&Token::RBrace) {
            let (_, f_span) = self.expect(Token::Identifier)?;
            let field_name = self.get_text(&f_span).to_string();

            let has_of = self.current_token() == Some(&Token::Of);
            if has_of {
                self.expect(Token::Of)?;
            }

            let ty = if has_of || self.is_type_start() {
                self.parse_type()?
            } else {
                TypeRef::BuiltinType(BuiltinType::Int)
            };

            if self.current_token() == Some(&Token::Semi) {
                self.advance();
            }

            fields.push(StructField {
                name: Identifier {
                    name: field_name,
                    span: f_span,
                },
                ty,
                span: f_span,
            });
        }

        self.expect(Token::RBrace)?;

        let span = start.merge(self.current_span());
        Ok(StructDefinition {
            name: Identifier { name, span: n_span },
            fields,
            span,
        })
    }

    pub(crate) fn parse_coroutine(&mut self) -> crate::Result<CoroutineDefinition> {
        let start = self.current_span();
        self.expect(Token::Coroutine)?;
        let sig = self.parse_signature()?;

        let mut body = Vec::new();
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
            if self.current_token() == Some(&Token::End) {
                break;
            }
        }
        if self.current_token() == Some(&Token::End) {
            self.expect(Token::End)?;
        }

        let span = start.merge(self.current_span());
        Ok(CoroutineDefinition {
            signature: sig,
            body,
            span,
        })
    }
}


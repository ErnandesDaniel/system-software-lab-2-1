use super::Parser;
use crate::ast::*;
use crate::lexer::Token;

impl<'source> Parser<'source> {
    pub(crate) fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token() {
            Some(Token::Return) => self.parse_return(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While) | Some(Token::Until) => self.parse_while(),
            Some(Token::Break) => self.parse_break(),
            Some(Token::Begin) | Some(Token::LBrace) => self.parse_block_like(),
            Some(Token::Do) => self.parse_repeat(),
            Some(Token::Identifier) => self.parse_identifier_based_statement(),
            _ => self.parse_expression_statement(),
        }
    }

    pub(crate) fn parse_identifier_based_statement(&mut self) -> Result<Statement, String> {
        let (_name, span) = self.expect(Token::Identifier)?;
        let var_name = self.get_text(&span).to_string();

        // Check if this is a slice assignment: identifier[expr] = expr
        if self.current_token() == Some(&Token::LBracket) {
            self.expect(Token::LBracket)?;
            let index = self.parse_expression(0)?;
            self.expect(Token::RBracket)?;

            // Create slice expression: arr[0]
            let slice = Expr::Slice(SliceExpr {
                array: Box::new(Expr::Identifier(Identifier {
                    name: var_name.clone(),
                    span,
                })),
                ranges: vec![Range {
                    start: index,
                    end: None,
                    span: Span::new(0, 0),
                }],
                span: span.merge(self.current_span()),
            });

            // Now check for assignment
            if self.current_token() == Some(&Token::Assign) {
                self.expect(Token::Assign)?;
                let right = self.parse_expression(0)?;
                if self.current_token() == Some(&Token::Semi) {
                    self.expect(Token::Semi)?;
                }
                let end_span = self.current_span();
                return Ok(Statement::Expression(ExpressionStatement {
                    expr: Expr::Binary(BinaryExpr {
                        left: Box::new(slice),
                        operator: BinaryOp::Assign,
                        right: Box::new(right),
                        span: span.merge(end_span),
                    }),
                    span: span.merge(end_span),
                }));
            }
        }

        // Regular identifier handling
        if self.current_token() == Some(&Token::Assign) {
            self.expect(Token::Assign)?;
            let expr = self.parse_expression(0)?;
            if self.current_token() == Some(&Token::Semi) {
                self.expect(Token::Semi)?;
            }
            let end_span = self.current_span();
            Ok(Statement::Expression(ExpressionStatement {
                expr: Expr::Binary(BinaryExpr {
                    left: Box::new(Expr::Identifier(Identifier {
                        name: var_name.clone(),
                        span,
                    })),
                    operator: BinaryOp::Assign,
                    right: Box::new(expr),
                    span: span.merge(end_span),
                }),
                span: span.merge(end_span),
            }))
        } else {
            let mut left = Expr::Identifier(Identifier {
                name: var_name,
                span,
            });

            while let Some(token) = self.current_token() {
                if matches!(token, Token::Assign) {
                    break;
                }
                if matches!(
                    token,
                    Token::End
                        | Token::Semi
                        | Token::Then
                        | Token::Else
                        | Token::Do
                        | Token::While
                        | Token::Until
                        | Token::Comma
                        | Token::RParen
                        | Token::RBracket
                ) {
                    break;
                }
                let token_copy = *token;
                let prec = Self::get_precedence(&token_copy);
                if prec == 0 {
                    break;
                }
                self.advance();
                left = self.parse_infix(left, token_copy, prec)?;
            }

            if self.current_token() == Some(&Token::Semi) {
                self.expect(Token::Semi)?;
            }

            let end_span = self.current_span();
            Ok(Statement::Expression(ExpressionStatement {
                expr: left,
                span: span.merge(end_span),
            }))
        }
    }

    pub(crate) fn parse_return(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        self.expect(Token::Return)?;
        let expr = if self.current_token().is_some()
            && self.current_token() != Some(&Token::Semi)
            && self.current_token() != Some(&Token::End)
        {
            Some(self.parse_expression(0)?)
        } else {
            None
        };
        if self.current_token() == Some(&Token::Semi) {
            self.expect(Token::Semi)?;
        }
        let span = start.merge(self.current_span());
        Ok(Statement::Return(ReturnStatement { expr, span }))
    }

    pub(crate) fn parse_if(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        self.expect(Token::If)?;
        let condition = self.parse_expression(0)?;
        self.expect(Token::Then)?;
        let consequence = Box::new(self.parse_statement()?);
        let alternative = if self.current_token() == Some(&Token::Else) {
            self.expect(Token::Else)?;
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        let span = start.merge(self.current_span());
        Ok(Statement::If(IfStatement {
            condition,
            consequence,
            alternative,
            span,
        }))
    }

    pub(crate) fn parse_while(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        let keyword = match self.current_token() {
            Some(Token::While) => {
                self.advance();
                LoopKeyword::While
            }
            Some(Token::Until) => {
                self.advance();
                LoopKeyword::Until
            }
            _ => return Err("Expected 'while' or 'until'".to_string()),
        };

        let condition = self.parse_condition_expression()?;

        let mut body = Vec::new();
        while let Some(tok) = self.current_token() {
            if tok == &Token::LoopEnd {
                self.expect(Token::LoopEnd)?;
                break;
            }
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
        }

        let span = start.merge(self.current_span());
        Ok(Statement::Loop(LoopStatement {
            keyword,
            condition,
            body,
            span,
        }))
    }

    pub(crate) fn parse_condition_expression(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_prefix()?;

        while let Some(token) = self.current_token() {
            if matches!(
                token,
                Token::End
                    | Token::Semi
                    | Token::Then
                    | Token::Else
                    | Token::Do
                    | Token::While
                    | Token::Until
                    | Token::Comma
                    | Token::RParen
                    | Token::RBracket
                    | Token::LBrace
            ) {
                break;
            }
            if matches!(token, Token::Assign) {
                break;
            }
            let token_copy = *token;
            let prec = Self::get_precedence(&token_copy);
            if prec == 0 {
                break;
            }
            self.advance();
            left = self.parse_infix(left, token_copy, prec)?;
        }

        Ok(left)
    }

    pub(crate) fn parse_repeat(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        self.expect(Token::Do)?;
        let body = Box::new(self.parse_statement()?);
        self.expect(Token::While)?;
        let condition = self.parse_expression(0)?;
        self.expect(Token::Semi)?;
        let span = start.merge(self.current_span());
        Ok(Statement::Repeat(RepeatStatement {
            body,
            keyword: LoopKeyword::Until,
            condition,
            span,
        }))
    }

    pub(crate) fn parse_break(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        self.expect(Token::Break)?;
        self.expect(Token::Semi)?;
        let span = start.merge(self.current_span());
        Ok(Statement::Break(BreakStatement { span }))
    }

    pub(crate) fn parse_block_like(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        let end_token = match self.current_token() {
            Some(Token::Begin) => {
                self.expect(Token::Begin)?;
                Token::End
            }
            Some(Token::LBrace) => {
                self.expect(Token::LBrace)?;
                Token::RBrace
            }
            _ => return Err("Expected 'begin' or '{'".to_string()),
        };
        let mut body = Vec::new();
        loop {
            if let Some(tok) = self.current_token() {
                if std::mem::discriminant(tok) == std::mem::discriminant(&end_token) {
                    break;
                }
            } else {
                break;
            }

            match self.parse_statement() {
                Ok(stmt) => {
                    body.push(stmt);
                    while self.current_token() == Some(&Token::Semi) {
                        self.advance();
                    }
                }
                Err(e) => {
                    if let Some(tok) = self.current_token() {
                        if std::mem::discriminant(tok) == std::mem::discriminant(&end_token) {
                            break;
                        }
                    }
                    return Err(e);
                }
            }

            if body.len() > 100 {
                break;
            }
        }
        self.expect(end_token)?;
        let span = start.merge(self.current_span());
        Ok(Statement::Block(BlockStatement { body, span }))
    }

    pub(crate) fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        let expr = self.parse_expression(0)?;
        if self.current_token() == Some(&Token::Semi) {
            self.expect(Token::Semi)?;
        }
        let span = start.merge(self.current_span());
        Ok(Statement::Expression(ExpressionStatement { expr, span }))
    }
}

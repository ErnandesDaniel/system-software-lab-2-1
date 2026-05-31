use super::Parser;
use crate::ast::{
    BinaryExpr, BinaryOp, BlockStatement, Expr, ExpressionStatement, Identifier, LoopKeyword, Range, RepeatStatement,
    SliceExpr, Span, Statement, VarDeclStatement,
};
use crate::lexer::Token;

impl Parser<'_> {
    pub(crate) fn parse_identifier_based_statement(&mut self) -> Result<Statement, String> {
        let (_name, span) = self.expect(Token::Identifier)?;
        let var_name = self.get_text(&span).to_string();

        if self.current_token() == Some(&Token::Of) {
            self.expect(Token::Of)?;
            let ty = self.parse_type()?;
            if self.current_token() == Some(&Token::Semi) {
                self.advance();
            }
            let end_span = self.current_span();
            return Ok(Statement::VarDecl(VarDeclStatement {
                name: Identifier { name: var_name, span },
                ty,
                span: span.merge(end_span),
            }));
        }

        // Check if this is an indexed expression: identifier[expr]
        if self.current_token() == Some(&Token::LBracket) {
            self.expect(Token::LBracket)?;
            let index = self.parse_expression(0)?;
            self.expect(Token::RBracket)?;

            // Create slice expression: arr[0]
            let mut left = Expr::Slice(SliceExpr {
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

            // Handle trailing .field and [sub] accesses (e.g., arr[i].field, arr[i][j])
            loop {
                match self.current_token() {
                    Some(Token::Dot) => {
                        self.advance();
                        let (_tok, f_span) = self.expect(Token::Identifier)?;
                        let field = Identifier {
                            name: self.get_text(&f_span).to_string(),
                            span: f_span,
                        };
                        left = Expr::FieldAccess(Box::new(left), field);
                    }
                    Some(Token::LBracket) => {
                        self.advance();
                        let sub_idx = self.parse_expression(0)?;
                        self.expect(Token::RBracket)?;
                        left = Expr::Slice(SliceExpr {
                            array: Box::new(left),
                            ranges: vec![Range {
                                start: sub_idx,
                                end: None,
                                span: Span::new(0, 0),
                            }],
                            span: span.merge(self.current_span()),
                        });
                    }
                    _ => break,
                }
            }

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
                        left: Box::new(left),
                        operator: BinaryOp::Assign,
                        right: Box::new(right),
                        span: span.merge(end_span),
                    }),
                    span: span.merge(end_span),
                }));
            }

            // No assignment: use Pratt loop for remaining infix operators
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
            return Ok(Statement::Expression(ExpressionStatement {
                expr: left,
                span: span.merge(end_span),
            }));
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
            let mut left = Expr::Identifier(Identifier { name: var_name, span });

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
        while let Some(tok) = self.current_token() {
            if *tok == end_token {
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
                    return Err(e);
                }
            }
        }
        self.expect(end_token)?;
        let span = start.merge(self.current_span());
        Ok(Statement::Block(BlockStatement { body, span }))
    }

    fn parse_repeat_tail(&mut self, body: Statement, body_span: Span) -> Result<Statement, String> {
        let keyword = match self.current_token() {
            Some(Token::While) => LoopKeyword::While,
            Some(Token::Until) => LoopKeyword::Until,
            _ => unreachable!(),
        };
        self.advance();
        let condition = self.parse_expression(0)?;
        self.expect(Token::Semi)?;
        let span = body_span.merge(self.current_span());
        Ok(Statement::Repeat(RepeatStatement {
            body: Box::new(body),
            keyword,
            condition,
            span,
        }))
    }

    pub(crate) fn parse_expression_statement(&mut self) -> Result<Statement, String> {
        let start = self.current_span();
        let expr = self.parse_expression(0)?;

        if matches!(self.current_token(), Some(Token::While | Token::Until)) {
            let body = Statement::Expression(ExpressionStatement {
                expr,
                span: start.merge(self.current_span()),
            });
            return self.parse_repeat_tail(body, start);
        }

        if self.current_token() == Some(&Token::Semi) {
            self.expect(Token::Semi)?;
        }
        let span = start.merge(self.current_span());
        Ok(Statement::Expression(ExpressionStatement { expr, span }))
    }
}

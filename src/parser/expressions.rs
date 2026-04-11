use super::Parser;
use crate::ast::*;
use crate::lexer::Token;

impl<'source> Parser<'source> {
    pub(crate) fn parse_expression(&mut self, min_prec: u8) -> Result<Expr, String> {
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
                    | Token::RParen
                    | Token::RBracket
                    | Token::CreateThread
                    | Token::Comma
            ) {
                break;
            }
            // Don't break on LParen when the left side is an identifier (function call)
            if matches!(token, Token::LParen) {
                if let Expr::Identifier(_) = left {
                    // Continue to parse function call
                } else {
                    break;
                }
            }
            if matches!(token, Token::Semi) {
                break;
            }
            if matches!(token, Token::Assign) {
                break;
            }
            let token_copy = *token;
            let prec = Self::get_precedence(&token_copy);
            if prec < min_prec || matches!(token_copy, Token::CreateThread) {
                break;
            }
            self.advance();
            left = self.parse_infix(left, token_copy, prec)?;
        }

        Ok(left)
    }

    pub(crate) fn parse_prefix(&mut self) -> Result<Expr, String> {
        match self.current_token() {
            Some(Token::CreateThread) => {
                let start = self.current_span();
                self.advance();
                self.expect(Token::LParen)?;
                let (_tok, span) = self.expect(Token::Identifier)?;
                let func_name = Identifier {
                    name: self.get_text(&span).to_string(),
                    span,
                };
                let mut scheduler = None;
                if self.current_token() == Some(&Token::Comma) {
                    self.advance();
                    // Parse string literal for scheduler: "FCFS" or "SPN"
                    let sched_name = match self.current_token() {
                        Some(Token::StringLiteral) => {
                            let (_tok, span) = self.expect(Token::StringLiteral)?;
                            let s = self.get_text(&span);
                            let unquoted = &s[1..s.len() - 1];
                            unquoted.to_string()
                        }
                        Some(Token::Fcfs) => {
                            self.advance();
                            "FCFS".to_string()
                        }
                        Some(Token::Spn) => {
                            self.advance();
                            "SPN".to_string()
                        }
                        Some(Token::Identifier) => {
                            let (_tok, span) = self.expect(Token::Identifier)?;
                            let name = self.get_text(&span).to_string();
                            if name == "FCFS" || name == "SPN" {
                                name
                            } else {
                                return Err("Expected FCFS or SPN".to_string());
                            }
                        }
                        _ => {
                            return Err(
                                "Expected scheduler (FCFS, SPN, or \"FCFS\", \"SPN\")".to_string()
                            );
                        }
                    };
                    scheduler = Some(Identifier {
                        name: sched_name,
                        span: Span::new(0, 0),
                    });
                }
                self.expect(Token::RParen)?;
                let span = start.merge(self.current_span());
                Ok(Expr::CreateThread(CreateThreadExpr {
                    function_name: func_name,
                    scheduler,
                    span,
                }))
            }
            Some(Token::DecLiteral) => {
                let (_tok, span) = self.expect(Token::DecLiteral)?;
                let value = self.get_text(&span).parse::<u64>().unwrap_or(0);
                Ok(Expr::Literal(Literal::Dec(value)))
            }
            Some(Token::HexLiteral) => {
                let (_tok, span) = self.expect(Token::HexLiteral)?;
                let text = self.get_text(&span);
                let value = u64::from_str_radix(&text[2..], 16).unwrap_or(0);
                Ok(Expr::Literal(Literal::Hex(value)))
            }
            Some(Token::BitsLiteral) => {
                let (_tok, span) = self.expect(Token::BitsLiteral)?;
                let text = self.get_text(&span);
                let value = u64::from_str_radix(&text[2..], 2).unwrap_or(0);
                Ok(Expr::Literal(Literal::Bits(value)))
            }
            Some(Token::StringLiteral) => {
                let (_tok, span) = self.expect(Token::StringLiteral)?;
                let s = self.get_text(&span);
                let unquoted = &s[1..s.len() - 1];
                let processed = unquoted
                    .replace("\\n", "\n")
                    .replace("\\r", "\r")
                    .replace("\\t", "\t")
                    .replace("\\\\", "\\")
                    .replace("\\\"", "\"");
                Ok(Expr::Literal(Literal::Str(processed)))
            }
            Some(Token::CharLiteral) => {
                let (_tok, span) = self.expect(Token::CharLiteral)?;
                let s = self.get_text(&span);
                let ch = s.chars().nth(1).unwrap_or('\0');
                Ok(Expr::Literal(Literal::Char(ch)))
            }
            Some(Token::True) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(true)))
            }
            Some(Token::False) => {
                self.advance();
                Ok(Expr::Literal(Literal::Bool(false)))
            }
            Some(Token::Identifier) => {
                let (_tok, span) = self.expect(Token::Identifier)?;
                Ok(Expr::Identifier(Identifier {
                    name: self.get_text(&span).to_string(),
                    span,
                }))
            }
            Some(Token::LParen) => {
                self.advance();
                let expr = self.parse_expression(0)?;
                self.expect(Token::RParen)?;
                Ok(Expr::Parenthesized(Box::new(expr)))
            }
            Some(Token::Minus) => {
                let start = self.current_span();
                self.advance();
                let operand = Box::new(self.parse_expression(90)?);
                let span = start.merge(self.current_span());
                Ok(Expr::Unary(UnaryExpr {
                    operator: UnaryOp::Negate,
                    operand,
                    span,
                }))
            }
            Some(Token::Bang) => {
                let start = self.current_span();
                self.advance();
                let operand = Box::new(self.parse_expression(90)?);
                let span = start.merge(self.current_span());
                Ok(Expr::Unary(UnaryExpr {
                    operator: UnaryOp::Not,
                    operand,
                    span,
                }))
            }
            Some(Token::Tilde) => {
                let start = self.current_span();
                self.advance();
                let operand = Box::new(self.parse_expression(90)?);
                let span = start.merge(self.current_span());
                Ok(Expr::Unary(UnaryExpr {
                    operator: UnaryOp::BitNot,
                    operand,
                    span,
                }))
            }
            _ => Err("Expected expression".to_string()),
        }
    }

    pub(crate) fn parse_infix(
        &mut self,
        left: Expr,
        token: Token,
        prec: u8,
    ) -> Result<Expr, String> {
        let start = left.span();

        match token {
            Token::Plus
            | Token::Minus
            | Token::Star
            | Token::Slash
            | Token::Percent
            | Token::Lt
            | Token::Gt
            | Token::Le
            | Token::Ge
            | Token::Eq
            | Token::Ne
            | Token::And
            | Token::Or => {
                let right = self.parse_expression(prec + 1)?;
                let end = right.span();
                let op = match token {
                    Token::Plus => BinaryOp::Add,
                    Token::Minus => BinaryOp::Subtract,
                    Token::Star => BinaryOp::Multiply,
                    Token::Slash => BinaryOp::Divide,
                    Token::Percent => BinaryOp::Modulo,
                    Token::Lt => BinaryOp::Less,
                    Token::Gt => BinaryOp::Greater,
                    Token::Le => BinaryOp::LessOrEqual,
                    Token::Ge => BinaryOp::GreaterOrEqual,
                    Token::Eq => BinaryOp::Equal,
                    Token::Ne => BinaryOp::NotEqual,
                    Token::And => BinaryOp::And,
                    Token::Or => BinaryOp::Or,
                    _ => return Err("Unknown operator".to_string()),
                };
                Ok(Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                    span: start.merge(end),
                }))
            }
            Token::LBracket => {
                let _index = self.parse_expression(0)?;
                self.expect(Token::RBracket)?;
                let span = start.merge(self.current_span());
                Ok(Expr::Slice(SliceExpr {
                    array: Box::new(left),
                    ranges: vec![],
                    span,
                }))
            }
            Token::LParen => {
                let mut args = Vec::new();
                while self.current_token() != Some(&Token::RParen) && self.current_token().is_some()
                {
                    if !args.is_empty() {
                        self.expect(Token::Comma)?;
                    }
                    args.push(self.parse_expression(0)?);
                }
                self.expect(Token::RParen)?;
                let span = start.merge(self.current_span());
                let func = Box::new(left);
                Ok(Expr::Call(CallExpr {
                    function: func,
                    arguments: args,
                    span,
                }))
            }
            Token::Assign => {
                let right = self.parse_expression(0)?;
                let span = start.merge(self.current_span());
                Ok(Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    operator: BinaryOp::Assign,
                    right: Box::new(right),
                    span,
                }))
            }
            Token::CreateThread => {
                let start = self.current_span();
                self.expect(Token::LParen)?;
                let (_tok, span) = self.expect(Token::Identifier)?;
                let func_name = Identifier {
                    name: self.get_text(&span).to_string(),
                    span,
                };
                let mut scheduler = None;
                if self.current_token() == Some(&Token::Comma) {
                    self.advance();
                    let sched_name = match self.current_token() {
                        Some(Token::StringLiteral) => {
                            let (_tok, span) = self.expect(Token::StringLiteral)?;
                            let s = self.get_text(&span);
                            let unquoted = &s[1..s.len() - 1];
                            unquoted.to_string()
                        }
                        Some(Token::Fcfs) => {
                            self.advance();
                            "FCFS".to_string()
                        }
                        Some(Token::Spn) => {
                            self.advance();
                            "SPN".to_string()
                        }
                        Some(Token::Identifier) => {
                            let (_tok, span) = self.expect(Token::Identifier)?;
                            let name = self.get_text(&span).to_string();
                            if name == "FCFS" || name == "SPN" {
                                name
                            } else {
                                return Err("Expected FCFS or SPN".to_string());
                            }
                        }
                        _ => {
                            return Err("Expected scheduler".to_string());
                        }
                    };
                    scheduler = Some(Identifier {
                        name: sched_name,
                        span: Span::new(0, 0),
                    });
                }
                self.expect(Token::RParen)?;
                let span = start.merge(self.current_span());
                return Ok(Expr::CreateThread(CreateThreadExpr {
                    function_name: func_name,
                    scheduler,
                    span,
                }));
            }
            _ => Ok(left),
        }
    }

    pub(crate) fn get_precedence(token: &Token) -> u8 {
        match token {
            Token::Or => 10,
            Token::And => 20,
            Token::Eq | Token::Ne => 30,
            Token::Lt | Token::Gt | Token::Le | Token::Ge => 40,
            Token::Plus | Token::Minus => 50,
            Token::Star | Token::Slash | Token::Percent => 60,
            Token::Assign => 5,
            Token::LParen => 70,
            Token::CreateThread => 80,
            _ => 0,
        }
    }
}

impl Expr {
    pub fn span(&self) -> Span {
        match self {
            Expr::Binary(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::Parenthesized(e) => e.span(),
            Expr::Call(e) => e.span,
            Expr::CreateThread(e) => e.span,
            Expr::Slice(e) => e.span,
            Expr::Identifier(e) => e.span,
            Expr::Literal(_) => Span::new(0, 0),
        }
    }
}

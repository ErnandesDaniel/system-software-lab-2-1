use crate::ast::{BinaryExpr, BinaryOp, CallExpr, Expr, Identifier, Range, SliceExpr};
use crate::error::CompilerError;
use crate::lexer::Token;
use crate::parser::Parser;

impl Parser<'_> {
    pub(crate) fn parse_infix(&mut self, left: Expr, token: Token, prec: u8) -> crate::Result<Expr> {
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
            | Token::Or
            | Token::BitAnd
            | Token::BitOr
            | Token::BitXor => {
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
                    Token::BitAnd => BinaryOp::BitAnd,
                    Token::BitOr => BinaryOp::BitOr,
                    Token::BitXor => BinaryOp::BitXor,
                    _ => return Err(CompilerError::Parse("Unknown operator".to_string())),
                };
                Ok(Expr::Binary(BinaryExpr {
                    left: Box::new(left),
                    operator: op,
                    right: Box::new(right),
                    span: start.merge(end),
                }))
            }
            Token::LBracket => {
                let index = self.parse_expression(0)?;
                let idx_span = index.span();
                let end_expr = if self.current_token() == Some(&Token::Range) {
                    self.advance();
                    Some(self.parse_expression(0)?)
                } else {
                    None
                };
                self.expect(Token::RBracket)?;
                let span = start.merge(self.current_span());
                Ok(Expr::Slice(SliceExpr {
                    array: Box::new(left),
                    ranges: vec![Range {
                        start: index,
                        end: end_expr,
                        span: idx_span,
                    }],
                    span,
                }))
            }
            Token::LParen => {
                let mut args = Vec::new();
                while self.current_token() != Some(&Token::RParen) && self.current_token().is_some() {
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
            Token::Dot => {
                let (_tok, f_span) = self.expect(Token::Identifier)?;
                let field = Identifier {
                    name: self.get_text(&f_span).to_string(),
                    span: f_span,
                };
                let _span = start.merge(self.current_span());
                Ok(Expr::FieldAccess(Box::new(left), field))
            }
            _ => Ok(left),
        }
    }
}

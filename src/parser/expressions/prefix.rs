use crate::parser::Parser;
use crate::ast::{Expr, Identifier, Literal, UnaryExpr, UnaryOp};
use crate::lexer::Token;

impl Parser<'_> {
    pub(crate) fn parse_prefix(&mut self) -> Result<Expr, String> {
        match self.current_token() {
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
            Some(Token::LBracket) => {
                self.advance();
                let mut elements = Vec::new();
                if self.current_token() != Some(&Token::RBracket) {
                    elements.push(self.parse_expression(0)?);
                    while self.current_token() == Some(&Token::Comma) {
                        self.advance();
                        elements.push(self.parse_expression(0)?);
                    }
                }
                self.expect(Token::RBracket)?;
                Ok(Expr::ArrayLiteral(elements))
            }
            Some(Token::Def) => {
                let func_def = self.parse_function()?;
                Ok(Expr::FuncLiteral(func_def))
            }
            _ => Err("Expected expression".to_string()),
        }
    }
}

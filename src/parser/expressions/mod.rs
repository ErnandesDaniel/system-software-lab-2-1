mod infix;
mod prefix;

use super::Parser;
use crate::ast::{Expr, Span};
use crate::lexer::Token;

impl Parser<'_> {
    pub(crate) fn parse_expression(&mut self, min_prec: u8) -> crate::Result<Expr> {
        let mut left = self.parse_prefix()?;

        while let Some(token) = self.current_token() {
            if matches!(
                token,
                Token::Semi
                    | Token::Else
                    | Token::While
                    | Token::Until
                    | Token::RParen
                    | Token::RBracket
                    | Token::RBrace
                    | Token::Comma
            ) {
                break;
            }
            if matches!(token, Token::LParen) {
            }
            if matches!(token, Token::LBracket) {
            }
            if matches!(
                token,
                Token::Semi
                    | Token::Identifier
                    | Token::Return
                    | Token::If
                    | Token::While
                    | Token::Until
                    | Token::Break
                    | Token::Def
                    | Token::Import
                    | Token::Global
                    | Token::Struct
                    | Token::Coroutine
                    | Token::Yield
                    | Token::LBrace
            ) {
                break;
            }
            let token_copy = *token;
            let prec = Self::get_precedence(&token_copy);
            if prec < min_prec {
                break;
            }
            self.advance();
            left = self.parse_infix(left, token_copy, prec)?;
        }

        Ok(left)
    }

    pub(crate) fn get_precedence(token: &Token) -> u8 {
        match token {
            Token::Or => 10,
            Token::And => 20,
            Token::BitOr => 22,
            Token::BitXor => 24,
            Token::BitAnd => 26,
            Token::Eq | Token::Ne => 30,
            Token::Lt | Token::Gt | Token::Le | Token::Ge => 40,
            Token::Plus | Token::Minus => 50,
            Token::Star | Token::Slash | Token::Percent => 60,
            Token::Assign => 5,
            Token::LParen | Token::LBracket => 70,
            Token::Dot => 80,
            _ => 0,
        }
    }
}

impl Expr {
    #[must_use]
    pub fn span(&self) -> Span {
        match self {
            Expr::Binary(e) => e.span,
            Expr::Unary(e) => e.span,
            Expr::Parenthesized(e) | Expr::FieldAccess(e, _) => e.span(),
            Expr::Call(c) => c.span,
            Expr::Slice(e) => e.span,
            Expr::Identifier(id) => id.span,
            Expr::FuncLiteral(f) => f.span,
            Expr::Literal(_, s) | Expr::ArrayLiteral(_, s) => *s,
        }
    }
}

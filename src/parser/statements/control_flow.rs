use crate::ast::*;
use crate::lexer::Token;
use super::Parser;

impl<'source> Parser<'source> {
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
}

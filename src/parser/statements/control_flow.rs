use super::Parser;
use crate::ast::{
    BreakStatement, ElseIfBranch, IfStatement, LoopKeyword, LoopStatement,
    ReturnStatement, Statement,
};
use crate::error::CompilerError;
use crate::lexer::Token;

impl Parser<'_> {
    pub(crate) fn parse_return(&mut self) -> crate::Result<Statement> {
        let start = self.current_span();
        self.expect(Token::Return)?;
        let expr = if self.current_token().is_some()
            && self.current_token() != Some(&Token::Semi)
            && self.current_token() != Some(&Token::RBrace)
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

    pub(crate) fn parse_if(&mut self) -> crate::Result<Statement> {
        let start = self.current_span();
        self.expect(Token::If)?;
        self.expect(Token::LParen)?;
        let condition = self.parse_expression(0)?;
        self.expect(Token::RParen)?;

        // if body: { ... }
        self.expect(Token::LBrace)?;
        let mut body = Vec::new();
        while self.current_token() != Some(&Token::RBrace) && self.current_token().is_some() {
            body.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;

        // else if chain and optional else
        let mut else_ifs = Vec::new();
        let mut else_body = None;

        while self.current_token() == Some(&Token::Else) {
            self.advance();
            if self.current_token() == Some(&Token::If) {
                // else if (...)
                let ei_start = self.current_span();
                self.expect(Token::If)?;
                self.expect(Token::LParen)?;
                let ei_cond = self.parse_expression(0)?;
                self.expect(Token::RParen)?;
                self.expect(Token::LBrace)?;
                let mut ei_body = Vec::new();
                while self.current_token() != Some(&Token::RBrace) && self.current_token().is_some() {
                    ei_body.push(self.parse_statement()?);
                }
                self.expect(Token::RBrace)?;
                let ei_span = ei_start.merge(self.current_span());
                else_ifs.push(ElseIfBranch {
                    condition: ei_cond,
                    body: ei_body,
                    span: ei_span,
                });
            } else {
                // else { ... }
                self.expect(Token::LBrace)?;
                let mut eb = Vec::new();
                while self.current_token() != Some(&Token::RBrace) && self.current_token().is_some() {
                    eb.push(self.parse_statement()?);
                }
                self.expect(Token::RBrace)?;
                else_body = Some(eb);
            }
        }

        let span = start.merge(self.current_span());
        Ok(Statement::If(IfStatement {
            condition,
            body,
            else_ifs,
            else_body,
            span,
        }))
    }

    pub(crate) fn parse_while(&mut self) -> crate::Result<Statement> {
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
            _ => return Err(CompilerError::Parse("Expected 'while' or 'until'".to_string())),
        };

        self.expect(Token::LParen)?;
        let condition = self.parse_expression(0)?;
        self.expect(Token::RParen)?;

        self.expect(Token::LBrace)?;
        let mut body = Vec::new();
        while self.current_token() != Some(&Token::RBrace) && self.current_token().is_some() {
            body.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;

        let span = start.merge(self.current_span());
        Ok(Statement::Loop(LoopStatement {
            keyword,
            condition,
            body,
            span,
        }))
    }

    pub(crate) fn parse_break(&mut self) -> crate::Result<Statement> {
        let start = self.current_span();
        self.expect(Token::Break)?;
        if self.current_token() == Some(&Token::Semi) {
            self.advance();
        }
        let span = start.merge(self.current_span());
        Ok(Statement::Break(BreakStatement { span }))
    }
}

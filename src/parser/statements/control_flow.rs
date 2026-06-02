use super::Parser;
use crate::ast::{
    BreakStatement, IfStatement, LoopKeyword, LoopStatement, RepeatStatement, ReturnStatement, Statement,
};
use crate::error::CompilerError;
use crate::lexer::Token;

impl Parser<'_> {
    pub(crate) fn parse_return(&mut self) -> crate::Result<Statement> {
        let start = self.current_span();
        self.expect(Token::Return)?;
        let expr = if self.current_token().is_some()
            && self.current_token() != Some(&Token::Semi)
            && self.current_token() != Some(&Token::End)
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
        let condition = self.parse_expression(0)?;
        self.expect(Token::Then)?;
        let is_block_consequence = matches!(self.current_token(), Some(Token::LBrace | Token::Begin));
        let consequence = Box::new(self.parse_statement()?);
        let alternative = if self.current_token() == Some(&Token::Else) {
            self.expect(Token::Else)?;
            Some(Box::new(self.parse_statement()?))
        } else {
            None
        };
        if !is_block_consequence && self.current_token() == Some(&Token::End) {
            self.expect(Token::End)?;
        }
        let span = start.merge(self.current_span());
        Ok(Statement::If(IfStatement {
            condition,
            consequence,
            alternative,
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

        let condition = self.parse_expression(0)?;

        while self.current_token() == Some(&Token::Semi) {
            self.advance();
        }

        let body_stmt = self.parse_statement()?;

        while self.current_token() == Some(&Token::Semi) {
            self.advance();
        }

        if self.current_token() == Some(&Token::LoopEnd) {
            self.expect(Token::LoopEnd)?;
        }

        let body = if let Statement::Block(block) = body_stmt {
            block.body
        } else {
            vec![body_stmt]
        };

        let span = start.merge(self.current_span());
        Ok(Statement::Loop(LoopStatement {
            keyword,
            condition,
            body,
            span,
        }))
    }

    pub(crate) fn parse_repeat(&mut self) -> crate::Result<Statement> {
        let start = self.current_span();
        // accept "repeat" or "do"
        match self.current_token() {
            Some(Token::Repeat) => { self.advance(); }
            Some(Token::Do) => { self.advance(); }
            _ => return Err(CompilerError::Parse("Expected 'repeat' or 'do'".to_string())),
        }
        let body = Box::new(self.parse_statement()?);
        // accept "until" or "while"
        let keyword = match self.current_token() {
            Some(Token::Until) => { self.advance(); LoopKeyword::Until }
            Some(Token::While) => { self.advance(); LoopKeyword::While }
            _ => return Err(CompilerError::Parse("Expected 'until' or 'while'".to_string())),
        };
        let condition = self.parse_expression(0)?;
        if self.current_token() == Some(&Token::Semi) {
            self.advance();
        }
        let span = start.merge(self.current_span());
        Ok(Statement::Repeat(RepeatStatement {
            body,
            keyword,
            condition,
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

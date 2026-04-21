mod control_flow;
mod assignments;

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
}

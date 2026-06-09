mod assignments;
mod control_flow;

use super::Parser;
use crate::ast::Statement;
use crate::lexer::Token;

impl Parser<'_> {
    pub(crate) fn parse_statement(&mut self) -> crate::Result<Statement> {
        match self.current_token() {
            Some(Token::Return) => self.parse_return(),
            Some(Token::If) => self.parse_if(),
            Some(Token::While | Token::Until) => self.parse_while(),
            Some(Token::Break) => self.parse_break(),
            Some(Token::LBrace) => self.parse_block_like(),
            Some(Token::Def) => {
                let func_def = self.parse_function()?;
                if self.current_token() == Some(&Token::Semi) {
                    self.advance();
                }
                Ok(Statement::FuncDef(func_def))
            }
            Some(Token::Identifier) => self.parse_identifier_based_statement(),
            _ => self.parse_expression_statement(),
        }
    }
}

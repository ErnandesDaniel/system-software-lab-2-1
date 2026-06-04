mod codegen_tests;
mod cross_target_tests;
mod integration_tests;
mod jvm_tests;
mod labs_tests;
mod lexer_tests;
mod parser_tests;
mod semantics_tests;

use crate::parser::Parser;

pub fn parse(source: &str) -> crate::ast::Program {
    let mut parser = Parser::new(source);
    parser.parse().unwrap()
}

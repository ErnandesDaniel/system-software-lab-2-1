use super::helpers::{has_error, tokens};
use crate::lexer::iter::Lexer;
use crate::lexer::Token;

#[test]
fn test_error_unexpected_char() {
    assert!(has_error("@"));
}

#[test]
fn test_error_unexpected_hash() {
    assert!(has_error("#"));
}

#[test]
fn test_error_unexpected_backtick() {
    assert!(has_error("`"));
}

#[test]
fn test_error_unexpected_dollar() {
    assert!(has_error("$"));
}

#[test]
fn test_unknown_char_in_expression() {
    let lex = Lexer::new("x @ y");
    let errors = lex.has_errors();
    assert!(errors);
    let t: Vec<_> = lex.filter_map(|r| r.ok().map(|(t, _)| t)).collect();
    assert_eq!(t.len(), 2);
}

#[test]
fn test_no_error_on_valid_code() {
    assert!(!has_error("def main() { return 0; }"));
}

#[test]
fn test_no_error_on_complex_program() {
    assert!(!has_error(
        "         struct Point { x of int; y of int; }
         def add(a of int, b of int) of int { return a + b; }
         global counter of int = 0;
         def main() of int { return 0; }"
    ));
}

#[test]
fn test_string_with_unicode_in_comment() {
    assert!(!has_error("// привет мир\ndef foo() {}"));
}

#[test]
fn test_empty_input() {
    assert_eq!(tokens(""), vec![]);
}

#[test]
fn test_only_whitespace() {
    assert_eq!(tokens("   \t\n  "), vec![]);
}

#[test]
fn test_only_comment() {
    assert_eq!(tokens("// just a comment"), vec![]);
}

#[test]
fn test_only_block_comment() {
    assert_eq!(tokens("/* block */"), vec![]);
}

#[test]
fn test_double_slash_expr() {
    let t = tokens("x // y");
    assert_eq!(t.len(), 1);
    assert_eq!(t[0], Token::Identifier);
}

#[test]
fn test_colon_is_not_token() {
    assert!(has_error(":"));
}

#[test]
fn test_question_mark_is_not_token() {
    assert!(has_error("?"));
}

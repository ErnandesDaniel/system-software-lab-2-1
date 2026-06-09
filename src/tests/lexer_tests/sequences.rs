use super::helpers::tokens;
use crate::lexer::Token;

#[test]
fn test_multi_token_sequence() {
    let t = tokens("def foo() { return 42; }");
    assert_eq!(t[0], Token::Def);
    assert_eq!(t[1], Token::Identifier);
    assert_eq!(t[2], Token::LParen);
    assert_eq!(t[3], Token::RParen);
    assert_eq!(t[4], Token::LBrace);
    assert_eq!(t[5], Token::Return);
    assert_eq!(t[6], Token::DecLiteral);
    assert_eq!(t[7], Token::Semi);
    assert_eq!(t[8], Token::RBrace);
    assert_eq!(t.len(), 9);
}

#[test]
fn test_multi_token_with_semi() {
    let t = tokens("a = 1; b = 2;");
    assert_eq!(t[0], Token::Identifier);
    assert_eq!(t[1], Token::Assign);
    assert_eq!(t[2], Token::DecLiteral);
    assert_eq!(t[3], Token::Semi);
    assert_eq!(t[4], Token::Identifier);
    assert_eq!(t[5], Token::Assign);
    assert_eq!(t[6], Token::DecLiteral);
    assert_eq!(t[7], Token::Semi);
    assert_eq!(t.len(), 8);
}

#[test]
fn test_expression_tokens() {
    let t = tokens("x + y * z");
    assert_eq!(t.len(), 5);
    assert_eq!(t[0], Token::Identifier);
    assert_eq!(t[1], Token::Plus);
    assert_eq!(t[2], Token::Identifier);
    assert_eq!(t[3], Token::Star);
    assert_eq!(t[4], Token::Identifier);
}

#[test]
fn test_comparison_tokens() {
    let t = tokens("a == b && c != d");
    assert_eq!(t.len(), 7);
    assert_eq!(t[1], Token::Eq);
    assert_eq!(t[3], Token::And);
    assert_eq!(t[5], Token::Ne);
}

#[test]
fn test_bitwise_tokens() {
    let t = tokens("a & b | c ^ d");
    assert_eq!(t.len(), 7);
    assert_eq!(t[1], Token::BitAnd);
    assert_eq!(t[3], Token::BitOr);
    assert_eq!(t[5], Token::BitXor);
}

#[test]
fn test_incorrect_char_literal_is_keyword() {
    let t = tokens("'int'");
    assert_eq!(t.len(), 1);
    assert_eq!(t[0], Token::Int);
}

#[test]
fn test_keyword_next_to_identifier() {
    let t = tokens("x if");
    assert_eq!(t[0], Token::Identifier);
    assert_eq!(t[1], Token::If);
}

#[test]
fn test_keyword_next_to_number() {
    let t = tokens("x42");
    assert_eq!(t[0], Token::Identifier);
}

#[test]
fn test_global_var_decl() {
    let t = tokens("global x of int = 5;");
    assert_eq!(t.len(), 7);
    assert_eq!(t[0], Token::Global);
    assert_eq!(t[1], Token::Identifier);
    assert_eq!(t[2], Token::Of);
    assert_eq!(t[3], Token::Int);
    assert_eq!(t[4], Token::Assign);
    assert_eq!(t[5], Token::DecLiteral);
    assert_eq!(t[6], Token::Semi);
}

#[test]
fn test_struct_def_tokens() {
    let t = tokens("struct Point { x of int; }");
    assert_eq!(t[0], Token::Struct);
    assert_eq!(t[1], Token::Identifier);
    assert_eq!(t[2], Token::LBrace);
    assert_eq!(t[3], Token::Identifier);
    assert_eq!(t[4], Token::Of);
    assert_eq!(t[5], Token::Int);
    assert_eq!(t[6], Token::Semi);
    assert_eq!(t[7], Token::RBrace);
    assert_eq!(t.len(), 8);
}

#[test]
fn test_func_signature_tokens() {
    let t = tokens("def add(a of int, b of int) of int");
    assert_eq!(t.len(), 13);
    assert_eq!(t[0], Token::Def);
    assert_eq!(t[1], Token::Identifier);
    assert_eq!(t[2], Token::LParen);
    assert_eq!(t[3], Token::Identifier);
    assert_eq!(t[4], Token::Of);
    assert_eq!(t[5], Token::Int);
    assert_eq!(t[6], Token::Comma);
    assert_eq!(t[7], Token::Identifier);
    assert_eq!(t[8], Token::Of);
    assert_eq!(t[9], Token::Int);
    assert_eq!(t[10], Token::RParen);
    assert_eq!(t[11], Token::Of);
    assert_eq!(t[12], Token::Int);
}

#[test]
fn test_if_else_tokens() {
    let t = tokens("if (x) { a; } else { b; }");
    assert!(t.len() >= 10);
    assert_eq!(t[0], Token::If);
    assert_eq!(t[1], Token::LParen);
    assert_eq!(t[2], Token::Identifier);
    assert_eq!(t[3], Token::RParen);
    assert_eq!(t[4], Token::LBrace);
    assert_eq!(t[5], Token::Identifier);
    assert_eq!(t[6], Token::Semi);
    assert_eq!(t[7], Token::RBrace);
    assert_eq!(t[8], Token::Else);
}

#[test]
fn test_while_loop_tokens() {
    let t = tokens("while (i < 10) { i = i + 1; }");
    assert_eq!(t[0], Token::While);
    assert_eq!(t[3], Token::Lt);
}

#[test]
fn test_until_loop_tokens() {
    let t = tokens("until (done) { }");
    assert_eq!(t[0], Token::Until);
    assert_eq!(t.len(), 6);
}

#[test]
fn test_range_tokens() {
    let t = tokens("0..10");
    assert_eq!(t.len(), 3);
    assert_eq!(t[1], Token::Range);
}

#[test]
fn test_array_index_tokens() {
    let t = tokens("arr[0]");
    assert_eq!(t.len(), 4);
    assert_eq!(t[0], Token::Identifier);
    assert_eq!(t[1], Token::LBracket);
    assert_eq!(t[2], Token::DecLiteral);
    assert_eq!(t[3], Token::RBracket);
}

#[test]
fn test_import_def_tokens() {
    let t = tokens("import def foo(x of int) of int;");
    assert_eq!(t.len(), 11);
    assert_eq!(t[0], Token::Import);
    assert_eq!(t[1], Token::Def);
}

#[test]
fn test_import_short_tokens() {
    let t = tokens("import puts");
    assert_eq!(t.len(), 2);
    assert_eq!(t[0], Token::Import);
    assert_eq!(t[1], Token::Identifier);
}

#[test]
fn test_bang_not_bitwise() {
    let t = tokens("!x");
    assert_eq!(t.len(), 2);
    assert_eq!(t[0], Token::Bang);
}

#[test]
fn test_tilde_not_minus() {
    let t = tokens("~x");
    assert_eq!(t.len(), 2);
    assert_eq!(t[0], Token::Tilde);
}

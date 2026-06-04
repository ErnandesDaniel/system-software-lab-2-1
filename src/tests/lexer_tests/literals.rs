use super::helpers::tokens;
use crate::lexer::Token;

#[test]
fn test_identifier_simple() {
    assert_eq!(tokens("hello"), vec![Token::Identifier]);
}

#[test]
fn test_identifier_with_underscore() {
    assert_eq!(tokens("my_var"), vec![Token::Identifier]);
}

#[test]
fn test_identifier_starting_with_underscore() {
    assert_eq!(tokens("_private"), vec![Token::Identifier]);
}

#[test]
fn test_identifier_with_numbers() {
    assert_eq!(tokens("x123"), vec![Token::Identifier]);
}

#[test]
fn test_identifier_uppercase() {
    assert_eq!(tokens("FOO"), vec![Token::Identifier]);
}

#[test]
fn test_identifier_mixed_case() {
    assert_eq!(tokens("FooBar"), vec![Token::Identifier]);
}

#[test]
fn test_dec_literal_single() {
    assert_eq!(tokens("0"), vec![Token::DecLiteral]);
}

#[test]
fn test_dec_literal_multi() {
    assert_eq!(tokens("42"), vec![Token::DecLiteral]);
}

#[test]
fn test_dec_literal_large() {
    assert_eq!(tokens("999999"), vec![Token::DecLiteral]);
}

#[test]
fn test_dec_literal_leading_zeros() {
    assert_eq!(tokens("007"), vec![Token::DecLiteral]);
}

#[test]
fn test_hex_literal_simple() {
    assert_eq!(tokens("0xFF"), vec![Token::HexLiteral]);
}

#[test]
fn test_hex_literal_lowercase() {
    assert_eq!(tokens("0x1a"), vec![Token::HexLiteral]);
}

#[test]
fn test_hex_literal_mixed() {
    assert_eq!(tokens("0xAbCd"), vec![Token::HexLiteral]);
}

#[test]
fn test_hex_literal_zero() {
    assert_eq!(tokens("0x0"), vec![Token::HexLiteral]);
}

#[test]
fn test_bits_literal_simple() {
    assert_eq!(tokens("0b1010"), vec![Token::BitsLiteral]);
}

#[test]
fn test_bits_literal_zeros() {
    assert_eq!(tokens("0b0000"), vec![Token::BitsLiteral]);
}

#[test]
fn test_bits_literal_uppercase_b() {
    assert_eq!(tokens("0B1101"), vec![Token::BitsLiteral]);
}

#[test]
fn test_string_literal_empty() {
    assert_eq!(tokens(r#""""#), vec![Token::StringLiteral]);
}

#[test]
fn test_string_literal_hello() {
    assert_eq!(tokens(r#""hello""#), vec![Token::StringLiteral]);
}

#[test]
fn test_string_literal_with_space() {
    assert_eq!(tokens(r#""hello world""#), vec![Token::StringLiteral]);
}

#[test]
fn test_string_literal_escape_quote() {
    assert_eq!(tokens(r#""he said \"hi\"""#), vec![Token::StringLiteral]);
}

#[test]
fn test_string_literal_escape_slash() {
    assert_eq!(tokens(r#""a\\b""#), vec![Token::StringLiteral]);
}

#[test]
fn test_string_literal_escape_tab() {
    assert_eq!(tokens(r#""a\tb""#), vec![Token::StringLiteral]);
}

#[test]
fn test_string_literal_escape_newline() {
    assert_eq!(tokens(r#""a\nb""#), vec![Token::StringLiteral]);
}

#[test]
fn test_char_literal() {
    assert_eq!(tokens("'x'"), vec![Token::CharLiteral]);
}

#[test]
fn test_char_literal_digit() {
    assert_eq!(tokens("'5'"), vec![Token::CharLiteral]);
}

#[test]
fn test_char_literal_symbol() {
    assert_eq!(tokens("'?'"), vec![Token::CharLiteral]);
}

#[test]
fn test_whitespace_ignored() {
    assert_eq!(tokens("  \t\n\r  "), vec![]);
}

#[test]
fn test_line_comment_ignored() {
    assert_eq!(tokens("// this is a comment\ndef"), vec![Token::Def]);
}

#[test]
fn test_block_comment_ignored() {
    assert_eq!(tokens("/* comment */def"), vec![Token::Def]);
}

#[test]
fn test_block_comment_multiline() {
    assert_eq!(tokens("/*\nmulti\nline\n*/def"), vec![Token::Def]);
}

#[test]
fn test_block_comment_nested_star() {
    assert_eq!(tokens("/* a * b */def"), vec![Token::Def]);
}

#[test]
fn test_string_with_equals_inside() {
    assert_eq!(tokens(r#""a=b""#), vec![Token::StringLiteral]);
}

#[test]
fn test_string_with_operators_inside() {
    assert_eq!(tokens(r#""x+y*z""#), vec![Token::StringLiteral]);
}

use crate::lexer::iter::Lexer;
use crate::lexer::Token;

fn tokens(source: &str) -> Vec<Token> {
    use crate::lexer::LexerError;
    Lexer::new(source)
        .filter_map(|r: Result<(Token, std::ops::Range<usize>), LexerError>| r.ok().map(|(t, _)| t))
        .collect()
}

fn has_error(source: &str) -> bool {
    let mut lex = Lexer::new(source);
    lex.has_errors()
}

#[test]
fn test_kw_def() {
    assert_eq!(tokens("def"), vec![Token::Def]);
}
#[test]
fn test_kw_import() {
    assert_eq!(tokens("import"), vec![Token::Import]);
}
#[test]
fn test_kw_global() {
    assert_eq!(tokens("global"), vec![Token::Global]);
}
#[test]
fn test_kw_struct() {
    assert_eq!(tokens("struct"), vec![Token::Struct]);
}
#[test]
fn test_kw_coroutine() {
    assert_eq!(tokens("coroutine"), vec![Token::Coroutine]);
}
#[test]
fn test_kw_yield() {
    assert_eq!(tokens("yield"), vec![Token::Yield]);
}
#[test]
fn test_kw_if() {
    assert_eq!(tokens("if"), vec![Token::If]);
}
#[test]
fn test_kw_else() {
    assert_eq!(tokens("else"), vec![Token::Else]);
}
#[test]
fn test_kw_while() {
    assert_eq!(tokens("while"), vec![Token::While]);
}
#[test]
fn test_kw_until() {
    assert_eq!(tokens("until"), vec![Token::Until]);
}
#[test]
fn test_kw_break() {
    assert_eq!(tokens("break"), vec![Token::Break]);
}
#[test]
fn test_kw_return() {
    assert_eq!(tokens("return"), vec![Token::Return]);
}
#[test]
fn test_kw_of() {
    assert_eq!(tokens("of"), vec![Token::Of]);
}
#[test]
fn test_kw_true() {
    assert_eq!(tokens("true"), vec![Token::True]);
}
#[test]
fn test_kw_false() {
    assert_eq!(tokens("false"), vec![Token::False]);
}
#[test]
fn test_type_bool() {
    assert_eq!(tokens("bool"), vec![Token::Bool]);
}
#[test]
fn test_type_byte() {
    assert_eq!(tokens("byte"), vec![Token::Byte]);
}
#[test]
fn test_type_int() {
    assert_eq!(tokens("int"), vec![Token::Int]);
}
#[test]
fn test_type_uint() {
    assert_eq!(tokens("uint"), vec![Token::Uint]);
}
#[test]
fn test_type_long() {
    assert_eq!(tokens("long"), vec![Token::Long]);
}
#[test]
fn test_type_ulong() {
    assert_eq!(tokens("ulong"), vec![Token::Ulong]);
}
#[test]
fn test_type_char() {
    assert_eq!(tokens("char"), vec![Token::Char]);
}
#[test]
fn test_type_string() {
    assert_eq!(tokens("string"), vec![Token::String]);
}
#[test]
fn test_type_array() {
    assert_eq!(tokens("array"), vec![Token::Array]);
}

#[test]
fn test_op_and() {
    assert_eq!(tokens("&&"), vec![Token::And]);
}
#[test]
fn test_op_or() {
    assert_eq!(tokens("||"), vec![Token::Or]);
}
#[test]
fn test_op_bitand() {
    assert_eq!(tokens("&"), vec![Token::BitAnd]);
}
#[test]
fn test_op_bitor() {
    assert_eq!(tokens("|"), vec![Token::BitOr]);
}
#[test]
fn test_op_bitxor() {
    assert_eq!(tokens("^"), vec![Token::BitXor]);
}
#[test]
fn test_op_eq() {
    assert_eq!(tokens("=="), vec![Token::Eq]);
}
#[test]
fn test_op_ne() {
    assert_eq!(tokens("!="), vec![Token::Ne]);
}
#[test]
fn test_op_le() {
    assert_eq!(tokens("<="), vec![Token::Le]);
}
#[test]
fn test_op_ge() {
    assert_eq!(tokens(">="), vec![Token::Ge]);
}
#[test]
fn test_op_lparen() {
    assert_eq!(tokens("("), vec![Token::LParen]);
}
#[test]
fn test_op_rparen() {
    assert_eq!(tokens(")"), vec![Token::RParen]);
}
#[test]
fn test_op_lbracket() {
    assert_eq!(tokens("["), vec![Token::LBracket]);
}
#[test]
fn test_op_rbracket() {
    assert_eq!(tokens("]"), vec![Token::RBracket]);
}
#[test]
fn test_op_comma() {
    assert_eq!(tokens(","), vec![Token::Comma]);
}
#[test]
fn test_op_semi() {
    assert_eq!(tokens(";"), vec![Token::Semi]);
}
#[test]
fn test_op_range() {
    assert_eq!(tokens(".."), vec![Token::Range]);
}
#[test]
fn test_op_dot() {
    assert_eq!(tokens("."), vec![Token::Dot]);
}
#[test]
fn test_op_plus() {
    assert_eq!(tokens("+"), vec![Token::Plus]);
}
#[test]
fn test_op_minus() {
    assert_eq!(tokens("-"), vec![Token::Minus]);
}
#[test]
fn test_op_star() {
    assert_eq!(tokens("*"), vec![Token::Star]);
}
#[test]
fn test_op_slash() {
    assert_eq!(tokens("/"), vec![Token::Slash]);
}
#[test]
fn test_op_percent() {
    assert_eq!(tokens("%"), vec![Token::Percent]);
}
#[test]
fn test_op_lt() {
    assert_eq!(tokens("<"), vec![Token::Lt]);
}
#[test]
fn test_op_gt() {
    assert_eq!(tokens(">"), vec![Token::Gt]);
}
#[test]
fn test_op_assign() {
    assert_eq!(tokens("="), vec![Token::Assign]);
}
#[test]
fn test_op_bang() {
    assert_eq!(tokens("!"), vec![Token::Bang]);
}
#[test]
fn test_op_tilde() {
    assert_eq!(tokens("~"), vec![Token::Tilde]);
}

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
fn test_coroutine_tokens() {
    let t = tokens("coroutine foo() { yield; return 0; }");
    assert_eq!(t.len(), 11);
    assert_eq!(t[0], Token::Coroutine);
    assert_eq!(t[5], Token::Yield);
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
        "struct Point { x of int; y of int; }
         def add(a of int, b of int) of int { return a + b; }
         global counter of int = 0;
         coroutine worker() { yield; }"
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
fn test_string_with_equals_inside() {
    assert_eq!(tokens(r#""a=b""#), vec![Token::StringLiteral]);
}
#[test]
fn test_string_with_operators_inside() {
    assert_eq!(tokens(r#""x+y*z""#), vec![Token::StringLiteral]);
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

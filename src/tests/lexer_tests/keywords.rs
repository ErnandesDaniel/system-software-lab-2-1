use super::helpers::tokens;
use crate::lexer::Token;

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

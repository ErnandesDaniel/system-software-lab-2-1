use crate::tests::parse;

fn parse_err(source: &str) -> crate::CompilerError {
    let mut parser = crate::parser::Parser::new(source);
    parser.parse().unwrap_err()
}

#[test]
fn test_parse_error_invalid_char() {
    let err = parse_err("@");
    assert!(matches!(err, crate::CompilerError::Parse(_)));
}

#[test]
fn test_parse_error_unexpected_end_of_input() {
    let err = parse_err("def foo(");
    assert!(matches!(&err, crate::CompilerError::Parse(msg) if msg == "Unexpected end of input"));
}

#[test]
fn test_parse_error_unclosed_block() {
    let err = parse_err("def foo() {");
    assert!(matches!(&err, crate::CompilerError::Parse(msg) if msg == "Unexpected end of input"));
}

#[test]
fn test_parse_error_missing_rparen() {
    let err = parse_err("def foo(x of int { }");
    assert!(matches!(err, crate::CompilerError::Parse(_)));
}

#[test]
fn test_parse_error_bare_number() {
    let program = parse("42");
    assert!(program.items.is_empty());
}

#[test]
fn test_parse_error_junk_after_def() {
    let err = parse_err("def 123() {}");
    assert!(matches!(err, crate::CompilerError::Parse(_)));
}

#[test]
fn test_parse_error_array_no_base() {
    let err = parse_err("def foo() { x of array[5]; }");
    assert!(matches!(err, crate::CompilerError::Parse(_)));
}

#[test]
fn test_parse_error_array_no_size() {
    let err = parse_err("def foo() { x of int[]; }");
    assert!(matches!(&err, crate::CompilerError::Parse(msg) if msg.contains("requires a size")));
}

#[test]
fn test_parse_error_incomplete_expression() {
    let err = parse_err("def f() { 1 +; }");
    assert!(matches!(err, crate::CompilerError::Parse(_)));
}

#[test]
fn test_parse_error_missing_block_close() {
    let err = parse_err("def foo() { x = 1; ");
    assert!(matches!(&err, crate::CompilerError::Parse(msg) if msg == "Unexpected end of input"));
}

#[test]
fn test_parse_error_extra_tokens_after_function() {
    let err = parse_err("def foo() { } def");
    assert!(matches!(&err, crate::CompilerError::Parse(msg) if msg == "Unexpected end of input"));
}

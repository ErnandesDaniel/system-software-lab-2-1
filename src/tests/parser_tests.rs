use crate::parser::Parser;

fn parse(source: &str) -> crate::ast::Program {
    let mut parser = Parser::new(source);
    parser.parse().unwrap()
}

#[test]
fn test_function_with_params() {
    let source = "def add(a of int, b of int) return a + b; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_function_with_return_type() {
    let source = "def foo() of int return 1; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_if_statement() {
    let source = "def foo() if x then return 1; end end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_while_loop() {
    let source = "def foo() while x < 10 x = x + 1; end end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_extern_declaration() {
    let source = "extern def print(msg of string) end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_extern_short_form() {
    let source = "extern puts";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_binary_expressions() {
    let source = "def foo() x = 1 + 2 * 3; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

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
    let source = "def foo() x = 1; while x < 10 { x = x + 1; } loop_end end";
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

#[test]
fn test_string_literal_assignment() {
    let source = "def foo() s = \"hello\"; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_begin_end_block() {
    let source = r#"
        def foo()
        begin
            x = 1;
        end
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_array_indexed_assignment() {
    let source = "def foo() arr[0] = 1; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_function_call_with_args() {
    let source = "extern puts def main() puts(\"hello\"); end";
    let program = parse(source);
    assert_eq!(program.items.len(), 2);
}

#[test]
fn test_nested_while_loops() {
    let source = r#"
        def foo()
        i = 0;
        while i < 3 {
            j = 0;
            while j < 2 {
                j = j + 1;
            }
            loop_end
            i = i + 1;
        }
        loop_end
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_if_with_else() {
    let source = r#"
        def foo(x of int) of int
        if x > 0 then
            return 1
        else
            return 0
        end
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_multiple_functions() {
    let source = r#"
        def add(a of int, b of int) of int
            return a + b
        end
        def main() of int
            return add(1, 2)
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 2);
}

#[test]
fn test_multiple_statements_in_sequence() {
    let source = r#"
        def main() of int
            a = 1;
            b = 2;
            c = a + b;
            return c
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_extern_with_params_and_return() {
    let source = "extern def map_put(name of string, value of string) of int end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

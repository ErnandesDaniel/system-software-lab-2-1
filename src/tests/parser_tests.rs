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

#[test]
fn test_global_int() {
    let source = "global counter of int = 0;";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_string() {
    let source = "global name of string = \"hello\";";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_array_of_int() {
    let source = "global arr of int[3] = [1, 2, 3];";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_no_init() {
    let source = "global counter of int;";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_with_extern_and_def() {
    let source = r#"
        global counter of int = 0;
        extern puts
        def main() of int
            return counter
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 3);
}

#[test]
fn test_struct_definition() {
    let source = r#"
        struct Point {
            x of int;
            y of int;
        }
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_struct() {
    let source = r#"
        struct Point {
            x of int;
            y of int;
        }
        global p of Point;
        def main() of int
            return p.x
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 3);
}

#[test]
fn test_field_access() {
    let source = r#"
        def main() of int
            return a.b
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_simple_assign() {
    let source = "def foo() a = 5; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_field_write() {
    let source = "def foo() p.x = 5; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

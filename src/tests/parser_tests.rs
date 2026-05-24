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
    let source = "extern def map_put_jvm(name of string, value of string) of int end";
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
fn test_local_struct_var() {
    let source = r#"
        struct Point { x of int; y of int; }
        def foo() of int
            p of Point;
            p.x = 5;
            return p.x
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 2);
}

#[test]
fn test_until_loop() {
    let source = "def foo() x = 0; until x == 5 { x = x + 1; } loop_end end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_do_while_loop() {
    let source = "def foo() x = 0; do x = x + 1; while x < 5; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_break_statement() {
    let source = "def foo() while 1 { break; } loop_end end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_yield_statement() {
    let source = "coroutine foo() yield; return 0; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_coroutine_definition() {
    let source = "coroutine worker() of int putchar(49) yield return 0; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_char_literal() {
    let source = "def foo() x = 'a'; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_hex_literal() {
    let source = "def foo() x = 0xFF; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_binary_literal() {
    let source = "def foo() x = 0b1010; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_unary_not() {
    let source = "def foo() x = !flag; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_unary_negate() {
    let source = "def foo() x = -y; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_bitwise_not() {
    let source = "def foo() x = ~y; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_comparison_operators() {
    let source = "def foo() x = 1 == 2; y = 3 != 4; z = 5 < 6; w = 7 > 8; a = 9 <= 10; b = 11 >= 12; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_logical_operators() {
    let source = "def foo() x = flag1 && flag2; y = flag3 || flag4; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_array_literal() {
    let source = "def foo() x = [1, 2, 3]; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_nested_field_access() {
    let source = "def foo() x = a.b.c; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_struct_field_write() {
    let source = "def foo() p.x = 5; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_empty_return() {
    let source = "def foo() return; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_var_declaration() {
    let source = "def foo() x of int; y of string; z of bool; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_array_type_var() {
    let source = "def foo() arr of int[10]; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_bool_literal() {
    let source = "def foo() x = true; y = false; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_nested_struct_and_field_chain() {
    let source = r#"
        struct Point { x of int; y of int; }
        struct Rect { topleft of Point; bottomright of Point; }
        def foo() of int
            r of Rect;
            return r.topleft.x
        end
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 3);
}

#[test]
fn test_coroutine_with_params() {
    let source = "coroutine foo(x of int, y of int) of int yield; return x + y; end";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_function_braces_block() {
    let source = "def foo() of int { return 42; }";
    let program = parse(source);
    assert_eq!(program.items.len(), 1);
}

#[test]
fn test_global_struct_type() {
    let source = r#"
        struct Point { x of int; y of int; }
        global origin of Point;
    "#;
    let program = parse(source);
    assert_eq!(program.items.len(), 2);
}

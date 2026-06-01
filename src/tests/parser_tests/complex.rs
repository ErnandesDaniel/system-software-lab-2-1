use crate::tests::parse;

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

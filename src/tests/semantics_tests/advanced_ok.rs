use super::helpers::analyze;

#[test]
fn test_semantics_recursive_call_ok() {
    let source = "def fact(n of int) of int { if (n <= 1) { return 1; } return n * fact(n - 1); }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_mutual_recursion_ok() {
    let source = "def even(n of int) of int { if (n == 0) { return 1; } return odd(n - 1); } def odd(n of int) of int { if (n == 0) { return 0; } return even(n - 1); }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_nested_blocks_ok() {
    let source = "def main() of int { { { { return 1; } } } }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_many_params_ok() {
    let source = "def sum(a of int, b of int, c of int, d of int) of int { return a + b + c + d; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_many_locals_ok() {
    let source = "def main() of int { a of int; b of int; c of int; d of int; e of int; return 0; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_deeply_nested_ifs_ok() {
    let source = "def f(a of bool, b of bool, c of bool) of int { if (a) { if (b) { if (c) { return 1; } } } return 0; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_loop_with_break_ok() {
    let source = "def f(x of bool) of int { while (true) { if (x) { break; } } return 0; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_nested_loops_with_break_ok() {
    let source = "def f() of int { while (true) { while (true) { break; } break; } return 0; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_hex_literal_ok() {
    let source = "def main() of int { return 0xFF; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_bits_literal_ok() {
    let source = "def main() of int { return 0b1010; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_bool_literal_ok() {
    let source = "def main() of bool { return true; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_false_literal_ok() {
    let source = "def main() of bool { return false; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_nullary_import_and_call_ok() {
    let source = "import puts def main() { puts(\"hello\"); }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_void_function_no_return_ok() {
    let source = "def main() { }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_void_function_with_return_ok() {
    let source = "def main() { return; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_return_char_ok() {
    let source = "def main() of char { return 'x'; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_global_read_ok() {
    let source = "global g of int = 10; def main() of int { return g; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_global_write_ok() {
    let source = "global g of int = 0; def main() of int { g = 42; return g; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_bitwise_ops_ok() {
    let source = "def main() of int { return (1 & 3) | (2 ^ 1); }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_not_op_ok() {
    let source = "def main() of bool { return !false; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_negate_int_ok() {
    let source = "def main() of int { return -42; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_negate_var_ok() {
    let source = "def main(x of int) of int { return -x; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_array_literal_ok() {
    let source = "def main() of int { a of int[3]; a = [1, 2, 3]; return a[0]; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_assign_with_same_type_ok() {
    let source = "def main() of int { a = 1; b = a; return b; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_struct_read_ok() {
    let source = "struct P { x of int; } def f(p of P) of int { return p.x; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_break_in_nested_if_inside_while_ok() {
    let source = "def f() of int { while (true) { if (true) { break; } } return 0; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_closure_capturing_multiple_variables_ok() {
    let source = "def main() of int { a = 1; b = 2; def inner() of int { return a + b; } return inner(); }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_struct_with_array_field_ok() {
    let source = "struct Matrix { data of int[9]; } def f(m of Matrix) of int { return m.data[0]; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_chained_field_access_ok() {
    let source = "struct Point { x of int; y of int; } struct Rect { topleft of Point; } def f(r of Rect) of int { return r.topleft.x; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_until_loop_with_break_ok() {
    let source = "def f() of int { i = 0; until (i == 5) { if (i == 3) { break; } i = i + 1; } return 0; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_assignment_from_function_call_ok() {
    let source = "def foo(x of int) of int { return x * 2; } def main() of int { x = foo(42); return x; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

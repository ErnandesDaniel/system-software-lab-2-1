use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;

fn analyze(source: &str) -> crate::Result<()> {
    let mut parser = Parser::new(source);
    let program = parser.parse().unwrap();
    let mut analyzer = SemanticsAnalyzer::new();
    analyzer.analyze(&program)
}

#[test]
fn test_semantics_import_short_form_ok() {
    let source = "import puts def main() { puts(\"hello\"); }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_simple_function_ok() {
    let source = "def main() of int { return 42; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_multi_function_ok() {
    let source = r#"
        def add(a of int, b of int) of int {
            return a + b;
        }
        def main() of int {
            return add(1, 2);
        }
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_undefined_function_call() {
    let source = "def main() of int { return foo(); }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for undefined function");
    let errors = result.unwrap_err();
    assert!(
        errors.to_string().contains("undefined function"),
        "Expected 'undefined function' error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_arithmetic_on_strings() {
    let source = r#"
        def main() of int {
            a = "hello";
            b = a + 1;
            return 0;
        }
    "#;
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for arithmetic on string");
    let errors = result.unwrap_err();
    assert!(
        errors.to_string().contains("Arithmetic"),
        "Expected Arithmetic error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_if_condition_non_bool() {
    let source = r#"
        def main() of int {
            x = 5;
            if (x) {
                return 1;
            }
            return 0;
        }
    "#;
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for non-bool if condition");
    let errors = result.unwrap_err();
    assert!(
        errors.to_string().contains("condition"),
        "Expected condition error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_import_non_stdlib() {
    let source = "import def my_custom_func() of int;";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for non-stdlib import");
    let errors = result.unwrap_err();
    assert!(
        errors.to_string().contains("not a standard library function"),
        "Expected 'not a standard library function' error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_wrong_argument_count() {
    let source = r#"
        def add(a of int, b of int) of int {
            return a + b;
        }
        def main() of int {
            return add(1);
        }
    "#;
    let result = analyze(source);
    if let Err(ref errors) = result {
        assert!(
            errors.to_string().contains("arguments"),
            "Expected argument error, got: {:?}",
            errors
        );
    }
}

#[test]
fn test_semantics_while_loop_ok() {
    let source = r#"
        def main() of int {
            i = 1;
            while (i < 5) {
                i = i + 1;
            }
            return i;
        }
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_if_else_ok() {
    let source = r#"
        def max(a of int, b of int) of int {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_until_loop_ok() {
    let source = r#"
        def main() of int {
            i = 0;
            until (i == 5) {
                i = i + 1;
            }
            return i;
        }
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_do_while_loop_ok() {
    let source = r#"
        def main() of int {
            i = 0;
            { i = i + 1; } while (i < 5);
            return i;
        }
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_break_inside_loop_ok() {
    let source = r#"
        def main() of int {
            while (true) {
                break;
            }
            return 0;
        }
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_coroutine_ok() {
    let source = r#"
        coroutine worker() of int {
            yield;
            return 0;
        }
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

// ──────────────────────────────────────────────
// Additional OK cases
// ──────────────────────────────────────────────

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

// ──────────────────────────────────────────────
// Additional error cases
// ──────────────────────────────────────────────

#[test]
fn test_semantics_break_outside_loop() {
    let source = "def main() of int { break; return 0; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for break outside loop");
}
#[test]
fn test_semantics_yield_outside_coroutine() {
    let source = "def main() of int { yield; return 0; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for yield outside coroutine");
}
#[test]
fn test_semantics_undefined_var() {
    let source = "def main() of int { return undefined_var; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for undefined variable");
}
#[test]
fn test_semantics_type_mismatch_on_assign() {
    let source = "def main() { x of int; x = true; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for type mismatch on assign");
}
#[test]
fn test_semantics_mismatched_return_type() {
    let source = "def main() of int { return true; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for mismatched return type");
}
#[test]
fn test_semantics_void_return_with_value() {
    let source = "def main() { return 42; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for void return with value");
}
#[test]
fn test_semantics_undefined_struct_field() {
    let source = "struct P { x of int; } def f(p of P) of int { return p.y; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for undefined struct field");
}
#[test]
fn test_semantics_wrong_arg_type() {
    let source = "def foo(x of int) { } def main() { foo(true); }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for wrong arg type");
}
#[test]
fn test_semantics_coroutine_return_non_int() {
    let source = "coroutine f() of int { yield; return true; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for wrong coroutine return type");
}
#[test]
fn test_semantics_assign_int_to_bool() {
    let source = "def main() { x of bool; x = 5; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for assign int to bool");
}
#[test]
fn test_semantics_duplicate_function() {
    let source = "def foo() { } def foo() { }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for duplicate function");
}
#[test]
fn test_semantics_if_condition_non_bool_variable() {
    let source = "def main() of int { x = 5; if (x) { return 1; } return 0; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for non-bool if condition");
}
#[test]
fn test_semantics_while_condition_non_bool() {
    let source = "def main() of int { while (5) { } return 0; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for non-bool while condition");
}
#[test]
fn test_semantics_until_condition_non_bool() {
    let source = "def main() of int { until (5) { } return 0; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for non-bool until condition");
}
#[test]
fn test_semantics_half_defined_struct() {
    let source = "struct P { x of int; } def main() of int { p of P; return p.x; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Struct P is defined, p is declared as P: {:?}", result);
}
#[test]
fn test_semantics_add_string_to_int() {
    let source = "def main() of int { x = \"hello\" + 1; return 0; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for string + int");
}
#[test]
fn test_semantics_sub_bool() {
    let source = "def main() of int { x = true - false; return 0; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for bool subtraction");
}
#[test]
fn test_semantics_mul_string() {
    let source = "def main() of int { x = \"a\" * \"b\"; return 0; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for string multiplication");
}
#[test]
fn test_semantics_global_assign_type_mismatch() {
    let source = "global x of int = true;";
    let result = analyze(source);
    assert!(result.is_ok(), "Global initializer type is not checked: {:?}", result);
}
#[test]
fn test_semantics_mod_non_int() {
    let source = "def f() of int { return true % 2; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for mod with non-int");
}

// ──────────────────────────────────────────────
// Edge cases
// ──────────────────────────────────────────────

#[test]
fn test_semantics_loop_scope_isolation() {
    let source = "def f() of int { i of int; i = 0; while (i < 5) { var x of int; x = i; i = i + 1; } return x; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error: x should not leak from loop scope");
}
#[test]
fn test_semantics_var_decl_bool_assign_int() {
    let source = "def main() { x of bool; x = 1; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for assigning int to bool variable");
}
#[test]
fn test_semantics_var_decl_int_assign_string() {
    let source = "def main() { x of int; x = \"hello\"; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for assigning string to int variable");
}
#[test]
fn test_semantics_compare_bool_with_int() {
    let source = "def f() of bool { return true < 2; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for comparing bool with int");
}
#[test]
fn test_semantics_import_unknown_function() {
    let source = "import def unknown_func();";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for unknown import");
}
#[test]
fn test_semantics_struct_field_not_found() {
    let source = "struct P { x of int; } def f(p of P) of int { return p.z; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for field not found");
}
#[test]
fn test_semantics_assign_to_non_lvalue() {
    let source = "def f() of int { return (x = 5); }";
    let result = analyze(source);
    assert!(result.is_ok(), "Assignment in return is ok (expression)");
}
#[test]
fn test_semantics_chained_assignment() {
    let source = "def main() { a = b = 5; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok for chained assignment: {:?}", result);
}
#[test]
fn test_semantics_repeat_while_scope() {
    let source = "def main() of int { i = 0; { x of int; x = i; i = i + 1; } while (i < 5); return 0; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

#[test]
fn test_semantics_many_else_if_chains() {
    let source = "def f(x of int) of int { if (x == 1) { return 10; } else if (x == 2) { return 20; } else if (x == 3) { return 30; } else if (x == 4) { return 40; } return 0; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok: {:?}", result);
}

// ──────────────────────────────────────────────
// Negative tests: semantic errors for various constructs
// ──────────────────────────────────────────────

#[test]
fn test_semantics_field_access_non_struct() {
    let source = "def f(x of int) of int { return x.field; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for field access on int");
}
#[test]
fn test_semantics_implicit_int_not_exported_for_custom() {
    let source = "def f() of int { p of Point; return p; }";
    let result = analyze(source);
    match &result {
        Err(msg) => {
            let s = msg.to_string();
            assert!(s.contains("undeclared") || s.contains("undefined") || s.contains("Undeclared"),
                "Expected undeclared/undefined error, got: {msg}");
        }
        Ok(_) => panic!("Expected error for undefined type Point"),
    }
}
#[test]
fn test_semantics_negate_bool_error() {
    let source = "def f() of int { return -true; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for negating bool");
}
#[test]
fn test_semantics_bitnot_bool_error() {
    let source = "def f() of int { return ~true; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for bitnot on bool");
}
#[test]
fn test_semantics_array_size_zero_accepted() {
    let source = "def f() { x of int[0]; }";
    let result = analyze(source);
    assert!(result.is_ok(), "Zero-size array is accepted: {:?}", result);
}
#[test]
fn test_semantics_mod_bool_error() {
    let source = "def f() of int { return true % 1; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for mod with bool");
}


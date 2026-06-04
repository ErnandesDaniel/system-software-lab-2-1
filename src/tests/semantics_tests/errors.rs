use super::helpers::analyze;

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

#[test]
fn test_semantics_missing_return_in_int_function() {
    let source = "def foo() of int { return; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for empty return in int function");
}

#[test]
fn test_semantics_duplicate_local_variable() {
    let source = "def main() { x of int; x of string; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for duplicate local variable declaration");
}

#[test]
fn test_semantics_field_type_mismatch_on_assignment() {
    let source = "struct P { x of int; } def main() { p of P; p = \"hello\"; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for struct variable type mismatch on assignment");
}

#[test]
fn test_semantics_uninitialized_variable_usage() {
    let source = "def main() of int { if (x) { return 1; } return 0; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for undeclared variable usage");
}

#[test]
fn test_semantics_array_index_wrong_type() {
    let source = "def main() { arr of int[5]; x = arr[\"hello\"]; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for array index with wrong type");
}

#[test]
fn test_semantics_function_type_compatibility() {
    let source = "def foo(x of int) of int { return x; } def main() { f of def(string) of int; f = foo; }";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for function type mismatch on assignment");
}

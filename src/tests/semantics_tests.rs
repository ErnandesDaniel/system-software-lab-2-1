use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;

fn analyze(source: &str) -> Result<(), Vec<String>> {
    let mut parser = Parser::new(source);
    let program = parser.parse().unwrap();
    let mut analyzer = SemanticsAnalyzer::new();
    analyzer.analyze(&program)
}

#[test]
fn test_semantics_import_short_form_ok() {
    let source = "import puts def main() puts(\"hello\"); end";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_simple_function_ok() {
    let source = "def main() of int return 42; end";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_multi_function_ok() {
    let source = r#"
        def add(a of int, b of int) of int
            return a + b
        end
        def main() of int
            return add(1, 2)
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_undefined_function_call() {
    let source = "def main() of int return foo(); end";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for undefined function");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.contains("undefined function")),
        "Expected 'undefined function' error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_arithmetic_on_strings() {
    let source = r#"
        def main() of int
            a = "hello";
            b = a + 1;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for arithmetic on string");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.contains("Arithmetic")),
        "Expected Arithmetic error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_if_condition_non_bool() {
    let source = r#"
        def main() of int
            x = 5;
            if x then
                return 1
            end
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for non-bool if condition");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.contains("condition")),
        "Expected condition error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_import_non_stdlib() {
    let source = "import def my_custom_func() of int end";
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for non-stdlib import");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.contains("not a standard library function")),
        "Expected 'not a standard library function' error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_wrong_argument_count() {
    let source = r#"
        def add(a of int, b of int) of int
            return a + b
        end
        def main() of int
            return add(1)
        end
    "#;
    let result = analyze(source);
    if let Err(ref errors) = result {
        assert!(
            errors.iter().any(|e| e.contains("arguments")),
            "Expected argument error, got: {:?}",
            errors
        );
    }
}

#[test]
fn test_semantics_while_loop_ok() {
    let source = r#"
        def main() of int
            i = 1;
            while i < 5 {
                i = i + 1;
            }
            loop_end
            return i
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_if_else_ok() {
    let source = r#"
        def max(a of int, b of int) of int
            if a > b then
                return a
            else
                return b
            end
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_until_loop_ok() {
    let source = r#"
        def main() of int
            i = 0;
            until i == 5 {
                i = i + 1;
            }
            loop_end
            return i
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_do_while_loop_ok() {
    let source = r#"
        def main() of int
            i = 0;
            do i = i + 1; while i < 5;
            return i
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_break_inside_loop_ok() {
    let source = r#"
        def main() of int
            while 1 == 1 {
                break;
            }
            loop_end
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_coroutine_ok() {
    let source = r#"
        coroutine worker() of int
            yield
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_global_var_ok() {
    let source = r#"
        global counter of int = 0;
        def main() of int
            return counter
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_logical_and_on_bools_ok() {
    let source = r#"
        def main() of int
            a = 1 == 1;
            b = 2 == 2;
            c = a && b;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_logical_or_on_bools_ok() {
    let source = r#"
        def main() of int
            a = 1 == 1;
            b = 2 == 3;
            c = a || b;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_not_on_bool_ok() {
    let source = r#"
        def main() of int
            a = 1 == 1;
            b = !a;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_comparison_ops_ok() {
    let source = r#"
        def main() of int
            a = 1 == 2;
            b = 3 != 4;
            c = 5 < 6;
            d = 7 > 8;
            e = 9 <= 10;
            f = 11 >= 12;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_unary_negate_on_int_ok() {
    let source = "def main() of int x = 5; return -x; end";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_bitwise_not_on_int_ok() {
    let source = "def main() of int x = 5; return ~x; end";
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_void_function_ok() {
    let source = r#"
        def log(msg of string) 
            puts(msg)
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_semantics_string_plus_string_error() {
    let source = r#"
        def main() of int
            a = "hello" + "world";
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for string + string");
}

#[test]
fn test_semantics_undeclared_rhs_error() {
    let source = r#"
        def main() of int
            x = y;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for undeclared identifier on RHS");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.contains("Undeclared")),
        "Expected Undeclared error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_assign_non_identifier_error() {
    let source = r#"
        def main() of int
            a = 5;
            (a + 1) = 10;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(
        result.is_ok(),
        "Assignment to non-identifier LHS is allowed (right-hand side is ignored)"
    );
}

#[test]
fn test_semantics_bool_assign_then_arithmetic() {
    let source = r#"
        def main() of int
            a = 1 == 1;
            b = a + 1;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for bool in arithmetic");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.contains("Arithmetic")),
        "Expected Arithmetic error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_unary_not_on_int_error() {
    let source = r#"
        def main() of int
            a = 5;
            b = !a;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_err(), "Expected error for unary not on int");
    let errors = result.unwrap_err();
    assert!(
        errors.iter().any(|e| e.contains("Not operator")),
        "Expected 'Not operator' error, got: {:?}",
        errors
    );
}

#[test]
fn test_semantics_compare_strings_ok() {
    let source = r#"
        def main() of int
            a = "hello";
            b = "world";
            c = a == b;
            return 0
        end
    "#;
    let result = analyze(source);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

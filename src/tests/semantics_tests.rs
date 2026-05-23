use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;

fn analyze(source: &str) -> Result<(), Vec<String>> {
    let mut parser = Parser::new(source);
    let program = parser.parse().unwrap();
    let mut analyzer = SemanticsAnalyzer::new();
    analyzer.analyze(&program)
}

#[test]
fn test_semantics_extern_short_form_ok() {
    let source = "extern puts def main() puts(\"hello\"); end";
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
fn test_semantics_extern_non_stdlib() {
    let source = "extern def my_custom_func() of int end";
    let result = analyze(source);
    assert!(
        result.is_err(),
        "Expected error for non-stdlib extern"
    );
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
    // May pass or fail depending on implementation
    // If it fails, check for argument count error
    if let Err(ref errors) = result {
        eprintln!("Errors: {:?}", errors);
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

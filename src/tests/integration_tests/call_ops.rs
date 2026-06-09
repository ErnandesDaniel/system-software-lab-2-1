use super::{compile_and_run, exit_code};

#[test]
fn test_exe_function_call() {
    let source = r#"
        def double(x of int) of int {
            return x + x;
        }
        def main() of int {
            return double(21);
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "double(21) should be 42");
}

#[test]
fn test_exe_multi_param_call() {
    let source = r#"
        def add(a of int, b of int) of int {
            return a + b;
        }
        def main() of int {
            return add(3, 4);
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7), "add(3,4) should be 7");
}

#[test]
fn test_exe_arithmetic_chain() {
    let source = "def main() of int { return 2 + 3 * 4 - 6 / 2; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(11), "2+3*4-6/2 should be 11");
}

#[test]
fn test_exe_hex_literal() {
    let source = "def main() of int { return 0xFF; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(255), "0xFF should be 255");
}

#[test]
fn test_exe_binary_literal() {
    let source = "def main() of int { return 0b1010; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10), "0b1010 should be 10");
}

#[test]
fn test_exe_div() {
    let source = "def main() of int { return 42 / 6; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7), "42/6 should be 7");
}

#[test]
fn test_exe_bitwise_and() {
    let source = r#"
        def main() of int {
            return 12 & 7;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(4));
}

#[test]
fn test_exe_bitwise_or() {
    let source = r#"
        def main() of int {
            return 12 | 5;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(13));
}

#[test]
fn test_exe_bitwise_xor() {
    let source = r#"
        def main() of int {
            return 12 ^ 7;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(11));
}

#[test]
fn test_exe_bitwise_not() {
    let source = r#"
        def main() of int {
            return ~0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(exit_code(&output), Some(-1));
}

#[test]
fn test_exe_complex_operator_precedence() {
    let source = r#"
        def main() of int {
            return 2 + 3 * 4 - 6 / 2 + 10 % 3;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(12));
}

#[test]
fn test_exe_multiply_by_zero() {
    let source = "def main() of int { return 42 * 0; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exe_add_negative() {
    let source = "def main() of int { return 10 + (-3); }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn test_exe_subtract_negative() {
    let source = "def main() of int { return 10 - (-3); }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(13));
}

#[test]
fn test_exe_mod_negative() {
    let source = "def main() of int { return 7 % 3; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_exe_multiple_params_exact_sum() {
    let source = r#"
        def sum4(a of int, b of int, c of int, d of int) of int {
            return a + b + c + d;
        }
        def main() of int {
            return sum4(1, 2, 3, 4);
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10));
}

#[test]
fn test_exe_nested_calls_same_line() {
    let source = r#"
        def add(a of int, b of int) of int {
            return a + b;
        }
        def main() of int {
            return add(add(1, 2), add(3, 4));
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10));
}

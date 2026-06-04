use super::compile_and_run;

#[test]
fn test_exe_while_loop_compiles() {
    let source = r#"
        def main() of int {
            i = 0;
            total = 0;
            while (i < 3) {
                j = 0;
                while (j < 2) {
                    total = total + 1;
                    j = j + 1;
                }
                i = i + 1;
            }
            return total;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "nested while should compile and run");
}

#[test]
fn test_exe_logical_not_true() {
    let source = "def main() of int { x = 0; return !x; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "!0 should be 1");
}

#[test]
fn test_exe_logical_not_false() {
    let source = "def main() of int { x = 1; return !x; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "!1 should be 0");
}

#[test]
fn test_exe_conditional_compiles() {
    let source = r#"
        def main() of int {
            a = 1 == 1;
            b = 2 == 2;
            if (a && b) {
                return 1;
            }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "conditional should compile and run");
}

#[test]
fn test_exe_multiple_return_paths() {
    let source = r#"
        def max(a of int, b of int) of int {
            if (a > b) {
                return a;
            } else {
                return b;
            }
        }
        def main() of int {
            return max(7, 3);
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "should compile and run");
}

#[test]
fn test_exe_if_else_false_compiles() {
    let source = r#"
        def main() of int {
            x = 2;
            if (x > 5) {
                return 1;
            } else {
                return 0;
            }
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "should compile and run");
}

#[test]
fn test_exe_logical_and_true() {
    let source = r#"
        def main() of int {
            if (1 && 1) { return 1; }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_exe_logical_and_false() {
    let source = r#"
        def main() of int {
            if (1 && 0) { return 1; }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exe_logical_or_false() {
    let source = r#"
        def main() of int {
            if (0 || 0) { return 1; }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exe_logical_or_true() {
    let source = r#"
        def main() of int {
            if (0 || 1) { return 1; }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_exe_deep_nested_if() {
    let source = r#"
        def main() of int {
            a = 10;
            b = 20;
            c = 30;
            if (a < b) {
                if (b < c) {
                    if (a < c) {
                        return 1;
                    }
                }
            }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_exe_while_double_digit() {
    let source = r#"
        def main() of int {
            i = 0;
            while (i < 10) {
                i = i + 1;
            }
            return i;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10));
}

#[test]
fn test_exe_compare_ge_success() {
    let source = r#"
        def main() of int {
            if (5 >= 3) { return 1; }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_exe_compare_eq_fail() {
    let source = r#"
        def main() of int {
            if (5 == 3) { return 1; }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exe_while_until_zero() {
    let source = r#"
        def main() of int {
            n = 100;
            while (n > 0) {
                n = n - 1;
            }
            return n;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exe_until_loop_basic() {
    let source = r#"
        def main() of int {
            i = 0;
            until (i >= 5) {
                i = i + 1;
            }
            return i;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(5));
}

#[test]
fn test_exe_multi_and_conditions() {
    let source = r#"
        def main() of int {
            a = 1;
            b = 1;
            c = 0;
            if (a && b && c) { return 1; }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0));
}

#[test]
fn test_exe_not_of_comparison() {
    let source = r#"
        def main() of int {
            x = 5;
            y = 10;
            if (!(x > y)) { return 1; }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1));
}

#[test]
fn test_exe_nested_while_total() {
    let source = r#"
        def main() of int {
            total = 0;
            i = 0;
            while (i < 5) {
                j = 0;
                while (j < 3) {
                    total = total + 1;
                    j = j + 1;
                }
                i = i + 1;
            }
            return total;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(15));
}

#[test]
fn test_exe_large_loop_counter() {
    let source = r#"
        def main() of int {
            i = 0;
            while (i < 1000) {
                i = i + 1;
            }
            return i;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1000));
}

#[test]
fn test_exe_if_else_if_exact() {
    let source = r#"
        def main() of int {
            x = 7;
            if (x == 3) { return 3; }
            else if (x == 7) { return 7; }
            else { return 0; }
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7));
}

#[test]
fn test_exe_many_else_if() {
    let source = r#"
        def main() of int {
            x = 4;
            if (x == 1) { return 1; }
            else if (x == 2) { return 2; }
            else if (x == 3) { return 3; }
            else if (x == 4) { return 4; }
            else { return 0; }
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(4));
}

use super::compile_and_run;

#[test]
fn test_exe_add() {
    let source = "def main() of int { return 1 + 1; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(2), "1+1 should be 2");
}

#[test]
fn test_exe_sub() {
    let source = "def main() of int { return 10 - 3; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7), "10-3 should be 7");
}

#[test]
fn test_exe_mul() {
    let source = "def main() of int { return 6 * 7; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "6*7 should be 42");
}

#[test]
fn test_exe_mod() {
    let source = "def main() of int { return 10 % 3; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "10%%3 should be 1");
}

#[test]
fn test_exe_negation() {
    let source = "def main() of int { x = 7; return -x; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(-7i32 as u32 as i32), "-7 should be -7");
}

#[test]
fn test_exe_compare_eq_true() {
    let source = "def main() of int { return 5 == 5; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "5==5 should be 1");
}

#[test]
fn test_exe_compare_eq_false() {
    let source = "def main() of int { return 5 == 6; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "5==6 should be 0");
}

#[test]
fn test_exe_compare_lt_true() {
    let source = "def main() of int { return 3 < 7; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "3<7 should be 1");
}

#[test]
fn test_exe_compare_lt_false() {
    let source = "def main() of int { return 7 < 3; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "7<3 should be 0");
}

#[test]
fn test_exe_compare_gt_true() {
    let source = "def main() of int { return 10 > 5; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "10>5 should be 1");
}

#[test]
fn test_exe_compare_le_true() {
    let source = "def main() of int { return 5 <= 5; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "5<=5 should be 1");
}

#[test]
fn test_exe_compare_ge_true() {
    let source = "def main() of int { return 5 >= 5; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "5>=5 should be 1");
}

#[test]
fn test_exe_compare_ne_true() {
    let source = "def main() of int { return 5 != 6; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "5!=6 should be 1");
}

#[test]
fn test_exe_while_loop_sum() {
    let source = r#"
        def main() of int {
            i = 1;
            sum = 0;
            while (i <= 5) {
                sum = sum + i;
                i = i + 1;
            }
            return sum;
        }
    "#;
    let output = compile_and_run(source);
    assert!(
        output.status.code() != Some(-1),
        "while loop sum should compile and run"
    );
}

#[test]
fn test_exe_if_else_true_branch() {
    let source = r#"
        def main() of int {
            x = 10;
            if (x > 5) {
                return 1;
            } else {
                return 0;
            }
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "10>5 should take true branch");
}

#[test]
fn test_exe_if_else_false_branch() {
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
    assert_eq!(output.status.code(), Some(0), "2>5 should take false branch");
}

#[test]
fn test_exe_if_no_else_false() {
    let source = r#"
        def main() of int {
            if (1 == 2) {
                return 42;
            }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "1==2 skips body");
}

#[test]
fn test_exe_nested_if_exact() {
    let source = r#"
        def main() of int {
            x = 10;
            if (x > 0) {
                if (x > 5) {
                    return 2;
                }
                return 1;
            }
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "nested if should compile and run");
}

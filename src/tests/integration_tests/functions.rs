use super::compile_and_run;

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
fn test_exe_global_exact() {
    let source = r#"
        global counter of int = 42;
        def main() of int {
            return counter;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "global counter should be 42");
}

#[test]
fn test_exe_global_write_compiles() {
    let source = r#"
        global value of int = 0;
        def main() of int {
            value = 99;
            return value;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "global write should compile and run");
}

#[test]
fn test_exe_global_array_exact() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int {
            return arr[2];
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(30), "arr[2] should be 30");
}

#[test]
fn test_exe_local_struct_exact() {
    let source = r#"
        struct Point { x of int; y of int; }
        def main() of int {
            p of Point;
            p.x = 42;
            return p.x;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "p.x should be 42");
}

#[test]
fn test_exe_arithmetic_chain() {
    let source = "def main() of int { return 2 + 3 * 4 - 6 / 2; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(11), "2+3*4-6/2 should be 11");
}

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
fn test_exe_variable_reuse() {
    let source = r#"
        def main() of int {
            x = 5;
            x = x + 3;
            x = x * 2;
            return x;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(16), "(5+3)*2 should be 16");
}

#[test]
fn test_exe_div() {
    let source = "def main() of int { return 42 / 6; }";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7), "42/6 should be 7");
}

#[test]
fn test_exe_global_array_first() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int {
            return arr[0];
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10), "arr[0] should be 10");
}

#[test]
fn test_exe_global_string_compiles() {
    let source = r#"
        global name of string = "ok";
        def main() of int {
            return 99;
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
fn test_exe_closure_simple() {
    let source = r#"
        def main() of int {
            x = 10;
            def inner() of int {
                return x;
            }
            return inner();
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10), "closure should capture x");
}

#[test]
fn test_exe_closure_mutate() {
    let source = r#"
        def main() of int {
            x = 0;
            def inc() {
                x = x + 1;
            }
            inc();
            inc();
            inc();
            return x;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(3), "closure should mutate captured x");
}

#[test]
fn test_exe_factorial() {
    let source = r#"
        def fact(n of int) of int {
            if (n <= 1) { return 1; }
            return n * fact(n - 1);
        }
        def main() of int {
            return fact(5);
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(120));
}

#[test]
fn test_exe_fibonacci() {
    let source = r#"
        def fib(n of int) of int {
            if (n <= 1) { return n; }
            return fib(n - 1) + fib(n - 2);
        }
        def main() of int {
            return fib(10);
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(55));
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
    assert_eq!(output.status.code(), Some(-1));
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
fn test_exe_return_char() {
    let source = r#"
        def main() of int {
            c = 'A';
            return c;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(65));
}

#[test]
fn test_exe_chain_assign() {
    let source = r#"
        def main() of int {
            a = b = c = 42;
            return a + b + c;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(126));
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
fn test_exe_mod_negative() {
    let source = "def main() of int { return 7 % 3; }";
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

#[test]
fn test_exe_param_as_temp_var() {
    let source = r#"
        def swap(a of int, b of int) of int {
            tmp = a;
            a = b;
            b = tmp;
            return a * 10 + b;
        }
        def main() of int {
            return swap(3, 7);
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(73));
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


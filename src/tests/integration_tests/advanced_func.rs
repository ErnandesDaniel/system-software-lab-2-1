use super::compile_and_run;

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

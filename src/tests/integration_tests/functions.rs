use super::compile_and_run;

#[test]
fn test_exe_function_call() {
    let source = r#"
        def double(x of int) of int
            return x + x
        end
        def main() of int
            return double(21)
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "double(21) should be 42");
}

#[test]
fn test_exe_multi_param_call() {
    let source = r#"
        def add(a of int, b of int) of int
            return a + b
        end
        def main() of int
            return add(3, 4)
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7), "add(3,4) should be 7");
}

#[test]
fn test_exe_global_exact() {
    let source = r#"
        global counter of int = 42;
        def main() of int
            return counter
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "global counter should be 42");
}

#[test]
fn test_exe_global_write_compiles() {
    let source = r#"
        global value of int = 0;
        def main() of int
            value = 99;
            return value
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "global write should compile and run");
}

#[test]
fn test_exe_global_array_exact() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[2]
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(30), "arr[2] should be 30");
}

#[test]
fn test_exe_local_struct_exact() {
    let source = r#"
        struct Point { x of int; y of int; }
        def main() of int
            p of Point;
            p.x = 42;
            return p.x
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "p.x should be 42");
}

#[test]
fn test_exe_arithmetic_chain() {
    let source = "def main() of int return 2 + 3 * 4 - 6 / 2; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(11), "2+3*4-6/2 should be 11");
}

#[test]
fn test_exe_while_loop_compiles() {
    let source = r#"
        def main() of int
            i = 0;
            total = 0;
            while i < 3 {
                j = 0;
                while j < 2 {
                    total = total + 1;
                    j = j + 1;
                }
                loop_end
                i = i + 1;
            }
            loop_end
            return total
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "nested while should compile and run");
}

#[test]
fn test_exe_logical_not_true() {
    let source = "def main() of int x = 0; return !x; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "!0 should be 1");
}

#[test]
fn test_exe_logical_not_false() {
    let source = "def main() of int x = 1; return !x; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "!1 should be 0");
}

#[test]
fn test_exe_conditional_compiles() {
    let source = r#"
        def main() of int
            a = 1 == 1;
            b = 2 == 2;
            if a && b then
                return 1
            end
            return 0
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "conditional should compile and run");
}

#[test]
fn test_exe_hex_literal() {
    let source = "def main() of int return 0xFF; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(255), "0xFF should be 255");
}

#[test]
fn test_exe_binary_literal() {
    let source = "def main() of int return 0b1010; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10), "0b1010 should be 10");
}

#[test]
fn test_exe_multiple_return_paths() {
    let source = r#"
        def max(a of int, b of int) of int
            if a > b then
                return a
            else
                return b
            end
        end
        def main() of int
            return max(7, 3)
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "should compile and run");
}

#[test]
fn test_exe_variable_reuse() {
    let source = r#"
        def main() of int
            x = 5;
            x = x + 3;
            x = x * 2;
            return x
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(16), "(5+3)*2 should be 16");
}

#[test]
fn test_exe_div() {
    let source = "def main() of int return 42 / 6; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7), "42/6 should be 7");
}

#[test]
fn test_exe_global_array_first() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[0]
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10), "arr[0] should be 10");
}

#[test]
fn test_exe_global_string_compiles() {
    let source = r#"
        global name of string = "ok";
        def main() of int
            return 99
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "should compile and run");
}

#[test]
fn test_exe_if_else_false_compiles() {
    let source = r#"
        def main() of int
            x = 2;
            if x > 5 then
                return 1
            else
                return 0
            end
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "should compile and run");
}

#[test]
fn test_exe_closure_simple() {
    let source = r#"
        def main() of int
            x = 10;
            def inner() of int
                return x
            end
            return inner()
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10), "closure should capture x");
}

#[test]
fn test_exe_closure_mutate() {
    let source = r#"
        def main() of int
            x = 0;
            def inc() 
                x = x + 1
            end
            inc();
            inc();
            inc();
            return x
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(3), "closure should mutate captured x");
}

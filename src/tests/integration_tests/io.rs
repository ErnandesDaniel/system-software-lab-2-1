use super::compile_and_run;
use super::compile_and_run_with_stdin;
use super::normalize_output;

#[test]
fn test_exe_break_aborts_loop() {
    let source = r#"
        import putchar
        def main() of int {
            i = 0;
            while (i < 5) {
                if (i == 3) { break; }
                putchar(65 + i);
                i = i + 1;
            }
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "ABC\n");
}

#[test]
fn test_exe_getchar_echo() {
    let source = r#"
        import getchar
        import putchar
        def main() of int {
            c = getchar();
            putchar(c);
            return 0;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "X");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "X");
}

#[test]
fn test_exe_getchar_echo_loop() {
    let source = r#"
        import getchar
        import putchar
        def main() of int {
            i = 0;
            while (i < 3) {
                c = getchar();
                putchar(c);
                i = i + 1;
            }
            return 0;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "XYZ");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "XYZ");
}

#[test]
fn test_exe_getchar_until_newline() {
    let source = r#"
        import getchar
        import putchar
        def main() of int {
            c = getchar();
            while (c != 10) {
                putchar(c);
                c = getchar();
            }
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "Hello\n");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "Hello\n");
}

#[test]
fn test_exe_pool_write_read_byte() {
    let source = r#"
        import putchar
        global pool of string = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        def main() of int {
            pool[100] = 65;
            putchar(pool[100]);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "A\n");
}

#[test]
fn test_exe_pool_write_read_multiple() {
    let source = r#"
        import putchar
        global pool of string = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        def main() of int {
            pool[0] = 72;
            pool[1] = 105;
            pool[2] = 0;
            putchar(pool[0]);
            putchar(pool[1]);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "Hi\n");
}

#[test]
fn test_exe_read_line_into_pool() {
    let source = r#"
        import getchar
        import putchar
        global pool of string = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        def main() of int {
            di = 0;
            c = getchar();
            while (c != 10) {
                pool[di] = c;
                di = di + 1;
                c = getchar();
            }
            pool[di] = 0;
            putchar(pool[0]);
            putchar(pool[1]);
            putchar(pool[2]);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "Hi!\n");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "Hi!\n");
}

#[test]
fn test_exe_multi_function_io() {
    let source = r#"
        import putchar
        def emit(a of int) {
            putchar(a);
        }
        def main() of int {
            emit(72);
            emit(105);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "Hi\n");
}

#[test]
fn test_exe_if_else_putchar() {
    let source = r#"
        import putchar
        def main() of int {
            x = 1;
            if (x == 1) {
                putchar(65);
            } else {
                putchar(66);
            }
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "A\n");
}

#[test]
fn test_exe_logical_or_shortcircuit() {
    let source = r#"
        import getchar
        import putchar
        def main() of int {
            c = getchar();
            if (c == 0 || c == 10) {
                putchar(79);
                putchar(75);
            } else {
                putchar(69);
            }
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "\n");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "OK\n");
}

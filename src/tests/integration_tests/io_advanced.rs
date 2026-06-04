use super::compile_and_run;
use super::compile_and_run_with_stdin;
use super::normalize_output;

const POOL_2K: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";

fn eof_source() -> String {
    "import getchar\nimport putchar\ndef main() of int {\n    c = getchar();\n    if (c == -1) {\n        putchar(69);\n        putchar(79);\n        putchar(70);\n    } else {\n        putchar(79);\n        putchar(75);\n    }\n    putchar(10);\n    return 0;\n}".to_string()
}

#[test]
fn test_exe_eof_detection() {
    let source = eof_source();
    let output = compile_and_run_with_stdin(&source, "");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "EOF\n");
}

#[test]
fn test_exe_while_loop_sum_stdin() {
    let source = r#"
        import getchar
        import putchar
        def main() of int {
            sum = 0;
            c = getchar();
            while (c != 10 && c != -1) {
                sum = sum + (c - 48);
                c = getchar();
            }
            putchar(48 + sum);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "123\n");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "6\n");
}

#[test]
fn test_exe_pool_readline_echo() {
    let source = r#"
        import getchar
        import putchar
        global pool of string = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        def main() of int {
            di = 0;
            c = getchar();
            while (c != 10 && c != -1) {
                pool[di] = c;
                di = di + 1;
                c = getchar();
            }
            pool[di] = 0;
            i = 0;
            while (pool[i] != 0) {
                putchar(pool[i]);
                i = i + 1;
            }
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "hello\n");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "hello\n");
}

#[test]
fn test_exe_pool_cat() {
    let source = r#"
        import getchar
        import putchar
        global pool of string
        def main() of int {
            pool = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";
            total = 0;
            c = getchar();
            while (c != -1) {
                pool[total] = c;
                total = total + 1;
                c = getchar();
            }
            pool[total] = 0;
            i = 0;
            while (pool[i] != 0) {
                putchar(pool[i]);
                i = i + 1;
            }
            return total;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "Hi!\n");
    assert_eq!(output.status.code(), Some(4), "pool_cat should count 4 chars (H,i,!,\\n)");
    assert_eq!(normalize_output(&output.stdout), "Hi!\n");
}

#[test]
fn test_exe_empty_stdin_immediate_eof() {
    let source = r#"
        import getchar
        def main() of int {
            c = getchar();
            return c;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "");
    assert_eq!(output.status.code(), Some(-1i32 as u32 as i32));
}

#[test]
fn test_exe_stdin_line_count() {
    let source = r#"
        import getchar
        import putchar
        def main() of int {
            lines = 0;
            c = getchar();
            while (c != -1) {
                if (c == 10) {
                    lines = lines + 1;
                }
                c = getchar();
            }
            putchar(48 + lines);
            putchar(10);
            return lines;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "a\nb\nc\n");
    assert_eq!(output.status.code(), Some(3));
    assert_eq!(normalize_output(&output.stdout), "3\n");
}

#[test]
fn test_exe_stdin_partial_read() {
    let source = r#"
        import getchar
        import putchar
        def main() of int {
            buf = "xxxxxxxxxxxx";
            i = 0;
            c = getchar();
            while (c != 10 && c != -1) {
                buf[i] = c;
                i = i + 1;
                c = getchar();
            }
            buf[i] = 0;
            j = 0;
            while (buf[j] != 0) {
                putchar(buf[j]);
                j = j + 1;
            }
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "partial");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "partial\n");
}

#[test]
fn test_exe_multi_eof_cycles() {
    let source = r#"
        import getchar
        import putchar
        def main() of int {
            count = 0;
            c = getchar();
            while (count < 3) {
                if (c == -1) {
                    putchar(69);
                    count = count + 1;
                }
                c = getchar();
            }
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run_with_stdin(source, "");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
}

#[test]
fn test_exe_daemon_create_get() {
    let source = r#"
        import getchar
        import putchar
        import puts
        import printf
        def main() of int {
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
}

#[test]
fn test_exe_daemon_set_delete() {
    let source = r#"
        import getchar
        import putchar
        import puts
        import printf
        global pool of string = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx"
        def main() of int {
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
}

#[test]
fn test_exe_daemon_list_exit() {
    let source = r#"
        import getchar
        import putchar
        import printf
        import puts
        def main() of int {
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
}

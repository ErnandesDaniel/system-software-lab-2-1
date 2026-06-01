use super::compile_and_run;
use super::compile_and_run_with_stdin;
use super::normalize_output;

const POOL_2K: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";

#[test]
fn test_exe_eof_detection() {
    let source = "import getchar\nimport putchar\ndef main() of int\n    c = getchar()\n    if c == -1 then {\n        putchar(69);\n        putchar(79);\n        putchar(70)\n    } else {\n        putchar(79);\n        putchar(75)\n    }\n    putchar(10)\n    return 0\nend\n";
    let output = compile_and_run_with_stdin(source, "");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "EOF\n");
}

#[test]
fn test_exe_while_loop_sum_stdin() {
    let source = r#"
        import getchar
        import putchar
        def main() of int
            sum = 0
            c = getchar()
            while c != 10 && c != -1 {
                sum = sum + (c - 48)
                c = getchar()
            }
            putchar(48 + sum)
            putchar(10)
            return 0
        end
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
        def main() of int
            di = 0
            c = getchar()
            while c != 10 && c != -1 {
                pool[di] = c
                di = di + 1
                c = getchar()
            }
            pool[di] = 10
            di = 0
            c2 = pool[di]
            while c2 != 10 && c2 != -1 {
                putchar(c2)
                di = di + 1
                c2 = pool[di]
            }
            putchar(10)
            return 0
        end
    "#;
    let output = compile_and_run_with_stdin(source, "Hello World\n");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "Hello World\n");
}

#[test]
fn test_exe_nested_while_io() {
    let source = r#"
        import putchar
        def main() of int
            i = 0
            while i < 3 {
                j = 0
                while j < 3 {
                    putchar(65 + j)
                    j = j + 1
                }
                i = i + 1
            }
            putchar(10)
            return 0
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "ABCABCABC\n");
}

#[test]
fn test_exe_break_nested_loop() {
    let source = r#"
        import putchar
        def main() of int
            i = 0
            while i < 3 {
                j = 0
                while j < 5 {
                    if j == 2 then { break; }
                    putchar(65 + j)
                    j = j + 1
                }
                i = i + 1
            }
            putchar(10)
            return 0
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "ABABAB\n");
}

#[test]
fn test_exe_globals_read_write() {
    let source = r#"
        import putchar
        global g of int = 42
        def main() of int
            putchar(48 + g)
            putchar(10)
            return 0
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "Z\n");
}

#[test]
fn test_exe_getchar_eof_then_valid() {
    let source = r#"
        import getchar
        import putchar
        def main() of int
            c1 = getchar()
            if c1 == -1 then
                putchar(69)
            else
                putchar(c1)
            end
            c2 = getchar()
            if c2 == -1 then
                putchar(69)
            else
                putchar(c2)
            end
            putchar(10)
            return 0
        end
    "#;
    let output = compile_and_run_with_stdin(source, "X");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    let stdout = normalize_output(&output.stdout);
    assert_eq!(stdout, "XE\n", "got: {}", stdout);
}

#[test]
fn test_mylang_simple() {
    let source = "import getchar\nimport putchar\ndef main() of int\n    c = getchar()\n    putchar(c)\n    putchar(10)\n    return 0\nend\n";
    let output = compile_and_run_with_stdin(source, "A\n");
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "A\n");
}

#[test]
fn test_mylang_break() {
    let source = "import putchar\ndef main() of int\n    i = 0\n    while i < 5 {\n        if i == 3 then { break; }\n        putchar(65 + i)\n        i = i + 1\n    }\n    putchar(10)\n    return 0\nend\n";
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "ABC\n");
}

#[test]
fn test_mylang_globals() {
    let source = "import putchar\ndef main() of int\n    putchar(65)\n    putchar(10)\n    return 0\nend\n";
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "A\n");
}

#[test]
fn test_mylang_multi() {
    let source = "import putchar\ndef foo(x of int) of int\n    putchar(x)\n    return 0\nend\ndef main() of int\n    foo(65)\n    putchar(10)\n    return 0\nend\n";
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "A\n");
}

#[test]
fn test_mylang_pool() {
    let source = &format!(
        "import putchar\nglobal pool of string = \"{}\"\ndef main() of int\n    pool[100] = 65\n    putchar(pool[100])\n    putchar(10)\n    return 0\nend\n",
        POOL_2K
    );
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "A\n");
}

#[test]
fn test_mylang_init() {
    let source = &format!(
        "import putchar\nglobal pool of string = \"{}\"\ndef main() of int\n    pool[1792] = 69\n    pool[1793] = 120\n    pool[1794] = 105\n    pool[1795] = 116\n    pool[1796] = 0\n    putchar(pool[1792])\n    putchar(pool[1793])\n    putchar(pool[1794])\n    putchar(pool[1795])\n    putchar(10)\n    return 0\nend\n",
        POOL_2K
    );
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "Exit\n");
}

#[test]
fn test_mylang_high() {
    let source = &format!(
        "import putchar\nglobal pool of string = \"{}\"\ndef main() of int\n    pool[1792] = 69\n    pool[1793] = 120\n    pool[1794] = 105\n    putchar(pool[1792])\n    putchar(pool[1793])\n    putchar(pool[1794])\n    putchar(10)\n    return 0\nend\n",
        POOL_2K
    );
    let output = compile_and_run(source);
    assert!(output.status.success(), "exit code: {:?}", output.status.code());
    assert_eq!(normalize_output(&output.stdout), "Exi\n");
}

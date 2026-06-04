use super::compile_and_run;
use super::normalize_output;

#[test]
fn test_coro_return_zero_on_empty() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import get_coroutine_state
        import putchar
        coroutine empty() of int {
            return 0;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            st = get_coroutine_state(0);
            if (st == -1) {
                putchar(79);
                putchar(75);
            } else {
                putchar(66);
                putchar(65);
            }
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "OK\n");
}

#[test]
fn test_coro_state_transitions() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import get_coroutine_state
        import putchar
        coroutine staged() of int {
            yield;
            return 99;
        }
        def main() of int {
            coro_init();
            s0 = get_coroutine_state(0);
            putchar(48 + s0);
            resume_coroutine(0);
            s1 = get_coroutine_state(0);
            putchar(48 + s1);
            resume_coroutine(0);
            s2 = get_coroutine_state(0);
            putchar(48 + s2);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "01/\n");
}

#[test]
fn test_coro_two_without_io() {
    let source = r#"
        import resume_coroutine
        import coro_init
        coroutine add5() of int {
            return 5;
        }
        coroutine add7() of int {
            return 7;
        }
        def main() of int {
            coro_init();
            r1 = resume_coroutine(0);
            r2 = resume_coroutine(1);
            return r1 + r2;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(12));
}

#[test]
fn test_coro_state_after_completion() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        import get_coroutine_state
        coroutine single() of int {
            putchar(83);
            return 0;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            st = get_coroutine_state(0);
            putchar(48 + st);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "S/\n");
}

#[test]
fn test_coro_initial_state() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        import get_coroutine_state
        coroutine idle() of int {
            return 0;
        }
        def main() of int {
            coro_init();
            st = get_coroutine_state(0);
            putchar(48 + st);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "0\n");
}

#[test]
fn test_coro_already_done_returns_one() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine once() of int {
            return 5;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            r = resume_coroutine(0);
            putchar(48 + r);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "1\n");
}

use super::compile_and_run;
use super::normalize_output;

#[test]
fn test_coro_simple_exact() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine worker() of int {
            putchar(49);
            yield;
            putchar(50);
            return 0;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            resume_coroutine(0);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "12\n");
}

#[test]
fn test_coro_multi_yield_exact() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine counter() of int {
            i = 0;
            yield;
            i = 10;
            yield;
            i = 20;
            return i;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            resume_coroutine(0);
            result = resume_coroutine(0);
            putchar(48 + result / 10);
            putchar(48 + result % 10);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "20\n");
}

#[test]
fn test_coro_return_value() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine worker() of int {
            return 99;
        }
        def main() of int {
            coro_init();
            result = resume_coroutine(0);
            putchar(48 + result / 10);
            putchar(48 + result % 10);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "99\n");
}

#[test]
fn test_coro_no_yield_immediate_return() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine fast() of int {
            return 77;
        }
        def main() of int {
            coro_init();
            r = resume_coroutine(0);
            putchar(48 + r / 10);
            putchar(48 + r % 10);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "77\n");
}

#[test]
fn test_coro_two_with_putchar() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine alpha() of int {
            putchar(65);
            yield;
            putchar(66);
            return 0;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            resume_coroutine(0);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "AB\n");
}

#[test]
fn test_coro_three_yields() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine seq() of int {
            putchar(49);
            yield;
            putchar(50);
            yield;
            putchar(51);
            yield;
            putchar(52);
            return 0;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            resume_coroutine(0);
            resume_coroutine(0);
            resume_coroutine(0);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "1234\n");
}

#[test]
fn test_coro_multiple_resume_after_done() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine single() of int {
            putchar(65);
            return 0;
        }
        def main() of int {
            coro_init();
            r1 = resume_coroutine(0);
            r2 = resume_coroutine(0);
            r3 = resume_coroutine(0);
            putchar(48 + r1);
            putchar(48 + r2);
            putchar(48 + r3);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "A011\n");
}

#[test]
fn test_coro_return_diff_values() {
    let source = r#"
        import resume_coroutine
        import coro_init
        coroutine gen() of int {
            yield;
            return 77;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            r = resume_coroutine(0);
            return r;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(77));
}

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
fn test_coro_with_putchar_inside() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine printer() of int {
            putchar(80);
            yield;
            putchar(81);
            yield;
            putchar(82);
            return 0;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            resume_coroutine(0);
            putchar(45);
            resume_coroutine(0);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "PQ-R\n");
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

#[test]
fn test_coro_yield_twice_then_return() {
    let source = r#"
        import resume_coroutine
        import coro_init
        import putchar
        coroutine stage() of int {
            putchar(65);
            yield;
            putchar(66);
            yield;
            putchar(67);
            return 3;
        }
        def main() of int {
            coro_init();
            resume_coroutine(0);
            resume_coroutine(0);
            r = resume_coroutine(0);
            putchar(48 + r);
            putchar(10);
            return 0;
        }
    "#;
    let output = compile_and_run(source);
    assert_eq!(normalize_output(&output.stdout), "ABC3\n");
}

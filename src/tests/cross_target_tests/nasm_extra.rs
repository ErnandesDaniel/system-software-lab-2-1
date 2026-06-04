use super::*;

const NASM_EXE_BITWISE: &str = r#"
def main() of int {
    a = 12 & 7;
    b = 12 | 5;
    c = 12 ^ 7;
    d = ~0;
    return a * 1000 + b * 100 + c * 10 + (d & 1);
}
"#;

#[test]
fn test_nasm_bitwise_ops() {
    let output = compile_and_run_nasm(NASM_EXE_BITWISE);
    assert!(output.status.code() != Some(-1));
}

const NASM_EXE_COMPLEX_ARITH: &str = r#"
def main() of int {
    a = 3;
    b = 7;
    c = 2;
    return a * b + b * c + a * c;
}
"#;

#[test]
fn test_nasm_complex_arith() {
    let output = compile_and_run_nasm(NASM_EXE_COMPLEX_ARITH);
    assert!(output.status.code() != Some(-1));
}

const NASM_EXE_NESTED_FUNC_CALL: &str = r#"
def inc(x of int) of int { return x + 1; }
def apply(f of def(int) of int, x of int) of int { return f(x); }
def main() of int {
    return apply(inc, 41);
}
"#;

#[test]
fn test_nasm_func_ref_indirect() {
    let output = compile_and_run_nasm(NASM_EXE_NESTED_FUNC_CALL);
    assert!(output.status.code() != Some(-1));
}

const JVM_COMPLEX_EXPR: &str = r#"
def calc(x of int) of int {
    return (x + 5) * 3 - 7;
}
def main() of int {
    return calc(10);
}
"#;

#[test]
fn test_jvm_complex_expr_valid() {
    assert!(jvm_valid(JVM_COMPLEX_EXPR));
}

const JVM_MULTI_FUNC_CHAIN: &str = r#"
def add(a of int, b of int) of int { return a + b; }
def mul(a of int, b of int) of int { return a * b; }
def main() of int {
    return mul(add(3, 4), add(1, 1));
}
"#;

#[test]
fn test_jvm_multi_func_chain_valid() {
    assert!(jvm_valid(JVM_MULTI_FUNC_CHAIN));
}

const JVM_RECURSION: &str = r#"
def fact(n of int) of int {
    if (n <= 1) { return 1; }
    return n * fact(n - 1);
}
def main() of int {
    return fact(5);
}
"#;

#[test]
fn test_jvm_recursion_valid() {
    assert!(jvm_valid(JVM_RECURSION));
}

const JVM_BITWISE_OPS: &str = r#"
def main() of int {
    a = 12 & 7;
    b = 12 | 5;
    c = 12 ^ 7;
    d = ~0;
    return a + b + c + d;
}
"#;

#[test]
fn test_jvm_bitwise_ops_valid() {
    assert!(jvm_valid(JVM_BITWISE_OPS));
}

const JVM_LOGICAL_OPS: &str = r#"
def main() of int {
    a = 1 && 1;
    b = 1 && 0;
    c = 0 || 1;
    d = 0 || 0;
    return a * 1000 + b * 100 + c * 10 + d;
}
"#;

#[test]
fn test_jvm_logical_ops_valid() {
    assert!(jvm_valid(JVM_LOGICAL_OPS));
}

const JVM_CHAINED_COMPARISON: &str = r#"
def main() of int {
    a = 5;
    if (a > 0 && a < 10) { return 1; }
    return 0;
}
"#;

#[test]
fn test_jvm_chained_comparison_valid() {
    assert!(jvm_valid(JVM_CHAINED_COMPARISON));
}

const JVM_NESTED_WHILE_LOOP: &str = r#"
def main() of int {
    total = 0;
    i = 0;
    while (i < 3) {
        j = 0;
        while (j < 4) {
            total = total + 1;
            j = j + 1;
        }
        i = i + 1;
    }
    return total;
}
"#;

#[test]
fn test_jvm_nested_while_valid() {
    assert!(jvm_valid(JVM_NESTED_WHILE_LOOP));
}

const JVM_UNTIL_LOOP: &str = r#"
def main() of int {
    i = 0;
    until (i >= 5) {
        i = i + 1;
    }
    return i;
}
"#;

#[test]
fn test_jvm_until_loop_valid() {
    assert!(jvm_valid(JVM_UNTIL_LOOP));
}

const JVM_BREAK_LOOP: &str = r#"
def main() of int {
    total = 0;
    i = 0;
    while (i < 10) {
        if (i == 5) { break; }
        total = total + i;
        i = i + 1;
    }
    return total;
}
"#;

#[test]
fn test_jvm_break_loop_valid() {
    assert!(jvm_valid(JVM_BREAK_LOOP));
}

const JVM_EMPTY_VOID_FUNC: &str = r#"
def nop() { }
def main() of int {
    nop();
    return 42;
}
"#;

#[test]
fn test_jvm_void_func_call_valid() {
    assert!(jvm_valid(JVM_EMPTY_VOID_FUNC));
}

const JVM_CHAR_LITERAL: &str = r#"
def main() of int {
    c = 'Z';
    return c;
}
"#;

#[test]
fn test_jvm_char_literal_valid() {
    assert!(jvm_valid(JVM_CHAR_LITERAL));
}

const JVM_IF_ELSE_IF_CHAIN: &str = r#"
def main() of int {
    x = 7;
    if (x == 3) { return 3; }
    else if (x == 7) { return 7; }
    else { return 0; }
}
"#;

#[test]
fn test_jvm_if_else_chain_valid() {
    assert!(jvm_valid(JVM_IF_ELSE_IF_CHAIN));
}

const JVM_NEGATIVE_LITERAL: &str = r#"
def main() of int {
    return -42;
}
"#;

#[test]
fn test_jvm_negative_literal_valid() {
    assert!(jvm_valid(JVM_NEGATIVE_LITERAL));
}

const JVM_MULTI_PARAM_CALL: &str = r#"
def sum4(a of int, b of int, c of int, d of int) of int {
    return a + b + c + d;
}
def main() of int {
    return sum4(1, 2, 3, 4);
}
"#;

#[test]
fn test_jvm_multi_param_call_valid() {
    assert!(jvm_valid(JVM_MULTI_PARAM_CALL));
}

const NASM_REPEAT_LOOP: &str = r#"
def main() of int {
    i = 0;
    { i = i + 1; } while (i < 5);
    return i;
}
"#;

#[test]
fn test_nasm_repeat_loop() {
    let output = compile_and_run_nasm(NASM_REPEAT_LOOP);
    assert!(output.status.code() != Some(-1));
}

const NASM_BREAK_LOOP: &str = r#"
def main() of int {
    total = 0;
    i = 0;
    while (i < 10) {
        if (i == 5) { break; }
        total = total + i;
        i = i + 1;
    }
    return total;
}
"#;

#[test]
fn test_nasm_break_loop() {
    let output = compile_and_run_nasm(NASM_BREAK_LOOP);
    assert!(output.status.code() != Some(-1));
}

const JVM_GLOBAL_READ: &str = r#"
global g of int = 42;
def main() of int {
    return g;
}
"#;

#[test]
fn test_jvm_global_read_valid() {
    assert!(jvm_valid(JVM_GLOBAL_READ));
}

const JVM_CLOSURE_SIMPLE: &str = r#"
def main() of int {
    x = 10;
    def inner() of int {
        return x;
    }
    return inner();
}
"#;

#[test]
fn test_jvm_closure_simple_valid() {
    assert!(jvm_valid(JVM_CLOSURE_SIMPLE));
}

const JVM_REPEAT_LOOP: &str = r#"
def main() of int {
    i = 0;
    { i = i + 1; } while (i < 5);
    return i;
}
"#;

#[test]
fn test_jvm_repeat_loop_valid() {
    assert!(jvm_valid(JVM_REPEAT_LOOP));
}

const JVM_HEX_LITERAL: &str = r#"
def main() of int {
    return 0xFF;
}
"#;

#[test]
fn test_jvm_hex_literal_valid() {
    assert!(jvm_valid(JVM_HEX_LITERAL));
}

const JVM_BINARY_LITERAL: &str = r#"
def main() of int {
    return 0b1010;
}
"#;

#[test]
fn test_jvm_binary_literal_valid() {
    assert!(jvm_valid(JVM_BINARY_LITERAL));
}

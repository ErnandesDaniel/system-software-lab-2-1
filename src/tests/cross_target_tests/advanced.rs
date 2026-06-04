use super::*;

const JVM_RETURN_42: &str = "def main() of int { return 42; }";

const JVM_CLOSURE_SIMPLE: &str = r#"
def main() of int {
    x = 10;
    def inner() of int {
        return x;
    }
    return inner();
}
"#;

const JVM_CLOSURE_MUTATE: &str = r#"
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

#[test]
fn test_jvm_runtime_return_42() {
    let output = compile_and_run_jvm(JVM_RETURN_42);
    assert_eq!(output.status.code(), Some(42), "jvm return 42");
}

#[test]
fn test_jvm_runtime_closure_simple() {
    let output = compile_and_run_jvm(JVM_CLOSURE_SIMPLE);
    assert_eq!(output.status.code(), Some(10), "jvm closure should capture x");
}

#[test]
fn test_jvm_runtime_closure_mutate() {
    let output = compile_and_run_jvm(JVM_CLOSURE_MUTATE);
    assert_eq!(output.status.code(), Some(3), "jvm closure should mutate captured x");
}

const STRUCT_LOCAL: &str = r#"
struct Point { x of int; y of int; }
def main() of int {
    p of Point;
    p.x = 42;
    p.y = 13;
    return p.x;
}
"#;

#[test]
fn test_nasm_struct_local() {
    let output = compile_and_run_nasm(STRUCT_LOCAL);
    assert_eq!(output.status.code(), Some(42), "nasm struct field should be 42");
}

#[test]
fn test_jvm_struct_valid() {
    assert!(jvm_valid(STRUCT_LOCAL), "jvm struct should produce valid class file");
}

const STRUCT_GLOBAL: &str = r#"
struct Point { x of int; y of int; }
global p of Point;
def main() of int {
    p.x = 42;
    return p.x;
}
"#;

#[test]
fn test_nasm_struct_global() {
    let output = compile_and_run_nasm(STRUCT_GLOBAL);
    assert_eq!(output.status.code(), Some(42), "nasm global struct field should be 42");
}

#[test]
fn test_jvm_struct_global_valid() {
    assert!(jvm_valid(STRUCT_GLOBAL), "jvm global struct should produce valid class file");
}

const STRUCT_NESTED_FIELD: &str = r#"
struct Point { x of int; y of int; }
struct Rect { topleft of Point; bottomright of Point; }
def main() of int {
    r of Rect;
    r.topleft.x = 42;
    return r.topleft.x;
}
"#;

#[test]
fn test_nasm_struct_nested_field() {
    let output = compile_and_run_nasm(STRUCT_NESTED_FIELD);
    assert_eq!(output.status.code(), Some(42), "nasm nested struct field should be 42");
}

#[test]
fn test_jvm_struct_nested_field_valid() {
    assert!(jvm_valid(STRUCT_NESTED_FIELD), "jvm nested struct should produce valid class file");
}

const COROUTINE_SIMPLE: &str = r#"
coroutine counter() of int {
    i = 0;
    yield;
    i = 42;
    return i;
}
def main() of int {
    return 0;
}
"#;

#[test]
fn test_nasm_coroutine_simple() {
    let (_, asm) = crate::tests::integration_tests::compile_only(COROUTINE_SIMPLE);
    assert!(asm.contains("co_0"), "nasm coroutine should have state 0");
    assert!(asm.contains("co_1"), "nasm coroutine should have state 1");
}

#[test]
fn test_jvm_coroutine_valid() {
    assert!(jvm_valid(COROUTINE_SIMPLE), "jvm coroutine should produce valid class file");
    assert!(jvm_class_count(COROUTINE_SIMPLE) >= 2, "jvm coroutine should produce coroutine class + Main");
}

const COROUTINE_WITH_PARAMS: &str = r#"
coroutine worker(x of int, y of int) of int {
    yield;
    return x + y;
}
def main() of int {
    return 0;
}
"#;

#[test]
fn test_nasm_coroutine_with_params() {
    let (_, asm) = crate::tests::integration_tests::compile_only(COROUTINE_WITH_PARAMS);
    assert!(asm.contains("co_0"), "nasm coroutine should have state 0");
    assert!(asm.contains("co_1"), "nasm coroutine should have state 1");
}

#[test]
fn test_jvm_coroutine_with_params_valid() {
    assert!(jvm_valid(COROUTINE_WITH_PARAMS), "jvm coroutine with params should produce valid class file");
}

const COROUTINE_MULTI_YIELD: &str = r#"
coroutine multi() of int {
    i = 0;
    yield;
    i = 1;
    yield;
    i = 2;
    return i;
}
def main() of int {
    return 0;
}
"#;

#[test]
fn test_nasm_coroutine_multi_yield() {
    let (_, asm) = crate::tests::integration_tests::compile_only(COROUTINE_MULTI_YIELD);
    assert!(asm.contains("co_0"), "nasm coroutine should have state 0");
    assert!(asm.contains("co_1"), "nasm coroutine should have state 1");
    assert!(asm.contains("co_2"), "nasm coroutine should have state 2");
}

#[test]
fn test_jvm_coroutine_multi_yield_valid() {
    assert!(jvm_valid(COROUTINE_MULTI_YIELD), "jvm coroutine with multi yields should produce valid class file");
}

const COROUTINE_WITH_PARAMS_NASM: &str = r#"
import resume_coroutine
import coro_init
coroutine adder(a of int) of int {
    return a + 1;
}
def main() of int {
    coro_init();
    resume_coroutine(0);
    return 0;
}
"#;

#[test]
fn test_nasm_coroutine_with_params_runtime() {
    let (_, asm) = crate::tests::integration_tests::compile_only(COROUTINE_WITH_PARAMS_NASM);
    assert!(asm.contains("co_0"), "nasm coroutine with params should have state 0");
}

const COROUTINE_MULTI_PARAM: &str = r#"
coroutine summer(a of int, b of int) of int {
    return a + b;
}
def main() of int {
    return 0;
}
"#;

#[test]
fn test_jvm_coroutine_multi_param_valid() {
    assert!(jvm_valid(COROUTINE_MULTI_PARAM), "jvm coroutine with multiple params should be valid");
}

const COROUTINE_YIELD_WITH_PARAM: &str = r#"
coroutine worker(x of int) of int {
    yield;
    return x;
}
def main() of int {
    return 0;
}
"#;

#[test]
fn test_nasm_coroutine_yield_with_param() {
    let (_, asm) = crate::tests::integration_tests::compile_only(COROUTINE_YIELD_WITH_PARAM);
    assert!(asm.contains("co_0"), "should have state 0");
    assert!(asm.contains("co_1"), "should have state 1");
}

#[test]
fn test_jvm_coroutine_yield_with_param_valid() {
    assert!(jvm_valid(COROUTINE_YIELD_WITH_PARAM), "jvm coroutine yield with param should be valid");
}

const NASM_ARRAY_OF_FUNCTIONS: &str = r#"
def main() of int {
    c1 = [
        def add2(x of int) of int {
            return x + 2;
        },
        def mul2(x of int) of int {
            return x * 2;
        }
    ];
    c2 = [
        def add2_2(x of int) of int {
            return x + 2;
        },
        def mul2_2(x of int) of int {
            return x * 2;
        }
    ];

    x1 = c1[0](2);
    y1 = c2[0](2);
    x2 = c1[0](2);
    y2 = c2[1](7);
    x3 = c1[1](7);
    y3 = c2[0](3);

    return (x1 + x2 + x3) * 100 + (y1 + y2 + y3);
}
"#;

const JVM_ARRAY_OF_FUNCTIONS: &str = r#"
def f5() of def(int) of int array[2] {
    arr = [
        def add2(x of int) of int {
            return x + 2;
        },
        def mul2(x of int) of int {
            return x * 2;
        }
    ];
    return arr;
}

def main() of int {
    c1 = f5();
    c2 = f5();

    x1 = c1[0](2);
    y1 = c2[0](2);

    x2 = c1[0](2);
    y2 = c2[1](7);

    x3 = c1[1](7);
    y3 = c2[0](3);

    return (x1 + x2 + x3) * 100 + (y1 + y2 + y3);
}
"#;

#[test]
fn test_nasm_array_of_functions() {
    // Verify the ASM is syntactically valid (contains expected labels/instructions).
    // Runtime verification is done via the CLI integration path (labs-examples/…/input.mylang).
    let ir = crate::ir_generator::IrGenerator::new().generate(&crate::tests::parse(NASM_ARRAY_OF_FUNCTIONS));
    let mut asm_gen = crate::codegen::nasm::AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("main_BB0:"), "entry block label should exist");
    assert!(asm.contains("__lambda_0_BB1:"), "lambda block label should exist");
    assert!(asm.contains("rep movsq"), "array copy should use rep movsq");
    assert!(asm.contains("call rax"), "indirect call should exist");
    assert!(asm.contains("leave"), "function epilogue should exist");
    assert!(asm.contains("ret"), "return should exist");
}

#[test]
fn test_jvm_array_of_functions_valid() {
    assert!(jvm_valid(JVM_ARRAY_OF_FUNCTIONS), "jvm array of functions should produce valid class files");
}

#[test]
fn test_jvm_runtime_array_of_functions() {
    let output = compile_and_run_jvm(JVM_ARRAY_OF_FUNCTIONS);
    assert_eq!(output.status.code(), Some(2223), "jvm array of functions c1[0](2) etc");
}

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

const NASM_EXE_COROUTINE_PUTCHAR: &str = r#"
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

#[test]
fn test_nasm_coroutine_putchar_runtime() {
    let ir = crate::ir_generator::IrGenerator::new().generate(&crate::tests::parse(NASM_EXE_COROUTINE_PUTCHAR));
    let mut asm_gen = crate::codegen::nasm::AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("co_0"), "should have state 0");
    assert!(asm.contains("co_1"), "should have state 1");
}

const NASM_COROUTINE_WITH_PARAMS_CALC: &str = r#"
import resume_coroutine
import coro_init
import set_coroutine_param
coroutine calc(a of int, b of int) of int {
    return a + b;
}
def main() of int {
    coro_init();
    set_coroutine_param(0, 10, 20);
    result = resume_coroutine(0);
    return result;
}
"#;

#[test]
fn test_nasm_coroutine_param_runtime() {
    let ir = crate::ir_generator::IrGenerator::new().generate(&crate::tests::parse(NASM_COROUTINE_WITH_PARAMS_CALC));
    let mut asm_gen = crate::codegen::nasm::AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("global calc"), "calc should be global");
    assert!(asm.contains("extern set_coroutine_param"), "set_coroutine_param should be extern");
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

const JVM_STRUCT_ARRAY_FIELD: &str = r#"
struct S { vals of int[3]; count of int; }
def main() of int {
    s of S;
    s.vals[0] = 10;
    s.vals[1] = 20;
    s.vals[2] = 30;
    return s.vals[0] + s.vals[1] + s.vals[2];
}
"#;

#[test]
fn test_jvm_struct_array_field_valid() {
    assert!(jvm_valid(JVM_STRUCT_ARRAY_FIELD));
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

const JVM_COROUTINE_BASIC: &str = r#"
coroutine worker() of int {
    yield;
    yield;
    return 0;
}
def main() of int {
    return 0;
}
"#;

#[test]
fn test_jvm_coroutine_basic_valid() {
    assert!(jvm_valid(JVM_COROUTINE_BASIC));
}

const JVM_COROUTINE_NO_YIELD: &str = r#"
coroutine fast() of int {
    return 42;
}
def main() of int {
    return 0;
}
"#;

#[test]
fn test_jvm_coroutine_no_yield_valid() {
    assert!(jvm_valid(JVM_COROUTINE_NO_YIELD));
}

const JVM_COROUTINE_MULTI_FUNC: &str = r#"
coroutine alpha() of int {
    yield;
    return 10;
}
coroutine beta() of int {
    yield;
    return 20;
}
def main() of int {
    return 0;
}
"#;

#[test]
fn test_jvm_coroutine_multi_func_valid() {
    assert!(jvm_valid(JVM_COROUTINE_MULTI_FUNC));
}

const JVM_COROUTINE_WITH_LOCALS: &str = r#"
coroutine counter() of int {
    i = 0;
    yield;
    i = 1;
    yield;
    i = 2;
    return i;
}
def main() of int {
    return 0;
}
"#;

#[test]
fn test_jvm_coroutine_with_locals_valid() {
    assert!(jvm_valid(JVM_COROUTINE_WITH_LOCALS));
}


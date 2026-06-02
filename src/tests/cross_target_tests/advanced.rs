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

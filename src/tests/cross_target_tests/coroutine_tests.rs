use super::*;

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

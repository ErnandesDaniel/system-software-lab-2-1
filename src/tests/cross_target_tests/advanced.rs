use super::*;

const JVM_RETURN_42: &str = "def main() of int return 42; end";

const JVM_CLOSURE_SIMPLE: &str = r#"
def main() of int
    x = 10;
    def inner() of int
        return x
    end
    return inner()
end
"#;

const JVM_CLOSURE_MUTATE: &str = r#"
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
def main() of int
    p of Point;
    p.x = 42;
    p.y = 13;
    return p.x
end
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
def main() of int
    p.x = 42;
    return p.x
end
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
def main() of int
    r of Rect;
    r.topleft.x = 42;
    return r.topleft.x
end
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
coroutine counter() of int
    i = 0;
    yield;
    i = 42;
    return i
end
def main() of int
    return 0
end
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
coroutine worker(x of int, y of int) of int
    yield;
    return x + y
end
def main() of int
    return 0
end
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
coroutine multi() of int
    i = 0;
    yield;
    i = 1;
    yield;
    i = 2;
    return i
end
def main() of int
    return 0
end
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
coroutine adder(a of int) of int
    return a + 1
end
def main() of int
    coro_init()
    resume_coroutine(0)
    return 0
end
"#;

#[test]
fn test_nasm_coroutine_with_params_runtime() {
    let (_, asm) = crate::tests::integration_tests::compile_only(COROUTINE_WITH_PARAMS_NASM);
    assert!(asm.contains("co_0"), "nasm coroutine with params should have state 0");
}

const COROUTINE_MULTI_PARAM: &str = r#"
coroutine summer(a of int, b of int) of int
    return a + b
end
def main() of int
    return 0
end
"#;

#[test]
fn test_jvm_coroutine_multi_param_valid() {
    assert!(jvm_valid(COROUTINE_MULTI_PARAM), "jvm coroutine with multiple params should be valid");
}

const COROUTINE_YIELD_WITH_PARAM: &str = r#"
coroutine worker(x of int) of int
    yield;
    return x
end
def main() of int
    return 0
end
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

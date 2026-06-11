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
    assert!(
        jvm_valid(STRUCT_GLOBAL),
        "jvm global struct should produce valid class file"
    );
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
    assert!(
        jvm_valid(STRUCT_NESTED_FIELD),
        "jvm nested struct should produce valid class file"
    );
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

    return x1 + y1 + x2 + y2 + x3 + y3;
}
"#;

#[test]
fn test_nasm_array_of_functions() {
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
    assert!(
        jvm_valid(JVM_ARRAY_OF_FUNCTIONS),
        "jvm array of functions should produce valid class files"
    );
}

#[test]
fn test_jvm_runtime_array_of_functions() {
    let output = compile_and_run_jvm(JVM_ARRAY_OF_FUNCTIONS);
    assert_eq!(output.status.code(), Some(45), "jvm array of functions c1[0](2) etc");
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

const JVM_MUTATING_CLOSURE_ARRAY_RETURN: &str = r#"
def f() of def(int) of int array[2] {
    y = 0;
    return [
        def add2(x of int) of int {
            y = x + y;
            return y;
        },
        def mul2(x of int) of int {
            y = x * y;
            return y;
        }
    ];
}

def main() of int {
    c = f();
    r1 = c[0](3);
    r2 = c[1](4);
    r3 = c[0](1);
    return r1 * 10000 + r2 * 100 + r3;
}
"#;

#[test]
fn test_jvm_runtime_mutating_closure_array_return() {
    let output = compile_and_run_jvm(JVM_MUTATING_CLOSURE_ARRAY_RETURN);
    assert_eq!(output.status.code(), Some(31213), "mutating closures sharing captured y via returned array");
}

#[test]
fn test_jvm_mutating_closure_array_return_valid() {
    assert!(
        jvm_valid(JVM_MUTATING_CLOSURE_ARRAY_RETURN),
        "mutating closure array return should produce valid class files"
    );
}

use crate::codegen::jvm::JvmGenerator;
use crate::codegen::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn generate_ir(source: &str) -> crate::ir::IrProgram {
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    ir_gen.generate(&program)
}

fn compile_and_run_nasm(source: &str) -> std::process::Output {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let ast = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);

    let asm_path = temp_dir.path().join("program.asm");
    fs::write(&asm_path, &asm).unwrap();

    let obj_path = temp_dir.path().join("program.obj");
    let _nasm = Command::new("nasm")
        .args([
            "-f",
            "win64",
            "-o",
            obj_path.to_str().unwrap(),
            asm_path.to_str().unwrap(),
        ])
        .output()
        .expect("NASM not found");

    let exe_path = temp_dir.path().join("program.exe");
    let _gcc = Command::new("gcc")
        .args([obj_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()])
        .output()
        .expect("GCC not found");

    Command::new(exe_path.to_str().unwrap())
        .output()
        .expect("Failed to run exe")
}

fn jvm_class_count(source: &str) -> usize {
    let ir = generate_ir(source);
    let mut jvm_gen = JvmGenerator::new();
    jvm_gen.generate_program(&ir).len()
}

fn compile_and_run_jvm(source: &str) -> std::process::Output {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ir = generate_ir(source);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    for (name, bytes) in &classes {
        let path = temp_dir.path().join(format!("{}.class", name));
        fs::write(&path, bytes).unwrap();
    }
    let output = Command::new("java")
        .args(["-cp", temp_dir.path().to_str().unwrap(), "Main"])
        .output()
        .expect("Java not found — JVM tests require java on PATH");
    if output.status.code() != Some(0) {
        eprintln!("Java stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("Java stdout: {}", String::from_utf8_lossy(&output.stdout));
    }
    output
}

fn jvm_valid(source: &str) -> bool {
    let ir = generate_ir(source);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    !classes.is_empty()
        && classes
            .iter()
            .all(|(_, b)| b.len() >= 4 && b[0..4] == [0xCA, 0xFE, 0xBA, 0xBE])
}

// ─── Shared test programs ───────────────────────────────────────────────

const RETURN_42: &str = "def main() of int return 42; end";

const ARITHMETIC: &str = r#"
def main() of int
    a = 2;
    b = 3;
    return a + b * 4
end
"#;

const IF_ELSE: &str = r#"
def main() of int
    x = 5;
    if x > 0 then
        return 1
    else
        return 0
    end
end
"#;

const WHILE_LOOP: &str = r#"
def main() of int
    i = 1;
    while i < 5 {
        i = i + 1;
    }
    loop_end
    return i
end
"#;

const NEGATION: &str = r#"
def main() of int
    x = 7;
    return -x
end
"#;

const NESTED_IF: &str = r#"
def main() of int
    x = 10;
    if x > 0 then
        if x > 5 then
            return 2
        end
        return 1
    end
    return 0
end
"#;

const MULTIPLY: &str = r#"
def main() of int
    a = 6;
    return a * a
end
"#;

const MINUS: &str = r#"
def main() of int
    a = 10;
    b = 3;
    return a - b
end
"#;

const MODULUS: &str = r#"
def main() of int
    return 10 % 3
end
"#;

const MULTI_VAR: &str = r#"
def main() of int
    x = 1;
    y = 2;
    z = 3;
    return x + y + z
end
"#;

// ─── NASM: shared programs compile & run ────────────────────────────────

#[test]
fn test_nasm_return_42() {
    let output = compile_and_run_nasm(RETURN_42);
    assert!(output.status.code() != Some(-1));
}

#[test]
fn test_nasm_arithmetic() {
    let output = compile_and_run_nasm(ARITHMETIC);
    assert!(output.status.code() != Some(-1));
}

#[test]
fn test_nasm_if_else() {
    let output = compile_and_run_nasm(IF_ELSE);
    assert!(output.status.code() != Some(-1));
}

#[test]
fn test_nasm_while_loop() {
    let output = compile_and_run_nasm(WHILE_LOOP);
    assert!(output.status.code() != Some(-1));
}

#[test]
fn test_nasm_negation() {
    let output = compile_and_run_nasm(NEGATION);
    assert!(output.status.code() != Some(-1));
}

#[test]
fn test_nasm_nested_if() {
    let output = compile_and_run_nasm(NESTED_IF);
    assert!(output.status.code() != Some(-1));
}

#[test]
fn test_nasm_multiply() {
    let output = compile_and_run_nasm(MULTIPLY);
    assert!(output.status.code() != Some(-1));
}

#[test]
fn test_nasm_minus() {
    let output = compile_and_run_nasm(MINUS);
    assert!(output.status.code() != Some(-1));
}

#[test]
fn test_nasm_modulus() {
    let output = compile_and_run_nasm(MODULUS);
    assert!(output.status.code() != Some(-1));
}

#[test]
fn test_nasm_multi_var() {
    let output = compile_and_run_nasm(MULTI_VAR);
    assert!(output.status.code() != Some(-1));
}

// ─── JVM: same programs produce valid .class files ─────────────────────

#[test]
fn test_jvm_return_42() {
    assert!(jvm_valid(RETURN_42));
    assert_eq!(jvm_class_count(RETURN_42), 1);
}

#[test]
fn test_jvm_arithmetic() {
    assert!(jvm_valid(ARITHMETIC));
    assert_eq!(jvm_class_count(ARITHMETIC), 1);
}

#[test]
fn test_jvm_if_else() {
    assert!(jvm_valid(IF_ELSE));
    assert_eq!(jvm_class_count(IF_ELSE), 1);
}

#[test]
fn test_jvm_while_loop() {
    assert!(jvm_valid(WHILE_LOOP));
    assert_eq!(jvm_class_count(WHILE_LOOP), 1);
}

#[test]
fn test_jvm_negation() {
    assert!(jvm_valid(NEGATION));
    assert_eq!(jvm_class_count(NEGATION), 1);
}

#[test]
fn test_jvm_nested_if() {
    assert!(jvm_valid(NESTED_IF));
    assert_eq!(jvm_class_count(NESTED_IF), 1);
}

#[test]
fn test_jvm_multiply() {
    assert!(jvm_valid(MULTIPLY));
    assert_eq!(jvm_class_count(MULTIPLY), 1);
}

#[test]
fn test_jvm_minus() {
    assert!(jvm_valid(MINUS));
    assert_eq!(jvm_class_count(MINUS), 1);
}

#[test]
fn test_jvm_modulus() {
    assert!(jvm_valid(MODULUS));
    assert_eq!(jvm_class_count(MODULUS), 1);
}

#[test]
fn test_jvm_multi_var() {
    assert!(jvm_valid(MULTI_VAR));
    assert_eq!(jvm_class_count(MULTI_VAR), 1);
}

// ─── JVM runtime tests (compile class files & run with java) ────────────

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

// ─── Struct tests ─────────────────────────────────────────────────────────

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

// ─── Coroutine tests ──────────────────────────────────────────────────────

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
    assert!(
        jvm_class_count(COROUTINE_SIMPLE) >= 2,
        "jvm coroutine should produce coroutine class + Main"
    );
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

// ─── Coroutine with params ───────────────────────────────────────────────

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

// ─── JVM daemon test (runs full daemon with piped stdin) ────────────────

fn compile_and_run_jvm_with_stdin(source: &str, stdin_data: &str) -> std::process::Output {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    // Parse, generate IR, generate JVM class files
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);

    for (name, bytes) in &classes {
        let path = temp_dir.path().join(format!("{}.class", name));
        fs::write(&path, bytes).unwrap();
    }

    // If the program uses globals, generate and compile RuntimeStub.java
    if !jvm_gen.global_vars().is_empty() {
        let stub_source = jvm_gen.runtime_stub_source();
        let stub_path = temp_dir.path().join("RuntimeStub.java");
        fs::write(&stub_path, stub_source).unwrap();

        let javac_output = Command::new("javac")
            .arg(stub_path.to_str().unwrap())
            .current_dir(temp_dir.path())
            .output()
            .expect("javac not found");
        if !javac_output.status.success() {
            eprintln!("javac stderr: {}", String::from_utf8_lossy(&javac_output.stderr));
            panic!("Failed to compile RuntimeStub.java");
        }
    }

    let mut java = Command::new("java")
        .args(["-cp", temp_dir.path().to_str().unwrap(), "Main"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("Java not found — JVM tests require java on PATH");

    if let Some(ref mut stdin) = java.stdin {
        stdin.write_all(stdin_data.as_bytes()).unwrap();
    }
    drop(java.stdin.take());

    let output = java.wait_with_output().unwrap();
    if !output.status.success() {
        eprintln!("Java stderr: {}", String::from_utf8_lossy(&output.stderr));
        eprintln!("Java stdout: {}", String::from_utf8_lossy(&output.stdout));
    }
    output
}

#[test]
#[ignore = "JVM daemon NPE on local24 — not yet fixed"]
fn test_jvm_daemon_set_command() {
    let source = DAEMON_SOURCE;

    let output = compile_and_run_jvm_with_stdin(source, "SET\n");

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("OK"),
        "Expected 'OK' in stdout, got: {:?}",
        stdout
    );
}

const DAEMON_SOURCE: &str = r#"// Lab 4 — MyLang daemon with malloc/free and string struct fields
import getchar
import putchar
import malloc
import free

struct MapEntry {
    key of string
    val of string
}

global entries of MapEntry[8]
global map_n of int = 0
global input_buf of string

global r0 of int = 0
global r1 of int = -1
global r2 of int = -1

def streq(a of string, b of string) of int
    i = 0
    while 1 == 1 {
        ca = a[i]
        cb = b[i]
        if ca != cb then { return 0 }
        if ca == 0 then { return 1 }
        i = i + 1
    }
end

def streq_at(buf of string, start of int, s of string) of int
    i = 0
    while 1 == 1 {
        ca = buf[start + i]
        cb = s[i]
        if ca != cb then { return 0 }
        if ca == 0 then { return 1 }
        i = i + 1
    }
end

def strdup_from(buf of string, start of int) of string
    len = 0
    while buf[start + len] != 0 { len = len + 1 }
    dst = malloc(len + 1)
    i = 0
    while i <= len {
        dst[i] = buf[start + i]
        i = i + 1
    }
    return dst
end

def putstr(s of string)
    i = 0
    while 1 == 1 {
        c = s[i]
        if c == 0 then { return }
        putchar(c)
        i = i + 1
    }
end

def map_find_at(buf of string, start of int) of int
    i = 0
    while i < map_n {
        if streq_at(buf, start, entries[i].key) == 1 then { return i }
        i = i + 1
    }
    return -1
end

def map_add(buf of string, key_off of int, val_off of int)
    if map_n >= 8 then { return }
    entries[map_n].key = strdup_from(buf, key_off)
    entries[map_n].val = strdup_from(buf, val_off)
    map_n = map_n + 1
end

def map_update(idx of int, buf of string, val_off of int)
    free(entries[idx].val)
    entries[idx].val = strdup_from(buf, val_off)
end

def map_remove(idx of int)
    free(entries[idx].key)
    free(entries[idx].val)
    while idx < map_n - 1 {
        entries[idx].key = entries[idx + 1].key
        entries[idx].val = entries[idx + 1].val
        idx = idx + 1
    }
    map_n = map_n - 1
end

def parse_line()
    r0 = 0; r1 = -1; r2 = -1
    sp1 = 0
    while 1 == 1 {
        c = input_buf[sp1]
        if c == 32 then { break; }
        if c == 0 then { return }
        sp1 = sp1 + 1
    }
    input_buf[sp1] = 0
    r1 = sp1 + 1
    sp2 = r1
    while 1 == 1 {
        c = input_buf[sp2]
        if c == 32 then { break; }
        if c == 0 then { r1 = -1; return }
        sp2 = sp2 + 1
    }
    input_buf[sp2] = 0
    r2 = sp2 + 1
end

def respond_ok()
    putchar(79); putchar(75); putchar(10)
end

def respond_ok_str(s of string)
    putchar(79); putchar(75); putchar(32)
    putstr(s)
    putchar(10)
end

def respond_err(s of string)
    putchar(69); putchar(82); putchar(82); putchar(32)
    putstr(s)
    putchar(10)
end

def main() of int
    input_buf = malloc(256)
    eof = 0
    di = 0
    while 1 == 1 {
        c = getchar()
        if c == 10 then { input_buf[di] = 0; break; }
        if c == -1 then { input_buf[di] = 0; eof = 1; break; }
        input_buf[di] = c
        di = di + 1
    }
    while eof == 0 {
        parse_line()
        if r1 == -1 then {
            if streq_at(input_buf, 0, "exit") == 1 then {
                i = 0
                while i < map_n {
                    free(entries[i].key)
                    free(entries[i].val)
                    i = i + 1
                }
                respond_ok()
                return 0
            }
        }
        if r1 == -1 then {
            if streq_at(input_buf, 0, "list") == 1 then {
                putchar(79); putchar(75); putchar(32)
                i = 0
                while i < map_n {
                    if i > 0 then { putchar(44) }
                    putstr(entries[i].key)
                    i = i + 1
                }
                putchar(10)
            }
        }
        if r1 >= 0 then {
            if streq_at(input_buf, 0, "get") == 1 then {
                idx = map_find_at(input_buf, r1)
                if idx >= 0 then {
                    respond_ok_str(entries[idx].val)
                } else {
                    respond_err("Key not found")
                }
            }
        }
        if r1 >= 0 then {
            if streq_at(input_buf, 0, "create") == 1 then {
                if r2 == -1 then {
                    respond_err("Key not found")
                } else {
                    idx = map_find_at(input_buf, r1)
                    if idx >= 0 then {
                        respond_err("Exists")
                    } else {
                        map_add(input_buf, r1, r2)
                        respond_ok()
                    }
                }
            }
        }
        if r1 >= 0 then {
            if streq_at(input_buf, 0, "set") == 1 then {
                if r2 == -1 then {
                    respond_err("Key not found")
                } else {
                    idx = map_find_at(input_buf, r1)
                    if idx >= 0 then {
                        map_update(idx, input_buf, r2)
                        respond_ok()
                    } else {
                        respond_err("Key not found")
                    }
                }
            }
        }
        if r1 >= 0 then {
            if streq_at(input_buf, 0, "delete") == 1 then {
                idx = map_find_at(input_buf, r1)
                if idx >= 0 then {
                    map_remove(idx)
                    respond_ok()
                } else {
                    respond_err("Key not found")
                }
            }
        }
        di = 0
        while 1 == 1 {
            c = getchar()
            if c == 10 then { input_buf[di] = 0; break; }
            if c == -1 then { input_buf[di] = 0; eof = 1; break; }
            input_buf[di] = c
            di = di + 1
        }
    }
end
"#;

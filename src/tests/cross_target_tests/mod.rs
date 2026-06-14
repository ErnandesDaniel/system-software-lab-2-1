mod closures_structs;
mod nasm_extra;

use crate::codegen::jvm::JvmGenerator;
use crate::codegen::nasm::AsmGenerator;
use crate::ir::IrOpcode;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn generate_ir(source: &str) -> crate::ir::IrProgram {
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    ir_gen.generate(&program)
}

pub fn compile_and_run_nasm(source: &str) -> std::process::Output {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let os_target = if cfg!(target_os = "linux") {
        crate::OsTarget::Linux
    } else {
        crate::OsTarget::Windows
    };
    let nasm_format = if cfg!(target_os = "linux") { "elf64" } else { "win64" };
    let obj_ext = if cfg!(target_os = "linux") { "o" } else { "obj" };
    let exe_name = if cfg!(target_os = "linux") {
        "program"
    } else {
        "program.exe"
    };

    let ast = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    let mut asm_gen = AsmGenerator::with_os(os_target);
    let asm = asm_gen.generate(&ir);

    let asm_path = temp_dir.path().join("program.asm");
    fs::write(&asm_path, &asm).unwrap();

    let obj_path = temp_dir.path().join(format!("program.{}", obj_ext));
    let nasm_result = Command::new("nasm")
        .args([
            "-f",
            nasm_format,
            "-O0",
            "-o",
            obj_path.to_str().unwrap(),
            asm_path.to_str().unwrap(),
        ])
        .output()
        .expect("NASM not found");
    if !nasm_result.status.success() {
        eprintln!("NASM FAILED: {}", String::from_utf8_lossy(&nasm_result.stderr));
    }

    let exe_path = temp_dir.path().join(exe_name);
    let mut extra_objs: Vec<std::path::PathBuf> = Vec::new();

    let mut gcc_cmd = Command::new("gcc");
    if cfg!(target_os = "linux") {
        gcc_cmd.arg("-no-pie");
    }
    let mut link_objs: Vec<&std::path::Path> = vec![&obj_path];
    for o in &extra_objs {
        link_objs.push(o);
    }
    let mut gcc_args = vec!["-o", exe_path.to_str().unwrap()];
    for o in &link_objs {
        gcc_args.push(o.to_str().unwrap());
    }
    let gcc_result = gcc_cmd.args(&gcc_args).output().expect("GCC not found");
    if !gcc_result.status.success() {
        eprintln!("gcc stdout: {}", String::from_utf8_lossy(&gcc_result.stdout));
        eprintln!("gcc stderr: {}", String::from_utf8_lossy(&gcc_result.stderr));
        eprintln!("asm:\n{}", &asm[..asm.len().min(2000)]);
    }

    if !exe_path.exists() {
        panic!("{} was not created by gcc", exe_name);
    }

    Command::new(exe_path.to_str().unwrap())
        .output()
        .expect("Failed to run exe")
}

pub fn jvm_class_count(source: &str) -> usize {
    let ir = generate_ir(source);
    let mut jvm_gen = JvmGenerator::new();
    jvm_gen.generate_program(&ir).len()
}

pub fn compile_and_run_jvm(source: &str) -> std::process::Output {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let ir = generate_ir(source);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    for (name, bytes) in &classes {
        let path = temp_dir.path().join(format!("{}.class", name));
        fs::write(&path, bytes).unwrap();
    }
    Command::new("java")
        .args(["-cp", temp_dir.path().to_str().unwrap(), "Main"])
        .output()
        .expect("Java not found — JVM tests require java on PATH")
}

pub fn jvm_valid(source: &str) -> bool {
    let ir = generate_ir(source);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    !classes.is_empty()
        && classes
            .iter()
            .all(|(_, b)| b.len() >= 4 && b[0..4] == [0xCA, 0xFE, 0xBA, 0xBE])
}

const RETURN_42: &str = "def main() of int { return 42; }";

const ARITHMETIC: &str = r#"
def main() of int {
    a = 2;
    b = 3;
    return a + b * 4;
}
"#;

const IF_ELSE: &str = r#"
def main() of int {
    x = 5;
    if (x > 0) {
        return 1;
    } else {
        return 0;
    }
}
"#;

const WHILE_LOOP: &str = r#"
def main() of int {
    i = 1;
    while (i < 5) {
        i = i + 1;
    }
    return i;
}
"#;

const NEGATION: &str = r#"
def main() of int {
    x = 7;
    return -x;
}
"#;

const NESTED_IF: &str = r#"
def main() of int {
    x = 10;
    if (x > 0) {
        if (x > 5) {
            return 2;
        }
        return 1;
    }
    return 0;
}
"#;

const MULTIPLY: &str = r#"
def main() of int {
    a = 6;
    return a * a;
}
"#;

const MINUS: &str = r#"
def main() of int {
    a = 10;
    b = 3;
    return a - b;
}
"#;

const MODULUS: &str = r#"
def main() of int {
    return 10 % 3;
}
"#;

const MULTI_VAR: &str = r#"
def main() of int {
    x = 1;
    y = 2;
    z = 3;
    return x + y + z;
}
"#;

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

mod advanced_func;
mod arithmetic;
mod basics;
mod call_ops;
mod control_flow;
mod globals;
mod io;
mod io_advanced;
mod jvm;
mod stdlib;
mod types;

use crate::codegen::nasm::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::parser::Parser;
use crate::OsTarget;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

fn test_os() -> OsTarget {
    if cfg!(target_os = "linux") {
        OsTarget::Linux
    } else {
        OsTarget::Windows
    }
}

fn test_nasm_format() -> &'static str {
    if cfg!(target_os = "linux") { "elf64" } else { "win64" }
}

fn test_obj_ext() -> &'static str {
    if cfg!(target_os = "linux") { "o" } else { "obj" }
}

fn test_exe_name() -> &'static str {
    if cfg!(target_os = "linux") { "program" } else { "program.exe" }
}

pub fn compile_only(source: &str) -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut parser = Parser::new(source);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => panic!("Parse error: {}", e),
    };

    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);

    let mut all_asm = String::new();
    let mut obj_files: Vec<std::path::PathBuf> = Vec::new();

    let global_names: Vec<String> = ir.globals.iter().map(|g| g.name.clone()).collect();
    for func in &ir.functions {
        let mut gen = AsmGenerator::with_os(test_os());
        gen.set_global_names(&global_names);
        let mut asm = gen.generate_single_function(func);
        if !ir.globals.is_empty() {
            let mut externs = String::new();
            for g in &ir.globals {
                externs.push_str(&format!("extern {}\n", g.name));
            }
            asm.insert_str(0, &externs);
        }
        let asm_path = temp_dir.path().join(format!("{}.asm", func.name));
        fs::write(&asm_path, &asm).unwrap();
        all_asm.push_str(&asm);

        let obj_path = temp_dir.path().join(format!("{}.{}", func.name, test_obj_ext()));
        let nasm_result = Command::new("nasm")
            .args(["-f", test_nasm_format(), "-o"])
            .arg(obj_path.to_str().unwrap())
            .arg(asm_path.to_str().unwrap())
            .output();
        if !nasm_result.as_ref().map(|o| o.status.success()).unwrap_or(false) {
            panic!(
                "NASM error for {}: {}",
                func.name,
                String::from_utf8_lossy(&nasm_result.unwrap().stderr)
            );
        }
        obj_files.push(obj_path);
    }

    if !ir.globals.is_empty() {
        let globals_asm = format!(
            "bits 64\ndefault rel\nsection .data\n{}",
            AsmGenerator::generate_globals_asm(&ir.globals)
        );
        all_asm.push_str(&globals_asm);
        let globals_path = temp_dir.path().join("globals.asm");
        fs::write(&globals_path, &globals_asm).unwrap();
        let globals_obj = temp_dir.path().join(format!("globals.{}", test_obj_ext()));
        let nasm_result = Command::new("nasm")
            .args(["-f", test_nasm_format(), "-o"])
            .arg(globals_obj.to_str().unwrap())
            .arg(globals_path.to_str().unwrap())
            .output();
        if nasm_result.as_ref().map(|o| o.status.success()).unwrap_or(false) {
            obj_files.push(globals_obj);
        }
    }

    let exe_path = temp_dir.path().join(test_exe_name());
    let mut gcc_args: Vec<std::ffi::OsString> = Vec::new();
    if cfg!(target_os = "linux") {
        gcc_args.push("-no-pie".into());
    }
    for obj in &obj_files {
        gcc_args.push(obj.into());
    }
    if cfg!(not(target_os = "linux")) {
        gcc_args.push("-l".into());
        gcc_args.push("msvcrt".into());
    }
    gcc_args.push("-o".into());
    gcc_args.push(exe_path.into());

    let gcc_result = Command::new("gcc").args(&gcc_args).output();
    if !gcc_result.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        panic!("GCC error: {}", String::from_utf8_lossy(&gcc_result.unwrap().stderr));
    }

    (temp_dir, all_asm)
}

pub fn normalize_exit_code(code: Option<i32>) -> Option<i32> {
    if cfg!(target_os = "linux") {
        code.map(|c| if c > 127 { c as i8 as i32 } else { c })
    } else {
        code
    }
}

pub fn exit_code(output: &std::process::Output) -> Option<i32> {
    normalize_exit_code(output.status.code())
}

pub fn compile_and_run(source: &str) -> std::process::Output {
    let (temp_dir, _) = compile_only(source);
    let exe_path = temp_dir.path().join(test_exe_name());
    Command::new(exe_path.to_str().unwrap())
        .current_dir(temp_dir.path())
        .output()
        .unwrap()
}

pub fn compile_and_run_with_files(source: &str, files: &[(&str, &str)]) -> std::process::Output {
    let (temp_dir, _) = compile_only(source);
    for (name, content) in files {
        fs::write(temp_dir.path().join(name), content).unwrap();
    }
    let exe_path = temp_dir.path().join(test_exe_name());
    Command::new(exe_path.to_str().unwrap())
        .current_dir(temp_dir.path())
        .output()
        .unwrap()
}

pub fn normalize_output(output: &[u8]) -> String {
    String::from_utf8_lossy(output).replace("\r\n", "\n")
}

fn compile_and_run_with_stdin(source: &str, input: &str) -> std::process::Output {
    let (temp_dir, _) = compile_only(source);
    let exe_path = temp_dir.path().join(test_exe_name());
    let mut child = Command::new(exe_path.to_str().unwrap())
        .current_dir(temp_dir.path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(input.as_bytes()).unwrap();
    child.wait_with_output().unwrap()
}

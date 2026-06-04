mod arithmetic;
mod basics;
mod coroutines;
mod functions;
mod globals;
mod io;
mod io_advanced;
mod stdlib;

use crate::codegen::nasm::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::parser::Parser;
use std::fs;
use std::io::Write;
use std::process::{Command, Stdio};
use tempfile::TempDir;

pub fn compile_only(source: &str) -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut parser = Parser::new(source);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => panic!("Parse error: {}", e),
    };

    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);

    let has_coroutines = ir.functions.iter().any(|f| f.is_coroutine);
    let mut all_asm = String::new();
    let mut obj_files: Vec<std::path::PathBuf> = Vec::new();

    let global_names: Vec<String> = ir.globals.iter().map(|g| g.name.clone()).collect();
    for func in &ir.functions {
        let mut gen = AsmGenerator::new();
        gen.set_global_names(&global_names);
        if func.is_coroutine {
            gen.set_coroutine(func.yield_count);
        }
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

        let obj_path = temp_dir.path().join(format!("{}.obj", func.name));
        let nasm_flags = if func.is_coroutine {
            vec!["-f", "win64", "-O0", "-o"]
        } else {
            vec!["-f", "win64", "-o"]
        };
        let nasm_result = Command::new("nasm")
            .args(&nasm_flags)
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
        let globals_obj = temp_dir.path().join("globals.obj");
        let nasm_result = Command::new("nasm")
            .args(["-f", "win64", "-o"])
            .arg(globals_obj.to_str().unwrap())
            .arg(globals_path.to_str().unwrap())
            .output();
        if nasm_result.as_ref().map(|o| o.status.success()).unwrap_or(false) {
            obj_files.push(globals_obj);
        }
    }

    if has_coroutines {
        let mut helper = String::from("bits 64\ndefault rel\n\n");
        helper.push_str("section .data\n");
        let coro_count = ir.functions.iter().filter(|f| f.is_coroutine).count().max(8);
        helper.push_str(&format!("co_states times {coro_count} dq 0\n"));
        for f in ir.functions.iter().filter(|f| f.is_coroutine) {
            let num_locals = f.locals.len();
            let ctx_bytes = 56 + num_locals * 8 + 4;
            let ctx_dwords = (ctx_bytes / 4).max(8);
            helper.push_str(&format!("state_{} times {} dd 0\n", f.name, ctx_dwords));
        }
        helper.push_str("\nsection .text\n");
        helper.push_str("global resume_coroutine\nresume_coroutine:\n");
        helper.push_str("    ; rcx = index\n");
        helper.push_str("    lea rax, [rel co_states]\n");
        helper.push_str("    mov rax, [rax + rcx * 8]\n");
        helper.push_str("    test rax, rax\n    jz .empty\n");
        helper.push_str("    mov rbx, rax\n");
        helper.push_str("    mov eax, [rbx]\n    cmp eax, -1\n    jne .go\n    mov eax, 1\n    ret\n.go:\n");
        helper.push_str("    push rbp\n    mov rbp, rsp\n    sub rsp, 40\n");
        helper.push_str("    mov [rbp + 32], rbx\n");
        helper.push_str("    mov rcx, rbx\n    mov rdx, [rbx + 32]\n    mov r8,  [rbx + 40]\n    mov r9,  [rbx + 48]\n");
        helper.push_str("    call [rbx + 8]\n");
        helper.push_str("    mov rbx, [rbp + 32]\n    mov eax, [rbx + 16]\n    leave\n    ret\n");
        helper.push_str(".empty:\n    mov eax, 1\n    ret\n\n");

        helper.push_str("global create_coroutine\ncreate_coroutine:\n");
        helper.push_str("    mov dword [rcx], 0\n    mov [rcx + 8], rdx\n    mov dword [rcx + 16], 0\n");
        helper.push_str("    mov [rcx + 24], r8\n    mov [rcx + 32], r9\n    ret\n\n");

        helper.push_str("global get_coroutine_state\nget_coroutine_state:\n");
        helper.push_str("    lea rax, [rel co_states]\n    mov rax, [rax + rcx * 8]\n    test rax, rax\n    jz .empty\n");
        helper.push_str("    mov eax, [rax]\n    ret\n.empty:\n    mov eax, -1\n    ret\n\n");

        helper.push_str("global set_coroutine_param\nset_coroutine_param:\n");
        helper.push_str("    ; rcx = index, rdx = p1, r8 = p2\n");
        helper.push_str("    lea rax, [rel co_states]\n    mov rax, [rax + rcx * 8]\n    test rax, rax\n    jz .empty\n");
        helper.push_str("    mov [rax + 24], edx\n    mov [rax + 32], r8d\n.empty:\n    ret\n\n");
        helper.push_str("global coro_init\n");
        for f in ir.functions.iter().filter(|f| f.is_coroutine) {
            helper.push_str(&format!("extern {}\n", f.name));
        }
        helper.push_str("coro_init:\n    push rbp\n    mov rbp, rsp\n");
        let mut idx = 0;
        for f in ir.functions.iter().filter(|f| f.is_coroutine) {
            helper.push_str(&format!("    lea rcx, [rel state_{}]\n", f.name));
            helper.push_str(&format!("    lea rdx, [rel {}]\n", f.name));
            helper.push_str("    sub rsp, 32\n    call create_coroutine\n    add rsp, 32\n");
            helper.push_str("    lea rax, [rel co_states]\n");
            helper.push_str(&format!("    lea rcx, [rel state_{}]\n", f.name));
            helper.push_str(&format!("    mov [rax + {}], rcx\n", idx * 8));
            idx += 1;
        }
        helper.push_str("    leave\n    ret\n\n");

        let helper_path = temp_dir.path().join("coro_helpers.asm");
        fs::write(&helper_path, &helper).unwrap();
        let helper_obj = temp_dir.path().join("coro_helpers.obj");
        let nasm_result = Command::new("nasm")
            .args(["-f", "win64", "-o"])
            .arg(helper_obj.to_str().unwrap())
            .arg(helper_path.to_str().unwrap())
            .output();
        if nasm_result.as_ref().map(|o| o.status.success()).unwrap_or(false) {
            obj_files.push(helper_obj);
        }
    }

    let exe_path = temp_dir.path().join("program.exe");
    let mut gcc_args: Vec<std::ffi::OsString> = Vec::new();
    for obj in &obj_files {
        gcc_args.push(obj.into());
    }
    gcc_args.push("-l".into());
    gcc_args.push("msvcrt".into());
    gcc_args.push("-o".into());
    gcc_args.push(exe_path.into());

    let gcc_result = Command::new("gcc").args(&gcc_args).output();
    if !gcc_result.as_ref().map(|o| o.status.success()).unwrap_or(false) {
        panic!("GCC error: {}", String::from_utf8_lossy(&gcc_result.unwrap().stderr));
    }

    (temp_dir, all_asm)
}

fn compile_and_run(source: &str) -> std::process::Output {
    let (temp_dir, _) = compile_only(source);
    let exe_path = temp_dir.path().join("program.exe");
    Command::new(exe_path.to_str().unwrap())
        .current_dir(temp_dir.path())
        .output()
        .unwrap()
}

fn compile_and_run_with_files(source: &str, files: &[(&str, &str)]) -> std::process::Output {
    let (temp_dir, _) = compile_only(source);
    for (name, content) in files {
        fs::write(temp_dir.path().join(name), content).unwrap();
    }
    let exe_path = temp_dir.path().join("program.exe");
    Command::new(exe_path.to_str().unwrap())
        .current_dir(temp_dir.path())
        .output()
        .unwrap()
}

fn normalize_output(output: &[u8]) -> String {
    String::from_utf8_lossy(output).replace("\r\n", "\n")
}

fn compile_and_run_with_stdin(source: &str, input: &str) -> std::process::Output {
    let (temp_dir, _) = compile_only(source);
    let exe_path = temp_dir.path().join("program.exe");
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

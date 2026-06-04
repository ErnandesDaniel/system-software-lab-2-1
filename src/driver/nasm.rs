use std::fs;
use std::path::Path;

use crate::codegen::nasm::AsmGenerator;
use crate::driver::CompilerDriver;
use crate::ir::{IrFunction, IrProgram};

impl CompilerDriver {
    pub fn generate_nasm(ir: &IrProgram, output_dir: &str) {
        use std::process::Command;

        let has_coroutines = ir.functions.iter().any(|f| f.is_coroutine);

        let global_names: Vec<String> = ir.globals.iter().map(|g| g.name.clone()).collect();
        for func in &ir.functions {
            let mut gen = AsmGenerator::new();
            gen.set_global_names(&global_names);
            if func.yield_count > 0 {
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
            let path = Path::new(output_dir).join(format!("{}.asm", func.name));

            if let Err(e) = fs::write(&path, &asm) {
                eprintln!("Failed to write assembly: {e}");
            }
        }

        if !ir.globals.is_empty() {
            let mut globals_asm = String::from("bits 64\ndefault rel\nsection .data\n");
            globals_asm.push_str(&AsmGenerator::generate_globals_asm(&ir.globals));
            let path = Path::new(output_dir).join("globals.asm");
            let _ = fs::write(&path, &globals_asm);
        }

        let mut obj_files = Vec::new();
        if has_coroutines {
            let mut helper = String::from("bits 64\ndefault rel\n\n");

            helper.push_str("section .data\n");
            let coro_count = ir.functions.iter().filter(|f| f.is_coroutine).count().max(8);
            helper.push_str(&format!("co_states times {coro_count} dq 0\n"));
            for f in ir.functions.iter().filter(|f| f.is_coroutine) {
                let ctx_dwords = (coro_state_needed(f, &global_names) + 3) / 4;
                helper.push_str(&format!("state_{} times {} dd 0\n", f.name, ctx_dwords));
            }

            helper.push_str("\nsection .text\n");
            helper.push_str("global resume_coroutine\nresume_coroutine:\n");
            helper.push_str("    lea rax, [rel co_states]\n");
            helper.push_str("    mov rax, [rax + rcx * 8]\n");
            helper.push_str("    test rax, rax\n    jz .empty\n");
            helper.push_str("    mov rbx, rax\n");
            helper.push_str("    mov eax, [rbx]\n    cmp eax, -1\n    jne .go\n    mov eax, 1\n    ret\n.go:\n");
            helper.push_str("    push rbp\n    mov rbp, rsp\n    sub rsp, 40\n");
    helper.push_str("    mov [rbp + 32], rbx\n");
    helper.push_str("    mov rcx, rbx\n");
    helper.push_str("    mov rdx, [rbx + 32]\n");
    helper.push_str("    mov r8,  [rbx + 40]\n");
    helper.push_str("    mov r9,  [rbx + 48]\n");
            helper.push_str("    call [rbx + 8]\n");
            helper.push_str("    mov rbx, [rbp + 32]\n");
            helper.push_str("    mov eax, [rbx + 16]\n    leave\n    ret\n");
            helper.push_str(".empty:\n    mov eax, 1\n    ret\n\n");

            helper.push_str("global create_coroutine\ncreate_coroutine:\n");
            helper.push_str("    mov dword [rcx], 0\n    mov [rcx + 8], rdx\n    mov dword [rcx + 16], 0\n");
            helper.push_str("    mov [rcx + 24], r8\n    mov [rcx + 32], r9\n    ret\n\n");

            helper.push_str("global get_coroutine_state\nget_coroutine_state:\n");
            helper.push_str("    lea rax, [rel co_states]\n    mov rax, [rax + rcx * 8]\n    test rax, rax\n    jz .empty\n");
            helper.push_str("    mov eax, [rax]\n    ret\n.empty:\n    mov eax, -1\n    ret\n\n");

            helper.push_str("global set_coroutine_param\nset_coroutine_param:\n");
            helper.push_str("    lea rax, [rel co_states]\n    mov rax, [rax + rcx * 8]\n    test rax, rax\n    jz .empty\n");
            helper.push_str("    mov [rax + 24], edx\n    mov [rax + 32], r8d\n.empty:\n    ret\n\n");

            helper.push_str("global coro_init\n");
            for f in ir.functions.iter().filter(|f| f.is_coroutine) {
                helper.push_str(&format!("extern {}\n", f.name));
            }
            helper.push_str("coro_init:\n    push rbp\n    mov rbp, rsp\n");
            for (idx, f) in ir.functions.iter().filter(|f| f.is_coroutine).enumerate() {
                helper.push_str(&format!("    lea rcx, [rel state_{}]\n", f.name));
                helper.push_str(&format!("    lea rdx, [rel {}]\n", f.name));
                helper.push_str("    sub rsp, 32\n    call create_coroutine\n    add rsp, 32\n");
                helper.push_str("    lea rax, [rel co_states]\n");
                helper.push_str(&format!("    lea rcx, [rel state_{}]\n", f.name));
                helper.push_str(&format!("    mov [rax + {}], rcx\n", idx * 8));
            }
            helper.push_str("    leave\n    ret\n\n");

            let coro_path = Path::new(output_dir).join("coro_helpers.asm");
            fs::write(&coro_path, &helper).ok();

            let obj = Path::new(output_dir).join("coro_helpers.obj");
            let output = Command::new("nasm")
                .args(["-f", "win64", "-O0", "-o", obj.to_str().expect("Path must be valid UTF-8"), coro_path.to_str().expect("Path must be valid UTF-8")])
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    obj_files.push(obj);
                }
            }
        }

        for func in &ir.functions {
            let asm_path = Path::new(output_dir).join(format!("{}.asm", func.name));
            let obj_path = Path::new(output_dir).join(format!("{}.obj", func.name));

            let output = Command::new("nasm")
                .args(["-f", "win64", "-O0", "-o", obj_path.to_str().expect("Path must be valid UTF-8"), asm_path.to_str().expect("Path must be valid UTF-8")])
                .output();

            match output {
                Ok(out) => {
                    if out.status.success() {
                        obj_files.push(obj_path);
                    } else {
                        eprintln!("NASM failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to run NASM: {e}"),
            }
        }

        let globals_asm = Path::new(output_dir).join("globals.asm");
        if globals_asm.exists() {
            let globals_obj = Path::new(output_dir).join("globals.obj");
            let output = Command::new("nasm")
                .args(["-f", "win64", "-O0", "-o", globals_obj.to_str().expect("Path must be valid UTF-8"), globals_asm.to_str().expect("Path must be valid UTF-8")])
                .output();
            if let Ok(out) = output {
                if out.status.success() {
                    obj_files.push(globals_obj);
                }
            }
        }

        if !obj_files.is_empty() {
            let exe_path = Path::new(output_dir).join("program.exe");
            let mut args: Vec<String> = obj_files.iter().map(|p| p.to_string_lossy().to_string()).collect();
            args.push("-Wl,/subsystem:console".to_string());
            args.push("-o".to_string());
            args.push(exe_path.to_string_lossy().to_string());

            let link = |linker: &str| Command::new(linker).args(&args).output();
            let result = link("clang").or_else(|_| link("gcc")).or_else(|_| link("mingw32-gcc"));
            match result {
                Ok(out) => {
                    if !out.status.success() {
                        eprintln!("Link failed: {}", String::from_utf8_lossy(&out.stderr));
                    }
                }
                Err(e) => eprintln!("Failed to run linker: {e}"),
            }
        }
    }
}

fn coro_state_needed(f: &IrFunction, global_names: &[String]) -> usize {
    let mut slot_set: std::collections::HashSet<String> = std::collections::HashSet::new();

    // Same logic as emit_prologue: locals + __co_ctx + temp results + params
    for local in &f.locals {
        if !global_names.contains(&local.name) { slot_set.insert(local.name.clone()); }
    }
    slot_set.insert("__co_ctx".to_string());
    for block in &f.blocks {
        for inst in &block.instructions {
            if let Some(ref result) = inst.result {
                if result.starts_with('t') && !slot_set.contains(result) && !global_names.iter().any(|g| g == result) {
                    slot_set.insert(result.clone());
                }
            }
        }
    }
    for (i, param) in f.parameters.iter().enumerate() {
        if i < 4 { slot_set.insert(param.name.clone()); }
    }

    56 + slot_set.len() * 8
}

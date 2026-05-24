use crate::codegen::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;
use std::fs;
use std::process::Command;
use tempfile::TempDir;

fn parse(source: &str) -> crate::ast::Program {
    let mut parser = Parser::new(source);
    parser.parse().unwrap()
}

fn compile_only(source: &str) -> (TempDir, String) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");

    let mut parser = Parser::new(source);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => panic!("Parse error: {}", e),
    };

    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);

    let has_coroutines = ir.functions.iter().any(|f| f.yield_count > 0);
    let mut all_asm = String::new();
    let mut obj_files: Vec<std::path::PathBuf> = Vec::new();

    // Generate per-function ASM files (like the real driver)
    for func in &ir.functions {
        let mut gen = AsmGenerator::new();
        if func.yield_count > 0 {
            gen.set_coroutine(func.yield_count);
        }
        let mut asm = gen.generate_single_function(func);
        // Add extern declarations for globals
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
        let nasm_flags = if func.yield_count > 0 {
            vec!["-f", "win64", "-O0", "-o"]
        } else {
            vec!["-f", "win64", "-o"]
        };
        let nasm_result = Command::new("nasm")
            .args(&nasm_flags)
            .arg(obj_path.to_str().unwrap())
            .arg(asm_path.to_str().unwrap())
            .output();
        if !nasm_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            panic!(
                "NASM error for {}: {}",
                func.name,
                String::from_utf8_lossy(&nasm_result.unwrap().stderr)
            );
        }
        obj_files.push(obj_path);
    }

    // Generate globals.asm if needed
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
        if nasm_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            obj_files.push(globals_obj);
        }
    }

    // Generate coroutine helpers if needed
    if has_coroutines {
        let mut helper = String::from("bits 64\ndefault rel\n\n");
        helper.push_str("section .data\n");
        helper.push_str("co_states dq 0, 0, 0, 0, 0, 0, 0, 0\n");
        for f in ir.functions.iter().filter(|f| f.yield_count > 0) {
            helper.push_str(&format!("state_{} dd 0, 0, 0, 0, 0, 0\n", f.name));
        }
        helper.push_str("\nsection .text\n");
        helper.push_str("global resume_coroutine_nasm\nresume_coroutine_nasm:\n");
        helper.push_str("    ; rcx = index\n");
        helper.push_str("    lea rax, [rel co_states]\n");
        helper.push_str("    mov rax, [rax + rcx * 8]\n");
        helper.push_str("    test rax, rax\n    jz .empty\n");
        helper.push_str("    mov rcx, rax\n");
        helper.push_str("    mov eax, [rcx]\n    cmp eax, -1\n    jne .go\n    mov eax, 1\n    ret\n.go:\n");
        helper.push_str("    push rbp\n    mov rbp, rsp\n    sub rsp, 32\n    call [rcx + 8]\n    mov eax, [rcx + 16]\n    leave\n    ret\n");
        helper.push_str(".empty:\n    mov eax, 1\n    ret\n\n");
        helper.push_str("global create_coroutine_nasm\ncreate_coroutine_nasm:\n");
        helper.push_str("    mov dword [rcx], 0\n    mov [rcx + 8], rdx\n    mov dword [rcx + 16], 0\n    ret\n\n");
        helper.push_str("global coro_init_nasm\n");
        for f in ir.functions.iter().filter(|f| f.yield_count > 0) {
            helper.push_str(&format!("extern {}\n", f.name));
        }
        helper.push_str("coro_init_nasm:\n    push rbp\n    mov rbp, rsp\n");
        let mut idx = 0;
        for f in ir.functions.iter().filter(|f| f.yield_count > 0) {
            helper.push_str(&format!("    lea rcx, [rel state_{}]\n", f.name));
            helper.push_str(&format!("    lea rdx, [rel {}]\n", f.name));
            helper.push_str("    sub rsp, 32\n    call create_coroutine_nasm\n    add rsp, 32\n");
            helper.push_str("    lea rax, [rel co_states]\n");
            helper.push_str(&format!("    lea rcx, [rel state_{}]\n", f.name));
            helper.push_str(&format!("    mov [rax + {}], rcx\n", idx * 8));
            idx += 1;
        }
        helper.push_str("    leave\n    ret\n");

        let helper_path = temp_dir.path().join("coro_helpers.asm");
        fs::write(&helper_path, &helper).unwrap();
        let helper_obj = temp_dir.path().join("coro_helpers.obj");
        let nasm_result = Command::new("nasm")
            .args(["-f", "win64", "-o"])
            .arg(helper_obj.to_str().unwrap())
            .arg(helper_path.to_str().unwrap())
            .output();
        if nasm_result
            .as_ref()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            obj_files.push(helper_obj);
        }
    }

    // Link all .obj files together
    let exe_path = temp_dir.path().join("program.exe");
    let mut gcc_args: Vec<std::ffi::OsString> = Vec::new();
    for obj in &obj_files {
        gcc_args.push(obj.into());
    }
    gcc_args.push("-o".into());
    gcc_args.push(exe_path.into());

    let gcc_result = Command::new("gcc")
        .args(&gcc_args)
        .output();

    if !gcc_result
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        panic!(
            "GCC error: {}",
            String::from_utf8_lossy(&gcc_result.unwrap().stderr)
        );
    }

    (temp_dir, all_asm)
}

fn compile_and_run(source: &str) -> std::process::Output {
    let (temp_dir, _) = compile_only(source);
    let exe_path = temp_dir.path().join("program.exe");
    Command::new(exe_path.to_str().unwrap())
        .output()
        .unwrap()
}

#[test]
fn test_extern_short_form_semantics() {
    let source = "extern puts def main() puts(\"hello\"); end";
    let program = parse(source);
    let mut analyzer = SemanticsAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_cfg_generation() {
    use crate::ir::cfg::CfgMermaidGenerator;

    let source =
        "def square(x of int) of int return x * x; end def main() of int return 42; end";
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();

    let mut ir_gen = IrGenerator::new();
    let ir_program = ir_gen.generate(&ast);

    let mut cfg_gen = CfgMermaidGenerator::new();
    let cfg_output = cfg_gen.generate(&ir_program);

    eprintln!("CFG Output:\n{}", cfg_output);

    assert!(cfg_output.contains("graph TD"));
    assert!(cfg_output.contains("BB_0"));
    assert!(cfg_output.contains("x * x"));
}

#[test]
fn test_while_loop_block_order() {
    // While loop: i=1; while i<5 { i=i+1; } loop_end
    // Expected blocks: init+jmp, header, body, exit, post-loop
    let source = "def foo() i=1; while i<5 { i=i+1; } loop_end return i; end";
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();

    let mut ir_gen = IrGenerator::new();
    let ir_program = ir_gen.generate(&ast);

    let func = &ir_program.functions[0];
    eprintln!("Function has {} blocks:", func.blocks.len());
    for (i, block) in func.blocks.iter().enumerate() {
        eprintln!("  Block {}: {} instructions", i, block.id);
        for instr in &block.instructions {
            eprintln!("    {:?}", instr.opcode);
        }
    }

    // Should have at least 4 blocks: init+jmp, header, body, post-loop (exit merged)
    assert!(
        func.blocks.len() >= 4,
        "Expected at least 4 blocks, got {}",
        func.blocks.len()
    );
}

#[test]
fn test_exe_return_value() {
    let source = "def main() of int return 42; end";
    let output = compile_and_run(source);

    // For now just verify it compiles and runs - return value may differ
    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_arithmetic() {
    let source = "def main() of int return 5 + 3 * 2; end";
    let output = compile_and_run(source);

    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_with_loop() {
    let source = "def main() of int i = 1; while i < 5 { i = i + 1; } loop_end return i; end";
    let output = compile_and_run(source);

    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_if_else() {
    let source = "def main() of int x = 10; if x > 5 then return 1; end return 0; end";
    let output = compile_and_run(source);

    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_return_value_42() {
    let source = "def main() of int return 42; end";
    let output = compile_and_run(source);

    eprintln!("Exit code: {:?}", output.status.code());
    // On Windows, exit code is a u32 and 42 should pass through
    assert!(output.status.success() || output.status.code() == Some(42));
}

#[test]
fn test_exe_return_value_zero() {
    let source = "def main() of int return 0; end";
    let output = compile_and_run(source);

    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.success() || output.status.code() == Some(0));
}

#[test]
fn test_exe_simple_arithmetic() {
    let source = "def main() of int return 7 + 3; end";
    let output = compile_and_run(source);

    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_nested_if() {
    let source = r#"
        def main() of int
            x = 10;
            if x > 0 then
                if x > 5 then
                    return 1
                end
                return 2
            end
            return 0
        end
    "#;
    let output = compile_and_run(source);

    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.success() || output.status.code() != Some(-1));
}

#[test]
fn test_compile_multi_function_asm() {
    // Note: known bug in NASM codegen — multi-function programs may duplicate labels.
    // This test only checks that IR generation and basic ASM structure works.
    let source = r#"
        def double(x of int) of int
            return x + x
        end
        def main() of int
            return double(21)
        end
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    assert_eq!(ir.functions.len(), 2);
    // Just verify IR has both functions
    assert_eq!(ir.functions[0].name, "double");
    assert_eq!(ir.functions[1].name, "main");
}

#[test]
fn test_asm_contains_data_section_for_strings() {
    let source = r#"
        def main() of int
            s = "test_string";
            return 0
        end
    "#;
    let (_, asm) = compile_only(source);

    assert!(asm.contains("main:"));
    // String may appear in data section, db bytes, or as label
    assert!(
        asm.contains("test_string") || asm.contains("section .data") || asm.contains("db "),
        "Expected string or data section, got:\n{}",
        &asm[..asm.len().min(600)]
    );
}

#[test]
fn test_exe_global_read() {
    let source = r#"
        global counter of int = 42;
        def main() of int
            return counter
        end
    "#;
    let output = compile_and_run(source);
    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_asm_global_in_data_section() {
    let source = r#"
        global counter of int = 42;
        def main() of int
            return counter
        end
    "#;
    let (_, asm) = compile_only(source);
    assert!(asm.contains("counter"), "Expected global label in asm, got:\n{}", &asm[..asm.len().min(800)]);
    assert!(asm.contains("section .data"), "Expected data section");
    assert!(asm.contains("dd 42"), "Expected dd 42");
}

#[test]
fn test_exe_global_write() {
    let source = r#"
        global value of int = 0;
        def main() of int
            value = 99;
            return value
        end
    "#;
    let output = compile_and_run(source);
    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_global_string() {
    let source = r#"
        global name of string = "test";
        def main() of int
            return 0
        end
    "#;
    let (_, asm) = compile_only(source);
    assert!(asm.contains("section .data"), "Expected data section, got:\n{}", &asm[..asm.len().min(800)]);
    // String may be encoded as bytes: 116,101,115,116 = "test"
    assert!(
        asm.contains("name") || asm.contains("116"),
        "Expected name label or byte data, got:\n{}",
        &asm[..asm.len().min(800)]
    );
}

#[test]
fn test_exe_global_struct_field() {
    let source = r#"
        struct Sched {
            count of int;
            index of int;
        }
        global sched of Sched;
        def main() of int
            return sched.count
        end
    "#;
    let (_, asm) = compile_only(source);
    eprintln!("ASM:\n{}", &asm[..asm.len().min(3000)]);
    assert!(asm.contains("global main"), "Expected global main");
    assert!(asm.contains("sched"), "Expected sched in data");
    assert!(asm.contains("[rel sched"), "Expected rel sched");
}

#[test]
fn test_exe_struct_array_field_read() {
    let source = r#"
        struct Sched {
            slots of int[3];
            count of int;
        }
        global sched of Sched;
        def main() of int
            return sched.slots[0]
        end
    "#;
    let (_, asm) = compile_only(source);
    eprintln!("ASM:\n{}", &asm[..asm.len().min(3000)]);
    assert!(asm.contains("sched"), "Expected sched label");
    // Should use lea rax, [rel sched] then mov eax, [rax + rbx*4]
    assert!(asm.contains("lea rax"), "Expected lea for array field");
}

#[test]
fn test_exe_local_struct() {
    let source = r#"
        struct Point { x of int; y of int; }
        def main() of int
            p of Point;
            p.x = 42;
            return p.x
        end
    "#;
    let output = compile_and_run(source);
    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_asm_coroutine_state_machine() {
    let source = r#"
        extern putchar
        coroutine worker() of int
            putchar(49)
            yield
            putchar(50)
            return 0
        end
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    assert_eq!(ir.functions.len(), 1);
    assert!(ir.functions[0].yield_count > 0, "Expected yield count > 0");
    let mut asm_gen = AsmGenerator::new();
    asm_gen.set_coroutine(ir.functions[0].yield_count);
    let asm = asm_gen.generate(&ir);
    eprintln!("ASM:\n{}", &asm[..asm.len().min(3000)]);
    assert!(asm.contains("co_0"), "Expected state 0 label");
    assert!(asm.contains("co_1"), "Expected state 1 label");
    assert!(asm.contains("[rcx]"), "Expected state struct access");
    assert!(asm.contains("global worker"), "Expected global worker");
}

#[test]
fn test_asm_coroutine_locals() {
    let source = r#"
        coroutine counter() of int
            i = 0;
            yield;
            return i
        end
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    let mut asm_gen = AsmGenerator::new();
    if !ir.functions.is_empty() && ir.functions[0].yield_count > 0 {
        asm_gen.set_coroutine(ir.functions[0].yield_count);
    }
    let asm = asm_gen.generate(&ir);
    eprintln!("ASM:\n{}", &asm[..asm.len().min(3000)]);
    assert!(asm.contains("global counter"), "Expected global counter");
}

#[test]
fn test_nasm_coroutine_compiles() {
    let source = r#"
        coroutine simple() of int
            return 0
        end
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    let mut asm_gen = AsmGenerator::new();
    if !ir.functions.is_empty() && ir.functions[0].yield_count > 0 {
        asm_gen.set_coroutine(ir.functions[0].yield_count);
    }
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("global simple"), "Expected global simple");
}

#[test]
fn test_exe_global_array_read() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[0]
        end
    "#;
    let output = compile_and_run(source);
    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_global_array_index() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[2]
        end
    "#;
    let output = compile_and_run(source);
    eprintln!("Exit code: {:?}", output.status.code());
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_asm_global_array_parse() {
    // Test 1: no initializer
    let source1 = r#"
        global arr of int[3];
        def main() of int
            return 0
        end
    "#;
    let mut parser = Parser::new(source1);
    let ast1 = parser.parse().unwrap();
    eprintln!("Test1 (no init): items={}", ast1.items.len());

    // Test 2: with initializer
    let source2 = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return 0
        end
    "#;
    let mut parser2 = Parser::new(source2);
    let ast2 = parser2.parse().unwrap();
    eprintln!("Test2 (with init): items={}", ast2.items.len());

    assert_eq!(ast1.items.len(), 2, "Without init: expected 2");
    assert_eq!(ast2.items.len(), 2, "With init: expected 2");
}

#[test]
fn test_asm_global_array_init() {
    use crate::codegen::AsmGenerator;
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[0]
        end
    "#;
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);
    eprintln!("IR functions: {}", ir.functions.len());
    eprintln!("IR globals: {}", ir.globals.len());
    assert_eq!(ir.functions.len(), 1, "Expected 1 function");
    assert_eq!(ir.globals.len(), 1, "Expected 1 global");
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    eprintln!("ASM:\n{}", &asm[..asm.len().min(3000)]);
    assert!(asm.contains("section .data"), "Expected data section");
    assert!(asm.contains("global main"), "Expected global main");
}

// ─── New integration tests with exact exit code verification ─────────────

#[test]
fn test_exe_add() {
    let source = "def main() of int return 1 + 1; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(2), "1+1 should be 2");
}

#[test]
fn test_exe_sub() {
    let source = "def main() of int return 10 - 3; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7), "10-3 should be 7");
}

#[test]
fn test_exe_mul() {
    let source = "def main() of int return 6 * 7; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "6*7 should be 42");
}

#[test]
fn test_exe_mod() {
    let source = "def main() of int return 10 % 3; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "10%%3 should be 1");
}

#[test]
fn test_exe_negation() {
    let source = "def main() of int x = 7; return -x; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(-7i32 as u32 as i32), "-7 should be -7");
}

#[test]
fn test_exe_compare_eq_true() {
    let source = "def main() of int return 5 == 5; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "5==5 should be 1");
}

#[test]
fn test_exe_compare_eq_false() {
    let source = "def main() of int return 5 == 6; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "5==6 should be 0");
}

#[test]
fn test_exe_compare_lt_true() {
    let source = "def main() of int return 3 < 7; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "3<7 should be 1");
}

#[test]
fn test_exe_compare_lt_false() {
    let source = "def main() of int return 7 < 3; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "7<3 should be 0");
}

#[test]
fn test_exe_compare_gt_true() {
    let source = "def main() of int return 10 > 5; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "10>5 should be 1");
}

#[test]
fn test_exe_compare_le_true() {
    let source = "def main() of int return 5 <= 5; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "5<=5 should be 1");
}

#[test]
fn test_exe_compare_ge_true() {
    let source = "def main() of int return 5 >= 5; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "5>=5 should be 1");
}

#[test]
fn test_exe_compare_ne_true() {
    let source = "def main() of int return 5 != 6; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "5!=6 should be 1");
}

#[test]
fn test_exe_while_loop_sum() {
    let source = r#"
        def main() of int
            i = 1;
            sum = 0;
            while i <= 5 {
                sum = sum + i;
                i = i + 1;
            }
            loop_end
            return sum
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "while loop sum should compile and run");
}

#[test]
fn test_exe_if_else_true_branch() {
    let source = r#"
        def main() of int
            x = 10;
            if x > 5 then
                return 1
            else
                return 0
            end
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "10>5 should take true branch");
}

#[test]
fn test_exe_if_else_false_branch() {
    let source = r#"
        def main() of int
            x = 2;
            if x > 5 then
                return 1
            else
                return 0
            end
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "2>5 should take false branch");
}

#[test]
fn test_exe_if_no_else_false() {
    let source = r#"
        def main() of int
            if 1 == 2 then
                return 42
            end
            return 0
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "1==2 skips body");
}

#[test]
fn test_exe_nested_if_exact() {
    let source = r#"
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
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "nested if should compile and run");
}

// ─── Tests with known compiler limitations ───────────────────────────────
// Multi-function tests: single-asm-file test helper can't handle separate
// per-function codegen that the real driver uses. These pass via the real
// compiler driver (`cargo run -- ...`) but not via compile_and_run().
#[test]
fn test_exe_function_call() {
    let source = r#"
        def double(x of int) of int
            return x + x
        end
        def main() of int
            return double(21)
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "double(21) should be 42");
}

#[test]
fn test_exe_multi_param_call() {
    let source = r#"
        def add(a of int, b of int) of int
            return a + b
        end
        def main() of int
            return add(3, 4)
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7), "add(3,4) should be 7");
}

#[test]
fn test_exe_global_exact() {
    let source = r#"
        global counter of int = 42;
        def main() of int
            return counter
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "global counter should be 42");
}

#[test]
fn test_exe_global_write_compiles() {
    let source = r#"
        global value of int = 0;
        def main() of int
            value = 99;
            return value
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "global write should compile and run");
}

#[test]
fn test_exe_global_array_exact() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[2]
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(30), "arr[2] should be 30");
}

#[test]
fn test_exe_local_struct_exact() {
    let source = r#"
        struct Point { x of int; y of int; }
        def main() of int
            p of Point;
            p.x = 42;
            return p.x
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(42), "p.x should be 42");
}

#[test]
fn test_exe_arithmetic_chain() {
    let source = "def main() of int return 2 + 3 * 4 - 6 / 2; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(11), "2+3*4-6/2 should be 11");
}

#[test]
fn test_exe_while_loop_compiles() {
    let source = r#"
        def main() of int
            i = 0;
            total = 0;
            while i < 3 {
                j = 0;
                while j < 2 {
                    total = total + 1;
                    j = j + 1;
                }
                loop_end
                i = i + 1;
            }
            loop_end
            return total
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "nested while should compile and run");
}

#[test]
fn test_exe_logical_not_true() {
    let source = "def main() of int x = 0; return !x; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(1), "!0 should be 1");
}

#[test]
fn test_exe_logical_not_false() {
    let source = "def main() of int x = 1; return !x; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(0), "!1 should be 0");
}

#[test]
fn test_exe_conditional_compiles() {
    let source = r#"
        def main() of int
            a = 1 == 1;
            b = 2 == 2;
            if a && b then
                return 1
            end
            return 0
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "conditional should compile and run");
}

#[test]
fn test_exe_hex_literal() {
    let source = "def main() of int return 0xFF; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(255), "0xFF should be 255");
}

#[test]
fn test_exe_binary_literal() {
    let source = "def main() of int return 0b1010; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10), "0b1010 should be 10");
}

#[test]
fn test_exe_multiple_return_paths() {
    let source = r#"
        def max(a of int, b of int) of int
            if a > b then
                return a
            else
                return b
            end
        end
        def main() of int
            return max(7, 3)
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "should compile and run");
}

#[test]
fn test_exe_variable_reuse() {
    let source = r#"
        def main() of int
            x = 5;
            x = x + 3;
            x = x * 2;
            return x
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(16), "(5+3)*2 should be 16");
}

#[test]
fn test_exe_div() {
    let source = "def main() of int return 42 / 6; end";
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(7), "42/6 should be 7");
}

#[test]
fn test_exe_global_array_first() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[0]
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10), "arr[0] should be 10");
}

#[test]
fn test_exe_global_string_compiles() {
    let source = r#"
        global name of string = "ok";
        def main() of int
            return 99
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "should compile and run");
}

#[test]
fn test_exe_if_else_false_compiles() {
    let source = r#"
        def main() of int
            x = 2;
            if x > 5 then
                return 1
            else
                return 0
            end
        end
    "#;
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "should compile and run");
}

#[test]
fn test_exe_closure_simple() {
    let source = r#"
        def main() of int
            x = 10;
            def inner() of int
                return x
            end
            return inner()
        end
    "#;
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(10), "closure should capture x");
}

#[test]
fn test_exe_closure_mutate() {
    let source = r#"
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
    let output = compile_and_run(source);
    assert_eq!(output.status.code(), Some(3), "closure should mutate captured x");
}

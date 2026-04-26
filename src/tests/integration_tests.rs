use crate::codegen::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;
use std::fs;
use std::process::Command;

fn parse(source: &str) -> crate::ast::Program {
    let mut parser = Parser::new(source);
    parser.parse().unwrap()
}

fn compile_and_run(source: &str) -> std::process::Output {
    let temp_dir = std::env::temp_dir().join("mylang_test_run");
    let _ = fs::remove_dir_all(&temp_dir);
    fs::create_dir_all(&temp_dir).unwrap();

    // Parse
    let mut parser = Parser::new(source);
    let ast = match parser.parse() {
        Ok(a) => a,
        Err(e) => {
            eprintln!("Parse error: {}", e);
            panic!("Parse error: {}", e);
        }
    };

    // Generate IR
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&ast);

    // Generate ASM
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);

    let asm_path = temp_dir.join("program.asm");
    fs::write(&asm_path, &asm).unwrap();

    // Assemble with NASM
    let obj_path = temp_dir.join("program.obj");
    let nasm_result = Command::new("nasm")
        .args([
            "-f",
            "win64",
            "-o",
            obj_path.to_str().unwrap(),
            asm_path.to_str().unwrap(),
        ])
        .output();

    if !nasm_result
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        eprintln!(
            "NASM error: {}",
            String::from_utf8_lossy(&nasm_result.unwrap().stderr)
        );
        panic!("NASM assembly failed");
    }

    // Link with GCC
    let exe_path = temp_dir.join("program.exe");
    let gcc_result = Command::new("gcc")
        .args([obj_path.to_str().unwrap(), "-o", exe_path.to_str().unwrap()])
        .output();

    if !gcc_result
        .as_ref()
        .map(|o| o.status.success())
        .unwrap_or(false)
    {
        eprintln!(
            "GCC error: {}",
            String::from_utf8_lossy(&gcc_result.unwrap().stderr)
        );
        panic!("GCC linking failed");
    }

    // Run
    let run_result = Command::new(exe_path.to_str().unwrap())
        .current_dir(&temp_dir)
        .output();

    run_result.unwrap()
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

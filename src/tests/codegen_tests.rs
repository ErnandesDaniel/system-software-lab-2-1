use crate::codegen::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::parser::Parser;

fn parse(source: &str) -> crate::ast::Program {
    let mut parser = Parser::new(source);
    parser.parse().unwrap()
}

#[test]
fn test_ir_generation_simple() {
    let source = "def foo() return 42; end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    assert_eq!(ir.functions.len(), 1);
    assert_eq!(ir.functions[0].name, "foo");
}

#[test]
fn test_asm_generation_simple() {
    let source = "def foo() return 42; end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("foo:"));
    assert!(asm.contains("ret"));
}

#[test]
fn test_extern_short_form_codegen() {
    let source = "extern puts def main() puts(\"hello\"); end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(
        asm.contains("extern puts"),
        "Expected 'extern puts' in asm output"
    );
}

#[test]
fn test_assembler_output_format() {
    let source = "def square(x of int) of int return x * x; end";
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();

    let mut ir_gen = IrGenerator::new();
    let ir_program = ir_gen.generate(&ast);

    let mut asm_gen = AsmGenerator::new();
    let asm_output = asm_gen.generate(&ir_program);

    eprintln!("ASM Output:\n{}", asm_output);

    assert!(asm_output.contains("bits 64"));
    assert!(asm_output.contains("default rel"));
    assert!(asm_output.contains("section .text"));
    assert!(asm_output.contains("global square"));
    assert!(asm_output.contains("square:"));
    assert!(asm_output.contains("BB_"));
    assert!(asm_output.contains("push rbp"));
    assert!(asm_output.contains("mov rbp, rsp"));
    assert!(asm_output.contains("imul"));
}

mod ir;
mod asm;

use crate::codegen::nasm::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;

#[test]
fn test_ir_generation_simple() {
    let source = "def foo() { return 42; }";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    assert_eq!(ir.functions.len(), 1);
    assert_eq!(ir.functions[0].name, "foo");
}

#[test]
fn test_asm_generation_simple() {
    let source = "def foo() { return 42; }";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("foo:"));
    assert!(asm.contains("ret"));
}

#[test]
fn test_import_short_form_codegen() {
    let source = "import puts def main() { puts(\"hello\"); }";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("extern puts"), "Expected 'extern puts' in asm output");
}

#[test]
fn test_assembler_output_format() {
    let ast = parse("def square(x of int) of int { return x * x; }");
    let mut ir_gen = IrGenerator::new();
    let ir_program = ir_gen.generate(&ast);
    let mut asm_gen = AsmGenerator::new();
    let asm_output = asm_gen.generate(&ir_program);
    assert!(asm_output.contains("bits 64"));
    assert!(asm_output.contains("default rel"));
    assert!(asm_output.contains("section .text"));
    assert!(asm_output.contains("global square"));
    assert!(asm_output.contains("square:"));
    assert!(asm_output.contains("square_BB"));
    assert!(asm_output.contains("push rbp"));
    assert!(asm_output.contains("mov rbp, rsp"));
    assert!(asm_output.contains("imul"));
}

#[test]
fn test_ir_generation_multiple_functions() {
    let source = r#"
        def add(a of int, b of int) of int {
            return a + b;
        }
        def main() of int {
            return add(1, 2);
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    assert_eq!(ir.functions.len(), 2);
    assert_eq!(ir.functions[0].name, "add");
    assert_eq!(ir.functions[1].name, "main");
}

use crate::codegen::nasm::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;

#[test]
fn test_asm_function_call() {
    let source = r#"
        def double(x of int) of int
            return x + x
        end
        def main() of int
            return double(21)
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("double:"));
    assert!(asm.contains("main:"));
    assert!(asm.contains("ret"));
}

#[test]
fn test_asm_if_else_structure() {
    let source = r#"
        def max(a of int, b of int) of int
            if a > b then
                return a
            else
                return b
            end
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("max:"));
    assert!(asm.contains("ret"));
    assert!(asm.contains("cmp"));
}

#[test]
fn test_asm_string_literal() {
    let source = r#"
        def main() of int
            s = "hello";
            return 0
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("main:"));
    assert!(asm.contains("ret"));
    assert!(
        asm.contains("hello") || asm.contains("db ") || asm.contains("section .data"),
        "Expected string data in asm, got: {}",
        &asm[..asm.len().min(500)]
    );
}

#[test]
fn test_asm_while_loop_structure() {
    let source = r#"
        def sum() of int
            i = 1;
            total = 0;
            while i <= 10 {
                total = total + i;
                i = i + 1;
            }
            loop_end
            return total
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("sum:"));
    assert!(asm.contains("ret"));
    assert!(asm.contains("cmp"));
    assert!(asm.contains("j"));
}

#[test]
fn test_asm_unary_negate() {
    let source = "def main() of int x = 7; return -x; end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("neg eax"));
}

#[test]
fn test_asm_logical_not() {
    let source = "def main() of int x = 0; return !x; end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("test") || asm.contains("xor eax, 1"));
}

#[test]
fn test_asm_comparison_eq() {
    let source = "def main() of int return 1 == 2; end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("cmp"));
    assert!(asm.contains("sete"));
}

#[test]
fn test_asm_comparison_lt() {
    let source = "def main() of int return 5 < 10; end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("cmp"));
    assert!(asm.contains("setl"));
}

#[test]
fn test_asm_comparison_gt() {
    let source = "def main() of int return 10 > 5; end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("cmp"));
    assert!(asm.contains("setg"));
}

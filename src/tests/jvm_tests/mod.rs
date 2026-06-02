mod advanced;
mod basic;

use crate::codegen::jvm::JvmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;

#[test]
fn test_jvm_generation_simple() {
    let source = "def main() of int { return 42; }";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Main");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_with_locals() {
    let source = r#"
        def main() of int {
            a = 10;
            b = 20;
            return a + b;
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Main");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_arithmetic() {
    let source = r#"
        def calc(x of int, y of int) of int {
            sum = x + y;
            diff = x - y;
            prod = x * y;
            return sum + diff + prod;
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Calc");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_comparison() {
    let source = r#"
        def compare(a of int, b of int) of int {
            if (a == b) { return 1; }
            return 0;
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_multiple_functions() {
    let source = r#"
        def add(a of int, b of int) of int {
            return a + b;
        }
        def main() of int {
            result = add(10, 20);
            return result;
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 2);
    for (_, bytes) in &classes {
        assert_eq!(bytes[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
    }
}

#[test]
fn test_jvm_generation_with_loop() {
    let source = r#"
        def main() of int {
            i = 1;
            while (i < 5) {
                i = i + 1;
            }
            return i;
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Main");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

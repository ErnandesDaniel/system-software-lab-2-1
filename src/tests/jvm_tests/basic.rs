use crate::codegen::jvm::JvmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;

#[test]
fn test_jvm_generation_negation() {
    let source = r#"
        def negate(x of int) of int
            return -x
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Negate");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_if_statement() {
    let source = r#"
        def max(a of int, b of int) of int
            if a > b then
                return a
            end
            return b
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Max");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_if_else_statement() {
    let source = r#"
        def abs(x of int) of int
            if x >= 0 then
                return x
            else
                return -x
            end
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Abs");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_nested_if() {
    let source = r#"
        def sign(x of int) of int
            if x > 0 then
                return 1
            else
                if x < 0 then
                    return -1
                else
                    return 0
                end
            end
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Sign");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_comparison_operators() {
    let source = r#"
        def cmp(a of int, b of int) of int
            if a < b then return 1; end
            if a > b then return 2; end
            if a <= b then return 3; end
            if a >= b then return 4; end
            if a != b then return 5; end
            return 0
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Cmp");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_string_return() {
    let source = r#"
        def greet() of string
            return "hello"
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Greet");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

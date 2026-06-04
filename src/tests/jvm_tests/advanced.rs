use crate::codegen::jvm::JvmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;

#[test]
fn test_jvm_generation_nested_while() {
    let source = r#"
        def main() of int {
            i = 0;
            while (i < 3) {
                j = 0;
                while (j < 2) {
                    j = j + 1;
                }
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

#[test]
fn test_jvm_generation_begin_end_block() {
    let source = r#"
        def foo() of int {
            x = 1;
            return x;
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Foo");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_import_call_with_string() {
    let source = r#"
        import def puts(msg of string);
        def main() of int {
            puts("hello from jvm");
            return 0;
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 2);
    assert_eq!(classes[0].0, "Main");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
    assert_eq!(classes[1].0, "RuntimeStub");
}

#[test]
fn test_jvm_generation_logical_not() {
    let source = r#"
        def not_test(x of int) of int {
            if (!(x == 0)) {
                return 1;
            }
            return 0;
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Not_test");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_minus_expression() {
    let source = r#"
        def diff(a of int, b of int) of int {
            return a - b;
        }
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);
    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Diff");
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

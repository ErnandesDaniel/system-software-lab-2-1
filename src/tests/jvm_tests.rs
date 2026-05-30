use crate::codegen::jvm::JvmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;

#[test]
fn test_jvm_generation_simple() {
    let source = "def main() of int return 42; end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);

    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);

    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Main");
    // Check that the class file was generated (has magic number 0xCAFEBABE)
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_with_locals() {
    let source = r#"
        def main() of int
            a = 10;
            b = 20;
            return a + b
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);

    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);

    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Main");
    // Class file should be valid
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_arithmetic() {
    let source = r#"
        def calc(x of int, y of int) of int
            sum = x + y;
            diff = x - y;
            prod = x * y;
            return sum + diff + prod
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);

    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);

    assert_eq!(classes.len(), 1);
    assert_eq!(classes[0].0, "Calc");
    // Class file should be valid
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_generation_comparison() {
    let source = r#"
        def compare(a of int, b of int) of int
            if a == b then return 1; end
            return 0
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);

    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);

    assert_eq!(classes.len(), 1);
    // Class file should be valid
    assert_eq!(classes[0].1[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
}

#[test]
fn test_jvm_multiple_functions() {
    let source = r#"
        def add(a of int, b of int) of int
            return a + b
        end
        
        def main() of int
            result = add(10, 20);
            return result
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);

    let mut jvm_gen = JvmGenerator::new();
    let classes = jvm_gen.generate_program(&ir);

    // Should generate 2 classes (one per function)
    assert_eq!(classes.len(), 2);

    // Both should be valid class files
    for (_, bytes) in &classes {
        assert_eq!(bytes[0..4], [0xCA, 0xFE, 0xBA, 0xBE]);
    }
}

#[test]
fn test_jvm_generation_with_loop() {
    let source = r#"
        def main() of int
            i = 1;
            while i < 5 {
                i = i + 1;
            }
            loop_end
            return i
        end
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
    // Class file should be valid
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
    // Class file should be valid
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
    // Class file should be valid
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

#[test]
fn test_jvm_generation_nested_while() {
    let source = r#"
        def main() of int
            i = 0;
            while i < 3 {
                j = 0;
                while j < 2 {
                    j = j + 1;
                }
                loop_end
                i = i + 1;
            }
            loop_end
            return i
        end
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
        def foo() of int
        begin
            x = 1;
            return x
        end
        end
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
        import def puts(msg of string) end
        def main() of int
            puts("hello from jvm");
            return 0
        end
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
fn test_jvm_generation_logical_not() {
    let source = r#"
        def not_test(x of int) of int
            if !(x == 0) then
                return 1
            end
            return 0
        end
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
        def diff(a of int, b of int) of int
            return a - b
        end
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

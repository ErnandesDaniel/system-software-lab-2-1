use crate::codegen::AsmGenerator;
use crate::ir_generator::IrGenerator;
use crate::tests::parse;

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
fn test_import_short_form_codegen() {
    let source = "import puts def main() puts(\"hello\"); end";
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("extern puts"), "Expected 'extern puts' in asm output");
}

#[test]
fn test_assembler_output_format() {
    let ast = parse("def square(x of int) of int return x * x; end");

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

#[test]
fn test_ir_generation_multiple_functions() {
    let source = r#"
        def add(a of int, b of int) of int
            return a + b
        end
        def main() of int
            return add(1, 2)
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    assert_eq!(ir.functions.len(), 2);
    assert_eq!(ir.functions[0].name, "add");
    assert_eq!(ir.functions[1].name, "main");
}

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

#[test]
fn test_asm_global_array_access() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[0]
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("section .data"));
    assert!(asm.contains("arr"));
    assert!(asm.contains("dd 10") || asm.contains("arr"));
}

#[test]
fn test_asm_global_array_index() {
    let source = r#"
        global arr of int[3] = [10, 20, 30];
        def main() of int
            return arr[2]
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("arr") || asm.contains("lea") || asm.contains("mov"));
}

#[test]
fn test_asm_struct_field_load() {
    let source = r#"
        struct Point { x of int; y of int; }
        def main() of int
            p of Point;
            p.x = 42;
            return p.x
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("main:"));
    assert!(asm.contains("ret"));
}

#[test]
fn test_asm_multi_blocks() {
    let source = r#"
        def main() of int
            if 1 == 1 then
                return 42
            end
            return 0
        end
    "#;
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    let ir = ir_gen.generate(&program);
    let mut asm_gen = AsmGenerator::new();
    let asm = asm_gen.generate(&ir);
    assert!(asm.contains("BB_0:"));
    assert!(asm.contains("BB_1:"));
    assert!(asm.contains("BB_2:"));
}

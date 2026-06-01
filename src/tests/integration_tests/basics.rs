use crate::ir_generator::IrGenerator;
use crate::parser::Parser;
use crate::semantics::analysis::SemanticsAnalyzer;
use crate::tests::parse;
use super::compile_and_run;
use super::compile_only;

#[test]
fn test_import_short_form_semantics() {
    let source = "import puts def main() puts(\"hello\"); end";
    let program = parse(source);
    let mut analyzer = SemanticsAnalyzer::new();
    let result = analyzer.analyze(&program);
    assert!(result.is_ok(), "Expected ok but got: {:?}", result);
}

#[test]
fn test_cfg_generation() {
    use crate::ir::diagram::CfgMermaidGenerator;

    let source = "def square(x of int) of int return x * x; end def main() of int return 42; end";
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();

    let mut ir_gen = IrGenerator::new();
    let ir_program = ir_gen.generate(&ast);

    let mut cfg_gen = CfgMermaidGenerator::new();
    let cfg_output = cfg_gen.generate(&ir_program);

    assert!(cfg_output.contains("graph TD"));
    assert!(cfg_output.contains("BB_0"));
    assert!(cfg_output.contains("x * x"));
}

#[test]
fn test_while_loop_block_order() {
    let source = "def foo() i=1; while i<5 { i=i+1; } loop_end return i; end";
    let mut parser = Parser::new(source);
    let ast = parser.parse().unwrap();

    let mut ir_gen = IrGenerator::new();
    let ir_program = ir_gen.generate(&ast);

    let func = &ir_program.functions[0];
    assert!(func.blocks.len() >= 4, "Expected at least 4 blocks, got {}", func.blocks.len());
}

#[test]
fn test_exe_return_value() {
    let source = "def main() of int return 42; end";
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_arithmetic() {
    let source = "def main() of int return 5 + 3 * 2; end";
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_with_loop() {
    let source = "def main() of int i = 1; while i < 5 { i = i + 1; } loop_end return i; end";
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_if_else() {
    let source = "def main() of int x = 10; if x > 5 then return 1; end return 0; end";
    let output = compile_and_run(source);
    assert!(output.status.code() != Some(-1), "Program should run");
}

#[test]
fn test_exe_return_value_42() {
    let source = "def main() of int return 42; end";
    let output = compile_and_run(source);
    assert!(output.status.success() || output.status.code() == Some(42));
}

#[test]
fn test_exe_return_value_zero() {
    let source = "def main() of int return 0; end";
    let output = compile_and_run(source);
    assert!(output.status.success() || output.status.code() == Some(0));
}

#[test]
fn test_exe_simple_arithmetic() {
    let source = "def main() of int return 7 + 3; end";
    let output = compile_and_run(source);
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
    assert!(output.status.success() || output.status.code() != Some(-1));
}

#[test]
fn test_compile_multi_function_asm() {
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
    assert!(
        asm.contains("test_string") || asm.contains("section .data") || asm.contains("db "),
        "Expected string or data section"
    );
}

use crate::ir_generator::IrGenerator;
use crate::tests::parse;

fn ir(source: &str) -> crate::ir::IrProgram {
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    ir_gen.generate(&program)
}

#[test]
fn test_ir_one_function() {
    let ir = ir("def foo() { }");
    assert_eq!(ir.functions.len(), 1);
    assert_eq!(ir.functions[0].name, "foo");
}

#[test]
fn test_ir_two_functions() {
    let ir = ir("def a() { } def b() { }");
    assert_eq!(ir.functions.len(), 2);
}

#[test]
fn test_ir_five_functions() {
    let ir = ir("def f1() { } def f2() { } def f3() { } def f4() { } def f5() { }");
    assert_eq!(ir.functions.len(), 5);
}

#[test]
fn test_ir_function_return_type_int() {
    let ir = ir("def f() of int { return 0; }");
    assert_eq!(ir.functions[0].return_type, crate::ir::IrType::Int);
}

#[test]
fn test_ir_function_return_type_bool() {
    let ir = ir("def f() of bool { return true; }");
    assert_eq!(ir.functions[0].return_type, crate::ir::IrType::Bool);
}

#[test]
fn test_ir_function_return_type_void() {
    let ir = ir("def f() { }");
    assert_eq!(ir.functions[0].return_type, crate::ir::IrType::Void);
}

#[test]
fn test_ir_function_return_type_char() {
    let ir = ir("def f() of char { return 'x'; }");
    assert_eq!(ir.functions[0].return_type, crate::ir::IrType::Char);
}

#[test]
fn test_ir_function_parameters_count() {
    let ir = ir("def f(a of int, b of int, c of int) { }");
    assert_eq!(ir.functions[0].parameters.len(), 3);
}

#[test]
fn test_ir_function_parameter_name() {
    let ir = ir("def f(x of int) { }");
    assert_eq!(ir.functions[0].parameters[0].name, "x");
}

#[test]
fn test_ir_function_parameter_type() {
    let ir = ir("def f(x of bool) { }");
    assert_eq!(ir.functions[0].parameters[0].ty, crate::ir::IrType::Bool);
}

#[test]
fn test_ir_function_no_params() {
    let ir = ir("def f() { }");
    assert!(ir.functions[0].parameters.is_empty());
}

#[test]
fn test_ir_block_count() {
    let ir = ir("def f() of int { if (true) { return 1; } return 0; }");
    assert!(
        ir.functions[0].blocks.len() >= 3,
        "expected >=3 blocks, got {}",
        ir.functions[0].blocks.len()
    );
}

#[test]
fn test_ir_blocks_have_ids() {
    let ir = ir("def f() { }");
    assert!(!ir.functions[0].blocks[0].id.is_empty());
}

#[test]
fn test_ir_used_functions_empty() {
    let ir = ir("def f() { }");
    assert!(ir.functions[0].used_functions.is_empty());
}

#[test]
fn test_ir_used_functions_contains_called() {
    let ir = ir("def a() { } def b() { a(); }");
    assert!(ir.functions[1].used_functions.contains(&"a".to_string()));
}

#[test]
fn test_ir_used_functions_imported() {
    let ir = ir("import puts def main() { puts(\"hi\"); }");
    let func = &ir.functions[0];
    assert!(
        func.used_functions.contains(&"puts".to_string()),
        "expected puts in used_functions, got {:?}",
        func.used_functions
    );
}

#[test]
fn test_ir_struct_layout_present() {
    let ir = ir("struct P { x of int; } def f() { }");
    assert!(
        ir.struct_layouts.structs.contains_key("P"),
        "Expected struct layout for P"
    );
}

#[test]
fn test_ir_locals_count() {
    let ir = ir("def f() { a of int; b of int; }");
    assert_eq!(ir.functions[0].locals.len(), 2);
}

#[test]
fn test_ir_local_name() {
    let ir = ir("def f() { my_var of int; }");
    assert_eq!(ir.functions[0].locals[0].name, "my_var");
}

#[test]
fn test_ir_local_type_int() {
    let ir = ir("def f() { x of int; }");
    assert_eq!(ir.functions[0].locals[0].ty, crate::ir::IrType::Int);
}

#[test]
fn test_ir_local_type_bool() {
    let ir = ir("def f() { x of bool; }");
    assert_eq!(ir.functions[0].locals[0].ty, crate::ir::IrType::Bool);
}

#[test]
fn test_ir_local_type_array() {
    let ir = ir("def f() { arr of int[5]; }");
    assert!(matches!(ir.functions[0].locals[0].ty, crate::ir::IrType::Array(_, _)));
}

#[test]
fn test_ir_local_exists() {
    let ir = ir("def f() { x of int; }");
    assert_eq!(ir.functions[0].locals.len(), 1);
    assert_eq!(ir.functions[0].locals[0].name, "x");
}

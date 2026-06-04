use crate::ir_generator::IrGenerator;
use crate::tests::parse;

fn ir(source: &str) -> crate::ir::IrProgram {
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    ir_gen.generate(&program)
}

#[test]
fn test_ir_no_globals() {
    let ir = ir("def f() { }");
    assert!(ir.globals.is_empty());
}

#[test]
fn test_ir_one_global() {
    let ir = ir("global x of int = 5; def f() { }");
    assert_eq!(ir.globals.len(), 1);
    assert_eq!(ir.globals[0].name, "x");
}

#[test]
fn test_ir_global_with_int_initializer() {
    let ir = ir("global x of int = 42; def f() { }");
    assert_eq!(ir.globals.len(), 1);
    assert!(ir.globals[0].initializer.is_some());
}

#[test]
fn test_ir_global_no_initializer() {
    let ir = ir("global x of int; def f() { }");
    assert!(ir.globals[0].initializer.is_none());
}

#[test]
fn test_ir_global_type_int() {
    let ir = ir("global x of int = 0; def f() { }");
    assert_eq!(ir.globals[0].ty, crate::ir::IrType::Int);
}

#[test]
fn test_ir_global_type_bool() {
    let ir = ir("global x of bool = true; def f() { }");
    assert_eq!(ir.globals[0].ty, crate::ir::IrType::Bool);
}

#[test]
fn test_ir_global_type_string() {
    let ir = ir(r#"global x of string = "hello"; def f() { }"#);
    assert_eq!(ir.globals[0].ty, crate::ir::IrType::String);
}

#[test]
fn test_ir_many_globals() {
    let src = (0..10).map(|i| format!("global g{i} of int = {i};")).collect::<Vec<_>>().join("\n");
    let ir = ir(&format!("{src} def f() {{ }}"));
    assert_eq!(ir.globals.len(), 10);
}

#[test]
fn test_ir_control_flow_has_branching() {
    let ir = ir("def f() of int { if (true) { return 1; } return 0; }");
    let has_branch = ir.functions[0].blocks.len() >= 3;
    assert!(has_branch, "Expected >=3 blocks for if-else, got {}", ir.functions[0].blocks.len());
}

#[test]
fn test_ir_block_successors_simple() {
    let ir = ir("def f() of int { if (true) { return 1; } return 0; }");
    let has_successors = ir.functions[0].blocks.iter()
        .any(|b| !b.successors.is_empty());
    assert!(has_successors, "Expected at least one block with successors");
}

#[test]
fn test_ir_multiple_blocks_function() {
    let ir = ir("def f() of int { if (true) { return 1; } else { return 2; } }");
    assert!(ir.functions[0].blocks.len() >= 3);
}

#[test]
fn test_ir_and_or_control_flow() {
    let ir = ir("def f() of bool { return true && false; }");
    let has_branch = ir.functions[0].blocks.len() >= 2;
    assert!(has_branch, "Expected multiple blocks for && short-circuit");
}

#[test]
fn test_ir_or_control_flow() {
    let ir = ir("def f() of bool { return true || false; }");
    let has_branch = ir.functions[0].blocks.len() >= 2;
    assert!(has_branch, "Expected multiple blocks for || short-circuit");
}

#[test]
fn test_ir_block_ordering() {
    let ir = ir("def f() of int { return 0; }");
    assert!(!ir.functions[0].blocks[0].id.is_empty());
}

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
    assert!(ir.functions[0].blocks.len() >= 3, "expected >=3 blocks, got {}", ir.functions[0].blocks.len());
}
#[test]
fn test_ir_blocks_have_ids() {
    let ir = ir("def f() { }");
    assert!(!ir.functions[0].blocks[0].id.is_empty());
}
#[test]
fn test_ir_ret_instruction() {
    let ir = ir("def f() of int { return 42; }");
    let block = &ir.functions[0].blocks[0];
    let has_ret = block.instructions.iter().any(|i| i.opcode == crate::ir::IrOpcode::Ret);
    assert!(has_ret, "Expected Ret instruction");
}
#[test]
fn test_ir_ret_void() {
    let ir = ir("def f() { }");
    let has_ret = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Ret);
    assert!(has_ret, "Expected Ret instruction");
}
#[test]
fn test_ir_add_instruction() {
    let ir = ir("def f() of int { return 1 + 2; }");
    let has_add = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Add);
    assert!(has_add, "Expected Add instruction");
}
#[test]
fn test_ir_sub_instruction() {
    let ir = ir("def f() of int { return 5 - 3; }");
    let has_sub = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Sub);
    assert!(has_sub, "Expected Sub instruction");
}
#[test]
fn test_ir_mul_instruction() {
    let ir = ir("def f() of int { return 3 * 4; }");
    let has_mul = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Mul);
    assert!(has_mul, "Expected Mul instruction");
}
#[test]
fn test_ir_div_instruction() {
    let ir = ir("def f() of int { return 8 / 2; }");
    let has_div = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Div);
    assert!(has_div, "Expected Div instruction");
}
#[test]
fn test_ir_eq_instruction() {
    let ir = ir("def f() of bool { return 1 == 2; }");
    let has_eq = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Eq);
    assert!(has_eq, "Expected Eq instruction");
}
#[test]
fn test_ir_lt_instruction() {
    let ir = ir("def f() of bool { return 1 < 2; }");
    let has_lt = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Lt);
    assert!(has_lt, "Expected Lt instruction");
}
#[test]
fn test_ir_call_instruction() {
    let ir = ir("import def foo(); def f() { foo(); }");
    let has_call = ir.functions.iter()
        .flat_map(|f| f.blocks.iter())
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Call);
    assert!(has_call, "Expected Call instruction");
}
#[test]
fn test_ir_call_with_args() {
    let ir = ir("import def foo(x of int); def f() { foo(42); }");
    let has_call = ir.functions.iter()
        .flat_map(|f| f.blocks.iter())
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Call);
    assert!(has_call);
}
#[test]
fn test_ir_cond_br_instruction() {
    let ir = ir("def f() of int { if (true) { return 1; } return 0; }");
    let has_cond = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::CondBr);
    assert!(has_cond, "Expected CondBr instruction");
}
#[test]
fn test_ir_control_flow_has_branching() {
    let ir = ir("def f() of int { if (true) { return 1; } return 0; }");
    let has_branch = ir.functions[0].blocks.len() >= 3;
    assert!(has_branch, "Expected >=3 blocks for if-else, got {}", ir.functions[0].blocks.len());
}
#[test]
fn test_ir_neg_instruction() {
    let ir = ir("def f(x of int) of int { return -x; }");
    let has_neg = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Neg);
    assert!(has_neg, "Expected Neg instruction");
}
#[test]
fn test_ir_not_instruction() {
    let ir = ir("def f(x of bool) of bool { return !x; }");
    let has_not = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Not);
    assert!(has_not, "Expected Not instruction");
}
#[test]
fn test_ir_assign_instruction() {
    let ir = ir("def f() { x = 42; }");
    let has_assign = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .find(|i| i.opcode == crate::ir::IrOpcode::Assign);
    assert!(has_assign.is_some(), "Expected Assign instruction");
}
#[test]
fn test_ir_load_or_assign_instruction() {
    let ir = ir("def f() of int { x = 42; return x; }");
    let has_load_or_assign = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| matches!(i.opcode, crate::ir::IrOpcode::Load | crate::ir::IrOpcode::Assign));
    assert!(has_load_or_assign, "Expected Load or Assign instruction");
}
#[test]
fn test_ir_assign_or_store_instruction() {
    let ir = ir("def f() { x of int; x = 5; }");
    let has_assign_or_store = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| matches!(i.opcode, crate::ir::IrOpcode::Assign | crate::ir::IrOpcode::Store));
    assert!(has_assign_or_store, "Expected Assign or Store instruction");
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
    assert!(func.used_functions.contains(&"puts".to_string()), "expected puts in used_functions, got {:?}", func.used_functions);
}
#[test]
fn test_ir_not_coroutine() {
    let ir = ir("def f() { }");
    assert!(!ir.functions[0].is_coroutine);
}
#[test]
fn test_ir_is_coroutine() {
    let ir = ir("coroutine f() of int { yield; return 0; }");
    assert!(ir.functions[0].is_coroutine);
}
#[test]
fn test_ir_coroutine_yield_instructions() {
    let ir = ir("coroutine f() of int { yield; return 0; }");
    let has_yield = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::CoroYield);
    assert!(has_yield, "Expected CoroYield instruction");
}
#[test]
fn test_ir_yield_count() {
    let ir = ir("coroutine f() of int { yield; yield; yield; return 0; }");
    assert_eq!(ir.functions[0].yield_count, 3);
}
#[test]
fn test_ir_no_yield_in_regular_func() {
    let ir = ir("def f() { }");
    assert_eq!(ir.functions[0].yield_count, 0);
}
#[test]
fn test_ir_struct_layout_present() {
    let ir = ir("struct P { x of int; } def f() { }");
    assert!(ir.struct_layouts.structs.contains_key("P"), "Expected struct layout for P");
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
fn test_ir_bitand_instruction() {
    let ir = ir("def f() of int { return 1 & 3; }");
    let has_bitand = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::BitAnd);
    assert!(has_bitand, "Expected BitAnd instruction");
}
#[test]
fn test_ir_bitor_instruction() {
    let ir = ir("def f() of int { return 1 | 2; }");
    let has_bitor = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::BitOr);
    assert!(has_bitor, "Expected BitOr instruction");
}
#[test]
fn test_ir_bitxor_instruction() {
    let ir = ir("def f() of int { return 1 ^ 3; }");
    let has_bitxor = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::BitXor);
    assert!(has_bitxor, "Expected BitXor instruction");
}
#[test]
fn test_ir_ge_instruction() {
    let ir = ir("def f() of bool { return 5 >= 3; }");
    let has_ge = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Ge);
    assert!(has_ge, "Expected Ge instruction");
}
#[test]
fn test_ir_le_instruction() {
    let ir = ir("def f() of bool { return 3 <= 5; }");
    let has_le = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Le);
    assert!(has_le, "Expected Le instruction");
}
#[test]
fn test_ir_ne_instruction() {
    let ir = ir("def f() of bool { return 1 != 2; }");
    let has_ne = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Ne);
    assert!(has_ne, "Expected Ne instruction");
}
#[test]
fn test_ir_gt_instruction() {
    let ir = ir("def f() of bool { return 10 > 5; }");
    let has_gt = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Gt);
    assert!(has_gt, "Expected Gt instruction");
}
#[test]
fn test_ir_bitnot_instruction() {
    let ir = ir("def f() of int { return ~1; }");
    let has_bitnot = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::BitNot);
    assert!(has_bitnot, "Expected BitNot instruction");
}
#[test]
fn test_ir_while_loop_instructions() {
    let ir = ir("def f() of int { i = 0; while (i < 10) { i = i + 1; } return i; }");
    let has_cmp = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| matches!(i.opcode, crate::ir::IrOpcode::Lt | crate::ir::IrOpcode::Le | crate::ir::IrOpcode::Ge | crate::ir::IrOpcode::Gt));
    assert!(has_cmp, "Expected comparison instruction in while loop");
}
#[test]
fn test_ir_loop_with_break_has_jump() {
    let ir = ir("def f() of int { while (true) { break; } return 0; }");
    let has_jump = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Jump);
    assert!(has_jump, "Expected Jump for break");
}
#[test]
fn test_ir_block_ordering() {
    let ir = ir("def f() of int { return 0; }");
    assert!(!ir.functions[0].blocks[0].id.is_empty());
}
#[test]
fn test_ir_mod_instruction() {
    let ir = ir("def f() of int { return 10 % 3; }");
    let has_mod = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::Mod);
    assert!(has_mod, "Expected Mod instruction");
}
#[test]
fn test_ir_load_for_array_access() {
    let ir = ir("def f() of int { a of int[5]; return a[0]; }");
    let has_load = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| matches!(i.opcode, crate::ir::IrOpcode::Load | crate::ir::IrOpcode::Slice));
    assert!(has_load, "Expected Load or Slice instruction for array access");
}
#[test]
fn test_ir_alloc_array_instruction() {
    let ir = ir("def f() { a = [1, 2, 3]; }");
    let has_alloc = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::AllocArray);
    assert!(has_alloc, "Expected AllocArray instruction for array literal");
}
#[test]
fn test_ir_empty_array_alloc() {
    let ir = ir("def f() { a = []; }");
    let has_alloc = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .any(|i| i.opcode == crate::ir::IrOpcode::AllocArray);
    assert!(has_alloc, "Expected AllocArray instruction for empty array literal");
}
#[test]
fn test_ir_field_access_load() {
    let ir = ir("struct P { x of int; } def f(p of P) of int { return p.x; }");
    let has_load = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .filter(|i| i.opcode == crate::ir::IrOpcode::Load)
        .count();
    assert!(has_load >= 1, "Expected at least 1 Load for field access, got {has_load}");
}
#[test]
fn test_ir_field_access_store() {
    let ir = ir("struct P { x of int; } def f(p of P) { p.x = 42; }");
    let has_store = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .find(|i| i.opcode == crate::ir::IrOpcode::Store);
    assert!(has_store.is_some(), "Expected Store for field assignment");
}
#[test]
fn test_ir_return_operand() {
    let ir = ir("def f() of int { return 42; }");
    let rets: Vec<_> = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .filter(|i| i.opcode == crate::ir::IrOpcode::Ret)
        .collect();
    assert!(!rets.is_empty(), "Expected at least one Ret instruction");
    let has_var_or_const = rets.iter().any(|i| {
        i.operands.iter().any(|o| {
            matches!(o, crate::ir::IrOperand::Variable(_, _) | crate::ir::IrOperand::Constant(_))
        })
    });
    assert!(has_var_or_const, "Expected variable or constant operand in Ret");
}
#[test]
fn test_ir_return_variable_operand() {
    let ir = ir("def f() of int { x = 1; return x; }");
    let rets: Vec<_> = ir.functions[0].blocks.iter()
        .flat_map(|b| &b.instructions)
        .filter(|i| i.opcode == crate::ir::IrOpcode::Ret)
        .collect();
    assert!(!rets.is_empty());
    let has_var = rets.iter().any(|i| {
        i.operands.iter().any(|o| matches!(o, crate::ir::IrOperand::Variable(_, _)))
    });
    assert!(has_var, "Expected variable operand in Ret");
}

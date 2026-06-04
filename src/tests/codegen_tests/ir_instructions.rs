use crate::ir_generator::IrGenerator;
use crate::tests::parse;

fn ir(source: &str) -> crate::ir::IrProgram {
    let program = parse(source);
    let mut ir_gen = IrGenerator::new();
    ir_gen.generate(&program)
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

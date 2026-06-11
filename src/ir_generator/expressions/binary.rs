use super::IrGenerator;
use crate::ast::{BinaryExpr, BinaryOp, Expr};
use crate::ir::{Constant, IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_binary_expr(&mut self, block: &mut IrBlock, expr: &BinaryExpr) -> (String, IrType) {
        // Short-circuit evaluation for && and ||
        if matches!(expr.operator, BinaryOp::And | BinaryOp::Or) {
            return self.visit_short_circuit(block, expr);
        }

        let (left_temp, left_type) = self.visit_expr(block, &expr.left);
        let (right_temp, right_type) = self.visit_expr(block, &expr.right);

        if matches!(expr.operator, BinaryOp::Assign) {
            return self.visit_assign(block, expr, &left_temp, &left_type, &right_temp, &right_type);
        }

        let result_temp = self.generate_temp();
        let (opcode, result_type) = match expr.operator {
            BinaryOp::Multiply => (IrOpcode::Mul, IrType::Int),
            BinaryOp::Divide => (IrOpcode::Div, IrType::Int),
            BinaryOp::Modulo => (IrOpcode::Mod, IrType::Int),
            BinaryOp::Add => (IrOpcode::Add, IrType::Int),
            BinaryOp::Subtract => (IrOpcode::Sub, IrType::Int),
            BinaryOp::Equal => (IrOpcode::Eq, IrType::Bool),
            BinaryOp::NotEqual => (IrOpcode::Ne, IrType::Bool),
            BinaryOp::Less => (IrOpcode::Lt, IrType::Bool),
            BinaryOp::Greater => (IrOpcode::Gt, IrType::Bool),
            BinaryOp::LessOrEqual => (IrOpcode::Le, IrType::Bool),
            BinaryOp::GreaterOrEqual => (IrOpcode::Ge, IrType::Bool),
            BinaryOp::BitAnd => (IrOpcode::BitAnd, IrType::Int),
            BinaryOp::BitOr => (IrOpcode::BitOr, IrType::Int),
            BinaryOp::BitXor => (IrOpcode::BitXor, IrType::Int),
            _ => unreachable!(),
        };

        block.instructions.push(IrInstruction {
            opcode,
            result: Some(result_temp.clone()),
            result_type: Some(result_type.clone()),
            operands: vec![
                IrOperand::Variable(left_temp, left_type),
                IrOperand::Variable(right_temp, right_type),
            ],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });
        (result_temp, result_type)
    }

    /// Short-circuit evaluation for `&&` and `||`:
    /// Produces 3 blocks:
    ///   ┌─ left block (caller's current block): evaluate a, cond_br → eval_right / shortcut
    ///   ├─ eval_right: evaluate b, result = b, jump continue
    ///   ├─ shortcut:   result = shortcut_value, jump continue
    ///   └─ continue:   (caller continues here with result_temp ready)
    fn visit_short_circuit(&mut self, block: &mut IrBlock, expr: &BinaryExpr) -> (String, IrType) {
        let is_and = matches!(expr.operator, BinaryOp::And);
        let result_temp = self.generate_temp();
        let eval_right_id = self.generate_block_id();
        let shortcut_id = self.generate_block_id();
        let continue_id = self.generate_block_id();

        // Evaluate left operand
        let (left_temp, _) = self.visit_expr(block, &expr.left);

        // For && : if left is false → shortcut (result=false), else eval_right
        // For || : if left is true  → shortcut (result=true),  else eval_right
        let shortcut_val = if is_and { 0i64 } else { 1i64 };
        let (true_target, false_target) = if is_and {
            (eval_right_id.clone(), shortcut_id.clone())
        } else {
            (shortcut_id.clone(), eval_right_id.clone())
        };

        block.instructions.push(IrInstruction {
            opcode: IrOpcode::CondBr,
            result: None,
            result_type: None,
            operands: vec![IrOperand::Variable(left_temp, IrType::Bool)],
            jump_target: None,
            true_target: Some(true_target),
            false_target: Some(false_target),
            span: expr.span,
        });
        block.successors.push(eval_right_id.clone());
        block.successors.push(shortcut_id.clone());

        // Save current block (left block) to block_stack, switch to eval_right
        let old_block = std::mem::replace(block, IrBlock::new(eval_right_id));
        self.block_stack.push(old_block);

        // Evaluate right operand in eval_right block
        let (right_temp, _) = self.visit_expr(block, &expr.right);
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Assign,
            result: Some(result_temp.clone()),
            result_type: Some(IrType::Bool),
            operands: vec![IrOperand::Variable(right_temp, IrType::Bool)],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Jump,
            result: None,
            result_type: None,
            operands: vec![],
            jump_target: Some(continue_id.clone()),
            true_target: None,
            false_target: None,
            span: expr.span,
        });
        block.successors.push(continue_id.clone());
        self.block_stack
            .push(std::mem::replace(block, IrBlock::new(shortcut_id)));

        // Shortcut block: assign shortcut value, then jump to continue
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Assign,
            result: Some(result_temp.clone()),
            result_type: Some(IrType::Bool),
            operands: vec![IrOperand::Constant(Constant::Int(shortcut_val))],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Jump,
            result: None,
            result_type: None,
            operands: vec![],
            jump_target: Some(continue_id.clone()),
            true_target: None,
            false_target: None,
            span: expr.span,
        });
        block.successors.push(continue_id.clone());
        self.block_stack
            .push(std::mem::replace(block, IrBlock::new(continue_id)));

        (result_temp, IrType::Bool)
    }

    fn visit_assign(
        &mut self,
        block: &mut IrBlock,
        expr: &BinaryExpr,
        _left_temp: &str,
        _left_type: &IrType,
        right_temp: &str,
        right_type: &IrType,
    ) -> (String, IrType) {
        match expr.left.as_ref() {
            Expr::FieldAccess(inner_base, field, _) => {
                self.assign_to_field(block, expr, inner_base, field, right_temp, right_type);
            }
            Expr::Slice(slice) => {
                self.assign_to_slice(block, expr, slice, right_temp, right_type);
            }
            left if matches!(left, Expr::Identifier(id) if self.captured_vars.contains_key(&id.name)) => {
                if let Expr::Identifier(id) = left {
                    return self.assign_to_captured(block, expr, &id.name, right_temp, right_type);
                }
                unreachable!()
            }
            Expr::Identifier(id) => {
                self.assign_to_identifier(block, expr, &id.name, right_temp, right_type);
            }
            _ => {
                self.assign_to_identifier(block, expr, "", right_temp, right_type);
            }
        }
        (right_temp.to_string(), right_type.clone())
    }

    fn assign_to_field(
        &mut self,
        block: &mut IrBlock,
        expr: &BinaryExpr,
        inner_base: &Expr,
        field: &crate::ast::Identifier,
        right_temp: &str,
        right_type: &IrType,
    ) {
        if let Expr::Slice(slice) = inner_base {
            if let Some(range) = slice.ranges.first() {
                let (arr_name, _) = self.visit_expr(block, &slice.array);
                let (idx_temp, _) = self.visit_expr(block, &range.start);
                let field_offset = self.find_field_offset_for_array(&arr_name, &field.name);
                let arr_type = self
                    .symbols
                    .global_types
                    .get(&arr_name)
                    .cloned()
                    .or_else(|| self.symbols.lookup(&arr_name).map(|l| l.ty.clone()))
                    .unwrap_or(IrType::Int);
                let elem_size = match &arr_type {
                    IrType::Array(elem, _) => elem.size() as i64,
                    _ => 4i64,
                };
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Store,
                    result: None,
                    result_type: None,
                    operands: vec![
                        IrOperand::Variable(arr_name, arr_type),
                        IrOperand::Constant(Constant::Int(field_offset as i64)),
                        IrOperand::Variable(right_temp.to_string(), right_type.clone()),
                        IrOperand::Variable(idx_temp, IrType::Int),
                        IrOperand::Constant(Constant::Int(elem_size)),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
                return;
            }
        }

        let (base_name, base_offset) = self.resolve_field_chain(expr.left.as_ref());
        let (base_type, _, _field_type) = self.resolve_field_info(&base_name, inner_base, field, 0);
        let total_offset = base_offset;

        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Store,
            result: None,
            result_type: None,
            operands: vec![
                IrOperand::Variable(base_name, base_type),
                IrOperand::Constant(Constant::Int(total_offset as i64)),
                IrOperand::Variable(right_temp.to_string(), right_type.clone()),
            ],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });
    }

    fn assign_to_slice(
        &mut self,
        block: &mut IrBlock,
        expr: &BinaryExpr,
        slice: &crate::ast::SliceExpr,
        right_temp: &str,
        right_type: &IrType,
    ) {
        if let Some(range) = slice.ranges.first() {
            // s.field[i] = value  → use base struct + field offset directly
            if let crate::ast::Expr::FieldAccess(ref base_expr, ref field_ident, _) = &*slice.array {
                let (base_name, _) = self.visit_expr(block, base_expr);
                let (idx_temp, _) = self.visit_expr(block, &range.start);
                let field_offset = self.find_field_offset_for_array(&base_name, &field_ident.name);
                let base_type = self
                    .symbols
                    .global_types
                    .get(&base_name)
                    .cloned()
                    .or_else(|| self.symbols.lookup(&base_name).map(|l| l.ty.clone()))
                    .unwrap_or(IrType::Int);
                let field_type = self.find_field_type_for_var(&base_name, &field_ident.name);
                let elem_size = match &field_type {
                    IrType::Array(elem, _) => elem.size() as i64,
                    _ => 4i64,
                };
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Store,
                    result: None,
                    result_type: None,
                    operands: vec![
                        IrOperand::Variable(base_name, base_type),
                        IrOperand::Constant(Constant::Int(field_offset as i64)),
                        IrOperand::Variable(right_temp.to_string(), right_type.clone()),
                        IrOperand::Variable(idx_temp, IrType::Int),
                        IrOperand::Constant(Constant::Int(elem_size)),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
                return;
            }

            let (idx, _) = self.visit_expr(block, &range.start);
            let (base_name, base_type) = self.visit_expr(block, &slice.array);

            if base_type == IrType::String {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::StrSetByte,
                    result: None,
                    result_type: None,
                    operands: vec![
                        IrOperand::Variable(base_name, base_type),
                        IrOperand::Variable(idx, IrType::Int),
                        IrOperand::Variable(right_temp.to_string(), right_type.clone()),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
            } else {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Store,
                    result: None,
                    result_type: None,
                    operands: vec![
                        IrOperand::Variable(base_name, base_type),
                        IrOperand::Constant(Constant::Int(0)),
                        IrOperand::Variable(right_temp.to_string(), right_type.clone()),
                        IrOperand::Variable(idx, IrType::Int),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
            }
        }
    }

    fn assign_to_identifier(
        &mut self,
        block: &mut IrBlock,
        expr: &BinaryExpr,
        target_name: &str,
        right_temp: &str,
        right_type: &IrType,
    ) {
        let name = target_name.to_string();
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Assign,
            result: Some(name.clone()),
            result_type: Some(right_type.clone()),
            operands: vec![IrOperand::Variable(right_temp.to_string(), right_type.clone())],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });
        if !name.is_empty() && !self.symbols.is_declared(&name) {
            self.symbols.define_local(&name, right_type.clone());
        }
    }

    fn assign_to_captured(
        &mut self,
        block: &mut IrBlock,
        expr: &BinaryExpr,
        target_name: &str,
        right_temp: &str,
        right_type: &IrType,
    ) -> (String, IrType) {
        let slot = self.captured_vars[target_name];
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::StoreCaptured,
            result: None,
            result_type: None,
            operands: vec![
                IrOperand::Variable("__env".to_string(), IrType::Int),
                IrOperand::Constant(Constant::Int(slot as i64)),
                IrOperand::Variable(right_temp.to_string(), right_type.clone()),
            ],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });
        (right_temp.to_string(), right_type.clone())
    }
}

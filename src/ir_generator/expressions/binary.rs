use super::IrGenerator;
use crate::ast::{BinaryExpr, BinaryOp, Expr};
use crate::ir::{Constant, IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_binary_expr(&mut self, block: &mut IrBlock, expr: &BinaryExpr) -> (String, IrType) {
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
            BinaryOp::Add => (IrOpcode::Add, if left_type.is_pointer() { left_type.clone() } else { IrType::Int }),
            BinaryOp::Subtract => (IrOpcode::Sub, if left_type.is_pointer() { left_type.clone() } else { IrType::Int }),
            BinaryOp::Equal => (IrOpcode::Eq, IrType::Bool),
            BinaryOp::NotEqual => (IrOpcode::Ne, IrType::Bool),
            BinaryOp::Less => (IrOpcode::Lt, IrType::Bool),
            BinaryOp::Greater => (IrOpcode::Gt, IrType::Bool),
            BinaryOp::LessOrEqual => (IrOpcode::Le, IrType::Bool),
            BinaryOp::GreaterOrEqual => (IrOpcode::Ge, IrType::Bool),
            BinaryOp::And => (IrOpcode::And, IrType::Bool),
            BinaryOp::Or => (IrOpcode::Or, IrType::Bool),
            BinaryOp::BitAnd => (IrOpcode::BitAnd, IrType::Int),
            BinaryOp::BitOr => (IrOpcode::BitOr, IrType::Int),
            BinaryOp::BitXor => (IrOpcode::BitXor, IrType::Int),
            _ => unreachable!(),
        };

        block.instructions.push(IrInstruction {
            opcode, result: Some(result_temp.clone()),
            result_type: Some(result_type.clone()),
            operands: vec![
                IrOperand::Variable(left_temp, left_type),
                IrOperand::Variable(right_temp, right_type),
            ],
            jump_target: None, true_target: None, false_target: None,
            span: expr.span,
        });
        (result_temp, result_type)
    }

    fn visit_assign(
        &mut self, block: &mut IrBlock, expr: &BinaryExpr,
        _left_temp: &str, _left_type: &IrType,
        right_temp: &str, right_type: &IrType,
    ) -> (String, IrType) {
        match expr.left.as_ref() {
            Expr::FieldAccess(inner_base, field) => {
                self.assign_to_field(block, expr, inner_base, field, right_temp, right_type);
            }
            Expr::Slice(slice) => {
                self.assign_to_slice(block, expr, slice, right_temp, right_type);
            }
            _ => {
                let target_name = match expr.left.as_ref() {
                    Expr::Identifier(id) => id.name.clone(),
                    _ => String::new(),
                };

                if !target_name.is_empty() && self.captured_vars.contains_key(&target_name) {
                    return self.assign_to_captured(block, expr, &target_name, right_temp, right_type);
                }

                self.assign_to_identifier(block, expr, &target_name, right_temp, right_type);
            }
        }
        (right_temp.to_string(), right_type.clone())
    }

    fn assign_to_field(
        &mut self, block: &mut IrBlock, expr: &BinaryExpr,
        inner_base: &Expr, field: &crate::ast::Identifier,
        right_temp: &str, right_type: &IrType,
    ) {
        if let Expr::Slice(slice) = inner_base {
            if let Some(range) = slice.ranges.first() {
                let (arr_name, _) = self.visit_expr(block, &slice.array);
                let (idx_temp, _) = self.visit_expr(block, &range.start);
                let field_offset = self.find_field_offset_for_array(&arr_name, &field.name);
                let arr_type = self.symbols.global_types.get(&arr_name).cloned().unwrap_or(IrType::Int);
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Store, result: None, result_type: None,
                    operands: vec![
                        IrOperand::Variable(arr_name, arr_type),
                        IrOperand::Constant(Constant::Int(field_offset as i64)),
                        IrOperand::Variable(right_temp.to_string(), right_type.clone()),
                        IrOperand::Variable(idx_temp, IrType::Int),
                    ],
                    jump_target: None, true_target: None, false_target: None, span: expr.span,
                });
                return;
            }
        }
        let (base_name, total_offset) = self.resolve_field_chain(expr.left.as_ref());
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Store, result: None, result_type: None,
            operands: vec![
                IrOperand::Variable(base_name, IrType::Int),
                IrOperand::Constant(Constant::Int(total_offset as i64)),
                IrOperand::Variable(right_temp.to_string(), right_type.clone()),
            ],
            jump_target: None, true_target: None, false_target: None, span: expr.span,
        });
    }

    fn assign_to_slice(
        &mut self, block: &mut IrBlock, expr: &BinaryExpr,
        slice: &crate::ast::SliceExpr,
        right_temp: &str, right_type: &IrType,
    ) {
        if let Expr::FieldAccess(_, _) = slice.array.as_ref() {
            if let Some(range) = slice.ranges.first() {
                let (idx, _) = self.visit_expr(block, &range.start);
                let (base_name, total_offset) = self.resolve_field_chain(slice.array.as_ref());
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Store, result: None, result_type: None,
                    operands: vec![
                        IrOperand::Variable(base_name, IrType::Int),
                        IrOperand::Constant(Constant::Int(total_offset as i64)),
                        IrOperand::Variable(right_temp.to_string(), right_type.clone()),
                        IrOperand::Variable(idx, IrType::Int),
                    ],
                    jump_target: None, true_target: None, false_target: None, span: expr.span,
                });
                return;
            }
        }
        if let Some(range) = slice.ranges.first() {
            let (idx, _) = self.visit_expr(block, &range.start);
            let (base_name, base_type) = self.visit_expr(block, &slice.array);
            let opcode = if base_type == IrType::String { IrOpcode::StrSetByte } else { IrOpcode::Store };
            let operands = if matches!(base_type, IrType::String) {
                vec![
                    IrOperand::Variable(base_name, base_type.clone()),
                    IrOperand::Variable(idx, IrType::Int),
                    IrOperand::Variable(right_temp.to_string(), right_type.clone()),
                ]
            } else {
                vec![
                    IrOperand::Variable(base_name, base_type.clone()),
                    IrOperand::Constant(Constant::Int(0)),
                    IrOperand::Variable(right_temp.to_string(), right_type.clone()),
                    IrOperand::Variable(idx, IrType::Int),
                ]
            };
            block.instructions.push(IrInstruction {
                opcode, result: None, result_type: None,
                operands,
                jump_target: None, true_target: None, false_target: None, span: expr.span,
            });
        }
    }

    fn assign_to_identifier(
        &mut self, block: &mut IrBlock, expr: &BinaryExpr,
        target_name: &str, right_temp: &str, right_type: &IrType,
    ) {
        let name = if target_name.is_empty() { String::new() } else { target_name.to_string() };
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Assign, result: Some(name.clone()),
            result_type: Some(right_type.clone()),
            operands: vec![IrOperand::Variable(right_temp.to_string(), right_type.clone())],
            jump_target: None, true_target: None, false_target: None, span: expr.span,
        });
        if !name.is_empty() && !self.symbols.is_declared(&name) {
            self.symbols.define_local(&name, right_type.clone());
        }
        if self.closure_envs.contains_key(right_temp) {
            let env_tmp = self.closure_envs[right_temp].clone();
            self.closure_envs.insert(name, env_tmp);
        }
    }

    fn assign_to_captured(
        &mut self, block: &mut IrBlock, expr: &BinaryExpr,
        target_name: &str, right_temp: &str, right_type: &IrType,
    ) -> (String, IrType) {
        let slot = self.captured_vars[target_name];
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::StoreCaptured, result: None, result_type: None,
            operands: vec![
                IrOperand::Variable("__env".to_string(), IrType::Int),
                IrOperand::Constant(Constant::Int(slot as i64)),
                IrOperand::Variable(right_temp.to_string(), right_type.clone()),
            ],
            jump_target: None, true_target: None, false_target: None, span: expr.span,
        });
        (right_temp.to_string(), right_type.clone())
    }
}

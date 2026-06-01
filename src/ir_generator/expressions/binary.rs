use super::IrGenerator;
use crate::ast::{BinaryExpr, BinaryOp, Expr};
use crate::ir::{Constant, IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_binary_expr(&mut self, block: &mut IrBlock, expr: &BinaryExpr) -> (String, IrType) {
        let left_temp = self.visit_expr(block, &expr.left).0;
        let (right_temp, right_type) = self.visit_expr(block, &expr.right);

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
            BinaryOp::And => (IrOpcode::And, IrType::Bool),
            BinaryOp::Or => (IrOpcode::Or, IrType::Bool),
            BinaryOp::BitAnd => (IrOpcode::BitAnd, IrType::Int),
            BinaryOp::BitOr => (IrOpcode::BitOr, IrType::Int),
            BinaryOp::BitXor => (IrOpcode::BitXor, IrType::Int),
            BinaryOp::Assign => {
                match expr.left.as_ref() {
                    Expr::FieldAccess(inner_base, field) => {
                        if let Expr::Slice(slice) = inner_base.as_ref() {
                            if let Some(range) = slice.ranges.first() {
                                let (arr_name, _) = self.visit_expr(block, &slice.array);
                                let (idx_temp, _) = self.visit_expr(block, &range.start);
                                let field_offset = self.find_field_offset_for_array(&arr_name, &field.name);
                                let arr_type = self.symbols.global_types.get(&arr_name).cloned().unwrap_or(IrType::Int);
                                block.instructions.push(IrInstruction {
                                    opcode: IrOpcode::Store,
                                    result: None,
                                    result_type: None,
                                    operands: vec![
                                        IrOperand::Variable(arr_name, arr_type),
                                        IrOperand::Constant(Constant::Int(field_offset as i64)),
                                        IrOperand::Variable(right_temp.clone(), right_type.clone()),
                                        IrOperand::Variable(idx_temp, IrType::Int),
                                    ],
                                    jump_target: None,
                                    true_target: None,
                                    false_target: None,
                                    span: expr.span,
                                });
                                return (right_temp, right_type);
                            }
                        }
                        let (base_name, total_offset) = self.resolve_field_chain(expr.left.as_ref());
                        block.instructions.push(IrInstruction {
                            opcode: IrOpcode::Store,
                            result: None,
                            result_type: None,
                            operands: vec![
                                IrOperand::Variable(base_name, IrType::Int),
                                IrOperand::Constant(Constant::Int(total_offset as i64)),
                                IrOperand::Variable(right_temp.clone(), right_type.clone()),
                            ],
                            jump_target: None,
                            true_target: None,
                            false_target: None,
                            span: expr.span,
                        });
                        return (right_temp, right_type);
                    }
                    Expr::Slice(slice) => {
                        if let Expr::FieldAccess(_, _) = slice.array.as_ref() {
                            let (base_name, total_offset) = self.resolve_field_chain(slice.array.as_ref());
                            if let Some(range) = slice.ranges.first() {
                                let (idx, _) = self.visit_expr(block, &range.start);
                                block.instructions.push(IrInstruction {
                                    opcode: IrOpcode::Store,
                                    result: None,
                                    result_type: None,
                                    operands: vec![
                                        IrOperand::Variable(base_name, IrType::Int),
                                        IrOperand::Constant(Constant::Int(total_offset as i64)),
                                        IrOperand::Variable(right_temp.clone(), right_type.clone()),
                                        IrOperand::Variable(idx, IrType::Int),
                                    ],
                                    jump_target: None,
                                    true_target: None,
                                    false_target: None,
                                    span: expr.span,
                                });
                                return (right_temp, right_type);
                            }
                        }
                        if let Some(range) = slice.ranges.first() {
                            let (idx, _) = self.visit_expr(block, &range.start);
                            let (base_name, base_type) = self.visit_expr(block, &slice.array);
                            if base_type == IrType::String {
                                block.instructions.push(IrInstruction {
                                    opcode: IrOpcode::StrSetByte,
                                    result: None,
                                    result_type: None,
                                    operands: vec![
                                        IrOperand::Variable(base_name, IrType::String),
                                        IrOperand::Variable(idx, IrType::Int),
                                        IrOperand::Variable(right_temp.clone(), right_type.clone()),
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
                                        IrOperand::Variable(right_temp.clone(), right_type.clone()),
                                        IrOperand::Variable(idx, IrType::Int),
                                    ],
                                    jump_target: None,
                                    true_target: None,
                                    false_target: None,
                                    span: expr.span,
                                });
                            }
                            return (right_temp, right_type);
                        }
                        let target_name = left_temp.clone();
                        block.instructions.push(IrInstruction {
                            opcode: IrOpcode::Assign,
                            result: Some(target_name.clone()),
                            result_type: Some(right_type.clone()),
                            operands: vec![IrOperand::Variable(right_temp.clone(), right_type.clone())],
                            jump_target: None,
                            true_target: None,
                            false_target: None,
                            span: expr.span,
                        });
                        return (right_temp, right_type);
                    }
                    _ => {
                        let target_name = match expr.left.as_ref() {
                            Expr::Identifier(id) => id.name.clone(),
                            _ => left_temp.clone(),
                        };

                        if let Some(slot) = self.captured_vars.get(&target_name) {
                            block.instructions.push(IrInstruction {
                                opcode: IrOpcode::StoreCaptured,
                                result: None,
                                result_type: None,
                                operands: vec![
                                    IrOperand::Variable("__env".to_string(), IrType::Int),
                                    IrOperand::Constant(Constant::Int(*slot as i64)),
                                    IrOperand::Variable(right_temp.clone(), right_type.clone()),
                                ],
                                jump_target: None,
                                true_target: None,
                                false_target: None,
                                span: expr.span,
                            });
                            return (right_temp, right_type);
                        }

                        block.instructions.push(IrInstruction {
                            opcode: IrOpcode::Assign,
                            result: Some(target_name.clone()),
                            result_type: Some(right_type.clone()),
                            operands: vec![IrOperand::Variable(right_temp.clone(), right_type.clone())],
                            jump_target: None,
                            true_target: None,
                            false_target: None,
                            span: expr.span,
                        });

                        if !self.symbols.is_declared(&target_name) {
                            self.symbols.define_local(&target_name, right_type.clone());
                        }
                        if self.closure_envs.contains_key(&right_temp) {
                            let env_tmp = self.closure_envs[&right_temp].clone();
                            self.closure_envs.insert(target_name.clone(), env_tmp);
                        }
                        return (right_temp, right_type);
                    }
                }
            }
        };

        block.instructions.push(IrInstruction {
            opcode,
            result: Some(result_temp.clone()),
            result_type: Some(result_type.clone()),
            operands: vec![
                IrOperand::Variable(left_temp, IrType::Int),
                IrOperand::Variable(right_temp, IrType::Int),
            ],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });

        (result_temp, result_type)
    }
}

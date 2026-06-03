use super::IrGenerator;
use crate::ast::SliceExpr;
use crate::ir::{Constant, IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_slice_expr(&mut self, block: &mut IrBlock, expr: &SliceExpr) -> (String, IrType) {
        if let crate::ast::Expr::FieldAccess(_, ref field_ident) = expr.array.as_ref() {
            let (base_name, total_offset) = self.resolve_field_chain(expr.array.as_ref());
            if let Some(range) = expr.ranges.first() {
                let (index_temp, _) = self.visit_expr(block, &range.start);
                let field_type = self.find_field_type_for_var(&base_name, &field_ident.name);
                let element_type = match &field_type {
                    IrType::Array(elem, _) => *elem.clone(),
                    _ => IrType::Int,
                };
                let result_temp = self.generate_temp();
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Load,
                    result: Some(result_temp.clone()),
                    result_type: Some(element_type.clone()),
                    operands: vec![
                        IrOperand::Variable(base_name, IrType::Int),
                        IrOperand::Constant(Constant::Int(total_offset as i64)),
                        IrOperand::Variable(index_temp, IrType::Int),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
                return (result_temp, element_type);
            }
        }

        let (array_temp, array_type) = self.visit_expr(block, &expr.array);

        let element_type = match &array_type {
            IrType::Array(ref elem, _) => *elem.clone(),
            IrType::String => IrType::Int,
            _ => IrType::Int,
        };

        if let Some(range) = expr.ranges.first() {
            let (start_temp, _) = self.visit_expr(block, &range.start);

            let result_temp = self.generate_temp();

            if let Some(ref end_expr) = range.end {
                let (end_temp, _) = self.visit_expr(block, end_expr);
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Slice,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Array(Box::new(element_type.clone()), 0)),
                    operands: vec![
                        IrOperand::Variable(array_temp, array_type.clone()),
                        IrOperand::Variable(start_temp, IrType::Int),
                        IrOperand::Variable(end_temp, IrType::Int),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
            } else if array_type == IrType::String {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::StrGetByte,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![
                        IrOperand::Variable(array_temp, IrType::String),
                        IrOperand::Variable(start_temp, IrType::Int),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
            } else {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Load,
                    result: Some(result_temp.clone()),
                    result_type: Some(element_type.clone()),
                    operands: vec![
                        IrOperand::Variable(array_temp, array_type.clone()),
                        IrOperand::Variable(start_temp, IrType::Int),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
            }

            return (result_temp, element_type);
        }

        (String::new(), IrType::Int)
    }
}

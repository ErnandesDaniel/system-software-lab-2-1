use super::IrGenerator;
use crate::ast::{Literal, Span};
use crate::ir::{Constant, IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_literal_expr(&mut self, block: &mut IrBlock, lit: &Literal, span: Span) -> (String, IrType) {
        let result_temp = self.generate_temp();

        match lit {
            Literal::Bool(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Bool),
                    operands: vec![IrOperand::Constant(Constant::Bool(*v))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span,
                });
                (result_temp, IrType::Bool)
            }
            Literal::Dec(v) | Literal::Hex(v) | Literal::Bits(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Int(*v as i64))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span,
                });
                (result_temp, IrType::Int)
            }
            Literal::Char(c) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Char(*c as u8))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span,
                });
                (result_temp, IrType::Int)
            }
            Literal::Str(s) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::String),
                    operands: vec![IrOperand::Constant(Constant::String(s.clone()))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span,
                });
                (result_temp, IrType::String)
            }
        }
    }
}

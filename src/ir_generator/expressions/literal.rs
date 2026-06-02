use super::IrGenerator;
use crate::ast::{Literal, Span};
use crate::ir::{Constant, IrBlock, IrInstruction, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_literal_expr(&mut self, block: &mut IrBlock, lit: &Literal, span: Span) -> (String, IrType) {
        let result_temp = self.generate_temp();
        match lit {
            Literal::Bool(v) => {
                let val = IrOperand::Constant(Constant::Bool(*v));
                block.instructions.push(IrInstruction::assign(result_temp.clone(), IrType::Bool, val, span));
                (result_temp, IrType::Bool)
            }
            Literal::Dec(v) | Literal::Hex(v) | Literal::Bits(v) => {
                let val = IrOperand::Constant(Constant::Int(*v as i64));
                block.instructions.push(IrInstruction::assign(result_temp.clone(), IrType::Int, val, span));
                (result_temp, IrType::Int)
            }
            Literal::Char(c) => {
                let val = IrOperand::Constant(Constant::Char(*c as u8));
                block.instructions.push(IrInstruction::assign(result_temp.clone(), IrType::Int, val, span));
                (result_temp, IrType::Int)
            }
            Literal::Str(s) => {
                let val = IrOperand::Constant(Constant::String(s.clone()));
                block.instructions.push(IrInstruction::assign(result_temp.clone(), IrType::String, val, span));
                (result_temp, IrType::String)
            }
        }
    }
}

use super::IrGenerator;
use crate::ast::{Literal, Span};
use crate::ir::{Constant, IrBlock, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_literal_expr(&mut self, block: &mut IrBlock, lit: &Literal, span: Span) -> (String, IrType) {
        let result_temp = self.generate_temp();
        match lit {
            Literal::Bool(v) => {
                block.inst(IrOpcode::Assign).with(Some(result_temp.clone()), Some(IrType::Bool),
                    vec![IrOperand::Constant(Constant::Bool(*v))], span);
                (result_temp, IrType::Bool)
            }
            Literal::Dec(v) | Literal::Hex(v) | Literal::Bits(v) => {
                block.inst(IrOpcode::Assign).with(Some(result_temp.clone()), Some(IrType::Int),
                    vec![IrOperand::Constant(Constant::Int(*v as i64))], span);
                (result_temp, IrType::Int)
            }
            Literal::Char(c) => {
                block.inst(IrOpcode::Assign).with(Some(result_temp.clone()), Some(IrType::Int),
                    vec![IrOperand::Constant(Constant::Char(*c as u8))], span);
                (result_temp, IrType::Int)
            }
            Literal::Str(s) => {
                block.inst(IrOpcode::Assign).with(Some(result_temp.clone()), Some(IrType::String),
                    vec![IrOperand::Constant(Constant::String(s.clone()))], span);
                (result_temp, IrType::String)
            }
        }
    }
}

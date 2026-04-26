use ristretto_classfile::attributes::Instruction;
use crate::ir::types::*;
use crate::codegen::jvm::JvmGenerator;
use crate::codegen::jvm::types::ComparisonOp;

impl JvmGenerator {
    pub fn generate_logical_and(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            let false_label = 0u16;
            let end_label = 0u16;
            code.push(Instruction::Ifeq(false_label));

            self.emit_load_operand(code, right);
            code.push(Instruction::Ifeq(false_label));

            code.push(Instruction::Iconst_1);
            code.push(Instruction::Goto(end_label));

            code.push(Instruction::Iconst_0);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    pub fn generate_logical_or(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            let true_label = 0u16;
            let end_label = 0u16;
            code.push(Instruction::Ifne(true_label));

            self.emit_load_operand(code, right);
            code.push(Instruction::Ifne(true_label));

            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(end_label));

            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    pub fn generate_logical_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            let true_label = 0u16;
            let end_label = 0u16;
            code.push(Instruction::Ifeq(true_label));
            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(end_label));
            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    pub fn generate_comparison(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, op: ComparisonOp) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            self.emit_load_operand(code, right);

            let true_label = 0u16;

            let branch_instr = match op {
                ComparisonOp::Eq => Instruction::If_icmpeq(true_label),
                ComparisonOp::Ne => Instruction::If_icmpne(true_label),
                ComparisonOp::Lt => Instruction::If_icmplt(true_label),
                ComparisonOp::Le => Instruction::If_icmple(true_label),
                ComparisonOp::Gt => Instruction::If_icmpgt(true_label),
                ComparisonOp::Ge => Instruction::If_icmpge(true_label),
            };
            code.push(branch_instr);

            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(0));

            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }
}

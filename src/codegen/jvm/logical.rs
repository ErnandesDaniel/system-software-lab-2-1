use crate::codegen::jvm::types::ComparisonOp;
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::IrInstruction;
use ristretto_classfile::attributes::Instruction;

impl JvmGenerator {
    fn emit_if_icmpeq(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmpeq(target_idx));
    }

    fn emit_if_icmpne(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmpne(target_idx));
    }

    fn emit_if_icmplt(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmplt(target_idx));
    }

    fn emit_if_icmple(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmple(target_idx));
    }

    fn emit_if_icmpgt(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmpgt(target_idx));
    }

    fn emit_if_icmpge(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmpge(target_idx));
    }

    fn emit_goto(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::Goto(target_idx));
    }

    pub fn generate_logical_and(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, global_offset: u16) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            self.emit_load_operand(code, left);
            let first_ifeq = code.len();
            code.push(Instruction::Ifeq(0)); // placeholder

            self.emit_load_operand(code, right);
            let second_ifeq = code.len();
            code.push(Instruction::Ifeq(0)); // placeholder

            code.push(Instruction::Iconst_1);
            code.push(Instruction::Goto(0)); // placeholder

            let iconst_0_idx = u16::try_from(code.len()).unwrap() + global_offset;
            code.push(Instruction::Iconst_0);
            let istore_idx = u16::try_from(code.len()).unwrap() + global_offset;
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));

            code[first_ifeq] = Instruction::Ifeq(iconst_0_idx);
            code[second_ifeq] = Instruction::Ifeq(iconst_0_idx);
            code[second_ifeq + 2] = Instruction::Goto(istore_idx);
        }
    }

    pub fn generate_logical_or(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, global_offset: u16) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            self.emit_load_operand(code, left);
            let first_ifne = code.len();
            code.push(Instruction::Ifne(0)); // placeholder

            self.emit_load_operand(code, right);
            let second_ifne = code.len();
            code.push(Instruction::Ifne(0)); // placeholder

            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(0)); // placeholder

            let iconst_1_idx = u16::try_from(code.len()).unwrap() + global_offset;
            code.push(Instruction::Iconst_1);
            let istore_idx = u16::try_from(code.len()).unwrap() + global_offset;
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));

            code[first_ifne] = Instruction::Ifne(iconst_1_idx);
            code[second_ifne] = Instruction::Ifne(iconst_1_idx);
            code[second_ifne + 2] = Instruction::Goto(istore_idx);
        }
    }

    pub fn generate_logical_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, global_offset: u16) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            let start_idx = code.len();
            code.push(Instruction::Ifeq(0)); // placeholder
            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(0)); // placeholder

            let iconst_1_idx = u16::try_from(code.len()).unwrap() + global_offset;
            code.push(Instruction::Iconst_1);
            let istore_idx = u16::try_from(code.len()).unwrap() + global_offset;
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));

            code[start_idx] = Instruction::Ifeq(iconst_1_idx);
            code[start_idx + 2] = Instruction::Goto(istore_idx);
        }
    }

    pub fn generate_comparison(
        &self,
        code: &mut Vec<Instruction>,
        inst: &IrInstruction,
        op: ComparisonOp,
        global_offset: u16,
    ) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            self.emit_load_operand(code, left);
            self.emit_load_operand(code, right);

            let start_idx = code.len();
            let iconst_1_idx = u16::try_from(start_idx + 3).unwrap() + global_offset;
            let istore_idx = u16::try_from(start_idx + 4).unwrap() + global_offset;

            match op {
                ComparisonOp::Eq => self.emit_if_icmpeq(code, iconst_1_idx),
                ComparisonOp::Ne => self.emit_if_icmpne(code, iconst_1_idx),
                ComparisonOp::Lt => self.emit_if_icmplt(code, iconst_1_idx),
                ComparisonOp::Le => self.emit_if_icmple(code, iconst_1_idx),
                ComparisonOp::Gt => self.emit_if_icmpgt(code, iconst_1_idx),
                ComparisonOp::Ge => self.emit_if_icmpge(code, iconst_1_idx),
            }

            code.push(Instruction::Iconst_0);
            self.emit_goto(code, istore_idx);

            code.push(Instruction::Iconst_1);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }
}

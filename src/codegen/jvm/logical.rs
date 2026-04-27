use ristretto_classfile::attributes::Instruction;
use crate::ir::types::*;
use crate::codegen::jvm::JvmGenerator;
use crate::codegen::jvm::types::ComparisonOp;

impl JvmGenerator {
    pub fn generate_logical_and(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            // If left is 0 (false), jump to push 0 (position 4)
            code.push(Instruction::Ifeq(4));

            self.emit_load_operand(code, right);
            // If right is 0 (false), jump to push 0 (position 4)
            code.push(Instruction::Ifeq(4));

            code.push(Instruction::Iconst_1);
            // Jump to store (position 5)
            code.push(Instruction::Goto(5));

            code.push(Instruction::Iconst_0);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    pub fn generate_logical_or(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            // If left is non-zero (true), jump to push 1 (position 4)
            code.push(Instruction::Ifne(4));

            self.emit_load_operand(code, right);
            // If right is non-zero (true), jump to push 1 (position 4)
            code.push(Instruction::Ifne(4));

            code.push(Instruction::Iconst_0);
            // Jump to store (position 5)
            code.push(Instruction::Goto(5));

            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    pub fn generate_logical_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            // If operand is 0 (false), jump to push 1 (position 3)
            code.push(Instruction::Ifeq(3));
            code.push(Instruction::Iconst_0);
            // Jump to store (position 4)
            code.push(Instruction::Goto(4));
            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    pub fn generate_comparison(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, op: ComparisonOp) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            self.emit_load_operand(code, right);

            // Branch structure for comparison (ABSOLUTE instruction positions):
            // Current position: 0
            // 0: If_icmpXX(3)  - if true, jump to position 3 (Iconst_1)
            // 1: Iconst_0      - else push 0
            // 2: Goto(4)       - jump to position 4 (Istore)
            // 3: Iconst_1      - push 1
            // 4: Istore(slot)  - store result
            
            // We need to calculate absolute positions
            let if_icmp_pos = 0u16;
            let iconst_0_pos = 1u16;
            let goto_pos = 2u16;
            let iconst_1_pos = 3u16;
            let istore_pos = 4u16;
            
            let branch_instr = match op {
                ComparisonOp::Eq => Instruction::If_icmpeq(iconst_1_pos),
                ComparisonOp::Ne => Instruction::If_icmpne(iconst_1_pos),
                ComparisonOp::Lt => Instruction::If_icmplt(iconst_1_pos),
                ComparisonOp::Le => Instruction::If_icmple(iconst_1_pos),
                ComparisonOp::Gt => Instruction::If_icmpgt(iconst_1_pos),
                ComparisonOp::Ge => Instruction::If_icmpge(iconst_1_pos),
            };
            code.push(branch_instr);

            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(istore_pos));

            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }
}

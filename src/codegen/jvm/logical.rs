use ristretto_classfile::attributes::Instruction;
use crate::ir::types::*;
use crate::codegen::jvm::JvmGenerator;
use crate::codegen::jvm::types::ComparisonOp;

impl JvmGenerator {
    pub fn generate_logical_and(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            // If left is 0 (false), jump to push 0 (instruction 5)
            code.push(Instruction::Ifeq(5));

            self.emit_load_operand(code, right);
            // If right is 0 (false), jump to push 0 (instruction 4)
            code.push(Instruction::Ifeq(4));

            code.push(Instruction::Iconst_1);
            // Jump to store (instruction 5)
            code.push(Instruction::Goto(5));

            code.push(Instruction::Iconst_0);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    pub fn generate_logical_or(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            // If left is non-zero (true), jump to push 1 (instruction 5)
            code.push(Instruction::Ifne(5));

            self.emit_load_operand(code, right);
            // If right is non-zero (true), jump to push 1 (instruction 4)
            code.push(Instruction::Ifne(4));

            code.push(Instruction::Iconst_0);
            // Jump to store (instruction 5)
            code.push(Instruction::Goto(5));

            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    pub fn generate_logical_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            // If operand is 0 (false), jump to push 1 (instruction 3)
            code.push(Instruction::Ifeq(3));
            code.push(Instruction::Iconst_0);
            // Jump to store (instruction 4)
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

            // Branch structure for comparison (relative instruction indices):
            // 0: If_icmpXX(3)  - if true, jump to instruction 3 (Iconst_1)
            // 1: Iconst_0      - else push 0
            // 2: Goto(4)       - jump to instruction 4 (Istore)
            // 3: Iconst_1      - push 1
            // 4: Istore(slot)  - store result
            
            let branch_instr = match op {
                ComparisonOp::Eq => Instruction::If_icmpeq(3),   // jump to Iconst_1
                ComparisonOp::Ne => Instruction::If_icmpne(3),   // jump to Iconst_1
                ComparisonOp::Lt => Instruction::If_icmplt(3),   // jump to Iconst_1
                ComparisonOp::Le => Instruction::If_icmple(3),   // jump to Iconst_1
                ComparisonOp::Gt => Instruction::If_icmpgt(3),   // jump to Iconst_1
                ComparisonOp::Ge => Instruction::If_icmpge(3),   // jump to Iconst_1
            };
            code.push(branch_instr);

            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(4));  // jump to Istore

            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }
}

use ristretto_classfile::attributes::Instruction;
use crate::ir::types::*;
use crate::codegen::jvm::JvmGenerator;
use crate::codegen::jvm::types::BinaryOp;

impl JvmGenerator {
    pub fn generate_instruction(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        match inst.opcode {
            IrOpcode::Assign => self.generate_assign(code, inst),
            IrOpcode::Add => self.generate_binary_op(code, inst, BinaryOp::Add),
            IrOpcode::Sub => self.generate_binary_op(code, inst, BinaryOp::Sub),
            IrOpcode::Mul => self.generate_binary_op(code, inst, BinaryOp::Mul),
            IrOpcode::Div => self.generate_binary_op(code, inst, BinaryOp::Div),
            IrOpcode::Mod => self.generate_binary_op(code, inst, BinaryOp::Mod),
            IrOpcode::Neg => self.generate_neg(code, inst),
            IrOpcode::Pos => self.generate_pos(code, inst),
            IrOpcode::And => self.generate_logical_and(code, inst),
            IrOpcode::Or => self.generate_logical_or(code, inst),
            IrOpcode::Not => self.generate_logical_not(code, inst),
            IrOpcode::BitAnd => self.generate_binary_op(code, inst, BinaryOp::BitAnd),
            IrOpcode::BitOr => self.generate_binary_op(code, inst, BinaryOp::BitOr),
            IrOpcode::BitNot => self.generate_bit_not(code, inst),
            IrOpcode::Eq => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Eq),
            IrOpcode::Ne => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Ne),
            IrOpcode::Lt => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Lt),
            IrOpcode::Le => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Le),
            IrOpcode::Gt => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Gt),
            IrOpcode::Ge => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Ge),
            IrOpcode::Call => self.generate_call(code, inst),
            IrOpcode::Ret => self.generate_return(code, inst),
            IrOpcode::Jump => self.generate_jump(code, inst),
            IrOpcode::CondBr => self.generate_conditional_branch(code, inst),
            IrOpcode::Load => self.generate_array_load(code, inst),
            IrOpcode::Slice => {}
            IrOpcode::Alloca => {}
            IrOpcode::Store => {}
            IrOpcode::Cast => {}
        }
    }

    fn generate_assign(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(ref operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            let slot = self.get_local_slot(result);
            
            match operand.get_type() {
                IrType::String => code.push(Instruction::Astore(slot as u8)),
                _ => code.push(Instruction::Istore(slot as u8)),
            }
        }
    }

    fn generate_binary_op(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, op: BinaryOp) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            self.emit_load_operand(code, right);

            let instr = match op {
                BinaryOp::Add => Instruction::Iadd,
                BinaryOp::Sub => Instruction::Isub,
                BinaryOp::Mul => Instruction::Imul,
                BinaryOp::Div => Instruction::Idiv,
                BinaryOp::Mod => Instruction::Irem,
                BinaryOp::BitAnd => Instruction::Iand,
                BinaryOp::BitOr => Instruction::Ior,
            };
            code.push(instr);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_neg(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Ineg);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_pos(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_bit_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Iconst_m1);
            code.push(Instruction::Ixor);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_call(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref target) = inst.jump_target {
            for operand in &inst.operands {
                self.emit_load_operand(code, operand);
            }

            let method_idx = self.method_refs.get(target).copied().unwrap_or(1);
            code.push(Instruction::Invokestatic(method_idx));

            if let Some(ref result) = inst.result {
                let slot = self.get_local_slot(result);
                code.push(Instruction::Istore(slot as u8));
            }
        }
    }

    fn generate_return(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Ireturn);
        } else {
            code.push(Instruction::Return);
        }
    }

    fn generate_jump(&self, _code: &mut Vec<Instruction>, _inst: &IrInstruction) {
        // Jump is handled entirely in generate_instruction_with_placeholders
        // to properly manage branch targets and avoid duplicate instructions
    }

    fn generate_conditional_branch(&self, _code: &mut Vec<Instruction>, _inst: &IrInstruction) {
        // CondBr is handled entirely in generate_instruction_with_placeholders
        // to properly manage branch targets and avoid duplicate instructions
    }

    fn generate_array_load(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(array), Some(index)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, array);
            self.emit_load_operand(code, index);
            code.push(Instruction::Iaload);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }
}

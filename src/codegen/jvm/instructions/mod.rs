mod closure;
mod control;
mod memory;

use crate::codegen::jvm::types::BinaryOp;
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{IrInstruction, IrOpcode, IrType};
use ristretto_classfile::attributes::Instruction;

impl JvmGenerator {
    pub fn generate_instruction(&mut self, code: &mut Vec<Instruction>, inst: &IrInstruction, global_offset: u16) {
        match inst.opcode {
            IrOpcode::Assign => self.generate_assign(code, inst),
            IrOpcode::Add => self.generate_binary_op(code, inst, BinaryOp::Add),
            IrOpcode::Sub => self.generate_binary_op(code, inst, BinaryOp::Sub),
            IrOpcode::Mul => self.generate_binary_op(code, inst, BinaryOp::Mul),
            IrOpcode::Div => self.generate_binary_op(code, inst, BinaryOp::Div),
            IrOpcode::Mod => self.generate_binary_op(code, inst, BinaryOp::Mod),
            IrOpcode::Neg => self.generate_neg(code, inst),
            IrOpcode::Pos => self.generate_pos(code, inst),
            IrOpcode::And => self.generate_logical_and(code, inst, global_offset),
            IrOpcode::Or => self.generate_logical_or(code, inst, global_offset),
            IrOpcode::Not => self.generate_logical_not(code, inst, global_offset),
            IrOpcode::BitAnd => self.generate_binary_op(code, inst, BinaryOp::BitAnd),
            IrOpcode::BitOr => self.generate_binary_op(code, inst, BinaryOp::BitOr),
            IrOpcode::BitXor => self.generate_binary_op(code, inst, BinaryOp::BitXor),
            IrOpcode::BitNot => self.generate_bit_not(code, inst),
            IrOpcode::Eq => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Eq, global_offset),
            IrOpcode::Ne => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Ne, global_offset),
            IrOpcode::Lt => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Lt, global_offset),
            IrOpcode::Le => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Le, global_offset),
            IrOpcode::Gt => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Gt, global_offset),
            IrOpcode::Ge => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Ge, global_offset),
            IrOpcode::Call => self.generate_call(code, inst),
            IrOpcode::Ret => self.generate_return(code, inst),
            IrOpcode::Jump => self.generate_jump(code, inst),
            IrOpcode::CondBr => self.generate_conditional_branch(code, inst),
            IrOpcode::Load => self.generate_array_load(code, inst),
            IrOpcode::Slice => self.generate_slice(code, inst),
            IrOpcode::Alloca => {}
            IrOpcode::Store => self.generate_store(code, inst),
            IrOpcode::Cast => {}
            IrOpcode::CoroYield => self.generate_coro_yield(code, inst),
            IrOpcode::CallIndirect => self.generate_call_indirect(code, inst),
            IrOpcode::MakeClosure => self.generate_make_closure(code, inst),
            IrOpcode::CallClosure => self.generate_call_closure(code, inst),
            IrOpcode::LoadCaptured => self.generate_load_captured(code, inst),
            IrOpcode::StoreCaptured => self.generate_store_captured(code, inst),
            IrOpcode::StrGetByte => self.generate_str_get_byte(code, inst),
            IrOpcode::StrSetByte => self.generate_str_set_byte(code, inst),
            IrOpcode::AllocArray => self.generate_alloc_array(code, inst),
        }
    }

    fn generate_assign(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            if self.wrapped_vars.contains(result) {
                let slot = self.get_local_slot(result);
                match slot {
                    0 => code.push(Instruction::Aload_0),
                    1 => code.push(Instruction::Aload_1),
                    2 => code.push(Instruction::Aload_2),
                    3 => code.push(Instruction::Aload_3),
                    _ => code.push(Instruction::Aload(slot as u8)),
                }
                code.push(Instruction::Iconst_0);
                self.emit_load_operand(code, operand);
                code.push(Instruction::Iastore);
            } else {
                self.emit_load_operand(code, operand);
                self.emit_store_result(code, result, &operand.get_type());
            }
        }
    }

    fn generate_binary_op(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, op: BinaryOp) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
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
                BinaryOp::BitXor => Instruction::Ixor,
            };
            code.push(instr);

            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    fn generate_neg(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Ineg);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    fn generate_pos(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    fn generate_bit_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Iconst_m1);
            code.push(Instruction::Ixor);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }
}

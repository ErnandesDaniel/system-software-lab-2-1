use crate::ir::types::{IrInstruction, IrOpcode};
use crate::ir::types::IrOperand;

use crate::codegen::nasm::AsmGenerator;

impl AsmGenerator {
    pub fn generate_instruction(&mut self, inst: &IrInstruction) {
        match inst.opcode {
            IrOpcode::Assign => self.generate_assign(inst),
            IrOpcode::Add => self.binary_op(inst, "add"),
            IrOpcode::Sub => self.binary_op(inst, "sub"),
            IrOpcode::Mul => self.binary_op(inst, "imul"),
            IrOpcode::Div => self.binary_op_div(inst),
            IrOpcode::Mod => self.binary_op_mod(inst),
            IrOpcode::Eq => self.compare_op(inst, "sete"),
            IrOpcode::Ne => self.compare_op(inst, "setne"),
            IrOpcode::Lt => self.compare_op(inst, "setl"),
            IrOpcode::Le => self.compare_op(inst, "setle"),
            IrOpcode::Gt => self.compare_op(inst, "setg"),
            IrOpcode::Ge => self.compare_op(inst, "setge"),
            IrOpcode::And => self.binary_op(inst, "and"),
            IrOpcode::Or => self.binary_op(inst, "or"),
            IrOpcode::Not => self.generate_not(inst),
            IrOpcode::Neg => self.generate_neg(inst),
            IrOpcode::Jump => self.generate_jump(inst),
            IrOpcode::Call => self.generate_call(inst),
            IrOpcode::Ret => self.generate_ret(inst),
            IrOpcode::CondBr => self.generate_cond_br(inst),
            IrOpcode::Load => self.generate_load(inst),
            IrOpcode::Slice => self.generate_slice(inst),
            IrOpcode::Alloca => {}
            IrOpcode::BitNot => self.generate_bitnot(inst),
            IrOpcode::BitAnd => self.binary_op(inst, "and"),
            IrOpcode::BitOr => self.binary_op(inst, "or"),
            IrOpcode::BitXor => self.binary_op(inst, "xor"),
            IrOpcode::Pos => self.generate_pos(inst),
            IrOpcode::StrGetByte => self.generate_str_get_byte(inst),
            IrOpcode::StrSetByte => self.generate_str_set_byte(inst),
            IrOpcode::Store => self.generate_store(inst),
            IrOpcode::Cast => {}
            IrOpcode::CoroYield => self.generate_yield(inst),
            IrOpcode::CallIndirect => self.generate_call_indirect(inst),
            IrOpcode::MakeClosure => self.generate_make_closure(inst),
            IrOpcode::CallClosure => self.generate_call_closure(inst),
            IrOpcode::LoadCaptured => self.generate_load_captured(inst),
            IrOpcode::StoreCaptured => self.generate_store_captured(inst),
            IrOpcode::AllocArray => {}
        }
    }

    pub fn generate_assign(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                let operand_ty = operand.get_type();
                let ty = inst.result_type.as_ref().unwrap_or(&operand_ty);
                let size = ty.size();
                if size > 8 {
                    self.copy_aggregate(operand, result, size);
                } else {
                    let is_pointer = operand.get_type().is_pointer();
                    let reg = if is_pointer || size == 8 { "rax" } else { "eax" };
                    self.load_operand(operand, reg, is_pointer || size == 8);
                    self.store_variable(result, reg, is_pointer || size == 8);
                }
            }
        }
    }

    fn copy_aggregate(&mut self, operand: &IrOperand, result: &str, size: u32) {
        let src_offset = match operand {
            IrOperand::Variable(name, _) => {
                self.locals.get(name).copied()
                    .or_else(|| self.temps.get(name).copied())
            }
            _ => None,
        };
        let dst_offset = self.locals.get(result).copied()
            .or_else(|| self.temps.get(result).copied());
        if let (Some(src_off), Some(dst_off)) = (src_offset, dst_offset) {
            let qwords = size / 8;
            if self.is_coroutine {
                if let IrOperand::Variable(name, _) = operand {
                    if self.locals.get(name).is_some() && self.temps.get(name).is_none() {
                        let co_off = 56 + (-src_off - 8);
                        self.restore_coro_ctx();
                        self.output.push_str(&format!("    lea rsi, [rcx + {co_off}]\n"));
                        self.output.push_str(&format!("    lea rdi, [rbp + {dst_off}]\n"));
                        self.output.push_str(&format!("    mov rcx, {qwords}\n"));
                        self.output.push_str("    rep movsq\n");
                        return;
                    }
                }
            }
            self.output.push_str(&format!("    lea rsi, [rbp + {src_off}]\n"));
            self.output.push_str(&format!("    lea rdi, [rbp + {dst_off}]\n"));
            self.output.push_str(&format!("    mov rcx, {qwords}\n"));
            self.output.push_str("    rep movsq\n");
        }
    }

    pub fn binary_op(&mut self, inst: &IrInstruction, op: &str) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            let is_ptr = left.get_type().is_pointer();
            let (lreg, rreg, sreg) = if is_ptr { ("rax", "rbx", "rax") } else { ("eax", "ebx", "eax") };
            self.load_operand(left, lreg, is_ptr);
            self.load_operand(right, rreg, is_ptr);
            self.output.push_str(&format!("    {op} {lreg}, {rreg}\n"));
            self.store_variable(result, sreg, is_ptr);
        }
    }

    pub fn binary_op_div(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cdq\n");
            self.output.push_str("    idiv ebx\n");
            self.store_variable(result, "eax", false);
        }
    }

    pub fn binary_op_mod(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cdq\n");
            self.output.push_str("    idiv ebx\n");
            self.store_variable(result, "edx", false);
        }
    }

    pub fn compare_op(&mut self, inst: &IrInstruction, set_op: &str) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cmp eax, ebx\n");
            self.output.push_str(&format!("    {set_op} al\n"));
            self.output.push_str("    movzx eax, al\n");
            self.store_variable(result, "eax", false);
        }
    }

    pub fn generate_not(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.output.push_str("    test eax, eax\n");
                self.output.push_str("    setz al\n");
                self.output.push_str("    movzx eax, al\n");
                self.store_variable(result, "eax", false);
            }
        }
    }

    pub fn generate_neg(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.output.push_str("    neg eax\n");
                self.store_variable(result, "eax", false);
            }
        }
    }

    pub fn generate_bitnot(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.output.push_str("    not eax\n");
                self.store_variable(result, "eax", false);
            }
        }
    }

    pub fn generate_pos(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.store_variable(result, "eax", false);
            }
        }
    }
}

use crate::codegen::nasm::{AsmGenerator, REGS_64};
use crate::ir::types::{IrInstruction, IrOpcode, IrOperand};

fn reg_low_byte(reg32: &str) -> &'static str {
    match reg32 {
        "eax" => "al", "ecx" => "cl", "edx" => "dl", "ebx" => "bl",
        "esi" => "sil", "edi" => "dil",
        "r8d" => "r8b", "r9d" => "r9b", "r10d" => "r10b", "r11d" => "r11b",
        "r12d" => "r12b", "r13d" => "r13b", "r14d" => "r14b", "r15d" => "r15b",
        _ => "al", // fallback (never reached with our register pool)
    }
}

impl AsmGenerator {
    pub fn generate_instruction(&mut self, inst: &IrInstruction) {
        match inst.opcode {
            IrOpcode::Assign => self.emit_assign(inst),
            IrOpcode::Add => self.emit_binary(inst, "add"),
            IrOpcode::Sub => self.emit_binary(inst, "sub"),
            IrOpcode::Mul => self.emit_binary(inst, "imul"),
            IrOpcode::Div => self.emit_div_mod(inst, false),
            IrOpcode::Mod => self.emit_div_mod(inst, true),
            IrOpcode::Eq => self.emit_compare(inst, "sete"),
            IrOpcode::Ne => self.emit_compare(inst, "setne"),
            IrOpcode::Lt => self.emit_compare(inst, "setl"),
            IrOpcode::Le => self.emit_compare(inst, "setle"),
            IrOpcode::Gt => self.emit_compare(inst, "setg"),
            IrOpcode::Ge => self.emit_compare(inst, "setge"),
            IrOpcode::And => self.emit_logical(inst, "and"),
            IrOpcode::Or => self.emit_logical(inst, "or"),
            IrOpcode::Not => self.emit_not(inst),
            IrOpcode::Neg => self.emit_neg(inst),
            IrOpcode::BitNot => self.emit_bitnot(inst),
            IrOpcode::BitAnd => self.emit_binary(inst, "and"),
            IrOpcode::BitOr => self.emit_binary(inst, "or"),
            IrOpcode::BitXor => self.emit_binary(inst, "xor"),
            IrOpcode::Jump => self.emit_jump(inst),
            IrOpcode::Call => self.emit_call(inst),
            IrOpcode::Ret => self.emit_ret(inst),
            IrOpcode::CondBr => self.emit_cond_br(inst),
            IrOpcode::Load => self.emit_load(inst),
            IrOpcode::Store => self.emit_store(inst),
            IrOpcode::Slice => self.emit_slice(inst),
            IrOpcode::StrGetByte => self.emit_str_get_byte(inst),
            IrOpcode::StrSetByte => self.emit_str_set_byte(inst),
            IrOpcode::CoroYield => self.emit_yield(inst),
            IrOpcode::CallIndirect => self.emit_call_indirect(inst),
            IrOpcode::MakeClosure => self.emit_make_closure(inst),
            IrOpcode::CallClosure => self.emit_call_closure(inst),
            IrOpcode::LoadCaptured => self.emit_load_captured(inst),
            IrOpcode::StoreCaptured => self.emit_store_captured(inst),
            IrOpcode::AllocArray => {
                if let Some(ref result) = inst.result {
                    if !self.global_names.contains(result.as_str()) && self.get_slot(result).is_none() {
                        let elem_size = inst.result_type.as_ref().map_or(4, |t| t.size());
                        self.alloc_slot(result, elem_size);
                    }
                }
            }
        }
        self.free_all_scratch();
    }

    fn emit_assign(&mut self, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            let op_ty = operand.get_type();
            let ty = inst.result_type.as_ref().unwrap_or(&op_ty);
            if ty.size() > 8 {
                self.copy_aggregate(operand, result, ty.size());
                return;
            }
            let wide = AsmGenerator::is_wide_type(ty);
            let reg = self.alloc_scratch(wide);
            self.load_operand(operand, reg);
            self.store_result(result, reg, ty);
        }
    }

    fn copy_aggregate(&mut self, operand: &IrOperand, result: &str, size: u32) {
        let src_name = match operand {
            IrOperand::Variable(name, _) => name.clone(),
            _ => return,
        };
        let src_mem = if let Some(co_off) = self.coro_offset(&src_name) {
            self.restore_coro_ctx();
            format!("[rcx + {co_off}]")
        } else {
            self.mem_for(&src_name)
        };
        let dst_mem = if let Some(co_off) = self.coro_offset(result) {
            self.restore_coro_ctx();
            format!("[rcx + {co_off}]")
        } else {
            self.mem_for(result)
        };

        let qwords = size / 8;
        self.line(&format!("lea rsi, {src_mem}"));
        self.line(&format!("lea rdi, {dst_mem}"));
        self.line(&format!("mov rcx, {qwords}"));
        self.line("rep movsq");
    }

    fn emit_binary(&mut self, inst: &IrInstruction, mnemonic: &str) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            let left_ty = left.get_type();
            let result_ty = inst.result_type.as_ref().unwrap_or(&left_ty);
            let wide = AsmGenerator::is_wide_type(result_ty);
            let r1 = self.alloc_scratch(wide);
            let r2 = self.alloc_scratch(wide);
            self.load_operand(left, r1);
            self.load_operand(right, r2);
            self.line(&format!("{mnemonic} {r1}, {r2}"));
            self.store_result(result, r1, result_ty);
        }
    }

    fn emit_logical(&mut self, inst: &IrInstruction, mnemonic: &str) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            let left_ty = left.get_type();
            let right_ty = right.get_type();
            let wide = AsmGenerator::is_wide_type(&left_ty) || AsmGenerator::is_wide_type(&right_ty);
            let r1 = self.alloc_scratch(wide);
            let r2 = self.alloc_scratch(wide);
            self.load_operand(left, r1);
            self.load_operand(right, r2);
            self.line(&format!("{mnemonic} {r1}, {r2}"));
            let bool_reg = if wide {
                self.reg_name(REGS_64.iter().position(|r| *r == r1).unwrap_or(0), false)
            } else {
                r1
            };
            let low = reg_low_byte(bool_reg);
            self.line(&format!("setnz {low}"));
            self.line(&format!("movzx {bool_reg}, {low}"));
            self.store_result(result, bool_reg, &crate::ir::IrType::Bool);
        }
    }

    fn emit_div_mod(&mut self, inst: &IrInstruction, is_mod: bool) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax");
            self.load_operand(right, "ebx");
            self.line("cdq");
            self.line("idiv ebx");
            let src = if is_mod { "edx" } else { "eax" };
            self.store_result(result, src, &crate::ir::IrType::Int);
        }
    }

    fn emit_compare(&mut self, inst: &IrInstruction, setcc: &str) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            let left_ty = left.get_type();
            let wide = AsmGenerator::is_wide_type(&left_ty);
            let r1 = self.alloc_scratch(wide);
            let r2 = self.alloc_scratch(wide);
            self.load_operand(left, r1);
            self.load_operand(right, r2);
            self.line(&format!("cmp {r1}, {r2}"));
            let cmp_reg = if wide {
                self.reg_name(REGS_64.iter().position(|r| *r == r1).unwrap_or(0), false)
            } else {
                r1
            };
            let low = reg_low_byte(cmp_reg);
            self.line(&format!("{setcc} {low}"));
            self.line(&format!("movzx {cmp_reg}, {low}"));
            self.store_result(result, cmp_reg, &crate::ir::IrType::Bool);
        }
    }

    fn emit_not(&mut self, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            let r = self.alloc_scratch(false);
            self.load_operand(operand, r);
            self.line(&format!("test {r}, {r}"));
            self.line("setz al");
            self.line("movzx eax, al");
            self.store_result(result, "eax", &crate::ir::IrType::Bool);
        }
    }

    fn emit_neg(&mut self, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            let op_ty = operand.get_type();
            let result_ty = inst.result_type.as_ref().unwrap_or(&op_ty);
            let wide = AsmGenerator::is_wide_type(result_ty);
            let r = self.alloc_scratch(wide);
            self.load_operand(operand, r);
            self.line(&format!("neg {r}"));
            self.store_result(result, r, result_ty);
        }
    }

    fn emit_bitnot(&mut self, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            let op_ty = operand.get_type();
            let result_ty = inst.result_type.as_ref().unwrap_or(&op_ty);
            let wide = AsmGenerator::is_wide_type(result_ty);
            let r = self.alloc_scratch(wide);
            self.load_operand(operand, r);
            self.line(&format!("not {r}"));
            self.store_result(result, r, result_ty);
        }
    }
}

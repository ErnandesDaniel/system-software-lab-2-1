use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::{IrInstruction, IrOperand};

impl AsmGenerator {
    pub fn emit_make_closure(&mut self, inst: &IrInstruction) {
        let result = match &inst.result {
            Some(r) => r.clone(),
            _ => return,
        };
        let env_base = self.get_slot("__env_slot_0").map_or(0, |s| s.offset);

        for (i, op) in inst.operands.iter().enumerate().skip(1) {
            if let IrOperand::Variable(name, _) = op {
                let slot_off = env_base + (i - 1) as i32 * 8;
                let mem = self.mem_for(name);
                self.line(&format!("lea rax, {mem}"));
                self.line(&format!("mov [rbp + {slot_off}], rax"));
            }
        }

        self.line(&format!("lea rax, [rbp + {env_base}]"));
        self.store_result(&result, "rax", &crate::ir::IrType::Int);
    }

    pub fn emit_call_closure(&mut self, inst: &IrInstruction) {
        if let (Some(func_op), Some(env_op)) = (inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(func_op, "rax");
            self.load_operand(env_op, "rcx");

            for (i, arg) in inst.operands.iter().enumerate().skip(2) {
                if (i - 2) < 3 {
                    let wide = AsmGenerator::is_wide_type(&arg.get_type());
                    let reg = match i - 2 {
                        0 => { if wide { "rdx" } else { "edx" } }
                        1 => { if wide { "r8" } else { "r8d" } }
                        2 => { if wide { "r9" } else { "r9d" } }
                        _ => "eax",
                    };
                    self.load_operand(arg, reg);
                }
            }

            self.line("sub rsp, 32");
            self.line("call rax");
            self.line("add rsp, 32");

            if let Some(ref result) = inst.result {
                let ret_ty = inst.result_type.as_ref().unwrap_or(&crate::ir::IrType::Int);
                let r = if AsmGenerator::is_wide_type(ret_ty) { "rax" } else { "eax" };
                self.store_result(result, r, ret_ty);
            }
        }
    }

    pub fn emit_load_captured(&mut self, inst: &IrInstruction) {
        if let (Some(env_op), Some(slot_op)) = (inst.operands.first(), inst.operands.get(1)) {
            let slot = match slot_op {
                IrOperand::Constant(crate::ir::Constant::Int(v)) => *v as usize,
                _ => 0,
            };
            let result = match &inst.result {
                Some(r) => r.clone(),
                _ => return,
            };
            self.load_operand(env_op, "rax");
            self.line(&format!("mov rax, [rax + {}]", slot * 8));
            let result_ty = inst.result_type.as_ref().unwrap_or(&crate::ir::IrType::Int);
            let ptr_reg = if AsmGenerator::is_wide_type(result_ty) { "rax" } else { "eax" };
            self.line(&format!("mov {ptr_reg}, [rax]"));
            self.store_result(&result, ptr_reg, result_ty);
        }
    }

    pub fn emit_store_captured(&mut self, inst: &IrInstruction) {
        if let (Some(env_op), Some(slot_op), Some(val_op)) =
            (inst.operands.first(), inst.operands.get(1), inst.operands.get(2))
        {
            let slot = match slot_op {
                IrOperand::Constant(crate::ir::Constant::Int(v)) => *v as usize,
                _ => 0,
            };
            self.load_operand(env_op, "rax");
            self.line(&format!("mov rax, [rax + {}]", slot * 8));
            let val_wide = AsmGenerator::is_wide_type(&val_op.get_type());
            let r = if val_wide { "rcx" } else { "ecx" };
            self.load_operand(val_op, r);
            self.line(&format!("mov [rax], {r}"));
        }
    }
}

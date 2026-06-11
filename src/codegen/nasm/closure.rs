use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::{IrInstruction, IrOperand, IrType};
use crate::OsTarget;

impl AsmGenerator {
    pub fn emit_make_closure(&mut self, inst: &IrInstruction) {
        let result = match &inst.result {
            Some(r) => r.clone(),
            _ => return,
        };

        let func_name = match inst.operands.first() {
            Some(IrOperand::FuncRef(name)) => name.clone(),
            _ => return,
        };

        let num_captures = inst.operands.len().saturating_sub(1);

        if num_captures == 0 {
            self.line(&format!("lea rax, [rel {func_name}]"));
            self.store_result(&result, "rax", &IrType::Closure(vec![], Box::new(IrType::Int)));
            return;
        }

        self.line(&format!("mov ecx, {}", num_captures * 8));
        if self.os == OsTarget::Windows {
            self.line("sub rsp, 32");
        }
        self.line("call xmalloc");
        if self.os == OsTarget::Windows {
            self.line("add rsp, 32");
        }
        self.line("mov r8, rax");

        for (i, op) in inst.operands.iter().enumerate().skip(1) {
            if let IrOperand::Variable(name, _) = op {
                if self.wrapped_vars.contains(name) {
                    let mem = self.mem_for(name);
                    self.line(&format!("mov r9, {mem}"));
                    self.line(&format!("mov [r8 + {}], r9", (i - 1) * 8));
                } else {
                    self.line("push r8");
                    self.line("mov ecx, 4");
                    if self.os == OsTarget::Windows {
                        self.line("sub rsp, 32");
                    }
                    self.line("call xmalloc");
                    if self.os == OsTarget::Windows {
                        self.line("add rsp, 32");
                    }
                    self.line("mov r9, rax");

                    self.load_operand(op, "eax");
                    self.line("mov [r9], eax");

                    self.line("pop r8");
                    self.line(&format!("mov [r8 + {}], r9", (i - 1) * 8));

                    let mem = self.mem_for(name);
                    self.line(&format!("mov {mem}, r9"));

                    self.wrapped_vars.insert(name.clone());
                }
            }
        }

        let slot_offset = self.ensure_16_byte_slot(&result);
        self.line(&format!("lea rdx, [rel {func_name}]"));
        self.line(&format!("mov [rbp + {}], rdx", slot_offset));
        self.line(&format!("mov [rbp + {}], r8", slot_offset + 8));
    }

    fn ensure_16_byte_slot(&mut self, name: &str) -> i32 {
        if let Some(slot) = self.get_slot(name) {
            return slot.offset;
        }
        self.alloc_slot(name, 16);
        self.get_slot(name).map_or(0, |s| s.offset)
    }

    pub fn emit_call_closure(&mut self, inst: &IrInstruction) {
        let Some(closure_op) = inst.operands.first() else {
            return;
        };

        let closure_name = match closure_op {
            IrOperand::Variable(name, _) => name.clone(),
            _ => return,
        };

        let closure_slot = match self.get_slot(&closure_name) {
            Some(s) => s,
            None => return,
        };

        self.line(&format!("mov rax, [rbp + {}]", closure_slot.offset));
        self.line(&format!("mov rcx, [rbp + {}]", closure_slot.offset + 8));

        for (i, arg) in inst.operands.iter().enumerate().skip(1) {
            let arg_ty = arg.get_type();
            let wide = AsmGenerator::is_wide_type(&arg_ty);
            let reg = if self.os == OsTarget::Linux {
                match i - 1 {
                    0 => { if wide { "rsi" } else { "esi" } }
                    1 => { if wide { "rdx" } else { "edx" } }
                    2 => { if wide { "rcx" } else { "ecx" } }
                    3 => { if wide { "r8" } else { "r8d" } }
                    4 => { if wide { "r9" } else { "r9d" } }
                    _ => { if wide { "rax" } else { "eax" } }
                }
            } else {
                match i - 1 {
                    0 => { if wide { "rdx" } else { "edx" } }
                    1 => { if wide { "r8" } else { "r8d" } }
                    2 => { if wide { "r9" } else { "r9d" } }
                    _ => { if wide { "rax" } else { "eax" } }
                }
            };
            self.load_operand(arg, reg);
        }

        if self.os == OsTarget::Windows {
            self.line("sub rsp, 32");
        }
        self.line("call rax");
        if self.os == OsTarget::Windows {
            self.line("add rsp, 32");
        }

        if let Some(ref result) = inst.result {
            let ret_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
            let r = if AsmGenerator::is_wide_type(ret_ty) { "rax" } else { "eax" };
            self.store_result(result, r, ret_ty);
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
            let result_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
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

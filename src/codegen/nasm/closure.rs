use crate::ir::types::{IrInstruction, IrOperand};

use crate::codegen::nasm::AsmGenerator;

impl AsmGenerator {
    pub fn generate_make_closure(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            let num_captures = inst.operands.len().saturating_sub(1);
            let env_base_offset = if num_captures > 0 {
                if let Some(&first_slot_offset) = self.locals.get("__env_slot_0") {
                    first_slot_offset
                } else {
                    0
                }
            } else {
                0
            };
            for (i, op) in inst.operands.iter().enumerate().skip(1) {
                if let IrOperand::Variable(name, _) = op {
                    let env_slot = i - 1;
                    if let Some(offset) = self.locals.get(name) {
                        self.output.push_str(&format!("    lea rax, [rbp + {offset}]\n"));
                    } else if let Some(offset) = self.temps.get(name) {
                        self.output.push_str(&format!("    lea rax, [rbp + {offset}]\n"));
                    } else {
                        self.output.push_str(&format!("    lea rax, [rel {name}]\n"));
                    }
                    self.output.push_str(&format!(
                        "    mov [rbp + {}], rax\n",
                        env_base_offset + env_slot as i32 * 8
                    ));
                }
            }
            self.output
                .push_str(&format!("    lea rax, [rbp + {env_base_offset}]\n"));
            self.store_variable(result, "rax", true);
        }
    }

    pub fn generate_call_closure(&mut self, inst: &IrInstruction) {
        if let (Some(func_op), Some(env_op)) = (inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(func_op, "rax", true);
            self.load_operand(env_op, "rcx", true);
            for (i, arg) in inst.operands.iter().enumerate().skip(2) {
                if (i - 2) < 3 {
                    let is_pointer = arg.get_type().is_pointer();
                    let reg = match i - 2 {
                        0 => {
                            if is_pointer { "rdx" } else { "edx" }
                        }
                        1 => {
                            if is_pointer { "r8" } else { "r8d" }
                        }
                        2 => {
                            if is_pointer { "r9" } else { "r9d" }
                        }
                        _ => "eax",
                    };
                    self.load_operand(arg, reg, is_pointer);
                }
            }
            self.output.push_str("    sub rsp, 32\n");
            self.output.push_str("    call rax\n");
            self.output.push_str("    add rsp, 32\n");
            if let Some(ref result) = inst.result {
                let is_pointer = inst
                    .result_type
                    .as_ref()
                    .is_some_and(crate::ir::types::IrType::is_pointer);
                let reg = if is_pointer { "rax" } else { "eax" };
                self.store_variable(result, reg, is_pointer);
            }
        }
    }

    pub fn generate_load_captured(&mut self, inst: &IrInstruction) {
        if let (Some(env_op), Some(slot_op)) = (inst.operands.first(), inst.operands.get(1)) {
            let slot = match slot_op {
                IrOperand::Constant(crate::ir::Constant::Int(v)) => *v as usize,
                _ => 0,
            };
            self.load_operand(env_op, "rax", true);
            self.output.push_str(&format!("    mov rax, [rax + {}]\n", slot * 8));
            let ptr_reg = if inst.result_type.as_ref().is_some_and(|t| t.is_pointer()) { "rax" } else { "eax" };
            self.output.push_str(&format!("    mov {ptr_reg}, [rax]\n"));
            if let Some(ref result) = inst.result {
                self.store_variable(result, ptr_reg, ptr_reg == "rax");
            }
        }
    }

    pub fn generate_store_captured(&mut self, inst: &IrInstruction) {
        if let (Some(env_op), Some(slot_op), Some(val_op)) =
            (inst.operands.first(), inst.operands.get(1), inst.operands.get(2))
        {
            let slot = match slot_op {
                IrOperand::Constant(crate::ir::Constant::Int(v)) => *v as usize,
                _ => 0,
            };
            self.load_operand(env_op, "rax", true);
            self.output.push_str(&format!("    mov rax, [rax + {}]\n", slot * 8));
            let is_pointer = val_op.get_type().is_pointer();
            let reg = if is_pointer { "rcx" } else { "ecx" };
            self.load_operand(val_op, reg, is_pointer);
            self.output
                .push_str(&format!("    mov [rax], {}\n", if is_pointer { "rcx" } else { "ecx" }));
        }
    }
}

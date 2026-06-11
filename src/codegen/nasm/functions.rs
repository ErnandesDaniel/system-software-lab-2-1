use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::{IrInstruction, IrOperand, IrType};
use crate::OsTarget;

impl AsmGenerator {
    pub fn emit_call(&mut self, inst: &IrInstruction) {
        let func_name = match &inst.jump_target {
            Some(f) => f.clone(),
            _ => return,
        };
        self.used_functions.push(func_name.clone());

        let max_params = if self.os == OsTarget::Linux { 6 } else { 4 };
        for (i, arg) in inst.operands.iter().enumerate() {
            if i < max_params {
                let arg_ty = arg.get_type();
                let wide = AsmGenerator::is_wide_type(&arg_ty) || arg_ty.size() > 8;
                let param_reg = self.param_register_name(i, wide);
                self.load_operand(arg, &param_reg);
            }
        }

        if self.os == OsTarget::Windows {
            self.line("sub rsp, 32");
        }
        self.line("xor eax, eax");
        self.line(&format!("call {func_name}"));
        if self.os == OsTarget::Windows {
            self.line("add rsp, 32");
        }

        if let Some(ref result) = inst.result {
            let ret_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
            let r = if AsmGenerator::is_wide_type(ret_ty) {
                "rax"
            } else {
                "eax"
            };
            self.store_result(result, r, ret_ty);
        }
    }

    pub fn emit_ret(&mut self, inst: &IrInstruction) {
        let has_wide_ret = inst
            .operands
            .first()
            .is_some_and(|op| AsmGenerator::is_wide_type(&op.get_type()));
        let ret_reg = if has_wide_ret { "rax" } else { "eax" };

        if let Some(operand) = inst.operands.first() {
            if let IrOperand::FuncRef(name) = operand {
                self.line(&format!("lea rax, [rel {name}]"));
            } else {
                self.load_operand(operand, ret_reg);
            }
        }

        self.line("leave");
        self.line("ret");
    }

    pub fn emit_call_indirect(&mut self, inst: &IrInstruction) {
        if let Some(func_ptr) = inst.operands.first() {
            self.load_operand(func_ptr, "rax");
            let max_params = if self.os == OsTarget::Linux { 6 } else { 4 };
            for (i, arg) in inst.operands.iter().enumerate().skip(1) {
                if i < max_params {
                    let arg_ty = arg.get_type();
                    let wide = AsmGenerator::is_wide_type(&arg_ty);
                    let param_reg = self.param_register_name(i - 1, wide);
                    self.load_operand(arg, &param_reg);
                }
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
                let r = if AsmGenerator::is_wide_type(ret_ty) {
                    "rax"
                } else {
                    "eax"
                };
                self.store_result(result, r, ret_ty);
            }
        }
    }

    fn param_register_name(&self, i: usize, wide: bool) -> String {
        if self.os == OsTarget::Linux {
            match i {
                0 => {
                    if wide {
                        "rdi".to_string()
                    } else {
                        "edi".to_string()
                    }
                }
                1 => {
                    if wide {
                        "rsi".to_string()
                    } else {
                        "esi".to_string()
                    }
                }
                2 => {
                    if wide {
                        "rdx".to_string()
                    } else {
                        "edx".to_string()
                    }
                }
                3 => {
                    if wide {
                        "rcx".to_string()
                    } else {
                        "ecx".to_string()
                    }
                }
                4 => {
                    if wide {
                        "r8".to_string()
                    } else {
                        "r8d".to_string()
                    }
                }
                5 => {
                    if wide {
                        "r9".to_string()
                    } else {
                        "r9d".to_string()
                    }
                }
                _ => {
                    if wide {
                        "rax".to_string()
                    } else {
                        "eax".to_string()
                    }
                }
            }
        } else {
            match i {
                0 => {
                    if wide {
                        "rcx".to_string()
                    } else {
                        "ecx".to_string()
                    }
                }
                1 => {
                    if wide {
                        "rdx".to_string()
                    } else {
                        "edx".to_string()
                    }
                }
                2 => {
                    if wide {
                        "r8".to_string()
                    } else {
                        "r8d".to_string()
                    }
                }
                3 => {
                    if wide {
                        "r9".to_string()
                    } else {
                        "r9d".to_string()
                    }
                }
                _ => {
                    if wide {
                        "rax".to_string()
                    } else {
                        "eax".to_string()
                    }
                }
            }
        }
    }
}

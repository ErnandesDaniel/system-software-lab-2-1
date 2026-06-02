use crate::ir::types::{Constant, IrInstruction, IrOperand};

use crate::codegen::nasm::AsmGenerator;

impl AsmGenerator {
    pub(crate) fn restore_coro_ctx(&mut self) {
        if self.is_coroutine {
            self.output
                .push_str(&format!("    mov rcx, [rbp + {}]\n", self.coro_ctx_offset));
        }
    }

    pub fn generate_call(&mut self, inst: &IrInstruction) {
        if let Some(ref func_name) = inst.jump_target {
            self.used_functions.push(func_name.clone());

            for (i, arg) in inst.operands.iter().enumerate() {
                if i < 4 {
                    let arg_ty = arg.get_type();
                    let is_pointer = arg_ty.is_pointer() || arg_ty.size() > 8;
                    let load_reg = self.get_param_register(i, is_pointer);
                    if is_pointer {
                        self.load_pointer_arg(arg, &load_reg);
                    } else if let IrOperand::Variable(name, _) = arg {
                        if !self.locals.contains_key(name)
                            && !self.temps.contains_key(name)
                            && !self.param_registers.contains(&name.clone())
                        {
                            let lea_reg = match load_reg.as_str() {
                                "ecx" => "rcx", "edx" => "rdx",
                                "r8d" => "r8", "r9d" => "r9",
                                _ => &load_reg,
                            };
                            self.output.push_str(&format!("    lea {lea_reg}, [rel {name}]\n"));
                        } else {
                            self.load_operand(arg, &load_reg, false);
                        }
                    } else {
                        self.load_operand(arg, &load_reg, false);
                    }
                }
            }

            self.output.push_str("    sub rsp, 32\n");
            self.output.push_str(&format!("    call {func_name}\n"));
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

    pub fn get_param_register(&self, i: usize, is_pointer: bool) -> String {
        match i {
            0 => {
                if is_pointer { "rcx".to_string() } else { "ecx".to_string() }
            }
            1 => {
                if is_pointer { "rdx".to_string() } else { "edx".to_string() }
            }
            2 => {
                if is_pointer { "r8".to_string() } else { "r8d".to_string() }
            }
            3 => {
                if is_pointer { "r9".to_string() } else { "r9d".to_string() }
            }
            _ => "ecx".to_string(),
        }
    }

    fn load_pointer_arg(&mut self, arg: &IrOperand, load_reg: &str) {
        let is_struct = matches!(arg, IrOperand::Variable(_, ty) if !ty.is_pointer() && ty.size() > 8);
        match arg {
            IrOperand::Constant(Constant::String(s)) => {
                let func = self.current_function.as_deref().unwrap_or("anon");
                let label = format!("{func}_str_{}", self.string_counter);
                self.string_counter += 1;
                self.emit_string_data(&label, s);
                self.output.push_str(&format!("    lea {load_reg}, [{label}]\n"));
            }
            IrOperand::Variable(name, ty) => {
                let use_lea = is_struct || (ty.is_pointer() && ty.size() > 8);
                if self.is_coroutine {
                    if let Some(offset) = self.locals.get(name) {
                        let co_off = 56 + (-offset - 8);
                        if use_lea {
                            self.restore_coro_ctx();
                            self.output.push_str(&format!("    lea rax, [rcx + {co_off}]\n"));
                            self.output.push_str(&format!("    mov {load_reg}, rax\n"));
                        } else if load_reg == "rcx" {
                            self.restore_coro_ctx();
                            self.output.push_str(&format!("    mov rax, [rcx + {co_off}]\n"));
                            self.output.push_str(&format!("    mov rcx, rax\n"));
                        } else {
                            self.output.push_str("    push rcx\n");
                            self.restore_coro_ctx();
                            self.output.push_str(&format!("    mov rax, [rcx + {co_off}]\n"));
                            self.output.push_str("    pop rcx\n");
                            self.output.push_str(&format!("    mov {load_reg}, rax\n"));
                        }
                    } else if let Some(offset) = self.temps.get(name) {
                        if use_lea {
                            self.output.push_str(&format!("    lea rax, [rbp + {offset}]\n"));
                        } else {
                            self.output.push_str(&format!("    mov rax, [rbp + {offset}]\n"));
                        }
                        self.output.push_str(&format!("    mov {load_reg}, rax\n"));
                    } else {
                        if use_lea {
                            self.output.push_str(&format!("    lea {load_reg}, [rel {name}]\n"));
                        } else {
                            self.output.push_str(&format!("    mov {load_reg}, [rel {name}]\n"));
                        }
                    }
                } else {
                    if let Some(offset) = self.locals.get(name) {
                        if use_lea {
                            self.output.push_str(&format!("    lea rax, [rbp + {offset}]\n"));
                        } else {
                            self.output.push_str(&format!("    mov rax, [rbp + {offset}]\n"));
                        }
                        self.output.push_str(&format!("    mov {load_reg}, rax\n"));
                    } else if let Some(offset) = self.temps.get(name) {
                        if use_lea {
                            self.output.push_str(&format!("    lea rax, [rbp + {offset}]\n"));
                        } else {
                            self.output.push_str(&format!("    mov rax, [rbp + {offset}]\n"));
                        }
                        self.output.push_str(&format!("    mov {load_reg}, rax\n"));
                    } else {
                        if use_lea {
                            self.output.push_str(&format!("    lea {load_reg}, [rel {name}]\n"));
                        } else {
                            self.output.push_str(&format!("    mov {load_reg}, [rel {name}]\n"));
                        }
                    }
                }
            }
            _ => self.load_operand(arg, load_reg, true),
        }
    }

    pub fn generate_yield(&mut self, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            if let IrOperand::Constant(c) = operand {
                self.load_constant(c, "eax");
                self.restore_coro_ctx();
                self.output.push_str("    mov [rcx], eax\n");
                self.output.push_str("    leave\n");
                self.output.push_str("    ret\n");
            }
        }
    }

    pub fn generate_ret(&mut self, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            match operand {
                IrOperand::Constant(c) => self.load_constant(c, "eax"),
                IrOperand::FuncRef(name) => {
                    self.output.push_str(&format!("    lea rax, [rel {name}]\n"));
                }
                IrOperand::Variable(name, ty) => {
                    let is_pointer = ty.is_pointer();
                    if let Some(offset) = self.temps.get(name) {
                        let reg = if is_pointer || self.param_registers.contains(name) { "rax" } else { "eax" };
                        self.output.push_str(&format!("    mov {reg}, [rbp + {offset}]\n"));
                    } else if let Some(offset) = self.locals.get(name) {
                        let reg = if is_pointer || self.param_registers.contains(name) { "rax" } else { "eax" };
                        self.output.push_str(&format!("    mov {reg}, [rbp + {offset}]\n"));
                    } else if self.param_registers.contains(name) {
                        let idx = self.param_registers.iter().position(|r| r == name).expect("Param not found in register list");
                        let src_reg = match idx {
                            0 => "rcx",
                            1 => "rdx",
                            2 => "r8",
                            3 => "r9",
                            _ => "rcx",
                        };
                        let ret_reg = if is_pointer { "rax" } else { "eax" };
                        self.output.push_str(&format!("    mov {ret_reg}, {src_reg}\n"));
                    }
                }
            }
        }
        if self.is_coroutine {
            self.restore_coro_ctx();
            self.output.push_str("    mov dword [rcx], -1\n");
            self.output.push_str("    mov dword [rcx + 16], eax\n");
        }
        self.output.push_str("    leave\n");
        self.output.push_str("    ret\n");
    }

    pub fn generate_call_indirect(&mut self, inst: &IrInstruction) {
        if let Some(func_ptr) = inst.operands.first() {
            self.load_operand(func_ptr, "rax", true);
            for (i, arg) in inst.operands.iter().enumerate().skip(1) {
                if i < 4 {
                    let is_pointer = arg.get_type().is_pointer();
                    let load_reg = self.get_param_register(i - 1, is_pointer);
                    self.load_operand(arg, &load_reg, is_pointer);
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
}

use crate::ir::types::*;

use super::AsmGenerator;

impl AsmGenerator {
    pub fn generate_call(&mut self, inst: &IrInstruction) {
        if let Some(ref func_name) = inst.jump_target {
            self.used_functions.push(func_name.clone());

            for (i, arg) in inst.operands.iter().enumerate() {
                if i < 4 {
                    let is_pointer = arg.get_type().is_pointer();
                    let load_reg = self.get_param_register(i, is_pointer);
                    if is_pointer {
                        self.load_pointer_arg(arg, &load_reg);
                    } else if let IrOperand::Variable(name, _) = arg {
                        // Check if it's a global variable (not local, not temp, not param)
                        if !self.locals.contains_key(name)
                            && !self.temps.contains_key(name)
                            && !self.param_registers.contains(&name.to_string())
                        {
                            // Global — pass by address
                            self.output.push_str(&format!("    lea {}, [rel {}]\n", load_reg, name));
                        } else {
                            self.load_operand(arg, &load_reg, false);
                        }
                    } else {
                        self.load_operand(arg, &load_reg, false);
                    }
                }
            }

            self.output.push_str("    sub rsp, 32\n");
            self.output.push_str(&format!("    call {}\n", func_name));
            self.output.push_str("    add rsp, 32\n");

            if let Some(ref result) = inst.result {
                let is_pointer = inst
                    .result_type
                    .as_ref()
                    .map(|t| t.is_pointer())
                    .unwrap_or(false);
                let reg = if is_pointer { "rax" } else { "eax" };
                self.store_variable(result, reg, is_pointer);
            }
        }
    }

    pub fn get_param_register(&self, i: usize, is_pointer: bool) -> String {
        match i {
            0 => {
                if is_pointer {
                    "rcx".to_string()
                } else {
                    "ecx".to_string()
                }
            }
            1 => {
                if is_pointer {
                    "rdx".to_string()
                } else {
                    "edx".to_string()
                }
            }
            2 => {
                if is_pointer {
                    "r8".to_string()
                } else {
                    "r8d".to_string()
                }
            }
            3 => {
                if is_pointer {
                    "r9".to_string()
                } else {
                    "r9d".to_string()
                }
            }
            _ => "ecx".to_string(),
        }
    }

    fn load_pointer_arg(&mut self, arg: &IrOperand, load_reg: &str) {
        match arg {
            IrOperand::Constant(Constant::String(s)) => {
                let label = format!("str_{}", self.string_counter);
                self.string_counter += 1;
                self.emit_string_data(&label, s);
                self.output
                    .push_str(&format!("    lea {}, [{}]\n", load_reg, label));
            }
            IrOperand::Variable(name, _) => {
                if let Some(offset) = self.locals.get(name) {
                    self.output
                        .push_str(&format!("    mov rax, [rbp + {}]\n", offset));
                    self.output
                        .push_str(&format!("    mov {}, rax\n", load_reg));
                } else if let Some(offset) = self.temps.get(name) {
                    self.output
                        .push_str(&format!("    mov rax, [rbp + {}]\n", offset));
                    self.output
                        .push_str(&format!("    mov {}, rax\n", load_reg));
                }
            }
            _ => self.load_operand(arg, load_reg, true),
        }
    }

    pub fn generate_ret(&mut self, inst: &IrInstruction) {
        if self.is_coroutine && !inst.operands.is_empty() {
            if let IrOperand::Constant(c) = &inst.operands[0] {
                // Coroutine yield: store state number to rip
                self.load_constant(c, "eax");
                self.output.push_str("    mov [rcx], eax\n");
                self.output.push_str("    leave\n");
                self.output.push_str("    ret\n");
                return;
            }
        }
        if let Some(operand) = inst.operands.first() {
            match operand {
                IrOperand::Constant(c) => self.load_constant(c, "eax"),
                IrOperand::Variable(name, ty) => {
                    let is_pointer = ty.is_pointer();
                    if let Some(offset) = self.temps.get(name) {
                        let reg = if is_pointer { "rax" } else { "eax" };
                        self.output
                            .push_str(&format!("    mov {}, [rbp + {}]\n", reg, offset));
                    } else if let Some(offset) = self.locals.get(name) {
                        let reg = if is_pointer { "rax" } else { "eax" };
                        self.output
                            .push_str(&format!("    mov {}, [rbp + {}]\n", reg, offset));
                    } else if self.param_registers.contains(name) {
                        let idx = self.param_registers.iter().position(|r| r == name).unwrap();
                        let src_reg = match idx {
                            0 => {
                                if is_pointer { "rcx" } else { "ecx" }
                            }
                            1 => {
                                if is_pointer { "rdx" } else { "edx" }
                            }
                            2 => {
                                if is_pointer { "r8" } else { "r8d" }
                            }
                            3 => {
                                if is_pointer { "r9" } else { "r9d" }
                            }
                            _ => "eax",
                        };
                        self.output.push_str(&format!("    mov eax, {}\n", src_reg));
                    }
                }
            }
        }
        if self.is_coroutine {
            // For coroutines: signal finished to scheduler
            self.output.push_str("    mov dword [rcx + 64], 1\n");
            self.output.push_str("    mov dword [rcx + 68], eax\n");
        }
        self.output.push_str("    leave\n");
        self.output.push_str("    ret\n");
    }

    pub fn load_operand(&mut self, operand: &IrOperand, dest: &str, _is_pointer: bool) {
        use crate::codegen::traits::OperandLoader;
        
        match operand {
            IrOperand::Variable(name, ty) => {
                let is_ptr = ty.is_pointer();
                let _is_temp = Self::is_temp(name);
                if let Some(offset) = self.locals.get(name) {
                    if self.is_coroutine {
                        let co_off = 72 + (-offset) as i32;
                        self.output.push_str(&format!("    mov {}, [rcx + {}]\n", dest, co_off));
                    } else {
                        self.output
                            .push_str(&format!("    mov {}, [rbp + {}]\n", dest, offset));
                    }
                } else if let Some(offset) = self.temps.get(name) {
                    self.output
                        .push_str(&format!("    mov {}, [rbp + {}]\n", dest, offset));
                } else if self.param_registers.contains(name) {
                    let idx = self.param_registers.iter().position(|r| r == name).unwrap();
                    let src_reg = match idx {
                        0 => {
                            if is_ptr {
                                "rcx"
                            } else {
                                "ecx"
                            }
                        }
                        1 => {
                            if is_ptr {
                                "rdx"
                            } else {
                                "edx"
                            }
                        }
                        2 => {
                            if is_ptr {
                                "r8"
                            } else {
                                "r8d"
                            }
                        }
                        3 => {
                            if is_ptr {
                                "r9"
                            } else {
                                "r9d"
                            }
                        }
                        _ => {
                            if is_ptr {
                                "rcx"
                            } else {
                                "ecx"
                            }
                        }
                    };
                    self.output
                        .push_str(&format!("    mov {}, {}\n", dest, src_reg));
                } else {
                    self.output
                        .push_str(&format!("    mov {}, [rel {}]\n", dest, name));
                }
            }
            IrOperand::Constant(c) => self.load_constant(c, dest),
        }
    }

    pub fn load_constant(&mut self, constant: &Constant, dest: &str) {
        match constant {
            Constant::Int(v) => self.output.push_str(&format!("    mov {}, {}\n", dest, v)),
            Constant::Bool(b) => {
                self.output
                    .push_str(&format!("    mov {}, {}\n", dest, if *b { 1 } else { 0 }))
            }
            Constant::String(s) => {
                let label = format!("str_{}", self.string_counter);
                self.string_counter += 1;
                self.emit_string_data(&label, s);
                let reg = if dest == "eax" { "rax" } else { dest };
                self.output
                    .push_str(&format!("    lea {}, [{}]\n", reg, label));
            }
            Constant::Char(c) => {
                let reg = if dest == "eax" { "eax" } else { dest };
                self.output
                    .push_str(&format!("    mov {}, {}\n", reg, *c as i32));
            }
            Constant::Array(_) => {
                self.output.push_str(&format!("    mov {}, 0\n", dest));
            }
        }
    }

    pub fn store_variable(&mut self, name: &str, src: &str, _is_pointer: bool) {
        if self.is_coroutine {
            if let Some(offset) = self.locals.get(name) {
                let co_off = 72 + (-offset) as i32;
                self.output.push_str(&format!("    mov [rcx + {}], {}\n", co_off, src));
                return;
            }
        }
        if let Some(offset) = self.locals.get(name) {
            self.output
                .push_str(&format!("    mov [rbp + {}], {}\n", offset, src));
        } else if let Some(offset) = self.temps.get(name) {
            self.output
                .push_str(&format!("    mov [rbp + {}], {}\n", offset, src));
        } else if name.starts_with('t') {
            let offset = -8 * (self.temp_counter as i32 + 1);
            self.temps.insert(name.to_string(), offset);
            self.temp_counter += 1;
            self.output
                .push_str(&format!("    mov [rbp + {}], {}\n", offset, src));
        } else {
            // Assume it's a global variable
            self.output.push_str(&format!("    mov [rel {}], {}\n", name, src));
        }
    }
}

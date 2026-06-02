use crate::ir::types::{Constant, IrOperand};

use crate::codegen::nasm::AsmGenerator;

impl AsmGenerator {
    pub fn load_operand(&mut self, operand: &IrOperand, dest: &str, is_pointer: bool) {
        match operand {
            IrOperand::Variable(name, ty) => {
                let _is_ptr = ty.is_pointer() || is_pointer;
                let dest64 = match dest {
                    "ecx" => "rcx",
                    "edx" => "rdx",
                    "r8d" => "r8",
                    "r9d" => "r9",
                    "eax" => "rax",
                    _ => dest,
                };
                if let Some(offset) = self.locals.get(name) {
                    if self.param_registers.contains(name) {
                        self.output.push_str(&format!("    mov {dest64}, [rbp + {offset}]\n"));
                    } else if self.is_coroutine {
                        let co_off = 56 + (-offset - 8);
                        self.restore_coro_ctx();
                        self.output.push_str(&format!("    mov {dest}, [rcx + {co_off}]\n"));
                    } else {
                        self.output.push_str(&format!("    mov {dest}, [rbp + {offset}]\n"));
                    }
                } else if let Some(offset) = self.temps.get(name) {
                    let temp_dest = if ty.is_pointer() && dest.starts_with('e') { dest64 } else { dest };
                    self.output.push_str(&format!("    mov {temp_dest}, [rbp + {offset}]\n"));
                } else if self.param_registers.contains(name) {
                    let idx = self.param_registers.iter().position(|r| r == name).expect("Param not found in register list");
                    let src_reg = match idx {
                        0 => "rcx",
                        1 => "rdx",
                        2 => "r8",
                        3 => "r9",
                        _ => "rcx",
                    };
                    self.output.push_str(&format!("    mov {dest64}, {src_reg}\n"));
                } else {
                    self.output.push_str(&format!("    mov {dest}, [rel {name}]\n"));
                }
            }
            IrOperand::Constant(c) => self.load_constant(c, dest),
            IrOperand::FuncRef(func_name) => {
                self.output.push_str(&format!("    lea rax, [rel {func_name}]\n"));
                if dest != "rax" {
                    self.output.push_str(&format!("    mov {dest}, rax\n"));
                }
            }
        }
    }

    pub fn load_constant(&mut self, constant: &Constant, dest: &str) {
        match constant {
            Constant::Int(v) => self.output.push_str(&format!("    mov {dest}, {v}\n")),
            Constant::Bool(b) => self.output.push_str(&format!("    mov {}, {}\n", dest, i32::from(*b))),
            Constant::String(s) => {
                let func = self.current_function.as_deref().unwrap_or("anon");
                let label = format!("{func}_str_{}", self.string_counter);
                self.string_counter += 1;
                self.emit_string_data(&label, s);
                let reg = if dest == "eax" { "rax" } else { dest };
                self.output.push_str(&format!("    lea {reg}, [{label}]\n"));
            }
            Constant::Char(c) => {
                let reg = if dest == "eax" { "eax" } else { dest };
                self.output.push_str(&format!("    mov {}, {}\n", reg, i32::from(*c)));
            }
            Constant::Array(_) => {
                self.output.push_str(&format!("    mov {dest}, 0\n"));
            }
        }
    }

    pub fn store_variable(&mut self, name: &str, src: &str, _is_pointer: bool) {
        if self.global_names.contains(name) {
            self.output.push_str(&format!("    mov [rel {name}], {src}\n"));
            return;
        }
        if self.is_coroutine {
            if let Some(offset) = self.locals.get(name) {
                let co_off = 56 + (-offset - 8);
                self.restore_coro_ctx();
                self.output.push_str(&format!("    mov [rcx + {co_off}], {src}\n"));
                return;
            }
        }
        if let Some(offset) = self.locals.get(name) {
            self.output.push_str(&format!("    mov [rbp + {offset}], {src}\n"));
        } else if let Some(offset) = self.temps.get(name) {
            self.output.push_str(&format!("    mov [rbp + {offset}], {src}\n"));
        } else if name.starts_with('t') {
            let offset = -8 * (self.temp_counter as i32 + 1);
            self.temps.insert(name.to_string(), offset);
            self.temp_counter += 1;
            self.output.push_str(&format!("    mov [rbp + {offset}], {src}\n"));
        } else {
            self.output.push_str(&format!("    mov [rel {name}], {src}\n"));
        }
    }
}

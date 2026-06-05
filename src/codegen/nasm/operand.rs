use crate::codegen::nasm::AsmGenerator;
use crate::codegen::nasm::REGS_32;
use crate::ir::types::{Constant, IrOperand, IrType};

impl AsmGenerator {
    pub fn mem_for(&self, name: &str) -> String {
        if self.global_names.contains(name) {
            format!("[rel {name}]")
        } else if let Some(slot) = self.get_slot(name) {
            format!("[rbp + {}]", slot.offset)
        } else {
            format!("[rel {name}]")
        }
    }

    pub fn coro_offset(&self, name: &str) -> Option<i32> {
        if self.is_coroutine {
            if let Some(slot) = self.get_slot(name) {
                return Some(56 + (-slot.offset - 8));
            }
        }
        None
    }

    pub fn load_operand(&mut self, operand: &IrOperand, reg: &str) {
        match operand {
            IrOperand::Variable(name, ty) => {
                if let Some(co_off) = self.coro_offset(name) {
                    let needs_rcx_save = reg != "rcx" && reg != "ecx" && reg != "rsp";
                    if needs_rcx_save { self.line("push rcx"); }
                    self.restore_coro_ctx();
                    let nreg = if reg == "ecx" || reg == "rcx" { "eax" } else { reg };
                    let use_lea = matches!(ty, IrType::Array(_, _) | IrType::Struct { .. });
                    if use_lea {
                        let lea_reg = if nreg.starts_with('e') {
                            self.reg_name(REGS_32.iter().position(|r| *r == nreg).unwrap_or(0), true)
                        } else {
                            nreg
                        };
                        self.line(&format!("lea {lea_reg}, [rcx + {co_off}]"));
                        if lea_reg != reg {
                            self.line(&format!("mov {reg}, {lea_reg}"));
                        }
                    } else if Self::is_wide_type(ty) {
                        let reg64 = if nreg.starts_with('e') {
                            self.reg_name(REGS_32.iter().position(|r| *r == nreg).unwrap_or(0), true)
                        } else {
                            nreg
                        };
                        self.line(&format!("mov {reg64}, [rcx + {co_off}]"));
                        if needs_rcx_save { self.line("pop rcx"); }
                        if reg64 != reg {
                            self.line(&format!("mov {reg}, {reg64}"));
                        }
                    } else {
                        self.line(&format!("mov {nreg}, [rcx + {co_off}]"));
                        if needs_rcx_save { self.line("pop rcx"); }
                        if nreg != reg {
                            if reg.starts_with('r') && nreg.starts_with('e') {
                                let second = reg.as_bytes().get(1).copied().unwrap_or(0);
                                let reg32 = if second.is_ascii_alphabetic() {
                                    format!("e{}", &reg[1..])
                                } else {
                                    format!("{}d", reg)
                                };
                                self.line(&format!("mov {reg32}, {nreg}"));
                            } else {
                                self.line(&format!("mov {reg}, {nreg}"));
                            }
                        }
                    }
                    if needs_rcx_save && use_lea { self.line("pop rcx"); }
                    return;
                }
                let mem = self.mem_for(name);
                let use_lea = matches!(ty, IrType::Array(_, _) | IrType::Struct { .. });
                if use_lea {
                    let lea_reg = if reg.starts_with('e') {
                        self.reg_name(REGS_32.iter().position(|r| *r == reg).unwrap_or(0), true)
                    } else {
                        reg
                    };
                    self.line(&format!("lea {lea_reg}, {mem}"));
                } else if Self::is_wide_type(ty) {
                    let reg64 = if reg.starts_with('e') {
                        self.reg_name(REGS_32.iter().position(|r| *r == reg).unwrap_or(0), true)
                    } else {
                        reg
                    };
                    self.line(&format!("mov {reg64}, {mem}"));
                } else {
                    self.line(&format!("mov {reg}, {mem}"));
                }
            }
            IrOperand::Constant(c) => self.load_constant(c, reg),
            IrOperand::FuncRef(func_name) => {
                self.line(&format!("lea rax, [rel {func_name}]"));
                if reg != "rax" {
                    self.line(&format!("mov {reg}, rax"));
                }
            }
        }
    }

    pub fn store_result(&mut self, name: &str, reg: &str, ty: &IrType) {
        if self.global_names.contains(name) {
            self.line(&format!("mov [rel {name}], {reg}"));
            return;
        }

        if let Some(co_off) = self.coro_offset(name) {
            self.restore_coro_ctx();
            if Self::is_wide_type(ty) {
                let reg64 = if reg.starts_with('e') {
                    self.reg_name(REGS_32.iter().position(|r| *r == reg).unwrap_or(0), true)
                } else {
                    reg
                };
                self.line(&format!("mov [rcx + {co_off}], {reg64}"));
            } else {
                let nreg = if reg == "ecx" || reg == "rcx" { "eax" } else { reg };
                if nreg != reg {
                    let second = reg.as_bytes().get(1).copied().unwrap_or(0);
                    let r32 = if second.is_ascii_alphabetic() {
                        format!("e{}", &reg[1..])
                    } else {
                        format!("{}d", reg)
                    };
                    self.line(&format!("mov {nreg}, {r32}"));
                }
                self.line(&format!("mov [rcx + {co_off}], {nreg}"));
            }
            return;
        }

        let slot_size = self.ensure_slot(name, ty);
        let mem = format!("[rbp + {}]", self.get_slot(name).unwrap().offset);

        if slot_size > 4 {
            let reg64 = if reg.starts_with('e') {
                self.reg_name(REGS_32.iter().position(|r| *r == reg).unwrap_or(0), true)
            } else {
                reg
            };
            self.line(&format!("mov {mem}, {reg64}"));
        } else {
            self.line(&format!("mov {mem}, {reg}"));
        }
    }

    fn ensure_slot(&mut self, name: &str, ty: &IrType) -> u32 {
        if let Some(slot) = self.get_slot(name) {
            return slot.size;
        }
        let size = ty.size().max(4);
        self.alloc_slot(name, size);
        size
    }

    pub fn load_constant(&mut self, constant: &Constant, reg: &str) {
        match constant {
            Constant::Int(v) => self.line(&format!("mov {reg}, {v}")),
            Constant::Bool(b) => self.line(&format!("mov {reg}, {}", i32::from(*b))),
            Constant::Char(c) => self.line(&format!("mov {reg}, {}", i32::from(*c))),
            Constant::String(s) => {
                let func = self.current_function.as_deref().unwrap_or("anon");
                let label = format!("{func}_str_{}", self.string_counter);
                self.string_counter += 1;
                self.emit_string_data(&label, s);
                let wide = if reg == "eax" { "rax" } else { reg };
                self.line(&format!("lea {wide}, [{label}]"));
            }
            Constant::Array(_) => {
                self.line(&format!("mov {reg}, 0"));
            }
        }
    }
}

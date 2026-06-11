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

    pub fn load_operand(&mut self, operand: &IrOperand, reg: &str) {
        match operand {
            IrOperand::Variable(name, ty) => {
                let mem = self.mem_for(name);
                let is_struct_ptr = matches!(ty, IrType::Struct { .. });
                if is_struct_ptr && self.param_names.contains(name) {
                    let wide_reg = if reg.starts_with('e') {
                        Self::reg_name(REGS_32.iter().position(|r| *r == reg).unwrap_or(0), true)
                    } else {
                        reg
                    };
                    self.line(&format!("mov {wide_reg}, {mem}"));
                } else if matches!(ty, IrType::Array(_, _) | IrType::Struct { .. }) {
                    let lea_reg = if reg.starts_with('e') {
                        Self::reg_name(REGS_32.iter().position(|r| *r == reg).unwrap_or(0), true)
                    } else {
                        reg
                    };
                    self.line(&format!("lea {lea_reg}, {mem}"));
                } else if Self::is_wide_type(ty) {
                    let reg64 = if reg.starts_with('e') {
                        Self::reg_name(REGS_32.iter().position(|r| *r == reg).unwrap_or(0), true)
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

        let slot_size = self.ensure_slot(name, ty);
        let mem = if let Some(slot) = self.get_slot(name) {
            format!("[rbp + {}]", slot.offset)
        } else {
            format!("[rel {name}]")
        };

        if slot_size > 4 {
            let reg64 = if reg.starts_with('e') {
                Self::reg_name(REGS_32.iter().position(|r| *r == reg).unwrap_or(0), true)
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

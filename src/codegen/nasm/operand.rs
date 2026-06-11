use crate::codegen::nasm::{AsmGenerator, REGS_32, REGS_64};
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

                if self.wrapped_vars.contains(name) {
                    self.line(&format!("mov rcx, {mem}"));
                    self.line(&format!("mov {reg}, [rcx]"));
                    return;
                }

                let is_env_param = name == "__env";
                let is_struct_ptr = matches!(ty, IrType::Struct { .. });
                if is_env_param || (is_struct_ptr && self.param_names.contains(name)) {
                    let wide_reg = Self::to_wide(reg);
                    self.line(&format!("mov {wide_reg}, {mem}"));
                } else if is_struct_ptr {
                    let wide_reg = Self::to_wide(reg);
                    self.line(&format!("lea {wide_reg}, {mem}"));
                } else if self.heap_allocated.contains(name) || matches!(ty, IrType::Closure(_, _)) {
                    let wide_reg = Self::to_wide(reg);
                    self.line(&format!("mov {wide_reg}, {mem}"));
                } else if Self::is_wide_type(ty) {
                    let reg64 = Self::to_wide(reg);
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

        if self.wrapped_vars.contains(name) {
            let mem = self.mem_for(name);
            self.line(&format!("mov rcx, {mem}"));
            self.line(&format!("mov [rcx], {reg}"));
            return;
        }

        let slot_size = self.ensure_slot(name, ty);
        let mem = if let Some(slot) = self.get_slot(name) {
            format!("[rbp + {}]", slot.offset)
        } else {
            format!("[rel {name}]")
        };

        if slot_size > 4 {
            let reg64 = Self::to_wide(reg);
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

    fn to_wide(reg: &str) -> &'static str {
        Self::reg_name(
            REGS_32.iter().position(|r| *r == reg)
                .or_else(|| REGS_64.iter().position(|r| *r == reg))
                .unwrap_or(0),
            true,
        )
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

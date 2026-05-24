use crate::ir::types::{Constant, IrInstruction, IrOperand};

use super::AsmGenerator;

impl AsmGenerator {
    fn restore_coro_ctx(&mut self) {
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
                    let is_pointer = arg.get_type().is_pointer();
                    let load_reg = self.get_param_register(i, is_pointer);
                    if is_pointer {
                        self.load_pointer_arg(arg, &load_reg);
                    } else if let IrOperand::Variable(name, _) = arg {
                        // Check if it's a global variable (not local, not temp, not param)
                        if !self.locals.contains_key(name)
                            && !self.temps.contains_key(name)
                            && !self.param_registers.contains(&name.clone())
                        {
                            // Global — pass by address
                            self.output.push_str(&format!("    lea {load_reg}, [rel {name}]\n"));
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
                    .is_some_and(super::super::ir::types::IrType::is_pointer);
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
                self.output.push_str(&format!("    lea {load_reg}, [{label}]\n"));
            }
            IrOperand::Variable(name, _) => {
                if let Some(offset) = self.locals.get(name) {
                    self.output.push_str(&format!("    mov rax, [rbp + {offset}]\n"));
                    self.output.push_str(&format!("    mov {load_reg}, rax\n"));
                } else if let Some(offset) = self.temps.get(name) {
                    self.output.push_str(&format!("    mov rax, [rbp + {offset}]\n"));
                    self.output.push_str(&format!("    mov {load_reg}, rax\n"));
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
                        let reg = if is_pointer { "rax" } else { "eax" };
                        self.output.push_str(&format!("    mov {reg}, [rbp + {offset}]\n"));
                    } else if let Some(offset) = self.locals.get(name) {
                        let reg = if is_pointer { "rax" } else { "eax" };
                        self.output.push_str(&format!("    mov {reg}, [rbp + {offset}]\n"));
                    } else if self.param_registers.contains(name) {
                        let idx = self.param_registers.iter().position(|r| r == name).unwrap();
                        let src_reg = match idx {
                            0 => {
                                if is_pointer {
                                    "rcx"
                                } else {
                                    "ecx"
                                }
                            }
                            1 => {
                                if is_pointer {
                                    "rdx"
                                } else {
                                    "edx"
                                }
                            }
                            2 => {
                                if is_pointer {
                                    "r8"
                                } else {
                                    "r8d"
                                }
                            }
                            3 => {
                                if is_pointer {
                                    "r9"
                                } else {
                                    "r9d"
                                }
                            }
                            _ => "eax",
                        };
                        self.output.push_str(&format!("    mov eax, {src_reg}\n"));
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
            // Load function pointer into rax
            self.load_operand(func_ptr, "rax", true);
            // Load arguments (operands[1..]) into registers
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
                    .is_some_and(super::super::ir::types::IrType::is_pointer);
                let reg = if is_pointer { "rax" } else { "eax" };
                self.store_variable(result, reg, is_pointer);
            }
        }
    }

    pub fn load_operand(&mut self, operand: &IrOperand, dest: &str, is_pointer: bool) {
        use crate::codegen::traits::OperandLoader;

        match operand {
            IrOperand::Variable(name, ty) => {
                let is_ptr = ty.is_pointer() || is_pointer;
                let _is_temp = Self::is_temp(name);
                if let Some(offset) = self.locals.get(name) {
                    if self.is_coroutine {
                        let co_off = 24 + (-offset);
                        self.restore_coro_ctx();
                        self.output.push_str(&format!("    mov {dest}, [rcx + {co_off}]\n"));
                    } else {
                        self.output.push_str(&format!("    mov {dest}, [rbp + {offset}]\n"));
                    }
                } else if let Some(offset) = self.temps.get(name) {
                    self.output.push_str(&format!("    mov {dest}, [rbp + {offset}]\n"));
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
                    self.output.push_str(&format!("    mov {dest}, {src_reg}\n"));
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
                let label = format!("str_{}", self.string_counter);
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
        if self.is_coroutine {
            if let Some(offset) = self.locals.get(name) {
                let co_off = 24 + (-offset);
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
            // Assume it's a global variable
            self.output.push_str(&format!("    mov [rel {name}], {src}\n"));
        }
    }

    pub fn generate_make_closure(&mut self, inst: &IrInstruction) {
        // operands[0] = FuncRef(mangled_name) — identifies the inner function
        // operands[1..] = captured variable operands
        // result = env temp name
        if let Some(ref result) = inst.result {
            let num_captures = inst.operands.len().saturating_sub(1);
            // Env slots are pre-allocated as locals (__env_slot_N) in the function frame
            // Find the first env slot offset (all slots are consecutive from env base)
            let env_base_offset = if num_captures > 0 {
                // The env slots are named "__env_slot_0", "__env_slot_1", etc.
                // They were added as locals during IR generation and are already in the frame
                if let Some(&first_slot_offset) = self.locals.get("__env_slot_0") {
                    first_slot_offset
                } else {
                    0
                }
            } else {
                0
            };
            // Store addresses of captured variables into env slots (rbp-relative)
            for (i, op) in inst.operands.iter().enumerate().skip(1) {
                if let IrOperand::Variable(name, _) = op {
                    let env_slot = i - 1;
                    // Get the address of the captured variable
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
            // Store env pointer into result temp (pointer to env struct in frame)
            self.output
                .push_str(&format!("    lea rax, [rbp + {env_base_offset}]\n"));
            self.store_variable(result, "rax", true);
        }
    }

    pub fn generate_call_closure(&mut self, inst: &IrInstruction) {
        // operands[0] = func ptr variable
        // operands[1] = env ptr variable
        // operands[2..] = regular args
        if let (Some(func_op), Some(env_op)) = (inst.operands.first(), inst.operands.get(1)) {
            // Load function pointer into rax
            self.load_operand(func_op, "rax", true);
            // Load env pointer into rcx (hidden first param)
            self.load_operand(env_op, "rcx", true);
            // Load regular args (shifted: params start from edx)
            for (i, arg) in inst.operands.iter().enumerate().skip(2) {
                if (i - 2) < 3 {
                    let is_pointer = arg.get_type().is_pointer();
                    let reg = match i - 2 {
                        0 => {
                            if is_pointer {
                                "rdx"
                            } else {
                                "edx"
                            }
                        }
                        1 => {
                            if is_pointer {
                                "r8"
                            } else {
                                "r8d"
                            }
                        }
                        2 => {
                            if is_pointer {
                                "r9"
                            } else {
                                "r9d"
                            }
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
                    .is_some_and(super::super::ir::types::IrType::is_pointer);
                let reg = if is_pointer { "rax" } else { "eax" };
                self.store_variable(result, reg, is_pointer);
            }
        }
    }

    pub fn generate_load_captured(&mut self, inst: &IrInstruction) {
        // operands[0] = env variable (__env)
        // operands[1] = slot index (constant)
        // result = value loaded through env
        if let (Some(env_op), Some(slot_op)) = (inst.operands.first(), inst.operands.get(1)) {
            let slot = match slot_op {
                IrOperand::Constant(crate::ir::Constant::Int(v)) => *v as usize,
                _ => 0,
            };
            // Load env pointer into rax
            self.load_operand(env_op, "rax", true);
            // Load captured variable's address from env[slot]
            self.output.push_str(&format!("    mov rax, [rax + {}]\n", slot * 8));
            // Load value from that address
            self.output.push_str("    mov eax, [rax]\n");
            if let Some(ref result) = inst.result {
                self.store_variable(result, "eax", false);
            }
        }
    }

    pub fn generate_store_captured(&mut self, inst: &IrInstruction) {
        // operands[0] = env variable (__env)
        // operands[1] = slot index (constant)
        // operands[2] = value to store
        if let (Some(env_op), Some(slot_op), Some(val_op)) =
            (inst.operands.first(), inst.operands.get(1), inst.operands.get(2))
        {
            let slot = match slot_op {
                IrOperand::Constant(crate::ir::Constant::Int(v)) => *v as usize,
                _ => 0,
            };
            // Load env pointer into rax
            self.load_operand(env_op, "rax", true);
            // Load captured variable's address from env[slot]
            self.output.push_str(&format!("    mov rax, [rax + {}]\n", slot * 8));
            // Load value into ecx
            let is_pointer = val_op.get_type().is_pointer();
            let reg = if is_pointer { "rcx" } else { "ecx" };
            self.load_operand(val_op, reg, is_pointer);
            // Store value through the address
            self.output
                .push_str(&format!("    mov [rax], {}\n", if is_pointer { "rcx" } else { "ecx" }));
        }
    }
}

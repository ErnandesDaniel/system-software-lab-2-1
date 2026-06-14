use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::{IrFunction, IrOpcode, IrType};
use std::collections::HashSet;

impl AsmGenerator {
    pub fn generate_single_function(&mut self, func: &IrFunction) -> String {
        self.output.clear();
        self.string_counter = 0;
        self.data_section.clear();

        self.line_global("bits 64");
        self.line_global("default rel");
        self.line_global("section .text");
        self.line_global(&format!("global {}", func.name));

        let externs = Self::collect_externs(func);
        if !externs.is_empty() {
            self.line_global("");
        }
        for ext in &externs {
            self.line_global(&format!("extern {ext}"));
        }
        if !externs.is_empty() {
            self.line_global("");
        }

        self.current_function = Some(func.name.clone());
        self.reset_for_function();

        self.emit_prologue(func);
        self.emit_body(func);

        if !self.data_section.is_empty() {
            self.output.push_str("\nsection .data\n");
            self.output.push_str(&self.data_section);
        }

        std::mem::take(&mut self.output)
    }

    pub(crate) fn map_extern_name(name: &str) -> String {
        match name {
            "fread_nasm" => "fread".to_string(),
            "fwrite_nasm" => "fwrite".to_string(),
            "fseek_nasm" => "fseek".to_string(),
            "socket_nasm" => "socket".to_string(),
            "setsockopt_nasm" => "setsockopt".to_string(),
            "fcntl_nasm" => "fcntl".to_string(),
            "bind_nasm" => "bind".to_string(),
            "listen_nasm" => "listen".to_string(),
            "accept_nasm" => "accept".to_string(),
            "send_nasm" => "send".to_string(),
            "recv_nasm" => "recv".to_string(),
            "close_nasm" => "close".to_string(),
            "getsockname_nasm" => "getsockname".to_string(),
            _ => name.to_string(),
        }
    }

    fn collect_externs(func: &IrFunction) -> Vec<String> {
        let mut set: HashSet<String> = HashSet::new();
        for ext in &func.used_functions {
            set.insert(Self::map_extern_name(ext));
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let IrOpcode::Call = inst.opcode {
                    if let Some(ref target) = inst.jump_target {
                        set.insert(Self::map_extern_name(target));
                    }
                }
                for op in &inst.operands {
                    if let crate::ir::IrOperand::FuncRef(name) = op {
                        set.insert(Self::map_extern_name(name));
                    }
                }
                if matches!(inst.opcode, IrOpcode::MakeClosure | IrOpcode::AllocArray) {
                    set.insert("malloc".to_string());
                }
            }
        }
        let mut list: Vec<String> = set.into_iter().collect();
        list.sort();
        list
    }

    fn emit_prologue(&mut self, func: &IrFunction) {
        let mut stack_size: i32 = 0;

        for local in &func.locals {
            if self.global_names.contains(&local.name) {
                continue;
            }
            let raw_size = local.ty.size().max(8) as u32;
            let aligned = raw_size.next_power_of_two() as i32;
            self.alloc_slot(&local.name, raw_size);
            stack_size += aligned;
        }

        self.param_registers.clear();
        self.param_names.clear();
        for (i, param) in func.parameters.iter().enumerate() {
            self.param_names.insert(param.name.clone());
            if i < 4 {
                self.param_registers.push(param.name.clone());
                if !self.slots.contains_key(&param.name) {
                    self.alloc_slot(&param.name, 8);
                    stack_size += 8;
                }
            }
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref result) = inst.result {
                    if result.starts_with('t')
                        && !self.slots.contains_key(result)
                        && !self.global_names.contains(result.as_str())
                    {
                        let slot_size = inst.result_type.as_ref().map_or(8, |t| {
                            if matches!(t, IrType::Array(_, _)) {
                                8u32
                            } else if matches!(t, IrType::Closure(_, _)) {
                                16u32
                            } else {
                                t.size().max(8) as u32
                            }
                        });
                        self.alloc_slot(result, slot_size);
                        stack_size += slot_size as i32;
                    }
                }
            }
        }

        self.line(&format!("{}:", func.name));
        self.line("push rbp");
        self.line("mov rbp, rsp");

        let aligned = ((stack_size as i64 + 15) / 16 * 16).max(16);
        self.line(&format!("sub rsp, {aligned}"));

        let param_names: Vec<String> = self.param_registers.to_vec();
        for (i, param_name) in param_names.iter().enumerate() {
            let reg = match (self.os, i) {
                (crate::OsTarget::Linux, 0) => "rdi",
                (crate::OsTarget::Linux, 1) => "rsi",
                (crate::OsTarget::Linux, 2) => "rdx",
                (crate::OsTarget::Linux, 3) => "rcx",
                (crate::OsTarget::Linux, 4) => "r8",
                (crate::OsTarget::Linux, 5) => "r9",
                (_, 0) => "rcx",
                (_, 1) => "rdx",
                (_, 2) => "r8",
                (_, 3) => "r9",
                _ => "rax",
            };
            let mem = self.mem_for(param_name);
            self.line(&format!("mov {mem}, {reg}"));
        }
    }

    fn emit_body(&mut self, func: &IrFunction) {
        let mut blocks: Vec<_> = func.blocks.iter().collect();
        blocks.sort_by_key(|b| {
            let num = b.id.trim_start_matches("BB").parse::<i32>().unwrap_or(0);
            num
        });
        for block in &blocks {
            self.emit_block(block);
        }
    }

    fn emit_block(&mut self, block: &crate::ir::IrBlock) {
        let label = self.format_block_label(&block.id);
        self.line(&format!("{label}:"));
        for inst in &block.instructions {
            self.generate_instruction(inst);
        }
    }

    fn line_global(&mut self, s: &str) {
        self.output.push_str(s);
        self.output.push('\n');
    }
}

#[cfg(test)]
impl AsmGenerator {
    pub fn generate(&mut self, program: &crate::ir::IrProgram) -> String {
        self.global_names = program.globals.iter().map(|g| g.name.clone()).collect();
        self.output.clear();
        self.string_counter = 0;
        self.data_section.clear();

        self.line_global("bits 64");
        self.line_global("default rel");
        self.line_global("section .text");
        self.line_global("");

        let mut all_func_names: std::collections::HashSet<String> = std::collections::HashSet::new();
        for func in &program.functions {
            all_func_names.insert(func.name.clone());
            self.line_global(&format!("global {}", func.name));
        }
        self.line_global("");

        let mut extern_set: std::collections::HashSet<String> = std::collections::HashSet::new();
        for func in &program.functions {
            for ext in &func.used_functions {
                extern_set.insert(Self::map_extern_name(ext));
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let IrOpcode::Call = inst.opcode {
                        if let Some(ref target) = inst.jump_target {
                            if !all_func_names.contains(target.as_str()) {
                                extern_set.insert(Self::map_extern_name(target));
                            }
                        }
                    }
                    for op in &inst.operands {
                        if let crate::ir::IrOperand::FuncRef(n) = op {
                            if !all_func_names.contains(n.as_str()) {
                                extern_set.insert(Self::map_extern_name(n));
                            }
                        }
                    }
                }
            }
        }
        let mut externs: Vec<String> = extern_set.into_iter().collect();
        externs.sort();
        for ext in &externs {
            self.line_global(&format!("extern {ext}"));
        }
        if !externs.is_empty() {
            self.line_global("");
        }

        for func in &program.functions {
            self.current_function = Some(func.name.clone());
            self.reset_for_function();
            self.emit_prologue(func);
            self.emit_body(func);
        }

        if !self.data_section.is_empty() || !program.globals.is_empty() {
            self.output.push_str("\nsection .data\n");
            self.output.push_str(&self.data_section);
            self.output.push_str(&Self::generate_globals_asm(&program.globals));
        }

        self.output.clone()
    }
}

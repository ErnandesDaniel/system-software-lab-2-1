use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::{IrFunction, IrOpcode};
use std::collections::{HashMap, HashSet};

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

    fn collect_externs(func: &IrFunction) -> Vec<String> {
        let mut set: HashSet<String> = HashSet::new();
        for ext in &func.used_functions {
            set.insert(ext.clone());
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                if let IrOpcode::Call = inst.opcode {
                    if let Some(ref target) = inst.jump_target {
                        set.insert(target.clone());
                    }
                }
                for op in &inst.operands {
                    if let crate::ir::IrOperand::FuncRef(name) = op {
                        set.insert(name.clone());
                    }
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
            let slot_size = local.ty.size().max(8) as i32;
            self.alloc_slot(&local.name, slot_size as u32);
            stack_size += slot_size;
        }

        self.param_registers.clear();
        for (i, param) in func.parameters.iter().enumerate() {
            if i < 4 {
                self.param_registers.push(param.name.clone());
                if !self.slots.contains_key(&param.name) {
                    self.alloc_slot(&param.name, 8);
                    stack_size += 8;
                }
            }
        }

        self.coro_ctx_offset = if self.is_coroutine {
            let off = self.alloc_slot("__co_ctx", 8);
            stack_size += 8;
            off
        } else {
            0
        };

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref result) = inst.result {
                    if result.starts_with('t')
                        && !self.slots.contains_key(result)
                        && !self.global_names.contains(result.as_str())
                    {
                        self.alloc_slot(result, 8);
                        stack_size += 8;
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
            let reg = if self.is_coroutine {
                match i {
                    0 => "rdx",
                    1 => "r8",
                    2 => "r9",
                    _ => "rax",
                }
            } else {
                match i {
                    0 => "rcx",
                    1 => "rdx",
                    2 => "r8",
                    3 => "r9",
                    _ => "rax",
                }
            };
            let mem = self.mem_for(param_name);
            self.line(&format!("mov {mem}, {reg}"));
        }

        if self.is_coroutine {
            let ctx_mem = self.mem_for("__co_ctx");
            self.line(&format!("mov {ctx_mem}, rcx"));
            self.line("mov eax, [rcx]");
            for s in 0..=self.yield_count {
                self.line(&format!("cmp eax, {s}"));
                self.line(&format!("je {f}_co_{s}", f = func.name));
            }
        }
    }

    fn emit_body(&mut self, func: &IrFunction) {
        if self.is_coroutine {
            let resume_map: HashMap<&str, usize> = func
                .coroutine_blocks
                .iter()
                .enumerate()
                .map(|(i, id)| (id.as_str(), i))
                .collect();
            for block in &func.blocks {
                if let Some(&state) = resume_map.get(block.id.as_str()) {
                    self.line(&format!("{f}_co_{state}:", f = func.name));
                }
                self.emit_block(block);
            }
        } else {
            let mut blocks: Vec<_> = func.blocks.iter().collect();
            blocks.sort_by_key(|b| {
                let num = b.id.trim_start_matches("BB").parse::<i32>().unwrap_or(0);
                num
            });
            for block in &blocks {
                self.emit_block(block);
            }
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
                extern_set.insert(ext.clone());
            }
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let IrOpcode::Call = inst.opcode {
                        if let Some(ref target) = inst.jump_target {
                            if !all_func_names.contains(target.as_str()) {
                                extern_set.insert(target.clone());
                            }
                        }
                    }
                    for op in &inst.operands {
                        if let crate::ir::IrOperand::FuncRef(n) = op {
                            if !all_func_names.contains(n.as_str()) {
                                extern_set.insert(n.clone());
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
            if func.is_coroutine {
                self.set_coroutine(func.yield_count);
            } else {
                self.is_coroutine = false;
                self.yield_count = 0;
            }
            self.current_function = Some(func.name.clone());
            self.reset_for_function();
            self.emit_prologue(func);
            self.emit_body(func);
            self.is_coroutine = false;
        }

        if !self.data_section.is_empty() || !program.globals.is_empty() {
            self.output.push_str("\nsection .data\n");
            self.output.push_str(&self.data_section);
            self.output.push_str(&Self::generate_globals_asm(&program.globals));
        }

        self.output.clone()
    }
}

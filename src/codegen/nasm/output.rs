use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::{IrFunction, IrOpcode, IrOperand};
use std::collections::HashMap;

#[cfg(test)]
use crate::ir::types::IrProgram;

impl AsmGenerator {
    #[cfg(test)]
    pub fn generate(&mut self, program: &IrProgram) -> String {
        self.global_names = program.globals.iter().map(|g| g.name.clone()).collect();
        self.output.push_str("bits 64\n");
        self.output.push_str("default rel\n");
        self.output.push_str("section .text\n\n");

        let mut extern_funcs: std::collections::HashSet<&String> = std::collections::HashSet::new();
        for func in &program.functions {
            for ext_func in &func.used_functions {
                extern_funcs.insert(ext_func);
            }
        }

        if extern_funcs.contains(&"createThread".to_string()) {
            self.output.push_str("extern createThread\n");
        }

        let mut user_funcs: std::collections::HashSet<&String> = std::collections::HashSet::new();
        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let IrOpcode::Call = inst.opcode {
                        if let Some(target) = &inst.jump_target {
                            if !extern_funcs.contains(target) {
                                user_funcs.insert(target);
                            }
                        }
                    }
                }
            }
        }

        for func_name in user_funcs {
            self.output.push_str(&format!("extern {}\n", func_name));
        }

        for ext_func in extern_funcs {
            self.output.push_str(&format!("extern {}\n", ext_func));
        }

        if !program.functions.is_empty() {
            self.output.push_str("\n");
        }

        for func in &program.functions {
            if func.is_coroutine {
                self.set_coroutine(func.yield_count);
            }
            self.generate_function_internal(func);
            self.is_coroutine = false;
        }

        if !self.data_section.is_empty() || !program.globals.is_empty() {
            self.output.push_str("\nsection .data\n");
            self.output.push_str(&self.data_section);
            self.output.push_str(&Self::generate_globals_asm(&program.globals));
        }

        self.output.clone()
    }

    pub fn generate_single_function(&mut self, func: &IrFunction) -> String {
        self.output.clear();
        self.string_counter = 0;
        self.data_section.clear();

        self.output.push_str("bits 64\n");
        self.output.push_str("default rel\n");
        self.output.push_str("section .text\n\n");

        let mut unique_externs: std::collections::HashSet<String> = std::collections::HashSet::new();
        for ext_func in &func.used_functions {
            unique_externs.insert(ext_func.clone());
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if inst.opcode == IrOpcode::Call {
                    if let Some(target) = &inst.jump_target {
                        unique_externs.insert(target.clone());
                    }
                }
                for op in &inst.operands {
                    if let IrOperand::FuncRef(name) = op {
                        unique_externs.insert(name.clone());
                    }
                }
            }
        }

        self.output.push_str("bits 64\n");
        self.output.push_str("default rel\n");
        self.output.push_str("section .text\n\n");
        self.output.push_str(&format!("global {}\n", func.name));

        let mut externs: Vec<_> = unique_externs.into_iter().collect();
        externs.sort();
        for ext_func in &externs {
            if !ext_func.is_empty() {
                self.output.push_str(&format!("extern {ext_func}\n"));
            }
        }

        if !externs.is_empty() {
            self.output.push('\n');
        }

        self.current_function = Some(func.name.clone());
        self.locals.clear();
        self.temps.clear();
        self.temp_counter = 0;

        let mut local_counter: i32 = 1;
        for local in &func.locals {
            if self.global_names.contains(&local.name) {
                continue;
            }
            let local_size = local.ty.size().max(8) as i32;
            let num_slots = (local_size + 7) / 8;
            let offset = -8 * local_counter - 8 * (num_slots - 1).max(0);
            local_counter += num_slots;
            self.locals.insert(local.name.clone(), offset);
        }

        self.param_registers.clear();
        for (i, param) in func.parameters.iter().enumerate() {
            if i < 4 {
                self.param_registers.push(param.name.clone());
            }
        }

        let mut param_save_offsets: Vec<i32> = Vec::new();
        for param_name in &self.param_registers {
            let offset = -8 * local_counter;
            local_counter += 1;
            param_save_offsets.push(offset);
            self.locals.insert(param_name.clone(), offset);
        }

        self.coro_ctx_offset = if self.is_coroutine {
            let off = -8 * local_counter;
            local_counter += 1;
            self.locals.insert("__co_ctx".to_string(), off);
            off
        } else {
            0
        };

        let mut temp_offset: i32 = -(8 * local_counter);
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref result) = inst.result {
                    if result.starts_with('t') && !self.global_names.contains(result.as_str()) && !self.temps.contains_key(result) {
                        self.temps.insert(result.clone(), temp_offset);
                        temp_offset -= 8;
                    }
                }
            }
        }

        let total_vars = local_counter + ((-temp_offset / 8) - local_counter);
        let stack_size = -8 * total_vars;
        let abs_stack = if stack_size < 0 { -stack_size } else { stack_size };
        let aligned = ((abs_stack + 15) / 16) * 16;
        let final_stack = aligned.max(16);

        self.output.push_str(&format!("{}:\n", func.name));

        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");
        self.output.push_str(&format!("    sub rsp, {final_stack}\n"));

        for (i, _param_name) in self.param_registers.iter().enumerate() {
            let reg = match i {
                0 => "rcx",
                1 => "rdx",
                2 => "r8",
                3 => "r9",
                _ => "rax",
            };
            let offset = param_save_offsets[i];
            self.output.push_str(&format!("    mov [rbp + {offset}], {reg}\n"));
        }

        if self.is_coroutine {
            self.output.push_str(&format!("    mov [rbp + {}], rcx\n", self.coro_ctx_offset));
            self.output.push_str("    mov eax, [rcx]\n");
            for s in 0..=self.yield_counter {
                self.output.push_str(&format!("    cmp eax, {s}\n"));
                self.output.push_str(&format!("    je {f}_co_{s}\n", f = func.name));
            }
        }

        if self.is_coroutine {
            let resume_map: HashMap<&str, usize> = func.coroutine_blocks.iter()
                .enumerate()
                .map(|(i, id)| (id.as_str(), i))
                .collect();
            for block in &func.blocks {
                if let Some(&state) = resume_map.get(block.id.as_str()) {
                    self.output.push_str(&format!("{f}_co_{state}:\n", f = func.name));
                }
                self.generate_block(block);
            }
        } else {
            let mut blocks: Vec<_> = func.blocks.iter().collect();
            blocks.sort_by_key(|b| {
                let num = b.id.trim_start_matches("BB").parse::<i32>().unwrap_or(0);
                num
            });
            for block in &blocks {
                self.generate_block(block);
            }
        }

        if !self.data_section.is_empty() {
            self.output.push_str("\nsection .data\n");
            self.output.push_str(&self.data_section);
        }

        std::mem::take(&mut self.output)
    }

    pub(crate) fn format_block_label(&self, id: &str) -> String {
        if id.starts_with("BB") {
            if let Some(ref func_name) = self.current_function {
                format!("{}_{}", func_name, id)
            } else {
                format!("BB_{}", id.trim_start_matches("BB"))
            }
        } else {
            id.to_string()
        }
    }
}

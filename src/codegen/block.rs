use crate::ir::*;

use super::AsmGenerator;

impl AsmGenerator {
    #[allow(dead_code)]
    pub fn generate_function_internal(&mut self, func: &IrFunction) {
        self.current_function = Some(func.name.clone());
        self.locals.clear();
        self.temps.clear();
        self.temp_counter = 0;

        for local in &func.locals {
            let offset = local.stack_offset.unwrap_or_else(|| {
                let off = -8 * (self.locals.len() as i32 + self.temps.len() as i32 + 1);
                off
            });

            if local.name.starts_with('t') {
                self.temps.insert(local.name.clone(), offset);
            } else {
                self.locals.insert(local.name.clone(), offset);
            }
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref result) = inst.result {
                    if result.starts_with('t') {
                        if !self.temps.contains_key(result) {
                            let offset = -8 * (self.temp_counter as i32 + 1);
                            self.temps.insert(result.clone(), offset);
                            self.temp_counter += 1;
                        }
                    }
                }
            }
        }

        self.param_registers.clear();
        for (i, param) in func.parameters.iter().enumerate() {
            if i < 4 {
                let _ = match i {
                    0 => "rcx",
                    1 => "rdx",
                    2 => "r8",
                    3 => "r9",
                    _ => "",
                };
                self.param_registers.push(param.name.clone());
            }
        }

        self.output.push_str(&format!("global {}\n", func.name));
        self.output.push_str(&format!("{}:\n", func.name));
        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");

        let frame_size = self.calculate_frame_size(func);
        if frame_size > 0 {
            self.output
                .push_str(&format!("    sub rsp, {}\n", frame_size));
        }

        for block in &func.blocks {
            self.generate_block(block);
        }

        eprintln!(
            "DEBUG: generate_function_internal: func.name={}, has_create_thread={}",
            func.name, self.has_create_thread
        );

        // If function has createThread calls, add scheduler call before returning
        if self.has_create_thread && func.name == "main" {
            self.used_functions.push("schedule_threads".to_string());
            self.output
                .push_str("    ; Call scheduler to run registered threads\n");
            self.output.push_str("    call schedule_threads\n");
        }

        self.output.push_str("    leave\n");
        self.output.push_str("    ret\n");
    }

    #[allow(dead_code)]
    pub fn calculate_frame_size(&self, func: &IrFunction) -> i32 {
        let mut size = 0;
        for local in &func.locals {
            if let Some(offset) = local.stack_offset {
                size = size.max(-offset);
            }
        }

        let aligned = ((size + 15) / 16) * 16;
        if aligned == 0 && size > 0 {
            aligned + 8
        } else {
            aligned
        }
    }

    pub fn generate_block(&mut self, block: &IrBlock) {
        let label = if block.id.starts_with("BB") {
            format!("BB_{}", block.id.trim_start_matches("BB"))
        } else {
            block.id.clone()
        };
        self.output.push_str(&format!("{}:\n", label));

        for inst in &block.instructions {
            self.generate_instruction(inst);
        }
    }
}

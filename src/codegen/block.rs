use crate::ir::types::*;

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

        if self.is_coroutine {
            self.output.push_str("    mov eax, [rcx]\n");
            for s in 0..=self.yield_counter {
                self.output.push_str(&format!("    cmp eax, {}\n", s));
                self.output.push_str(&format!("    je co_{}\n", s));
            }
        }

        // Allocate stack slots for parameters and save them from registers
        // This ensures function pointer params survive across calls (rcx is caller-saved)
        let param_save_count = self.param_registers.len();
        let mut param_save_offsets: Vec<i32> = Vec::new();
        for (i, param_name) in self.param_registers.iter().enumerate() {
            let offset = -8 * (self.locals.len() as i32 + self.temps.len() as i32 + 1 + i as i32);
            param_save_offsets.push(offset);
            self.locals.insert(param_name.clone(), offset);
        }

        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");

        let mut frame_size = self.calculate_frame_size(func);
        // Ensure frame is large enough for param save slots
        if param_save_count > 0 {
            let param_area = (param_save_count as i32) * 8;
            if frame_size < param_area {
                frame_size = ((param_area + 15) / 16) * 16;
            }
        }
        if frame_size > 0 {
            self.output
                .push_str(&format!("    sub rsp, {}\n", frame_size));
        }

        // Save parameter register values to the allocated stack slots
        // Always use full 64-bit registers — __env is a pointer typed as IrType::Int
        for (i, _param_name) in self.param_registers.iter().enumerate() {
            let reg = match i {
                0 => "rcx",
                1 => "rdx",
                2 => "r8",
                3 => "r9",
                _ => "rax",
            };
            let offset = param_save_offsets[i];
            self.output.push_str(&format!("    mov [rbp + {}], {}\n", offset, reg));
        }

        let mut blocks: Vec<_> = func.blocks.iter().collect();
        blocks.sort_by_key(|b| {
            b.id.trim_start_matches("BB").parse::<i32>().unwrap_or(0)
        });
        for block in &blocks {
            self.generate_block(block);
        }
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

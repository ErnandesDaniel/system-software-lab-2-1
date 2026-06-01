use crate::ir::types::IrBlock;

#[cfg(test)]
use crate::ir::types::IrFunction;

use crate::codegen::nasm::AsmGenerator;

impl AsmGenerator {
    #[cfg(test)]
    pub fn generate_function_internal(&mut self, func: &IrFunction) {
        self.current_function = Some(func.name.clone());
        self.locals.clear();
        self.temps.clear();
        self.temp_counter = 0;

        let mut slot_counter: i32 = 1;
        for local in &func.locals {
            let local_size = local.ty.size().max(8) as i32;
            let num_slots = (local_size + 7) / 8;
            let offset = -8 * slot_counter;
            slot_counter += num_slots;

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
                            let offset = -8 * slot_counter;
                            self.temps.insert(result.clone(), offset);
                            slot_counter += 1;
                        }
                    }
                }
            }
        }

        self.param_registers.clear();
        for (i, param) in func.parameters.iter().enumerate() {
            if i < 4 {
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

        let mut param_save_offsets: Vec<i32> = Vec::new();
        for param_name in self.param_registers.iter() {
            let offset = -8 * slot_counter;
            slot_counter += 1;
            param_save_offsets.push(offset);
            self.locals.insert(param_name.clone(), offset);
        }

        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");

        let total_bytes = 8 * (slot_counter - 1);
        let frame_size = ((total_bytes + 15) / 16).max(1) * 16;
        self.output.push_str(&format!("    sub rsp, {}\n", frame_size));

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
        blocks.sort_by_key(|b| b.id.trim_start_matches("BB").parse::<i32>().unwrap_or(0));
        for block in &blocks {
            self.generate_block(block);
        }
    }

    pub fn generate_block(&mut self, block: &IrBlock) {
        let label = self.format_block_label(&block.id);
        self.output.push_str(&format!("{label}:\n"));

        for inst in &block.instructions {
            self.generate_instruction(inst);
        }
    }
}

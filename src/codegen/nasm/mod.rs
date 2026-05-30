mod block;
mod functions;
mod instructions;

use crate::ir::types::{IrFunction, IrOpcode, IrOperand, IrType};
use std::collections::{HashMap, HashSet};

#[cfg(test)]
use crate::ir::types::IrProgram;

pub struct AsmGenerator {
    output: String,
    string_counter: usize,
    data_section: String,
    locals: HashMap<String, i32>,
    temps: HashMap<String, i32>,
    used_functions: Vec<String>,
    current_function: Option<String>,
    param_registers: Vec<String>,
    temp_counter: usize,
    is_coroutine: bool,
    yield_counter: usize,
    coro_ctx_offset: i32,
    global_names: HashSet<String>,
}

impl AsmGenerator {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {
            output: String::new(),
            string_counter: 0,
            data_section: String::new(),
            locals: HashMap::new(),
            temps: HashMap::new(),
            used_functions: Vec::new(),
            current_function: None,
            param_registers: Vec::new(),
            temp_counter: 0,
            is_coroutine: false,
            yield_counter: 0,
            coro_ctx_offset: 0,
            global_names: HashSet::new(),
        }
    }

    pub fn set_global_names(&mut self, names: &[String]) {
        self.global_names = names.iter().cloned().collect();
    }

    pub fn set_coroutine(&mut self, yield_count: usize) {
        self.is_coroutine = true;
        self.yield_counter = yield_count;
    }

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
            if func.yield_count > 0 {
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
            let offset = -8 * local_counter;
            local_counter += 1;
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
                    if result.starts_with('t') && !self.temps.contains_key(result) {
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
            self.output
                .push_str(&format!("    mov [rbp + {}], rcx\n", self.coro_ctx_offset));
            self.output.push_str("    mov eax, [rcx]\n");
            for s in 0..=self.yield_counter {
                self.output.push_str(&format!("    cmp eax, {s}\n"));
                self.output.push_str(&format!("    je co_{s}\n"));
            }
        }

        if self.is_coroutine {
            use std::collections::HashMap;
            let resume_map: HashMap<&str, usize> = func.coroutine_blocks.iter()
                .enumerate()
                .map(|(i, id)| (id.as_str(), i))
                .collect();
            for block in &func.blocks {
                if let Some(&state) = resume_map.get(block.id.as_str()) {
                    self.output.push_str(&format!("co_{state}:\n"));
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

    fn emit_string_data(&mut self, label: &str, s: &str) {
        let bytes = self.escape_string(s);
        self.data_section.push_str(&format!("{label} db "));

        if bytes.is_empty() {
            self.data_section.push('0');
        } else {
            for (i, b) in bytes.iter().enumerate() {
                if i > 0 {
                    self.data_section.push_str(", ");
                }
                self.data_section.push_str(&format!("{b}"));
            }
        }
        self.data_section.push_str(", 0\n");
    }

    fn escape_string(&self, s: &str) -> Vec<u8> {
        let mut result = Vec::new();
        for c in s.chars() {
            match c {
                '\n' => { result.push(10); }
                '\r' => { result.push(13); }
                '\t' => { result.push(9); }
                '\\' => { result.push(92); }
                '"' => { result.push(34); }
                _ => {
                    let mut buf = [0u8; 4];
                    let encoded = c.encode_utf8(&mut buf);
                    result.extend_from_slice(encoded.as_bytes());
                }
            }
        }
        result
    }

    fn format_block_label(&self, id: &str) -> String {
        if id.starts_with("BB") {
            format!("BB_{}", id.trim_start_matches("BB"))
        } else {
            id.to_string()
        }
    }

    pub fn generate_globals_asm(globals: &[crate::ir::IrGlobal]) -> String {
        let mut output = String::new();
        if globals.is_empty() {
            return output;
        }
        for global in globals {
            output.push_str(&format!("global {}\n", global.name));
            match &global.ty {
                IrType::Int | IrType::Bool => {
                    let val = match &global.initializer {
                        Some(crate::ir::Constant::Int(v)) => *v,
                        _ => 0,
                    };
                    output.push_str(&format!("{} dd {}\n", global.name, val));
                }
                IrType::String => {
                    if let Some(crate::ir::Constant::String(s)) = &global.initializer {
                        let slabel = format!("{}_str", global.name);
                        let bytes: Vec<u8> = s.bytes().collect();
                        output.push_str(&format!("{slabel} db "));
                        if bytes.is_empty() {
                            output.push('0');
                        } else {
                            for (j, b) in bytes.iter().enumerate() {
                                if j > 0 {
                                    output.push_str(", ");
                                }
                                output.push_str(&format!("{b}"));
                            }
                        }
                        output.push_str(", 0\n");
                        output.push_str(&format!("{} dq {}\n", global.name, slabel));
                    } else {
                        output.push_str(&format!("{} dq 0\n", global.name));
                    }
                }
                IrType::Array(elem_type, size) => {
                    let label = global.name.clone();
                    match elem_type.as_ref() {
                        IrType::Int => {
                            output.push_str(&format!("{label} dd "));
                            if let Some(crate::ir::Constant::Array(elems)) = &global.initializer {
                                for (i, elem) in elems.iter().enumerate() {
                                    if i > 0 {
                                        output.push_str(", ");
                                    }
                                    if let crate::ir::Constant::Int(v) = elem {
                                        output.push_str(&format!("{v}"));
                                    } else {
                                        output.push('0');
                                    }
                                }
                                for i in elems.len()..*size {
                                    if i > 0 || !elems.is_empty() {
                                        output.push_str(", ");
                                    }
                                    output.push('0');
                                }
                            } else {
                                for i in 0..*size {
                                    if i > 0 {
                                        output.push_str(", ");
                                    }
                                    output.push('0');
                                }
                            }
                            output.push('\n');
                        }
                        IrType::String => {
                            let init_elems = match &global.initializer {
                                Some(crate::ir::Constant::Array(elems)) => elems.as_slice(),
                                _ => &[],
                            };
                            for i in 0..*size {
                                let slabel = format!("{}_{}", global.name, i);
                                let s = init_elems.get(i).and_then(|e| {
                                    if let crate::ir::Constant::String(s) = e { Some(s.as_str()) } else { None }
                                }).unwrap_or("");
                                output.push_str(&format!("{slabel} db "));
                                let bytes: Vec<u8> = s.bytes().collect();
                                if bytes.is_empty() {
                                    output.push('0');
                                } else {
                                    for (j, b) in bytes.iter().enumerate() {
                                        if j > 0 {
                                            output.push_str(", ");
                                        }
                                        output.push_str(&format!("{b}"));
                                    }
                                    output.push_str(", 0");
                                }
                                output.push('\n');
                            }
                            output.push_str(&format!("{label} dq "));
                            for i in 0..*size {
                                if i > 0 {
                                    output.push_str(", ");
                                }
                                output.push_str(&format!("{}_{}", global.name, i));
                            }
                            output.push('\n');
                        }
                        _ => {}
                    }
                }
                _ => {
                    let size = global.ty.size() as usize;
                    output.push_str(&format!("{} times {} db 0\n", global.name, size));
                }
            }
        }
        output
    }
}

impl Default for AsmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

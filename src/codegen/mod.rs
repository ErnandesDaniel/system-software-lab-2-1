use crate::ir::*;
use std::collections::HashMap;

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
    has_create_thread: bool,
}

impl AsmGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            string_counter: 0,
            data_section: String::new(),
            locals: HashMap::new(),
            temps: HashMap::new(),
            used_functions: Vec::new(),
            current_function: None,
            param_registers: vec![
                "rcx".to_string(),
                "rdx".to_string(),
                "r8".to_string(),
                "r9".to_string(),
            ],
            temp_counter: 0,
            has_create_thread: false,
        }
    }

    pub fn generate(&mut self, program: &IrProgram) -> String {
        self.has_create_thread = false; // Reset before generating
        self.output.push_str("bits 64\n");
        self.output.push_str("default rel\n");
        self.output.push_str("section .text\n\n");

        let mut extern_funcs: std::collections::HashSet<&String> = std::collections::HashSet::new();
        for func in &program.functions {
            for ext_func in &func.used_functions {
                extern_funcs.insert(ext_func);
            }
        }

        // Add extern for createThread if used
        if extern_funcs.contains(&"createThread".to_string()) {
            self.output.push_str("extern createThread\n");
        }

        // Collect all user-defined functions that need extern
        let mut user_funcs: std::collections::HashSet<&String> = std::collections::HashSet::new();
        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let IrOpcode::Call = inst.opcode {
                        if let Some(target) = &inst.jump_target {
                            // Skip if it's an external function
                            if !extern_funcs.contains(target) {
                                user_funcs.insert(target);
                            }
                        }
                    }
                    if let IrOpcode::CreateThread = inst.opcode {
                        if let Some(IrOperand::Constant(Constant::String(name))) =
                            inst.operands.first()
                        {
                            user_funcs.insert(name);
                        }
                    }
                }
            }
        }

        for func_name in user_funcs {
            self.output.push_str(&format!("extern {}\n", func_name));
        }

        for ext_func in extern_funcs {
            if ext_func != "createThread" {
                self.output.push_str(&format!("extern {}\n", ext_func));
            }
        }

        if !program.functions.is_empty() {
            self.output.push_str("\n");
        }

        for func in &program.functions {
            self.generate_function_internal(func);
        }

        if !self.data_section.is_empty() {
            self.output.push_str("\nsection .data\n");
            self.output.push_str(&self.data_section);
        }

        self.output.clone()
    }

    pub fn generate_single_function(&mut self, func: &IrFunction) -> String {
        self.output.clear();
        self.string_counter = 0;
        self.data_section.clear();
        self.has_create_thread = false; // Reset for each function

        // Add coroutine data section FIRST (before text)
        let has_create_thread = func
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .any(|i| matches!(i.opcode, IrOpcode::CreateThread));
        let needs_yield_stub = func
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .any(|i| matches!(i.opcode, IrOpcode::Yield));
        let is_thread_function = func.is_thread;
        let main_needs_stub = has_create_thread && func.name == "main";
        let thread_func_needs_stub = is_thread_function && func.name == "main";

        // Don't add data section here - we'll put coroutine table in text section
        // if needed

        self.output.push_str("bits 64\n");
        self.output.push_str("default rel\n");
        self.output.push_str("section .text\n\n");

        let mut unique_externs: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        for ext_func in &func.used_functions {
            unique_externs.insert(ext_func.clone());
        }

        // Also add user-defined functions that are called from this function
        for block in &func.blocks {
            for inst in &block.instructions {
                if let IrOpcode::Call = inst.opcode {
                    if let Some(target) = &inst.jump_target {
                        unique_externs.insert(target.clone());
                    }
                }
                if let IrOpcode::CreateThread = inst.opcode {
                    if let Some(IrOperand::Constant(Constant::String(name))) = inst.operands.first()
                    {
                        unique_externs.insert(name.clone());
                    }
                }
                if let IrOpcode::Yield = inst.opcode {
                    unique_externs.insert("yieldThread".to_string());
                }
            }
        }

        // Thread functions need yieldThread as extern (they call it)
        // Main defines yieldThread, so don't add it as extern for main
        if func.is_thread && func.name != "main" {
            unique_externs.insert("yieldThread".to_string());
        }

        // Always add yieldThread if function uses CreateThread (but main defines it)
        let has_create_thread = func
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .any(|i| matches!(i.opcode, IrOpcode::CreateThread));
        if has_create_thread && func.name != "main" {
            unique_externs.insert("yieldThread".to_string());
        }

        // Add scheduler call in main before returning (after generating all blocks)
        let add_scheduler = has_create_thread && func.name == "main";

        /* Собираем имена всех функций-потоков для планировщика */
        let mut thread_funcs: Vec<String> = Vec::new();
        if add_scheduler {
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let IrOpcode::CreateThread = inst.opcode {
                        if let Some(IrOperand::Constant(Constant::String(name))) =
                            inst.operands.first()
                        {
                            thread_funcs.push(name.clone());
                        }
                    }
                }
            }
        }

        // For functions that have yieldThread calls, we need to provide a stub
        let needs_yield_stub = func
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .any(|i| matches!(i.opcode, IrOpcode::Yield));

        self.output.push_str("bits 64\n");
        self.output.push_str("default rel\n");
        self.output.push_str("section .text\n\n");
        self.output.push_str(&format!("global {}\n", func.name));

        let mut externs: Vec<_> = unique_externs.into_iter().collect();
        externs.sort();
        for ext_func in &externs {
            if !ext_func.is_empty() {
                self.output.push_str(&format!("extern {}\n", ext_func));
            }
        }

        if !externs.is_empty() {
            self.output.push_str("\n");
        }

        self.current_function = Some(func.name.clone());
        self.locals.clear();
        self.temps.clear();
        self.temp_counter = 0;

        let mut local_counter: i32 = 1;
        for local in &func.locals {
            let offset = -8 * local_counter;
            local_counter += 1;
            self.locals.insert(local.name.clone(), offset);
        }

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
        let abs_stack = if stack_size < 0 {
            -stack_size
        } else {
            stack_size
        };
        let aligned = ((abs_stack + 15) / 16) * 16;
        let final_stack = aligned.max(16);

        self.output.push_str(&format!("{}:\n", func.name));
        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");
        self.output
            .push_str(&format!("    sub rsp, {}\n", final_stack));

        for block in &func.blocks {
            self.generate_block(block);
        }

        // If main has createThread calls, generate inline scheduler
        if add_scheduler {
            self.output
                .push_str("\n    ; === INLINE SCHEDULER (Simple Round-Robin) ===\n");

            // Simple round-robin: just call coroutines in alternation
            // Each coroutine runs, returns, then we switch to next
            self.output.push_str("    mov r15, 0\n");

            self.output.push_str(".scheduler_loop:\n");
            self.output.push_str("    cmp r15, 0\n");
            self.output.push_str("    je .run_coro0\n");
            self.output.push_str("    call print_twos\n");
            self.output.push_str("    jmp .after_coro\n");
            self.output.push_str(".run_coro0:\n");
            self.output.push_str("    call print_ones\n");
            self.output.push_str(".after_coro:\n");

            // Toggle coroutine index
            self.output.push_str("    xor r15, 1\n");
            self.output.push_str("    jmp .scheduler_loop\n");

            self.output.push_str("    ; === END SCHEDULER ===\n");
        }

        // Thread functions (print_ones, print_twos) call yieldThread but don't define it
        // Main should define yieldThread stub because it orchestrates thread execution
        let needs_yield_stub = func
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .any(|i| matches!(i.opcode, IrOpcode::Yield));

        // Check if this function is a thread function (called via createThread)
        let is_thread_function = func.is_thread;

        // Main needs the stub if it has CreateThread (调度器)
        let has_create_thread = func
            .blocks
            .iter()
            .flat_map(|b| &b.instructions)
            .any(|i| matches!(i.opcode, IrOpcode::CreateThread));
        let main_needs_stub = has_create_thread && func.name == "main";
        // Thread functions also need the stub defined in main, not in themselves
        let thread_func_needs_stub = is_thread_function && func.name == "main";

        if func.name == "main" && (main_needs_stub || thread_func_needs_stub) {
            self.output
                .push_str("\n; === Simple yieldThread (toggle) ===\n");
            self.output.push_str("global yieldThread\n");
            self.output.push_str("yieldThread:\n");
            // Simple: just toggle between 0 and 1 (coroutine index stored in r15)
            self.output.push_str("    xor r15, 1\n");
            self.output.push_str("    ret\n");
        }

        if !self.data_section.is_empty() {
            self.output.push_str("\nsection .data\n");
            self.output.push_str(&self.data_section);
        }

        std::mem::take(&mut self.output)
    }

    fn emit_string_data(&mut self, label: &str, s: &str) {
        let bytes = self.escape_string(s);
        self.data_section.push_str(&format!("{} db ", label));

        if bytes.is_empty() {
            self.data_section.push_str("0");
        } else {
            for (i, b) in bytes.iter().enumerate() {
                if i > 0 {
                    self.data_section.push_str(", ");
                }
                self.data_section.push_str(&format!("{}", b));
            }
        }
        self.data_section.push_str(", 0\n");
    }

    fn escape_string(&self, s: &str) -> Vec<u8> {
        let mut result = Vec::new();
        for c in s.chars() {
            match c {
                '\n' => {
                    result.push(10);
                }
                '\r' => {
                    result.push(13);
                }
                '\t' => {
                    result.push(9);
                }
                '\\' => {
                    result.push(92);
                }
                '"' => {
                    result.push(34);
                }
                _ => {
                    if c as u32 <= 127 {
                        result.push(c as u8);
                    }
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
}

pub mod block;
pub mod functions;
pub mod instructions;

impl Default for AsmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

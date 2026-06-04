mod closure;
mod control;
mod data;
mod functions;
mod globals;
mod instructions;
mod memory;
mod operand;
mod output;

use std::collections::{HashMap, HashSet};

pub struct AsmGenerator {
    output: String,
    data_section: String,
    string_counter: usize,
    current_function: Option<String>,
    slots: HashMap<String, StackSlot>,
    next_stack_offset: i32,
    global_names: HashSet<String>,
    used_functions: Vec<String>,
    param_registers: Vec<String>,
    is_coroutine: bool,
    yield_count: usize,
    coro_ctx_offset: i32,
    regs_used_32: u32,
    regs_used_64: u32,
}

#[derive(Debug, Clone, Copy)]
pub struct StackSlot {
    pub offset: i32,
    pub size: u32,
}

const REGS_32: &[&str] = &["eax", "ecx", "edx", "ebx", "esi", "edi", "r8d", "r9d", "r10d", "r11d"];
const REGS_64: &[&str] = &["rax", "rcx", "rdx", "rbx", "rsi", "rdi", "r8", "r9", "r10", "r11"];

impl AsmGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            data_section: String::new(),
            string_counter: 0,
            current_function: None,
            slots: HashMap::new(),
            next_stack_offset: 0,
            global_names: HashSet::new(),
            used_functions: Vec::new(),
            param_registers: Vec::new(),
            is_coroutine: false,
            yield_count: 0,
            coro_ctx_offset: 0,
            regs_used_32: 0,
            regs_used_64: 0,
        }
    }

    pub fn set_global_names(&mut self, names: &[String]) {
        self.global_names = names.iter().cloned().collect();
    }

    pub fn set_coroutine(&mut self, yield_count: usize) {
        self.is_coroutine = true;
        self.yield_count = yield_count;
    }

    pub fn reset_for_function(&mut self) {
        self.slots.clear();
        self.next_stack_offset = 0;
        self.param_registers.clear();
        self.regs_used_32 = 0;
        self.regs_used_64 = 0;
    }

    pub fn alloc_slot(&mut self, name: &str, size: u32) -> i32 {
        let aligned = size.max(8).next_power_of_two() as i32;
        self.next_stack_offset -= aligned;
        let offset = self.next_stack_offset;
        self.slots.insert(name.to_string(), StackSlot { offset, size });
        offset
    }

    pub fn get_slot(&self, name: &str) -> Option<StackSlot> {
        self.slots.get(name).copied()
    }

    pub fn reg_name(&self, index: usize, wide: bool) -> &'static str {
        if wide {
            REGS_64.get(index).unwrap_or(&"rax")
        } else {
            REGS_32.get(index).unwrap_or(&"eax")
        }
    }

    pub fn alloc_scratch(&mut self, wide: bool) -> &'static str {
        let (used, regs) = if wide {
            (&mut self.regs_used_64, REGS_64)
        } else {
            (&mut self.regs_used_32, REGS_32)
        };
        for (i, _) in regs.iter().enumerate() {
            if *used & (1 << i) == 0 {
                *used |= 1 << i;
                return self.reg_name(i, wide);
            }
        }
        regs[0]
    }

    pub fn free_all_scratch(&mut self) {
        self.regs_used_32 = 0;
        self.regs_used_64 = 0;
    }

    pub fn is_wide_type(ty: &crate::ir::IrType) -> bool {
        ty.size() > 4
    }

    fn func_local_label(&mut self) -> String {
        let n = self.string_counter;
        self.string_counter += 1;
        format!(".copy_{}", n)
    }

    pub fn format_block_label(&self, id: &str) -> String {
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

    pub(crate) fn line(&mut self, s: &str) {
        self.output.push_str("    ");
        self.output.push_str(s);
        self.output.push('\n');
    }
}

impl Default for AsmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

mod block;
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
}

impl Default for AsmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

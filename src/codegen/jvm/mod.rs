use crate::codegen::traits::OperandLoader;
use crate::codegen::jvm::types::{capitalize_first, get_method_descriptor, ir_type_to_jvm_descriptor};
use crate::ir::types::*;
use ristretto_classfile::attributes::Instruction;
use ristretto_classfile::ConstantPool;
use std::collections::HashMap;

mod types;
mod classfile;
mod instructions;
mod loaders;
mod logical;

#[derive(Debug, Clone)]
enum JumpPlaceholder {
    Goto { block_id: String },
    Ifne { block_id: String },
    Ifeq { block_id: String },
}

#[derive(Debug, Clone)]
enum JvmInst {
    Real(Instruction),
    Placeholder(JumpPlaceholder),
}

pub struct JvmGenerator {
    locals: HashMap<String, u16>,
    next_local_slot: u16,
    constant_pool: ConstantPool<'static>,
    method_refs: HashMap<String, u16>,
    string_consts: HashMap<String, u16>,
    current_function_name: String,
    current_params: Vec<IrParameter>,
    current_return_type: IrType,
}

impl JvmGenerator {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            next_local_slot: 0,
            constant_pool: ConstantPool::default(),
            method_refs: HashMap::new(),
            string_consts: HashMap::new(),
            current_function_name: String::new(),
            current_params: Vec::new(),
            current_return_type: IrType::Void,
        }
    }

    pub fn generate_program(&mut self, program: &IrProgram) -> Vec<(String, Vec<u8>)> {
        let mut classes = Vec::new();

        for func in &program.functions {
            let class_name = if func.name == "main" {
                "Main".to_string()
            } else {
                capitalize_first(&func.name)
            };

            let class_bytes = self.generate_function_class(func, &class_name);
            classes.push((class_name, class_bytes));
        }

        classes
    }

    fn generate_function_class(&mut self, func: &IrFunction, class_name: &str) -> Vec<u8> {
        self.reset_state();
        self.current_function_name = func.name.clone();
        self.current_params = func.parameters.clone();
        self.current_return_type = func.return_type.clone();

        self.setup_local_variables(func);
        self.collect_external_calls(func);
        let code = self.generate_bytecode(func);
        self.build_class_file(class_name, func, code)
    }
    
    fn collect_external_calls(&mut self, func: &IrFunction) {
        let runtime_stub_class = self.constant_pool.add_class("RuntimeStub").unwrap();
        
        for block in &func.blocks {
            for inst in &block.instructions {
                for operand in &inst.operands {
                    if let IrOperand::Constant(crate::ir::Constant::String(s)) = operand {
                        if !self.string_consts.contains_key(s) {
                            if let Ok(idx) = self.constant_pool.add_string(s) {
                                self.string_consts.insert(s.clone(), idx);
                            }
                        }
                    }
                }
                
                if let IrOpcode::Call = inst.opcode {
                    if let Some(ref target) = inst.jump_target {
                        if self.method_refs.contains_key(target) {
                            continue;
                        }
                        
                        let param_types: Vec<IrType> = inst.operands.iter()
                            .map(|op| op.get_type())
                            .collect();
                        let return_type = inst.result_type.clone();
                        
                        let (class_idx, method_name, descriptor) = if self.is_external_function(target) {
                            let desc = get_method_descriptor(target);
                            (runtime_stub_class, target.clone(), desc)
                        } else {
                            let class_name = capitalize_first(target);
                            let user_class = self.constant_pool.add_class(&class_name).unwrap();
                            let desc = self.build_user_method_descriptor(&param_types, return_type.as_ref());
                            (user_class, "call".to_string(), desc)
                        };
                        
                        let method_idx = self.constant_pool
                            .add_method_ref(class_idx, &method_name, &descriptor)
                            .unwrap();
                        
                        self.method_refs.insert(target.clone(), method_idx);
                    }
                }
            }
        }
    }
    
    fn is_external_function(&self, name: &str) -> bool {
        matches!(name, "puts" | "putchar" | "getchar" | "printf" | "rand" | "srand" | "time" | "Sleep"
            | "map_put" | "map_get" | "map_remove" | "map_has" | "map_size" | "map_key" | "map_list"
            | "shm_read_state" | "shm_read_byte" | "shm_read_str" | "shm_write_state" | "shm_write_resp" | "shm_wait_event"
            | "shm_find_null")
    }
    
    fn build_user_method_descriptor(&self, param_types: &[IrType], return_type: Option<&IrType>) -> String {
        let param_desc: String = param_types.iter()
            .map(|t| ir_type_to_jvm_descriptor(t))
            .collect();
        let ret_desc = return_type.as_ref()
            .map(|t| ir_type_to_jvm_descriptor(t))
            .unwrap_or_else(|| "I".to_string());
        format!("({}){}", param_desc, ret_desc)
    }

    fn reset_state(&mut self) {
        self.locals.clear();
        self.next_local_slot = 0;
        self.constant_pool = ConstantPool::default();
        self.method_refs.clear();
        self.string_consts.clear();
    }

    fn setup_local_variables(&mut self, func: &IrFunction) {
        for param in &func.parameters {
            self.locals.insert(param.name.clone(), self.next_local_slot);
            self.next_local_slot += 1;
        }

        for local in &func.locals {
            if !Self::is_temp(&local.name) && !self.locals.contains_key(&local.name) {
                self.locals.insert(local.name.clone(), self.next_local_slot);
                self.next_local_slot += 1;
            }
        }

        let mut temps_used: Vec<String> = Vec::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref result) = inst.result {
                    if Self::is_temp(result) && !temps_used.contains(result) {
                        temps_used.push(result.clone());
                    }
                }
            }
        }

        for temp in temps_used {
            if !self.locals.contains_key(&temp) {
                self.locals.insert(temp, self.next_local_slot);
                self.next_local_slot += 1;
            }
        }
    }

    fn reorder_blocks_for_jvm<'a>(&self, blocks: &'a [IrBlock]) -> Vec<&'a IrBlock> {
        if blocks.is_empty() {
            return Vec::new();
        }
        
        let mut ordered = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let block_map: std::collections::HashMap<String, &IrBlock> = 
            blocks.iter().map(|b| (b.id.clone(), b)).collect();
        
        // Find entry block (first block that is not referenced by others)
        let mut referenced = std::collections::HashSet::new();
        for block in blocks {
            for succ in &block.successors {
                referenced.insert(succ.clone());
            }
        }
        
        // Entry is first block not in referenced, or just first block
        let entry_idx = blocks.iter()
            .position(|b| !referenced.contains(&b.id))
            .unwrap_or(0);
        
        // BFS from entry to get correct order
        let mut queue = std::collections::VecDeque::new();
        queue.push_back(&blocks[entry_idx]);
        
        while let Some(block) = queue.pop_front() {
            if visited.insert(block.id.clone()) {
                ordered.push(block);
                
                // Add successors (body first for loops, then alternative)
                for succ_id in &block.successors {
                    if !visited.contains(succ_id) {
                        if let Some(succ_block) = block_map.get(succ_id) {
                            // Check if already in queue by id
                            let already_in_queue = queue.iter().any(|b| b.id == *succ_id);
                            if !already_in_queue {
                                queue.push_back(succ_block);
                            }
                        }
                    }
                }
            }
        }
        
        // Add any remaining blocks
        for block in blocks {
            if !visited.contains(&block.id) {
                ordered.push(block);
            }
        }
        
        ordered
    }

    fn generate_bytecode(&self, func: &IrFunction) -> Vec<Instruction> {
        let mut instructions: Vec<JvmInst> = Vec::new();
        let mut block_to_inst_idx: HashMap<String, usize> = HashMap::new();

        // Initialize all local slots for verifier type consistency across loop boundaries
        let string_slots = collect_string_slots(self, func);
        for slot in 0..self.next_local_slot {
            if self.locals.values().any(|&s| s == slot) {
                if string_slots.contains(&slot) {
                    instructions.push(JvmInst::Real(Instruction::Aconst_null));
                    instructions.push(JvmInst::Real(Instruction::Astore(slot as u8)));
                } else {
                    instructions.push(JvmInst::Real(Instruction::Iconst_0));
                    instructions.push(JvmInst::Real(Instruction::Istore(slot as u8)));
                }
            }
        }

    // Helper to find all string-typed local slots
    fn collect_string_slots(jg: &JvmGenerator, func: &IrFunction) -> Vec<u16> {
        let mut slots = Vec::new();
        for param in &func.parameters {
            if param.ty == IrType::String {
                if let Some(&slot) = jg.locals.get(&param.name) {
                    slots.push(slot);
                }
            }
        }
        for local in &func.locals {
            if local.ty == IrType::String {
                if let Some(&slot) = jg.locals.get(&local.name) {
                    if !slots.contains(&slot) {
                        slots.push(slot);
                    }
                }
            }
        }
        for block in &func.blocks {
            for inst in &block.instructions {
                for op in &inst.operands {
                    if let IrOperand::Variable(name, ty) = op {
                        if *ty == IrType::String {
                            if let Some(&slot) = jg.locals.get(name) {
                                if !slots.contains(&slot) {
                                    slots.push(slot);
                                }
                            }
                        }
                    }
                }
                if let Some(ref result) = inst.result {
                    if Some(IrType::String) == inst.result_type || inst.operands.first().map_or(false, |op| op.get_type() == IrType::String) {
                        if let Some(&slot) = jg.locals.get(result) {
                            if !slots.contains(&slot) {
                                slots.push(slot);
                            }
                        }
                    }
                }
            }
        }
        slots.sort();
        slots.dedup();
        slots
    }

        // Reorder blocks for correct branch targets
        let ordered_blocks = self.reorder_blocks_for_jvm(&func.blocks);

        // First pass: generate all instructions with placeholders, track block start indices
        let mut inst_idx = instructions.len();
        for block in &ordered_blocks {
            block_to_inst_idx.insert(block.id.clone(), inst_idx);

            for ir_inst in &block.instructions {
                let jvm_insts = self.generate_instruction_with_placeholders(ir_inst, inst_idx as u16);
                inst_idx += jvm_insts.len();
                instructions.extend(jvm_insts);
            }
        }

        // Map block IDs to instruction indices
        let block_inst_indices: HashMap<String, u16> = block_to_inst_idx.iter()
            .map(|(id, &idx)| (id.clone(), idx as u16))
            .collect();

        // Resolve placeholders to instruction-index-based branch instructions
        let result: Vec<Instruction> = instructions.into_iter().map(|jvm_inst| {
            match jvm_inst {
                JvmInst::Real(instr) => instr,
                JvmInst::Placeholder(p) => {
                    let target_block = match &p {
                        JumpPlaceholder::Goto { block_id } => block_id,
                        JumpPlaceholder::Ifne { block_id } => block_id,
                        JumpPlaceholder::Ifeq { block_id } => block_id,
                    };

                    let target_idx = block_inst_indices.get(target_block).copied().unwrap_or(0);
                    match &p {
                        JumpPlaceholder::Goto { .. } => Instruction::Goto(target_idx),
                        JumpPlaceholder::Ifne { .. } => Instruction::Ifne(target_idx),
                        JumpPlaceholder::Ifeq { .. } => Instruction::Ifeq(target_idx),
                    }
                }
            }
        }).collect();

        // Ensure all branch targets are within bounds; add Nop if needed
        let total = result.len() as u16;
        let has_out_of_bounds = block_inst_indices.values().any(|&idx| idx >= total);
        if has_out_of_bounds {
            let mut extended = result;
            extended.push(Instruction::Nop);
            extended
        } else {
            result
        }
    }
    
    fn generate_instruction_with_placeholders(&self, inst: &IrInstruction, global_offset: u16) -> Vec<JvmInst> {
        let mut code: Vec<Instruction> = Vec::new();

        self.generate_instruction(&mut code, inst, global_offset);

        match inst.opcode {
            IrOpcode::Jump => {
                if let Some(ref target) = inst.jump_target {
                    vec![JvmInst::Placeholder(JumpPlaceholder::Goto {
                        block_id: target.clone()
                    })]
                } else {
                    vec![JvmInst::Real(Instruction::Nop)]
                }
            }
            IrOpcode::CondBr => {
                // CondBr uses true_target and false_target, not jump_target
                if let (Some(ref true_target), Some(ref false_target)) = (&inst.true_target, &inst.false_target) {
                    if let Some(operand) = inst.operands.first() {
                        self.emit_load_operand(&mut code, operand);
                    }
                    // Generate: ifeq false_target (jump to false branch if condition is false/0)
                    //           goto true_target (fall through to true branch)
                    code.into_iter().map(JvmInst::Real).chain(
                        vec![
                            JvmInst::Placeholder(JumpPlaceholder::Ifeq {
                                block_id: false_target.clone()
                            }),
                            JvmInst::Placeholder(JumpPlaceholder::Goto {
                                block_id: true_target.clone()
                            })
                        ]
                    ).collect()
                } else if let Some(ref target) = inst.jump_target {
                    // Fallback for legacy IR using jump_target
                    if let Some(operand) = inst.operands.first() {
                        self.emit_load_operand(&mut code, operand);
                    }
                    code.into_iter().map(JvmInst::Real).chain(
                        vec![JvmInst::Placeholder(JumpPlaceholder::Ifne {
                            block_id: target.clone()
                        })]
                    ).collect()
                } else {
                    code.into_iter().map(JvmInst::Real).collect()
                }
            }
            _ => {
                code.into_iter().map(JvmInst::Real).collect()
            }
        }
    }

    pub fn get_local_slot(&self, name: &str) -> u16 {
        *self.locals.get(name).unwrap_or(&0)
    }
}

impl Default for JvmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

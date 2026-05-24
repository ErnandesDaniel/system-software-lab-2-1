use crate::codegen::traits::OperandLoader;
use crate::codegen::jvm::types::{capitalize_first, get_fn_interface_name, get_method_descriptor, ir_type_to_jvm_descriptor};
use crate::ir::types::*;
use ristretto_classfile::attributes::Instruction;
use ristretto_classfile::ConstantPool;
use std::collections::{HashMap, HashSet};

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
    env_vars: HashSet<String>,
    closure_targets: HashMap<String, String>,
    anewarray_int_class_idx: Option<u16>,
    wrapped_vars: HashSet<String>,
    func_ref_targets: HashSet<String>,
    interface_method_refs: HashMap<String, u16>,
    func_ref_init_refs: HashMap<String, (u16, u16)>, // func_name → (class_idx, init_ref_idx)
    func_ref_instance_slots: HashMap<String, u16>,  // func_name → local slot of lambda instance
    func_ref_env_field_refs: HashMap<String, u16>,  // func_name → field ref for __env [[I
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
            env_vars: HashSet::new(),
            closure_targets: HashMap::new(),
            anewarray_int_class_idx: None,
            wrapped_vars: HashSet::new(),
            func_ref_targets: HashSet::new(),
            interface_method_refs: HashMap::new(),
            func_ref_init_refs: HashMap::new(),
            func_ref_instance_slots: HashMap::new(),
            func_ref_env_field_refs: HashMap::new(),
        }
    }

    pub fn generate_program(&mut self, program: &IrProgram) -> Vec<(String, Vec<u8>)> {
        let mut classes = Vec::new();

        // Pre-pass: collect all FuncRef targets across all functions
        self.func_ref_targets.clear();
        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    for op in &inst.operands {
                        if let IrOperand::FuncRef(name) = op {
                            self.func_ref_targets.insert(name.clone());
                        }
                    }
                }
            }
        }

        // Pre-pass: generate all needed functional interface class files
        let mut generated_ifaces = HashSet::new();
        for func in &program.functions {
            if self.func_ref_targets.contains(&func.name) {
                let user_params: Vec<IrType> = func.parameters.iter()
                    .filter(|p| p.name != "__env")
                    .map(|p| p.ty.clone())
                    .collect();
                let iface_name = get_fn_interface_name(&user_params, &func.return_type);
                if generated_ifaces.insert(iface_name.clone()) {
                    let class_data = self.generate_fn_interface(&user_params, &func.return_type);
                    classes.push((iface_name, class_data));
                }
            }
        }

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
        
        // Pre-pass: collect FuncRef targets for new/init refs
        for block in &func.blocks {
            for inst in &block.instructions {
                for op in &inst.operands {
                    if let IrOperand::FuncRef(func_name) = op {
                        if !self.func_ref_init_refs.contains_key(func_name) {
                            let class_name = crate::codegen::jvm::types::capitalize_first(func_name);
                            let class_idx = self.constant_pool.add_class(&class_name).unwrap();
                            let init_ref = self.constant_pool
                                .add_method_ref(class_idx, "<init>", "()V")
                                .unwrap();
                            self.func_ref_init_refs.insert(func_name.clone(), (class_idx, init_ref));
                        }
                    }
                }
            }
        }

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
                
                match inst.opcode {
                    IrOpcode::Call => {
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
                    IrOpcode::CallClosure => {
                        // operands[1] = env_ptr → look up lambda name from closure_targets
                        if let Some(env_operand) = inst.operands.get(1) {
                            if let IrOperand::Variable(env_name, _) = env_operand {
                                if let Some(lambda_name) = self.closure_targets.get(env_name) {
                                    if self.method_refs.contains_key(lambda_name) {
                                        continue;
                                    }
                                    
                                    let class_name = capitalize_first(lambda_name);
                                    let user_class = self.constant_pool.add_class(&class_name).unwrap();
                                    
                                    // Build descriptor: ([[I<arg_types>)<return_type>
                                    let mut param_desc = "[[I".to_string();
                                    for arg in inst.operands.iter().skip(2) {
                                        param_desc.push_str(&ir_type_to_jvm_descriptor(&arg.get_type()));
                                    }
                                    let ret_desc = inst.result_type.as_ref()
                                        .map(|t| ir_type_to_jvm_descriptor(t))
                                        .unwrap_or_else(|| "V".to_string());
                                    let desc = format!("({}){}", param_desc, ret_desc);
                                    
                                    let method_idx = self.constant_pool
                                        .add_method_ref(user_class, "call", &desc)
                                        .unwrap();
                                    
                                    self.method_refs.insert(lambda_name.clone(), method_idx);
                                }
                            }
                        }
                    }
                    IrOpcode::MakeClosure => {
                        // Pre-compute the [I class index for anewarray
                        if self.anewarray_int_class_idx.is_none() {
                            self.anewarray_int_class_idx = Some(self.constant_pool.add_class("[I").unwrap());
                        }
                        // Pre-compute __env field ref for closure lambda classes
                        if let Some(IrOperand::FuncRef(func_name)) = inst.operands.first() {
                            if !self.func_ref_env_field_refs.contains_key(func_name) {
                                let class_name = crate::codegen::jvm::types::capitalize_first(func_name);
                                let class_idx = self.constant_pool.add_class(&class_name).unwrap();
                                let field_ref = self.constant_pool
                                    .add_field_ref(class_idx, "__env", "[[I")
                                    .unwrap();
                                self.func_ref_env_field_refs.insert(func_name.clone(), field_ref);
                            }
                        }
                    }
                    IrOpcode::CallIndirect => {
                        // Register invokeinterface method ref for the functional interface
                        if let Some(func_op) = inst.operands.first() {
                            if let IrType::Function(params, ret) = func_op.get_type() {
                                let iface_name = get_fn_interface_name(&params, &ret);
                                if self.interface_method_refs.contains_key(&iface_name) {
                                    continue;
                                }
                                let iface_class = self.constant_pool.add_class(&iface_name).unwrap();
                                let method_desc = self.build_user_method_descriptor(&params, Some(&ret));
                                let method_idx = self.constant_pool
                                    .add_interface_method_ref(iface_class, "apply", &method_desc)
                                    .unwrap();
                                self.interface_method_refs.insert(iface_name, method_idx);
                            }
                        }
                    }
                    _ => {}
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
        self.env_vars.clear();
        self.closure_targets.clear();
        self.anewarray_int_class_idx = None;
        self.wrapped_vars.clear();
        self.interface_method_refs.clear();
        self.func_ref_init_refs.clear();
        self.func_ref_instance_slots.clear();
        self.func_ref_env_field_refs.clear();
        // func_ref_targets is NOT cleared — it's populated in generate_program pre-pass
    }

    fn setup_local_variables(&mut self, func: &IrFunction) {
        for param in &func.parameters {
            if param.name == "__env" {
                self.env_vars.insert(param.name.clone());
            }
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
        let mut func_ref_instance_temps: HashMap<String, String> = HashMap::new(); // func_name → temp
        for block in &func.blocks {
            for inst in &block.instructions {
                if inst.opcode == IrOpcode::Assign {
                    if let Some(IrOperand::FuncRef(name)) = inst.operands.first() {
                        if let Some(ref result) = inst.result {
                            func_ref_instance_temps.insert(name.clone(), result.clone());
                        }
                    }
                }
                match inst.opcode {
                    IrOpcode::MakeClosure => {
                        if let Some(ref result) = inst.result {
                            if !temps_used.contains(result) {
                                temps_used.push(result.clone());
                            }
                            self.env_vars.insert(result.clone());
                            if let Some(ref target) = inst.jump_target {
                                self.closure_targets.insert(result.clone(), target.clone());
                            }
                        }
                        // Track captured variables as wrapped vars (int[1] wrappers)
                        for op in inst.operands.iter().skip(1) {
                            if let IrOperand::Variable(name, _) = op {
                                self.wrapped_vars.insert(name.clone());
                            }
                        }
                    }
                    _ => {
                        if let Some(ref result) = inst.result {
                            if Self::is_temp(result) && !temps_used.contains(result) {
                                temps_used.push(result.clone());
                            }
                        }
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

        // Populate func_ref_instance_slots from temps
        for (func_name, temp_name) in &func_ref_instance_temps {
            if let Some(&slot) = self.locals.get(temp_name) {
                self.func_ref_instance_slots.insert(func_name.clone(), slot);
            }
        }
    }

    fn reorder_blocks_for_jvm<'a>(&self, blocks: &'a [IrBlock]) -> Vec<&'a IrBlock> {
        if blocks.is_empty() {
            return Vec::new();
        }
        
        let block_map: std::collections::HashMap<String, &IrBlock> = 
            blocks.iter().map(|b| (b.id.clone(), b)).collect();
        
        // Find entry block (first block that is not referenced by others)
        let mut referenced = std::collections::HashSet::new();
        for block in blocks {
            for succ in &block.successors {
                referenced.insert(succ.clone());
            }
        }
        
        let entry_idx = blocks.iter()
            .position(|b| !referenced.contains(&b.id))
            .unwrap_or(0);
        
        // DFS pre-order from entry. This visits body blocks before exit blocks
        // because body is the first successor of header, and exit is the second.
        // Back edges (body → header) are skipped since header is already visited.
        let mut ordered = Vec::new();
        let mut visited = std::collections::HashSet::new();
        let mut stack = vec![&blocks[entry_idx]];
        
        while let Some(block) = stack.pop() {
            if visited.insert(block.id.clone()) {
                ordered.push(block);
                for succ_id in block.successors.iter().rev() {
                    if !visited.contains(succ_id) {
                        if let Some(succ_block) = block_map.get(succ_id) {
                            stack.push(succ_block);
                        }
                    }
                }
            }
        }
        
        // Add any remaining blocks not reachable from entry
        for block in blocks {
            if !visited.contains(&block.id) {
                ordered.push(block);
            }
        }
        
        // Use REVERSE of original block order from generate_function.
        // The original order is: [exit, ...blocks..., entry]
        // but with blocks in stack order (header, then, else, body, ...).
        // We need: [entry, ...reachable..., exit]
        // So: collect reachable in DFS pre-order, ensure entry first, exit last.
        let mut seen = std::collections::HashSet::new();
        let mut result = Vec::new();
        
        // Start DFS from entry, collect in DFS pre-order
        let mut dfs_stack = vec![&blocks[entry_idx]];
        while let Some(b) = dfs_stack.pop() {
            if seen.insert(b.id.clone()) {
                result.push(b);
                for succ_id in b.successors.iter().rev() {
                    if !seen.contains(succ_id) {
                        if let Some(succ) = block_map.get(succ_id) {
                            dfs_stack.push(succ);
                        }
                    }
                }
            }
        }
        
        // Append any remaining blocks (shouldn't happen, but safety)
        for b in blocks {
            if !seen.contains(&b.id) {
                result.push(b);
            }
        }
        

        
        result
    }

    fn generate_bytecode(&self, func: &IrFunction) -> Vec<Instruction> {
        let mut instructions: Vec<JvmInst> = Vec::new();
        let mut block_to_inst_idx: HashMap<String, usize> = HashMap::new();

        // Initialize all local (non-parameter) slots for verifier type consistency
        let string_slots = collect_string_slots(self, func);
        let env_slot_nums: HashSet<u16> = self.env_vars.iter()
            .filter_map(|name| self.locals.get(name))
            .copied()
            .collect();
        let wrapped_slot_nums: HashSet<u16> = self.wrapped_vars.iter()
            .filter_map(|name| self.locals.get(name))
            .copied()
            .collect();
        let fn_slot_nums: HashSet<u16> = func.locals.iter()
            .filter(|l| matches!(l.ty, IrType::Function(_, _)))
            .filter_map(|l| self.locals.get(&l.name))
            .copied()
            .collect();
        let num_params = func.parameters.len() as u16;
        for slot in num_params..self.next_local_slot {
            if self.locals.values().any(|&s| s == slot) {
                if string_slots.contains(&slot) || env_slot_nums.contains(&slot) || fn_slot_nums.contains(&slot) {
                    instructions.push(JvmInst::Real(Instruction::Aconst_null));
                    instructions.push(JvmInst::Real(Instruction::Astore(slot as u8)));
                } else if wrapped_slot_nums.contains(&slot) {
                    instructions.push(JvmInst::Real(Instruction::Iconst_1));
                    instructions.push(JvmInst::Real(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Int)));
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

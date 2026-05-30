use crate::codegen::jvm::types::{
    capitalize_first, get_fn_interface_name, get_method_descriptor, ir_type_to_jvm_descriptor,
};
use crate::codegen::traits::OperandLoader;
use crate::ir::types::{Constant, IrBlock, IrFunction, IrInstruction, IrOpcode, IrOperand, IrParameter, IrProgram, IrType};
use ristretto_classfile::attributes::Instruction;
use ristretto_classfile::ConstantPool;
use std::collections::{HashMap, HashSet};

mod classfile;
mod instructions;
mod loaders;
mod logical;
mod types;

#[derive(Debug, Clone)]
enum JumpPlaceholder {
    Goto { block_id: String },
    Ifne { block_id: String },
    Ifeq { block_id: String },
    IfIcmpeq { block_id: String },
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
    func_ref_instance_slots: HashMap<String, u16>,   // func_name → local slot of lambda instance
    func_ref_env_field_refs: HashMap<String, u16>,   // func_name → field ref for __env [[I
    is_coroutine: bool,
    coroutine_field_refs: HashMap<String, u16>,      // var_name → field ref index
    coroutine_state_field: u16,                      // field ref for 'state'
    coroutine_result_field: u16,                     // field ref for 'result'
    coroutine_field_entries: Vec<(u16, u16)>,       // (name_utf8, desc_utf8) for class file fields
    string_slice_ref: u16,                           // method ref RuntimeStub.string_slice([BII)[B
    struct_field_types: HashMap<String, Vec<(usize, IrType)>>,  // struct_var → [(byte_offset, field_type)]
    struct_uses_object_array: HashSet<String>,        // struct vars needing Object[]
    integer_value_of_ref: u16,                        // method ref Integer.valueOf(I)Ljava/lang/Integer;
    integer_int_value_ref: u16,                       // method ref Integer.intValue()I
    integer_class_idx: u16,                           // class ref java/lang/Integer
    byte_array_class_idx: u16,                        // class ref [B
    object_class_idx: u16,                            // class ref java/lang/Object
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
            is_coroutine: false,
            coroutine_field_refs: HashMap::new(),
            coroutine_state_field: 0,
            coroutine_result_field: 0,
            coroutine_field_entries: Vec::new(),
            string_slice_ref: 0,
            struct_field_types: HashMap::new(),
            struct_uses_object_array: HashSet::new(),
            integer_value_of_ref: 0,
            integer_int_value_ref: 0,
            integer_class_idx: 0,
            byte_array_class_idx: 0,
            object_class_idx: 0,
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
                let user_params: Vec<IrType> = func
                    .parameters
                    .iter()
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

        // Generate RuntimeStub if there are coroutines
        if program.functions.iter().any(|f| f.is_coroutine) {
            let stub_bytes = self.generate_runtime_stub(&program.functions);
            classes.push(("RuntimeStub".to_string(), stub_bytes));
        }

        classes
    }

    fn generate_function_class(&mut self, func: &IrFunction, class_name: &str) -> Vec<u8> {
        self.reset_state();
        self.current_function_name = func.name.clone();
        self.current_params = func.parameters.clone();
        self.current_return_type = func.return_type.clone();

        if func.is_coroutine {
            self.is_coroutine = true;
            self.setup_coroutine_fields(func, class_name);
        }

        self.scan_struct_field_types(func);

        if !self.struct_uses_object_array.is_empty() {
            self.integer_class_idx = self.constant_pool.add_class("java/lang/Integer").unwrap();
            self.byte_array_class_idx = self.constant_pool.add_class("[B").unwrap();
            self.object_class_idx = self.constant_pool.add_class("java/lang/Object").unwrap();
            self.ensure_int_value_ref();
            self.ensure_value_of_ref();
        }

        self.setup_local_variables(func);

        if self.is_coroutine {
            self.next_local_slot = 1; // only slot 0 = this
            self.locals.clear();
        }
        self.collect_external_calls(func);
        let code = self.generate_bytecode(func);
        self.build_class_file(class_name, func, code)
    }

    fn scan_struct_field_types(&mut self, func: &IrFunction) {
        self.struct_field_types.clear();
        self.struct_uses_object_array.clear();
        for block in &func.blocks {
            for inst in &block.instructions {
                let base = inst.operands.first().and_then(|o| {
                    if let IrOperand::Variable(n, ty) = o {
                        if matches!(ty, IrType::Array(_, _)) { Some(n.clone()) } else { None }
                    } else { None }
                });
                let Some(ref base_name) = base else { continue };
                let byte_off = inst.operands.get(1).and_then(|o| {
                    if let IrOperand::Constant(Constant::Int(n)) = o { Some(*n as usize) } else { None }
                }).unwrap_or(0);
                let field_ty = inst.result_type.clone().unwrap_or(IrType::Int);
                let fields = self.struct_field_types.entry(base_name.clone()).or_default();
                if !fields.iter().any(|(o, _)| *o == byte_off) {
                    fields.push((byte_off, field_ty.clone()));
                    if !matches!(field_ty, IrType::Int) {
                        self.struct_uses_object_array.insert(base_name.clone());
                    }
                }
            }
        }
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
                            let init_ref = self.constant_pool.add_method_ref(class_idx, "<init>", "()V").unwrap();
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

                            let param_types: Vec<IrType> = inst
                                .operands
                                .iter()
                                .map(super::super::ir::types::IrOperand::get_type)
                                .collect();
                            let return_type = inst.result_type.clone();

                            let (class_idx, method_name, descriptor) = if Self::is_external_function(target) {
                                let desc = get_method_descriptor(target);
                                (runtime_stub_class, target.clone(), desc)
                            } else {
                                let class_name = capitalize_first(target);
                                let user_class = self.constant_pool.add_class(&class_name).unwrap();
                                let desc = Self::build_user_method_descriptor(&param_types, return_type.as_ref());
                                (user_class, "call".to_string(), desc)
                            };

                            let method_idx = self
                                .constant_pool
                                .add_method_ref(class_idx, &method_name, &descriptor)
                                .unwrap();

                            self.method_refs.insert(target.clone(), method_idx);
                        }
                    }
                    IrOpcode::CallClosure => {
                        // operands[1] = env_ptr → look up lambda name from closure_targets
                        if let Some(IrOperand::Variable(env_name, _)) = inst.operands.get(1) {
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
                                let ret_desc = inst
                                    .result_type
                                    .as_ref()
                                    .map_or_else(|| "V".to_string(), ir_type_to_jvm_descriptor);
                                let desc = format!("({param_desc}){ret_desc}");

                                let method_idx = self.constant_pool.add_method_ref(user_class, "call", &desc).unwrap();

                                self.method_refs.insert(lambda_name.clone(), method_idx);
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
                                let field_ref = self.constant_pool.add_field_ref(class_idx, "__env", "[[I").unwrap();
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
                                let method_desc = Self::build_user_method_descriptor(&params, Some(&ret));
                                let method_idx = self
                                    .constant_pool
                                    .add_interface_method_ref(iface_class, "apply", &method_desc)
                                    .unwrap();
                                self.interface_method_refs.insert(iface_name, method_idx);
                            }
                        }
                    }
                    IrOpcode::Slice => {
                        if self.string_slice_ref == 0 {
                            let stub_class = self.constant_pool.add_class("RuntimeStub").unwrap();
                            self.string_slice_ref = self
                                .constant_pool
                                .add_method_ref(stub_class, "string_slice", "([BII)[B")
                                .unwrap();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn is_external_function(name: &str) -> bool {
        matches!(
            name,
            "puts"
                | "putchar"
                | "getchar"
                | "printf"
                | "rand"
                | "srand"
                | "time"
                | "Sleep"
                | "map_put_jvm"
                | "map_get_jvm"
                | "map_remove_jvm"
                | "map_has_jvm"
                | "map_size_jvm"
                | "map_key_jvm"
                | "map_list_jvm"
                | "shm_read_state_jvm"
                | "shm_read_byte_jvm"
                | "shm_read_str_jvm"
                | "shm_write_state_jvm"
                | "shm_write_resp_jvm"
                | "shm_wait_event_jvm"
                | "shm_find_null_jvm"
        )
    }

    fn build_user_method_descriptor(param_types: &[IrType], return_type: Option<&IrType>) -> String {
        let param_desc: String = param_types.iter().map(ir_type_to_jvm_descriptor).collect();
        let ret_desc = return_type
            .as_ref()
            .map_or_else(|| "I".to_string(), |t| ir_type_to_jvm_descriptor(t));
        format!("({param_desc}){ret_desc}")
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
        self.is_coroutine = false;
        self.coroutine_field_refs.clear();
        self.coroutine_state_field = 0;
        self.coroutine_result_field = 0;
        self.coroutine_field_entries.clear();
        self.string_slice_ref = 0;
        self.struct_field_types.clear();
        self.struct_uses_object_array.clear();
        self.integer_value_of_ref = 0;
        self.integer_int_value_ref = 0;
        self.integer_class_idx = 0;
        self.byte_array_class_idx = 0;
        self.object_class_idx = 0;
        // func_ref_targets is NOT cleared — it's populated in generate_program pre-pass
    }

    fn setup_coroutine_fields(&mut self, func: &IrFunction, class_name: &str) {
        let this_class = self.constant_pool.add_class(class_name).unwrap();

        // Register state field (int)
        let state_name_idx = self.constant_pool.add_utf8("state").unwrap();
        let state_desc_idx = self.constant_pool.add_utf8("I").unwrap();
        let state_name = "state".to_string();
        let state_desc = "I".to_string();
        self.coroutine_state_field = self
            .constant_pool
            .add_field_ref(this_class, &state_name, &state_desc)
            .unwrap();
        self.coroutine_field_entries.push((state_name_idx, state_desc_idx));

        // Register result field (int)
        let result_name_idx = self.constant_pool.add_utf8("result").unwrap();
        let result_desc_idx = self.constant_pool.add_utf8("I").unwrap();
        let result_name = "result".to_string();
        let result_desc = "I".to_string();
        self.coroutine_result_field = self
            .constant_pool
            .add_field_ref(this_class, &result_name, &result_desc)
            .unwrap();
        self.coroutine_field_entries.push((result_name_idx, result_desc_idx));

        // Build a set of unique local variable names to create as fields
        let mut field_names: Vec<String> = Vec::new();
        let mut seen_names: HashSet<String> = HashSet::new();

        // Parameters
        for param in &func.parameters {
            if param.name == "__env" {
                continue;
            }
            if seen_names.insert(param.name.clone()) {
                field_names.push(param.name.clone());
            }
        }
        // Locals
        for local in &func.locals {
            if seen_names.insert(local.name.clone()) {
                field_names.push(local.name.clone());
            }
        }

        for name in &field_names {
            if self.coroutine_field_refs.contains_key(name) {
                continue;
            }
            let field_name_idx = self.constant_pool.add_utf8(&format!("var_{name}")).unwrap();
            let field_desc_idx = self.constant_pool.add_utf8("I").unwrap();
            let field_name = format!("var_{name}");
            let field_desc = "I".to_string();
            let field_ref = self
                .constant_pool
                .add_field_ref(this_class, &field_name, &field_desc)
                .unwrap();
            self.coroutine_field_refs.insert(name.clone(), field_ref);
            self.coroutine_field_entries.push((field_name_idx, field_desc_idx));
        }
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

    fn reorder_blocks_for_jvm<'a>(blocks: &'a [IrBlock]) -> Vec<&'a IrBlock> {
        if blocks.is_empty() {
            return Vec::new();
        }

        let block_map: std::collections::HashMap<String, &IrBlock> = blocks.iter().map(|b| (b.id.clone(), b)).collect();

        // Find entry block (first block that is not referenced by others)
        let mut referenced = std::collections::HashSet::new();
        for block in blocks {
            for succ in &block.successors {
                referenced.insert(succ.clone());
            }
        }

        let entry_idx = blocks.iter().position(|b| !referenced.contains(&b.id)).unwrap_or(0);

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

        // For coroutines: emit state machine dispatch
        if self.is_coroutine {
            instructions.push(JvmInst::Real(Instruction::Aload_0));
            instructions.push(JvmInst::Real(Instruction::Getfield(self.coroutine_state_field)));
            for state_idx in 0..=func.yield_count {
                if let Some(block_id) = func.coroutine_blocks.get(state_idx) {
                    instructions.push(JvmInst::Real(Instruction::Dup));
                    instructions.push(JvmInst::Real(Instruction::Bipush(state_idx as i8)));
                    instructions.push(JvmInst::Placeholder(JumpPlaceholder::IfIcmpeq {
                        block_id: block_id.clone(),
                    }));
                }
            }
            instructions.push(JvmInst::Real(Instruction::Pop));
        }

        // Initialize all local (non-parameter) slots for verifier type consistency
        if !self.is_coroutine {
            let string_slots = collect_string_slots(self, func);
            let env_slot_nums: HashSet<u16> = self
                .env_vars
                .iter()
                .filter_map(|name| self.locals.get(name))
                .copied()
                .collect();
            let wrapped_slot_nums: HashSet<u16> = self
                .wrapped_vars
                .iter()
                .filter_map(|name| self.locals.get(name))
                .copied()
                .collect();
            let fn_slot_nums: HashSet<u16> = func
                .locals
                .iter()
                .filter(|l| matches!(l.ty, IrType::Function(_, _)))
                .filter_map(|l| self.locals.get(&l.name))
                .copied()
                .collect();
            let struct_slot_nums: HashSet<u16> = func
                .locals
                .iter()
                .filter(|l| matches!(&l.ty, IrType::Array(et, _) if **et == IrType::Int))
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
                        instructions.push(JvmInst::Real(Instruction::Newarray(
                            ristretto_classfile::attributes::ArrayType::Int,
                        )));
                        instructions.push(JvmInst::Real(Instruction::Astore(slot as u8)));
                    } else if struct_slot_nums.contains(&slot) {
                        let name = func.locals.iter().find(|l| self.locals.get(&l.name) == Some(&slot)).map(|l| l.name.clone());
                        if name.as_ref().is_some_and(|n| self.struct_uses_object_array.contains(n)) {
                            let arr_size = func.locals.iter()
                                .find(|l| self.locals.get(&l.name) == Some(&slot))
                                .map_or(1, |l| match &l.ty {
                                    IrType::Array(_, size) => *size as u8,
                                    _ => 1,
                                });
                            instructions.push(JvmInst::Real(Instruction::Bipush(arr_size as i8)));
                            instructions.push(JvmInst::Real(Instruction::Anewarray(self.object_class_idx)));
                            instructions.push(JvmInst::Real(Instruction::Astore(slot as u8)));
                        } else {
                            let arr_size = func.locals.iter()
                                .find(|l| self.locals.get(&l.name) == Some(&slot))
                                .map_or(1, |l| match &l.ty {
                                    IrType::Array(_, size) => *size as u8,
                                    _ => 1,
                                });
                            instructions.push(JvmInst::Real(Instruction::Bipush(arr_size as i8)));
                            instructions.push(JvmInst::Real(Instruction::Newarray(
                                ristretto_classfile::attributes::ArrayType::Int,
                            )));
                            instructions.push(JvmInst::Real(Instruction::Astore(slot as u8)));
                        }
                    } else {
                        instructions.push(JvmInst::Real(Instruction::Iconst_0));
                        instructions.push(JvmInst::Real(Instruction::Istore(slot as u8)));
                    }
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
                        if Some(IrType::String) == inst.result_type
                            || inst.operands.first().is_some_and(|op| op.get_type() == IrType::String)
                        {
                            if let Some(&slot) = jg.locals.get(result) {
                                if !slots.contains(&slot) {
                                    slots.push(slot);
                                }
                            }
                        }
                    }
                }
            }
            slots.sort_unstable();
            slots.dedup();
            slots
        }

        // Reorder blocks for correct branch targets
        let ordered_blocks = Self::reorder_blocks_for_jvm(&func.blocks);

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
        let block_inst_indices: HashMap<String, u16> = block_to_inst_idx
            .iter()
            .map(|(id, &idx)| (id.clone(), idx as u16))
            .collect();

        // Resolve placeholders to instruction-index-based branch instructions
        let result: Vec<Instruction> = instructions
            .into_iter()
            .map(|jvm_inst| match jvm_inst {
                JvmInst::Real(instr) => instr,
                JvmInst::Placeholder(p) => {
                    let target_block = match &p {
                        JumpPlaceholder::Goto { block_id }
                        | JumpPlaceholder::Ifne { block_id }
                        | JumpPlaceholder::Ifeq { block_id }
                        | JumpPlaceholder::IfIcmpeq { block_id } => block_id,
                    };

                    let target_idx = block_inst_indices.get(target_block).copied().unwrap_or(0);
                    match &p {
                        JumpPlaceholder::Goto { .. } => Instruction::Goto(target_idx),
                        JumpPlaceholder::Ifne { .. } => Instruction::Ifne(target_idx),
                        JumpPlaceholder::Ifeq { .. } => Instruction::Ifeq(target_idx),
                        JumpPlaceholder::IfIcmpeq { .. } => Instruction::If_icmpeq(target_idx),
                    }
                }
            })
            .collect();

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
                        block_id: target.clone(),
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
                    code.into_iter()
                        .map(JvmInst::Real)
                        .chain(vec![
                            JvmInst::Placeholder(JumpPlaceholder::Ifeq {
                                block_id: false_target.clone(),
                            }),
                            JvmInst::Placeholder(JumpPlaceholder::Goto {
                                block_id: true_target.clone(),
                            }),
                        ])
                        .collect()
                } else if let Some(ref target) = inst.jump_target {
                    // Fallback for legacy IR using jump_target
                    if let Some(operand) = inst.operands.first() {
                        self.emit_load_operand(&mut code, operand);
                    }
                    code.into_iter()
                        .map(JvmInst::Real)
                        .chain(vec![JvmInst::Placeholder(JumpPlaceholder::Ifne {
                            block_id: target.clone(),
                        })])
                        .collect()
                } else {
                    code.into_iter().map(JvmInst::Real).collect()
                }
            }
            _ => code.into_iter().map(JvmInst::Real).collect(),
        }
    }

    pub fn get_local_slot(&self, name: &str) -> u16 {
        *self.locals.get(name).unwrap_or(&0)
    }

    pub fn emit_store_result(&self, code: &mut Vec<Instruction>, name: &str, ty: &IrType) {
        if self.is_coroutine {
            if let Some(&field_ref) = self.coroutine_field_refs.get(name) {
                code.push(Instruction::Aload_0);
                code.push(Instruction::Swap);
                code.push(Instruction::Putfield(field_ref));
            }
        } else {
            let slot = self.get_local_slot(name);
            match ty {
                IrType::String | IrType::Function(_, _) | IrType::Array(..) => {
                    code.push(Instruction::Astore(slot as u8));
                }
                _ => code.push(Instruction::Istore(slot as u8)),
            }
        }
    }

    pub fn is_struct_var(&self, operand: &IrOperand) -> bool {
        if let IrOperand::Variable(name, _) = operand {
            self.struct_uses_object_array.contains(name)
        } else {
            false
        }
    }

    pub fn ensure_int_value_ref(&mut self) -> u16 {
        if self.integer_int_value_ref == 0 {
            let int_class = self.constant_pool.add_class("java/lang/Integer").unwrap();
            self.integer_int_value_ref = self
                .constant_pool
                .add_method_ref(int_class, "intValue", "()I")
                .unwrap();
        }
        self.integer_int_value_ref
    }

    pub fn ensure_value_of_ref(&mut self) -> u16 {
        if self.integer_value_of_ref == 0 {
            let int_class = self.constant_pool.add_class("java/lang/Integer").unwrap();
            self.integer_value_of_ref = self
                .constant_pool
                .add_method_ref(int_class, "valueOf", "(I)Ljava/lang/Integer;")
                .unwrap();
        }
        self.integer_value_of_ref
    }

    pub fn emit_boxed_field_load(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, _byte_off: usize) {
        let field_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
        match field_ty {
            IrType::String => {
                code.push(Instruction::Checkcast(self.byte_array_class_idx));
            }
            _ => {
                code.push(Instruction::Checkcast(self.integer_class_idx));
                code.push(Instruction::Invokevirtual(self.integer_int_value_ref));
            }
        }
    }

    pub fn emit_boxed_field_store(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, _byte_off: usize) {
        let is_string = inst.operands.get(2).is_some_and(|o| matches!(o, IrOperand::Constant(crate::ir::Constant::String(_))));
        if is_string {
            code.push(Instruction::Aastore);
        } else {
            code.push(Instruction::Invokestatic(self.integer_value_of_ref));
            code.push(Instruction::Aastore);
        }
    }
}

impl Default for JvmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

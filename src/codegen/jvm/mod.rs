use crate::ir::types::{
    Constant, IrFunction, IrOpcode, IrOperand, IrParameter, IrProgram, IrType,
};
use ristretto_classfile::attributes::Instruction;
use ristretto_classfile::ConstantPool;
use std::collections::{HashMap, HashSet};

mod build;
mod bytecode;
mod collect;
mod context;
mod coro_build;
mod helpers;
mod instructions;
mod loaders;
mod logical;
mod runtime;
mod stacks;
mod stubs;
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
    func_ref_init_refs: HashMap<String, (u16, u16)>,
    func_ref_instance_slots: HashMap<String, u16>,
    func_ref_env_field_refs: HashMap<String, u16>,
    is_coroutine: bool,
    coroutine_field_refs: HashMap<String, u16>,
    coroutine_state_field: u16,
    coroutine_result_field: u16,
    coroutine_field_entries: Vec<(u16, u16)>,
    coroutine_param_field_refs: Vec<(Option<u16>, Option<u16>)>,
    string_slice_ref: u16,
    struct_field_types: HashMap<String, Vec<(usize, IrType)>>,
    struct_uses_object_array: HashSet<String>,
    integer_value_of_ref: u16,
    integer_int_value_ref: u16,
    integer_class_idx: u16,
    byte_array_class_idx: u16,
    object_class_idx: u16,
    object_array_class_idx: u16,
    global_vars: HashMap<String, IrType>,
    global_uses_object_array: HashSet<String>,
    global_struct_offset_sets: HashMap<String, Vec<usize>>,
    global_field_refs: HashMap<String, u16>,
    runtime_stub_class_ref: u16,
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
            coroutine_param_field_refs: Vec::new(),
            string_slice_ref: 0,
            struct_field_types: HashMap::new(),
            struct_uses_object_array: HashSet::new(),
            integer_value_of_ref: 0,
            integer_int_value_ref: 0,
            integer_class_idx: 0,
            byte_array_class_idx: 0,
            object_class_idx: 0,
            object_array_class_idx: 0,
            global_vars: HashMap::new(),
            global_uses_object_array: HashSet::new(),
            global_struct_offset_sets: HashMap::new(),
            global_field_refs: HashMap::new(),
            runtime_stub_class_ref: 0,
        }
    }

    pub fn generate_program(&mut self, program: &IrProgram) -> Vec<(String, Vec<u8>)> {
        let mut classes = Vec::new();
        self.coroutine_param_field_refs.clear();

        self.global_vars.clear();
        self.global_uses_object_array.clear();
        for g in &program.globals {
            self.global_vars.insert(g.name.clone(), g.ty.clone());
        }

        self.global_struct_offset_sets.clear();
        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    let base = inst.operands.first().and_then(|o| {
                        if let IrOperand::Variable(n, ty) = o {
                            if self.global_vars.contains_key(n) && matches!(ty, IrType::Array(_, _)) {
                                Some(n.clone())
                            } else { None }
                        } else { None }
                    });
                    if let Some(ref base_name) = base {
                        if inst.opcode == IrOpcode::Load {
                            if !matches!(inst.result_type.as_ref().unwrap_or(&IrType::Int), IrType::Int) {
                                self.global_uses_object_array.insert(base_name.clone());
                            }
                        }
                        if inst.opcode == IrOpcode::Store {
                            if let Some(val_op) = inst.operands.get(2) {
                                let vt = val_op.get_type();
                                if !matches!(vt, IrType::Int) {
                                    self.global_uses_object_array.insert(base_name.clone());
                                }
                            }
                        }
                        if let Some(IrOperand::Constant(Constant::Int(byte_off))) = inst.operands.get(1) {
                            let offsets = self.global_struct_offset_sets.entry(base_name.clone()).or_default();
                            if !offsets.contains(&(*byte_off as usize)) {
                                offsets.push(*byte_off as usize);
                                offsets.sort_unstable();
                            }
                        }
                    }
                }
            }
        }

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

        let mut generated_ifaces = HashSet::new();
        for func in &program.functions {
            if self.func_ref_targets.contains(&func.name) {
                let user_params: Vec<IrType> = func
                    .parameters
                    .iter()
                    .filter(|p| p.name != "__env")
                    .map(|p| p.ty.clone())
                    .collect();
                let iface_name = crate::codegen::jvm::types::get_fn_interface_name(&user_params, &func.return_type);
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
                crate::codegen::jvm::types::capitalize_first(&func.name)
            };

            let class_bytes = self.generate_function_class(func, &class_name);
            classes.push((class_name, class_bytes));
        }

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

        if !self.struct_uses_object_array.is_empty() || !self.global_uses_object_array.is_empty() {
            self.integer_class_idx = self.constant_pool.add_class("java/lang/Integer").expect("Failed to add to constant pool");
            self.byte_array_class_idx = self.constant_pool.add_class("[B").expect("Failed to add to constant pool");
            self.object_class_idx = self.constant_pool.add_class("java/lang/Object").expect("Failed to add to constant pool");
            self.object_array_class_idx = self.constant_pool.add_class("[Ljava/lang/Object;").expect("Failed to add to constant pool");
            self.ensure_int_value_ref();
            self.ensure_value_of_ref();
        }

        self.setup_local_variables(func);

        if self.is_coroutine {
            self.next_local_slot = 1;
            self.locals.clear();
        }
        self.collect_external_calls(func);
        let code = self.generate_bytecode(func);
        self.build_class_file(class_name, func, code)
    }
}

impl Default for JvmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

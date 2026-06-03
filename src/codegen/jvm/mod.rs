use crate::ir::types::{
    Constant, IrFunction, IrOperand, IrParameter, IrProgram, IrType,
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

pub struct JvmPoolState {
    pub constant_pool: ConstantPool<'static>,
    pub method_refs: HashMap<String, u16>,
    pub string_consts: HashMap<String, u16>,
    pub interface_method_refs: HashMap<String, u16>,
    pub func_ref_init_refs: HashMap<String, (u16, u16)>,
    pub func_ref_env_field_refs: HashMap<String, u16>,
    pub func_ref_instance_slots: HashMap<String, u16>,
    pub anewarray_int_class_idx: Option<u16>,
    pub string_slice_ref: u16,
    pub integer_value_of_ref: u16,
    pub integer_int_value_ref: u16,
    pub integer_class_idx: u16,
    pub byte_array_class_idx: u16,
    pub object_class_idx: u16,
    pub object_array_class_idx: u16,
    pub runtime_stub_class_ref: u16,
}

pub struct JvmFuncState {
    pub locals: HashMap<String, u16>,
    pub next_local_slot: u16,
    pub current_function_name: String,
    pub current_params: Vec<IrParameter>,
    pub current_return_type: IrType,
}

pub struct JvmClosureState {
    pub env_vars: HashSet<String>,
    pub closure_targets: HashMap<String, String>,
    pub wrapped_vars: HashSet<String>,
    pub func_ref_targets: HashSet<String>,
}

pub struct JvmCoroState {
    pub is_coroutine: bool,
    pub coroutine_field_refs: HashMap<String, u16>,
    pub coroutine_state_field: u16,
    pub coroutine_result_field: u16,
    pub coroutine_field_entries: Vec<(u16, u16)>,
    pub coroutine_param_field_refs: Vec<(Option<u16>, Option<u16>)>,
}

pub struct JvmStructState {
    pub struct_field_types: HashMap<String, Vec<(usize, IrType)>>,
    pub struct_uses_object_array: HashSet<String>,
}

pub struct JvmGlobalState {
    pub global_vars: HashMap<String, IrType>,
    pub global_uses_object_array: HashSet<String>,
    pub global_struct_offset_sets: HashMap<String, Vec<usize>>,
    pub global_field_refs: HashMap<String, u16>,
    pub struct_names: HashSet<String>,
}

pub struct JvmGenerator {
    pub pool: JvmPoolState,
    pub func: JvmFuncState,
    pub closure: JvmClosureState,
    pub coro: JvmCoroState,
    pub st: JvmStructState,
    pub global: JvmGlobalState,
}

impl JvmGenerator {
    pub fn new() -> Self {
        Self {
            pool: JvmPoolState {
                constant_pool: ConstantPool::default(),
                method_refs: HashMap::new(),
                string_consts: HashMap::new(),
                interface_method_refs: HashMap::new(),
                func_ref_init_refs: HashMap::new(),
                func_ref_env_field_refs: HashMap::new(),
                func_ref_instance_slots: HashMap::new(),
                anewarray_int_class_idx: None,
                string_slice_ref: 0,
                integer_value_of_ref: 0,
                integer_int_value_ref: 0,
                integer_class_idx: 0,
                byte_array_class_idx: 0,
                object_class_idx: 0,
                object_array_class_idx: 0,
                runtime_stub_class_ref: 0,
            },
            func: JvmFuncState {
                locals: HashMap::new(),
                next_local_slot: 0,
                current_function_name: String::new(),
                current_params: Vec::new(),
                current_return_type: IrType::Void,
            },
            closure: JvmClosureState {
                env_vars: HashSet::new(),
                closure_targets: HashMap::new(),
                wrapped_vars: HashSet::new(),
                func_ref_targets: HashSet::new(),
            },
            coro: JvmCoroState {
                is_coroutine: false,
                coroutine_field_refs: HashMap::new(),
                coroutine_state_field: 0,
                coroutine_result_field: 0,
                coroutine_field_entries: Vec::new(),
                coroutine_param_field_refs: Vec::new(),
            },
            st: JvmStructState {
                struct_field_types: HashMap::new(),
                struct_uses_object_array: HashSet::new(),
            },
            global: JvmGlobalState {
                global_vars: HashMap::new(),
                global_uses_object_array: HashSet::new(),
                global_struct_offset_sets: HashMap::new(),
                global_field_refs: HashMap::new(),
                struct_names: HashSet::new(),
            },
        }
    }

    pub fn generate_program(&mut self, program: &IrProgram) -> Vec<(String, Vec<u8>)> {
        let mut classes = Vec::new();
        self.coro.coroutine_param_field_refs.clear();

        self.global.global_vars.clear();
        self.global.global_uses_object_array.clear();
        self.global.struct_names.clear();
        for g in &program.globals {
            self.global.global_vars.insert(g.name.clone(), g.ty.clone());
        }
        for (sname, _) in &program.struct_layouts.structs {
            self.global.struct_names.insert(sname.clone());
        }

        self.global.global_struct_offset_sets.clear();
        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    let base = inst.operands.first().and_then(|o| {
                        if let IrOperand::Variable(n, ty) = o {
                            if self.global.global_vars.contains_key(n) && matches!(ty, IrType::Array(_, _)) {
                                Some(n.clone())
                            } else { None }
                        } else { None }
                    });
                    if let Some(ref base_name) = base {
                        let is_struct_offset = inst.operands.get(1).and_then(|o| {
                            if let IrOperand::Constant(Constant::Int(byte_off)) = o {
                                Some(*byte_off > 0)
                            } else { None }
                        }).unwrap_or(false);
                        if is_struct_offset {
                            self.global.global_uses_object_array.insert(base_name.clone());
                        }
                        if let Some(IrOperand::Constant(Constant::Int(byte_off))) = inst.operands.get(1) {
                            let offsets = self.global.global_struct_offset_sets.entry(base_name.clone()).or_default();
                            if !offsets.contains(&(*byte_off as usize)) {
                                offsets.push(*byte_off as usize);
                                offsets.sort_unstable();
                            }
                        }
                    }
                }
            }
        }

        self.closure.func_ref_targets.clear();
        for func in &program.functions {
            for block in &func.blocks {
                for inst in &block.instructions {
                    for op in &inst.operands {
                        if let IrOperand::FuncRef(name) = op {
                            self.closure.func_ref_targets.insert(name.clone());
                        }
                    }
                }
            }
        }

        let mut generated_ifaces = HashSet::new();
        for func in &program.functions {
            if self.closure.func_ref_targets.contains(&func.name) {
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

        let stub_bytes = self.generate_runtime_stub(&program.functions);
        classes.push(("RuntimeStub".to_string(), stub_bytes));

        classes
    }

    fn generate_function_class(&mut self, func: &IrFunction, class_name: &str) -> Vec<u8> {
        self.reset_state();
        self.func.current_function_name = func.name.clone();
        self.func.current_params = func.parameters.clone();
        self.func.current_return_type = func.return_type.clone();

        if func.is_coroutine {
            self.coro.is_coroutine = true;
            self.setup_coroutine_fields(func, class_name);
        }

        self.scan_struct_field_types(func);

        if !self.st.struct_uses_object_array.is_empty() || !self.global.global_uses_object_array.is_empty() {
            self.pool.integer_class_idx = self.pool.constant_pool.add_class("java/lang/Integer").expect("Failed to add to constant pool");
            self.pool.byte_array_class_idx = self.pool.constant_pool.add_class("[B").expect("Failed to add to constant pool");
            self.pool.object_class_idx = self.pool.constant_pool.add_class("java/lang/Object").expect("Failed to add to constant pool");
            self.pool.object_array_class_idx = self.pool.constant_pool.add_class("[Ljava/lang/Object;").expect("Failed to add to constant pool");
            self.ensure_int_value_ref();
            self.ensure_value_of_ref();
        }

        self.setup_local_variables(func);

        if self.coro.is_coroutine {
            self.func.next_local_slot = 1;
            self.func.locals.clear();
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

use crate::ir::types::{IrParameter, IrType};
use ristretto_classfile::attributes::Instruction;
use ristretto_classfile::ConstantPool;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
pub(crate) enum JumpPlaceholder {
    Goto { block_id: String },
    Ifne { block_id: String },
    Ifeq { block_id: String },
    IfIcmpeq { block_id: String },
}

#[derive(Debug, Clone)]
pub(crate) enum JvmInst {
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
    pub nullscan_ref: u16,
    pub large_int_refs: HashMap<i64, u16>,
}

pub struct JvmFuncState {
    pub locals: HashMap<String, u16>,
    pub next_local_slot: u16,
    pub current_function_name: String,
    pub current_params: Vec<IrParameter>,
    pub current_return_type: IrType,
    pub param_type_map: HashMap<String, IrType>,
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
    pub coroutine_field_entries: Vec<(u16, u16, String)>,
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
    pub stub_needed: bool,
}

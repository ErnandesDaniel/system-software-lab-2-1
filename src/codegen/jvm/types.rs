use crate::ir::types::IrType;

pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
}

pub enum ComparisonOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

/// Build a class-name-safe element descriptor from an IrType (no L...; wrapping).
/// Used to construct functional interface names.
fn ir_type_short_descriptor(ty: &IrType) -> &'static str {
    match ty {
        IrType::Void => "V",
        IrType::Bool => "Z",
        IrType::Int => "I",
        IrType::String => "Str",
        IrType::Array(_, _) => "Arr",
        IrType::Function(_, _) => "Fn",
    }
}

/// Deterministic functional interface class name for a function type.
/// Example: `get_fn_interface_name(&[], &IrType::Int)` → "FnV_I"
///          `get_fn_interface_name(&[IrType::Int], &IrType::Int)` → "FnI_I"
pub fn get_fn_interface_name(params: &[IrType], ret: &IrType) -> String {
    let mut name = String::from("Fn");
    for p in params {
        name.push_str(ir_type_short_descriptor(p));
    }
    name.push('_');
    name.push_str(ir_type_short_descriptor(ret));
    name
}

pub fn ir_type_to_jvm_descriptor(ty: &IrType) -> String {
    match ty {
        IrType::Void => "V".to_string(),
        IrType::Bool => "Z".to_string(),
        IrType::Int => "I".to_string(),
        IrType::String => "Ljava/lang/String;".to_string(),
        IrType::Array(elem, _) => format!("[{}]", ir_type_to_jvm_descriptor(elem)),
        IrType::Function(params, ret) => {
            let iface_name = get_fn_interface_name(params, ret);
            format!("L{};", iface_name)
        }
    }
}

pub fn get_method_descriptor(target: &str) -> String {
    match target {
        "puts" => "(Ljava/lang/String;)I".to_string(),
        "putchar" => "(I)I".to_string(),
        "getchar" => "()I".to_string(),
        "printf" => "(Ljava/lang/String;I)I".to_string(),
        "rand" => "()I".to_string(),
        "srand" => "(I)V".to_string(),
        "time" => "(I)I".to_string(),
        "Sleep" => "(I)V".to_string(),
        "map_put_jvm" => "(Ljava/lang/String;Ljava/lang/String;)I".to_string(),
        "map_get_jvm" => "(Ljava/lang/String;)Ljava/lang/String;".to_string(),
        "map_remove_jvm" => "(Ljava/lang/String;)I".to_string(),
        "map_has_jvm" => "(Ljava/lang/String;)I".to_string(),
        "map_size_jvm" => "()I".to_string(),
        "map_key_jvm" => "(I)Ljava/lang/String;".to_string(),
        "map_list_jvm" => "()Ljava/lang/String;".to_string(),
        "shm_read_state_jvm" => "()I".to_string(),
        "shm_read_byte_jvm" => "(I)I".to_string(),
        "shm_read_str_jvm" => "(I)Ljava/lang/String;".to_string(),
        "shm_write_state_jvm" => "(I)V".to_string(),
        "shm_write_resp_jvm" => "(ILjava/lang/String;)V".to_string(),
        "shm_wait_event_jvm" => "()V".to_string(),
        "shm_find_null_jvm" => "(I)I".to_string(),
        _ => "()I".to_string(),
    }
}

pub fn capitalize_first(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => s.to_string(),
    }
}

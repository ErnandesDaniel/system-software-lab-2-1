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

pub fn ir_type_to_jvm_descriptor(ty: &IrType) -> String {
    match ty {
        IrType::Void => "V".to_string(),
        IrType::Bool => "Z".to_string(),
        IrType::Int => "I".to_string(),
        IrType::String => "Ljava/lang/String;".to_string(),
        IrType::Array(elem, _) => format!("[{}]", ir_type_to_jvm_descriptor(elem)),
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

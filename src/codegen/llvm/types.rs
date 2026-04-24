use crate::ir::IrType;
use super::LlvmGenerator;

impl LlvmGenerator {
    pub(crate) fn ir_type_to_llvm(&self, ty: &IrType) -> String {
        match ty {
            IrType::Int | IrType::Bool => "i32".to_string(),
            IrType::Void => "void".to_string(),
            IrType::String => "i8*".to_string(),
            IrType::Array(t, sz) => format!("[{} x {}]", sz, self.ir_type_to_llvm(t)),
        }
    }

    pub(crate) fn block_id_to_label(&self, id: &str) -> String {
        format!("bb_{}", id.trim_start_matches("BB"))
    }

    pub(crate) fn get_extern_signature(&self, name: &str) -> String {
        match name {
            "puts" => "declare i32 @puts(i8*)\n".to_string(),
            "printf" => "declare i32 @printf(i8*, ...)\n".to_string(),
            "srand" => "declare void @srand(i32)\n".to_string(),
            "rand" => "declare i32 @rand()\n".to_string(),
            "time" => "declare i32 @time(i32)\n".to_string(),
            "getchar" => "declare i32 @getchar()\n".to_string(),
            "putchar" => "declare i32 @putchar(i32)\n".to_string(),
            "scanf" => "declare i32 @scanf(i8*, ...)\n".to_string(),
            "Sleep" => "declare void @Sleep(i32)\n".to_string(),
            _ => format!("declare i32 @{}(i32)\n", name),
        }
    }
}

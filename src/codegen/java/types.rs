use crate::ir::*;
use std::collections::HashMap;

#[derive(Clone)]
pub struct JasmFunction {
    pub name: String,
    pub params: Vec<(String, IrType)>,
    pub return_type: IrType,
    pub instructions: Vec<JasmInstruction>,
    pub local_types: HashMap<String, IrType>,
}

#[derive(Clone)]
pub struct JasmInstruction {
    pub opcode: String,
    pub operands: Vec<String>,
}
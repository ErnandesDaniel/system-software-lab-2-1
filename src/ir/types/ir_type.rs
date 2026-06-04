use serde::{Deserialize, Serialize};
use crate::ast::Span;
use crate::struct_layout::LayoutDatabase;
use super::instruction::{IrInstruction, IrOperand, IrOpcode, Constant};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
    pub globals: Vec<IrGlobal>,
    pub struct_layouts: LayoutDatabase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrGlobal {
    pub name: String,
    pub ty: IrType,
    pub initializer: Option<Constant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrFunction {
    pub name: String,
    pub return_type: IrType,
    pub parameters: Vec<IrParameter>,
    pub blocks: Vec<IrBlock>,
    pub locals: Vec<IrLocal>,
    pub used_functions: Vec<String>,
    pub yield_count: usize,
    pub coroutine_blocks: Vec<String>,
    pub is_coroutine: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrParameter {
    pub name: String,
    pub ty: IrType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrLocal {
    pub name: String,
    pub ty: IrType,
    pub stack_offset: Option<i32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum IrType {
    Void,
    Bool,
    Byte,
    Int,
    Uint,
    Long,
    Ulong,
    Char,
    String,
    Array(Box<IrType>, usize),
    Function(Vec<IrType>, Box<IrType>),
    Struct {
        name: String,
        fields: Vec<(String, IrType)>,
        size: usize,
    },
}

impl IrType {
    #[allow(dead_code)]
    #[must_use]
    pub fn size(&self) -> u32 {
        match self {
            IrType::Void => 0,
            IrType::Bool | IrType::Byte | IrType::Char => 4,
            IrType::Int | IrType::Uint => 4,
            IrType::Long | IrType::Ulong => 8,
            IrType::String | IrType::Function(_, _) => 8,
            IrType::Array(elem, size) => elem.size() * *size as u32,
            IrType::Struct { size, .. } => *size as u32,
        }
    }

    #[must_use]
    pub fn is_pointer(&self) -> bool {
        matches!(self, IrType::String | IrType::Function(_, _) | IrType::Array(_, _))
    }

    #[must_use]
    pub fn struct_name(&self) -> Option<&str> {
        match self {
            IrType::Struct { name, .. } => Some(name.as_str()),
            _ => None,
        }
    }

    #[must_use]
    pub fn struct_fields(&self) -> Option<&[(String, IrType)]> {
        match self {
            IrType::Struct { fields, .. } => Some(fields.as_slice()),
            _ => None,
        }
    }

    #[must_use]
    pub fn is_int_like(&self) -> bool {
        matches!(self, IrType::Byte | IrType::Int | IrType::Uint | IrType::Long | IrType::Ulong | IrType::Char)
    }

    #[must_use]
    pub fn is_bool(&self) -> bool {
        matches!(self, IrType::Bool)
    }

    #[must_use]
    pub fn is_byte(&self) -> bool {
        matches!(self, IrType::Byte | IrType::Char)
    }

    #[must_use]
    pub fn is_uint(&self) -> bool {
        matches!(self, IrType::Uint)
    }

    #[must_use]
    pub fn is_long(&self) -> bool {
        matches!(self, IrType::Long | IrType::Ulong)
    }

    pub fn jvm_descriptor(&self) -> String {
        match self {
            IrType::Void => "V".to_string(),
            IrType::Bool => "Z".to_string(),
            IrType::Byte => "B".to_string(),
            IrType::Int | IrType::Uint | IrType::Char => "I".to_string(),
            IrType::Long | IrType::Ulong => "J".to_string(),
            IrType::String => "[B".to_string(),
            IrType::Array(elem, _) => format!("[{}", elem.jvm_descriptor()),
            IrType::Function(_, _) => "Ljava/lang/Object;".to_string(),
            IrType::Struct { .. } => "I".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrBlock {
    pub id: String,
    pub instructions: Vec<IrInstruction>,
    pub successors: Vec<String>,
}

impl IrBlock {
    pub fn new(id: String) -> Self {
        Self { id, instructions: Vec::new(), successors: Vec::new() }
    }

    pub fn inst(&mut self, opcode: IrOpcode) -> IrInstBuilder<'_> {
        IrInstBuilder { block: self, opcode }
    }
}

pub struct IrInstBuilder<'a> {
    block: &'a mut IrBlock,
    opcode: IrOpcode,
}

impl IrInstBuilder<'_> {
    pub fn with(self, result: Option<String>, result_type: Option<IrType>, operands: Vec<IrOperand>, span: Span) {
        self.block.instructions.push(IrInstruction {
            opcode: self.opcode,
            result, result_type, operands,
            jump_target: None, true_target: None, false_target: None, span,
        });
    }

    pub fn jump(self, result: Option<String>, result_type: Option<IrType>, operands: Vec<IrOperand>, target: String, span: Span) {
        self.block.instructions.push(IrInstruction {
            opcode: self.opcode,
            result, result_type, operands,
            jump_target: Some(target), true_target: None, false_target: None, span,
        });
    }

    pub fn cond(self, operands: Vec<IrOperand>, true_target: String, false_target: String, span: Span) {
        self.block.instructions.push(IrInstruction {
            opcode: self.opcode,
            result: None, result_type: None, operands,
            jump_target: None, true_target: Some(true_target), false_target: Some(false_target), span,
        });
    }
}

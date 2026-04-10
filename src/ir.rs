use crate::ast::Span;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrProgram {
    pub functions: Vec<IrFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrFunction {
    pub name: String,
    pub return_type: IrType,
    pub parameters: Vec<IrParameter>,
    pub blocks: Vec<IrBlock>,
    pub locals: Vec<IrLocal>,
    pub used_functions: Vec<String>,
    pub is_thread: bool,
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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(dead_code)]
pub enum IrType {
    Void,
    Bool,
    Int,
    String,
    Array(Box<IrType>, usize),
}

impl IrType {
    #[allow(dead_code)]
    pub fn size(&self) -> u32 {
        match self {
            IrType::Void => 0,
            IrType::Bool | IrType::Int => 4,
            IrType::String => 8,
            IrType::Array(elem, size) => elem.size() * *size as u32,
        }
    }

    pub fn is_pointer(&self) -> bool {
        matches!(self, IrType::String)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrBlock {
    pub id: String,
    pub instructions: Vec<IrInstruction>,
    pub successors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IrInstruction {
    pub opcode: IrOpcode,
    pub result: Option<String>,
    pub result_type: Option<IrType>,
    pub operands: Vec<IrOperand>,
    pub jump_target: Option<String>,
    pub true_target: Option<String>,
    pub false_target: Option<String>,
    pub span: Span,
}

#[allow(dead_code)]
impl IrInstruction {
    pub fn new(opcode: IrOpcode) -> Self {
        Self {
            opcode,
            result: None,
            result_type: None,
            operands: Vec::new(),
            jump_target: None,
            true_target: None,
            false_target: None,
            span: Span::new(0, 0),
        }
    }

    #[allow(dead_code)]
    pub fn with_result(mut self, result: String, ty: IrType) -> Self {
        self.result = Some(result);
        self.result_type = Some(ty);
        self
    }

    #[allow(dead_code)]
    pub fn with_operand(mut self, operand: IrOperand) -> Self {
        self.operands.push(operand);
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IrOpcode {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    And,
    Or,
    Not,
    BitNot,
    BitAnd,
    BitOr,
    Neg,
    Pos,
    Assign,
    Call,
    CreateThread,
    Yield,
    CoroutineCreate,
    CoroutineYield,
    CoroutineResume,
    Jump,
    CondBr,
    Ret,
    Load,
    Store,
    Slice,
    Alloca,
    Cast,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IrOperand {
    Variable(String, IrType),
    Constant(Constant),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constant {
    Int(i64),
    Bool(bool),
    String(String),
    Char(u8),
}

#[allow(dead_code)]
impl IrOperand {
    pub fn get_type(&self) -> IrType {
        match self {
            IrOperand::Variable(_, ty) => ty.clone(),
            IrOperand::Constant(c) => match c {
                Constant::Int(_) => IrType::Int,
                Constant::Bool(_) => IrType::Bool,
                Constant::String(_) => IrType::String,
                Constant::Char(_) => IrType::Int,
            },
        }
    }

    #[allow(dead_code)]
    pub fn int(value: i64) -> Self {
        IrOperand::Constant(Constant::Int(value))
    }

    #[allow(dead_code)]
    pub fn bool(value: bool) -> Self {
        IrOperand::Constant(Constant::Bool(value))
    }

    #[allow(dead_code)]
    pub fn string(value: impl Into<String>) -> Self {
        IrOperand::Constant(Constant::String(value.into()))
    }

    #[allow(dead_code)]
    pub fn variable(name: impl Into<String>, ty: IrType) -> Self {
        IrOperand::Variable(name.into(), ty)
    }
}

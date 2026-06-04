use serde::{Deserialize, Serialize};
use crate::ast::Span;
use super::ir_type::IrType;

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
    #[must_use]
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
    #[must_use]
    pub fn with_result(mut self, result: String, ty: IrType) -> Self {
        self.result = Some(result);
        self.result_type = Some(ty);
        self
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn with_operand(mut self, operand: IrOperand) -> Self {
        self.operands.push(operand);
        self
    }

    pub fn add(result: String, ty: IrType, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Add, result: Some(result), result_type: Some(ty), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn sub(result: String, ty: IrType, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Sub, result: Some(result), result_type: Some(ty), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn mul(result: String, ty: IrType, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Mul, result: Some(result), result_type: Some(ty), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn div(result: String, ty: IrType, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Div, result: Some(result), result_type: Some(ty), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn modulo(result: String, ty: IrType, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Mod, result: Some(result), result_type: Some(ty), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn eq(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Eq, result: Some(result), result_type: Some(IrType::Bool), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn ne(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Ne, result: Some(result), result_type: Some(IrType::Bool), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn lt(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Lt, result: Some(result), result_type: Some(IrType::Bool), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn le(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Le, result: Some(result), result_type: Some(IrType::Bool), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn gt(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Gt, result: Some(result), result_type: Some(IrType::Bool), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn ge(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Ge, result: Some(result), result_type: Some(IrType::Bool), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn and(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::And, result: Some(result), result_type: Some(IrType::Bool), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn or(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Or, result: Some(result), result_type: Some(IrType::Bool), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn unary(opcode: IrOpcode, result: String, ty: IrType, operand: IrOperand, span: Span) -> Self {
        Self { opcode, result: Some(result), result_type: Some(ty), operands: vec![operand], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn assign(target: String, ty: IrType, value: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Assign, result: Some(target), result_type: Some(ty), operands: vec![value], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn load(result: String, ty: IrType, source: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Load, result: Some(result), result_type: Some(ty), operands: vec![source], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn store(base: IrOperand, offset: IrOperand, value: IrOperand, index: Option<IrOperand>, span: Span) -> Self {
        let mut operands = vec![base, offset, value];
        if let Some(idx) = index { operands.push(idx); }
        Self { opcode: IrOpcode::Store, result: None, result_type: None, operands, jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn call(func: String, args: Vec<IrOperand>, result: Option<String>, ty: IrType, span: Span) -> Self {
        Self { opcode: IrOpcode::Call, result, result_type: Some(ty), operands: args, jump_target: Some(func), true_target: None, false_target: None, span }
    }
    pub fn jump(target: String, span: Span) -> Self {
        Self { opcode: IrOpcode::Jump, result: None, result_type: None, operands: vec![], jump_target: Some(target), true_target: None, false_target: None, span }
    }
    pub fn cond_br(cond: IrOperand, true_target: String, false_target: String, span: Span) -> Self {
        Self { opcode: IrOpcode::CondBr, result: None, result_type: None, operands: vec![cond], jump_target: None, true_target: Some(true_target), false_target: Some(false_target), span }
    }
    pub fn ret(operand: Option<IrOperand>, span: Span) -> Self {
        let operands = operand.map_or(vec![], |o| vec![o]);
        Self { opcode: IrOpcode::Ret, result: None, result_type: None, operands, jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn ret_void(span: Span) -> Self {
        Self { opcode: IrOpcode::Ret, result: None, result_type: None, operands: vec![], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn make_closure(func: String, captured_vars: Vec<IrOperand>, result: String, span: Span) -> Self {
        let mut operands = vec![IrOperand::FuncRef(func.clone())];
        operands.extend(captured_vars);
        Self { opcode: IrOpcode::MakeClosure, result: Some(result), result_type: Some(IrType::Int), operands, jump_target: Some(func), true_target: None, false_target: None, span }
    }
    pub fn call_closure(closure: IrOperand, env: IrOperand, args: Vec<IrOperand>, result: Option<String>, ty: IrType, span: Span) -> Self {
        let mut operands = vec![closure, env];
        operands.extend(args);
        Self { opcode: IrOpcode::CallClosure, result, result_type: Some(ty), operands, jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn call_indirect(target: IrOperand, args: Vec<IrOperand>, result: Option<String>, ty: IrType, span: Span) -> Self {
        let mut operands = vec![target];
        operands.extend(args);
        Self { opcode: IrOpcode::CallIndirect, result, result_type: Some(ty), operands, jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn load_captured(result: String, env: IrOperand, slot: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::LoadCaptured, result: Some(result), result_type: Some(IrType::Int), operands: vec![env, slot], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn store_captured(env: IrOperand, slot: IrOperand, value: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::StoreCaptured, result: None, result_type: None, operands: vec![env, slot, value], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn str_get_byte(result: String, string: IrOperand, index: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::StrGetByte, result: Some(result), result_type: Some(IrType::Int), operands: vec![string, index], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn str_set_byte(string: IrOperand, index: IrOperand, value: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::StrSetByte, result: None, result_type: None, operands: vec![string, index, value], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn alloc_array(result: String, ty: IrType, elements: Vec<IrOperand>, span: Span) -> Self {
        let mut operands = vec![];
        operands.extend(elements);
        Self { opcode: IrOpcode::AllocArray, result: Some(result), result_type: Some(ty), operands, jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn bit_and(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::BitAnd, result: Some(result), result_type: Some(IrType::Int), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn bit_or(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::BitOr, result: Some(result), result_type: Some(IrType::Int), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn bit_xor(result: String, left: IrOperand, right: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::BitXor, result: Some(result), result_type: Some(IrType::Int), operands: vec![left, right], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn slice(result: String, ty: IrType, base: IrOperand, start: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::Slice, result: Some(result), result_type: Some(ty), operands: vec![base, start], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn coro_yield(state: i64, span: Span) -> Self {
        Self { opcode: IrOpcode::CoroYield, result: None, result_type: Some(IrType::Int), operands: vec![IrOperand::Constant(Constant::Int(state))], jump_target: None, true_target: None, false_target: None, span }
    }
    pub fn bit_not(result: String, operand: IrOperand, span: Span) -> Self {
        Self { opcode: IrOpcode::BitNot, result: Some(result), result_type: Some(IrType::Int), operands: vec![operand], jump_target: None, true_target: None, false_target: None, span }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum IrOpcode {
    Add, Sub, Mul, Div, Mod,
    Eq, Ne, Lt, Le, Gt, Ge,
    And, Or, Not, BitNot, BitAnd, BitOr, BitXor,
    StrGetByte, StrSetByte, Neg,
    Assign, Call, Jump, CondBr, Ret, Load, Store, Slice, CoroYield,
    CallIndirect, MakeClosure, CallClosure, LoadCaptured, StoreCaptured, AllocArray,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IrOperand {
    Variable(String, IrType),
    Constant(Constant),
    FuncRef(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Constant {
    Int(i64),
    Bool(bool),
    String(String),
    Char(u8),
    Array(Vec<Constant>),
}

impl Constant {
    #[must_use]
    pub fn get_type(&self) -> IrType {
        match self {
            Constant::Int(_) => IrType::Int,
            Constant::Bool(_) => IrType::Bool,
            Constant::String(_) => IrType::String,
            Constant::Char(_) => IrType::Char,
            Constant::Array(elems) => {
                let elem_ty = elems.first().map_or(IrType::Int, |e| e.get_type());
                IrType::Array(Box::new(elem_ty), elems.len())
            }
        }
    }
}

#[allow(dead_code)]
impl IrOperand {
    #[must_use]
    pub fn get_type(&self) -> IrType {
        match self {
            IrOperand::Variable(_, ty) => ty.clone(),
            IrOperand::Constant(c) => match c {
                Constant::Bool(_) => IrType::Bool,
                Constant::String(_) => IrType::String,
                Constant::Int(_) => IrType::Int,
                Constant::Char(_) => IrType::Char,
                Constant::Array(elems) => {
                    let elem_ty = elems.first().map_or(IrType::Int, |e| e.get_type());
                    IrType::Array(Box::new(elem_ty), elems.len())
                }
            },
            IrOperand::FuncRef(_) => IrType::Function(Vec::new(), Box::new(IrType::Int)),
        }
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn int(value: i64) -> Self {
        IrOperand::Constant(Constant::Int(value))
    }

    #[allow(dead_code)]
    #[must_use]
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

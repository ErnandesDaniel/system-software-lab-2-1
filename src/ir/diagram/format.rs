use crate::ir::*;

pub fn format_instruction(inst: &IrInstruction) -> String {
    match &inst.opcode {
        IrOpcode::Add => format_binary(inst, "+"),
        IrOpcode::Sub => format_binary(inst, "-"),
        IrOpcode::Mul => format_binary(inst, "*"),
        IrOpcode::Div => format_binary(inst, "/"),
        IrOpcode::Mod => format_binary(inst, "%"),
        IrOpcode::Eq => format_cmp_result(inst, "=="),
        IrOpcode::Ne => format_cmp_result(inst, "!="),
        IrOpcode::Lt => format_cmp_result(inst, "<"),
        IrOpcode::Gt => format_cmp_result(inst, ">"),
        IrOpcode::Le => format_cmp_result(inst, "<="),
        IrOpcode::Ge => format_cmp_result(inst, ">="),
        IrOpcode::And => format_binary(inst, "&&"),
        IrOpcode::Or => format_binary(inst, "||"),
        IrOpcode::Not => format_unary(inst, "!"),
        IrOpcode::Neg => format_unary(inst, "-"),
        IrOpcode::Pos => format_unary(inst, "+"),
        IrOpcode::BitNot => format_unary(inst, "~"),
        IrOpcode::BitAnd => format_binary(inst, "&"),
        IrOpcode::BitOr => format_binary(inst, "|"),
        IrOpcode::BitXor => format_binary(inst, "^"),
        IrOpcode::Assign => format_assign(inst),
        IrOpcode::Call => format_call(inst),
        IrOpcode::Jump => format_jump(inst),
        IrOpcode::CondBr => format_cond_br(inst),
        IrOpcode::CoroYield => "yield".to_string(),
        IrOpcode::CallIndirect => format_call_indirect(inst),
        IrOpcode::Ret => format_ret(inst),
        IrOpcode::Load => format_load(inst),
        IrOpcode::Store => format_store(inst),
        IrOpcode::Slice => format_slice(inst),
        IrOpcode::Alloca => format_alloca(inst),
        IrOpcode::Cast => format_cast(inst),
        IrOpcode::StrGetByte => format_str_get_byte(inst),
        IrOpcode::StrSetByte => format_str_set_byte(inst),
        IrOpcode::MakeClosure => format_make_closure(inst),
        IrOpcode::CallClosure => format_call_closure(inst),
        IrOpcode::LoadCaptured => format_load_captured(inst),
        IrOpcode::StoreCaptured => "store_captured".to_string(),
        IrOpcode::AllocArray => format_alloc_array(inst),
    }
}

fn result_name(inst: &IrInstruction) -> &str {
    inst.result.as_deref().unwrap_or("?")
}

fn format_binary(inst: &IrInstruction, op: &str) -> String {
    let (a, b) = get_operands(inst);
    format!("{} = {} {} {}", result_name(inst), a, op, b)
}

fn format_unary(inst: &IrInstruction, op: &str) -> String {
    let a = get_single_operand(inst);
    format!("{} = {}{}", result_name(inst), op, a)
}

fn format_cmp_result(inst: &IrInstruction, op: &str) -> String {
    let (a, b) = get_operands(inst);
    format!("{} = ({} {} {})", result_name(inst), a, op, b)
}

fn format_assign(inst: &IrInstruction) -> String {
    if let Some(ref result) = inst.result {
        if let Some(op) = inst.operands.first() {
            let a = format_operand(op);
            if a == *result {
                return a;
            }
            return format!("{result} = {a}");
        }
    }
    "=".to_string()
}

fn format_call(inst: &IrInstruction) -> String {
    let target = inst.jump_target.as_deref().unwrap_or("?");
    let args = inst
        .operands
        .iter()
        .map(format_operand)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{} = {}({})", result_name(inst), target, args)
}

fn format_jump(inst: &IrInstruction) -> String {
    let target = inst.jump_target.as_deref().unwrap_or("?");
    format!("goto {}", format_block_id(target))
}

fn format_cond_br(inst: &IrInstruction) -> String {
    let cond = get_single_operand(inst);
    format!(
        "if {} goto {} else {}",
        cond,
        format_block_id(inst.true_target.as_deref().unwrap_or("?")),
        format_block_id(inst.false_target.as_deref().unwrap_or("?"))
    )
}

fn format_call_indirect(inst: &IrInstruction) -> String {
    let target = get_single_operand(inst);
    let args = inst
        .operands
        .iter()
        .skip(1)
        .map(format_operand)
        .collect::<Vec<_>>()
        .join(", ");
    format!("{} = {}({}) (indirect)", result_name(inst), target, args)
}

fn format_ret(inst: &IrInstruction) -> String {
    if inst.operands.is_empty() {
        "return".to_string()
    } else {
        let a = get_single_operand(inst);
        format!("return {a}")
    }
}

fn format_load(inst: &IrInstruction) -> String {
    let (arr, idx) = get_operands(inst);
    format!("{} = {}[{}]", result_name(inst), arr, idx)
}

fn format_store(inst: &IrInstruction) -> String {
    let (arr, idx) = get_operands(inst);
    format!("{}[{}] = {}", arr, idx, result_name(inst))
}

fn format_slice(inst: &IrInstruction) -> String {
    let (arr, start) = get_operands(inst);
    format!("{} = {}[{}:]", result_name(inst), arr, start)
}

fn format_alloca(inst: &IrInstruction) -> String {
    format!("alloca {}", result_name(inst))
}

fn format_cast(inst: &IrInstruction) -> String {
    let a = get_single_operand(inst);
    format!("{} = cast({})", result_name(inst), a)
}

fn format_str_get_byte(inst: &IrInstruction) -> String {
    let (str_name, idx_name) = get_operands(inst);
    format!("{} = {}[{}] (str)", result_name(inst), str_name, idx_name)
}

fn format_str_set_byte(inst: &IrInstruction) -> String {
    let (str_name, idx_name) = get_operands(inst);
    format!(
        "{}[{}] = {} (str)",
        str_name,
        idx_name,
        inst.operands
            .get(2)
            .map(format_operand)
            .unwrap_or_else(|| "?".to_string())
    )
}

fn format_make_closure(inst: &IrInstruction) -> String {
    let target = get_single_operand(inst);
    format!("{} = make_closure({})", result_name(inst), target)
}

fn format_call_closure(inst: &IrInstruction) -> String {
    let target = get_single_operand(inst);
    let args: Vec<String> = inst
        .operands
        .iter()
        .skip(2)
        .map(format_operand)
        .collect();
    format!("{} = {}({}) (closure)", result_name(inst), target, args.join(", "))
}

fn format_load_captured(inst: &IrInstruction) -> String {
    format!("load_captured {}", result_name(inst))
}

fn format_alloc_array(inst: &IrInstruction) -> String {
    format!("alloc_array {}", result_name(inst))
}

pub(crate) fn format_block_id(id: &str) -> String {
    id.replace("BB", "BB_")
}

fn get_operands(inst: &IrInstruction) -> (String, String) {
    let a = inst
        .operands
        .first()
        .map(format_operand)
        .unwrap_or_else(|| "?".to_string());
    let b = inst
        .operands
        .get(1)
        .map(format_operand)
        .unwrap_or_else(|| "?".to_string());
    (a, b)
}

fn get_single_operand(inst: &IrInstruction) -> String {
    inst.operands
        .first()
        .map(format_operand)
        .unwrap_or_else(|| "?".to_string())
}

pub fn format_operand(op: &IrOperand) -> String {
    match op {
        IrOperand::Variable(name, _) | IrOperand::FuncRef(name) => name.clone(),
        IrOperand::Constant(c) => match c {
            Constant::Int(n) => n.to_string(),
            Constant::Bool(b) => b.to_string(),
            Constant::String(s) => s
                .replace('\\', "\\\\")
                .replace('\n', "\\n")
                .replace('\r', "\\r")
                .replace('\t', "\\t"),
            Constant::Char(c) => format!("'{}'", *c as char),
            Constant::Array(a) => format!("[{}]", a.len()),
        },
    }
}

pub fn format_block_label(id: &str) -> String {
    if id.starts_with("BB") {
        format!("BB_{}", id.trim_start_matches("BB"))
    } else {
        id.to_string()
    }
}

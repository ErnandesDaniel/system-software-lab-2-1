use crate::ir::*;

use super::generator::JasmGenerator;
use super::types::JasmInstruction;

pub fn translate_instruction(gen: &mut JasmGenerator, inst: &IrInstruction, func: &IrFunction, instructions: &mut Vec<JasmInstruction>) {
    match inst.opcode {
        IrOpcode::Assign => {
            if let Some(result) = &inst.result {
                if let Some(operand) = inst.operands.first() {
                    let is_void = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::Void));
                    if is_void { return; }
                    if let IrOperand::Variable(_, ty) = operand {
                        if matches!(ty, IrType::Void) { return; }
                    }
                    let local_idx = gen.get_local_index(result);
                    gen.push_value(operand, instructions);
                    if let Some(idx) = local_idx {
                        let is_string = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::String));
                        let store_opcode = if is_string { "astore" } else { "istore" };
                        instructions.push(JasmInstruction { opcode: store_opcode.to_string(), operands: vec![idx.to_string()] });
                    }
                }
            }
        }
        IrOpcode::Add | IrOpcode::Sub | IrOpcode::Mul | IrOpcode::Div | IrOpcode::Mod | IrOpcode::And | IrOpcode::Or => {
            if let Some(result) = &inst.result {
                if let (Some(left), Some(right)) = (inst.operands.get(0), inst.operands.get(1)) {
                    let local_idx = gen.get_local_index(result);
                    gen.push_value(left, instructions);
                    gen.push_value(right, instructions);
                    let op = match inst.opcode {
                        IrOpcode::Add => "iadd", IrOpcode::Sub => "isub", IrOpcode::Mul => "imul",
                        IrOpcode::Div => "idiv", IrOpcode::Mod => "irem", IrOpcode::And => "iand", IrOpcode::Or => "ior", _ => "iadd",
                    };
                    instructions.push(JasmInstruction { opcode: op.to_string(), operands: vec![] });
                    if let Some(idx) = local_idx {
                        instructions.push(JasmInstruction { opcode: "istore".to_string(), operands: vec![idx.to_string()] });
                    }
                }
            }
        }
        IrOpcode::Neg => {
            if let Some(result) = &inst.result {
                if let Some(operand) = inst.operands.first() {
                    let local_idx = gen.get_local_index(result);
                    gen.push_value(operand, instructions);
                    instructions.push(JasmInstruction { opcode: "ineg".to_string(), operands: vec![] });
                    if let Some(idx) = local_idx {
                        instructions.push(JasmInstruction { opcode: "istore".to_string(), operands: vec![idx.to_string()] });
                    }
                }
            }
        }
        IrOpcode::Ret => {
            let returns_void = matches!(func.return_type, IrType::Void);
            let is_main = func.name == "main";
            if returns_void || is_main {
                instructions.push(JasmInstruction { opcode: "return".to_string(), operands: vec![] });
            } else {
                if let Some(operand) = inst.operands.first() {
                    gen.push_value(operand, instructions);
                }
                instructions.push(JasmInstruction { opcode: "ireturn".to_string(), operands: vec![] });
            }
        }
        IrOpcode::Jump => {
            if let Some(ref target) = inst.jump_target {
                instructions.push(JasmInstruction { opcode: "goto".to_string(), operands: vec![target.clone()] });
            }
        }
        IrOpcode::CondBr => {
            if let Some(operand) = inst.operands.first() {
                gen.push_value(operand, instructions);
                if let Some(ref true_t) = inst.true_target {
                    if let Some(ref false_t) = inst.false_target {
                        instructions.push(JasmInstruction { opcode: "ifne".to_string(), operands: vec![true_t.clone()] });
                        instructions.push(JasmInstruction { opcode: "goto".to_string(), operands: vec![false_t.clone()] });
                    }
                }
            }
        }
        IrOpcode::Eq | IrOpcode::Ne | IrOpcode::Lt | IrOpcode::Le | IrOpcode::Gt | IrOpcode::Ge => {
            if let Some(result) = &inst.result {
                if let (Some(left), Some(right)) = (inst.operands.get(0), inst.operands.get(1)) {
                    let local_idx = gen.get_local_index(result);
                    gen.push_value(left, instructions);
                    gen.push_value(right, instructions);
                    let jop = match inst.opcode {
                        IrOpcode::Eq => "if_icmpeq", IrOpcode::Ne => "if_icmpne", IrOpcode::Lt => "if_icmplt",
                        IrOpcode::Le => "if_icmple", IrOpcode::Gt => "if_icmpgt", IrOpcode::Ge => "if_icmpge", _ => "if_icmpeq",
                    };
                    let true_label = format!("{}_true", result);
                    let false_label = format!("{}_false", result);
                    instructions.push(JasmInstruction { opcode: jop.to_string(), operands: vec![true_label.clone()] });
                    instructions.push(JasmInstruction { opcode: "bipush".to_string(), operands: vec!["0".to_string()] });
                    instructions.push(JasmInstruction { opcode: "goto".to_string(), operands: vec![false_label.clone()] });
                    instructions.push(JasmInstruction { opcode: "label".to_string(), operands: vec![true_label] });
                    instructions.push(JasmInstruction { opcode: "bipush".to_string(), operands: vec!["1".to_string()] });
                    instructions.push(JasmInstruction { opcode: "label".to_string(), operands: vec![false_label] });
                    if let Some(idx) = local_idx {
                        instructions.push(JasmInstruction { opcode: "istore".to_string(), operands: vec![idx.to_string()] });
                    }
                }
            }
        }
        IrOpcode::Not => {
            if let Some(result) = &inst.result {
                if let Some(operand) = inst.operands.first() {
                    let local_idx = gen.get_local_index(result);
                    gen.push_value(operand, instructions);
                    instructions.push(JasmInstruction { opcode: "bipush".to_string(), operands: vec!["1".to_string()] });
                    instructions.push(JasmInstruction { opcode: "ixor".to_string(), operands: vec![] });
                    if let Some(idx) = local_idx {
                        instructions.push(JasmInstruction { opcode: "istore".to_string(), operands: vec![idx.to_string()] });
                    }
                }
            }
        }
        IrOpcode::Call => {
            super::calls::translate_call(gen, inst, instructions);
        }
        _ => {}
    }
}
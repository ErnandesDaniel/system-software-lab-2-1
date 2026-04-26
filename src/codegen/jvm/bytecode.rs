use ristretto_classfile::attributes::Instruction;
use crate::ir::types::*;
use std::collections::HashMap;

pub fn resolve_bytecode(
    func: &IrFunction,
    locals: &HashMap<String, u16>,
    method_refs: &HashMap<String, u16>,
    string_consts: &HashMap<String, u16>,
) -> Vec<Instruction> {
    use ristretto_classfile::attributes::Instruction as Ri;
    
    // First pass: collect all instructions and compute block positions
    let mut all_instructions: Vec<(String, Ri)> = Vec::new();  // (block_id, instruction)
    let mut block_positions: HashMap<String, usize> = HashMap::new();
    let mut current_pos = 0usize;
    
    for block in &func.blocks {
        block_positions.insert(block.id.clone(), current_pos);
        
        for inst in &block.instructions {
            let insts = generate_inst(inst, locals, method_refs, string_consts);
            for i in insts {
                current_pos += instr_size(&i);
                all_instructions.push((block.id.clone(), i));
            }
        }
    }
    
    // Second pass: resolve labels
    let mut result = Vec::new();
    current_pos = 0;
    
    for (block_id, instr) in all_instructions {
        let resolved = match instr {
            // Branch to block - resolve offset
            Ri::Goto(_) => {
                // Find target block from the original instruction
                // This is tricky - we need to know the target
                // For now, just use placeholder
                instr
            }
            _ => instr,
        };
        
        result.push(resolved);
        current_pos += instr_size(&instr);
    }
    
    result
}

fn instr_size(instr: &Instruction) -> usize {
    use ristretto_classfile::attributes::Instruction as Ri;
    match instr {
        Ri::Iconst_m1 | Ri::Iconst_0 | Ri::Iconst_1 | Ri::Iconst_2 |
        Ri::Iconst_3 | Ri::Iconst_4 | Ri::Iconst_5 | Ri::Iadd | Ri::Isub |
        Ri::Imul | Ri::Idiv | Ri::Irem | Ri::Ineg | Ri::Iand | Ri::Ior |
        Ri::Ixor | Ri::Iaload | Ri::Ireturn | Ri::Return => 1,
        Ri::Bipush(_) => 2,
        Ri::Sipush(_) => 3,
        Ri::Ldc(_) => 2,
        Ri::Ldc_w(_) => 3,
        Ri::Iload_0 | Ri::Iload_1 | Ri::Iload_2 | Ri::Iload_3 |
        Ri::Aload_0 | Ri::Aload_1 | Ri::Aload_2 | Ri::Aload_3 |
        Ri::Istore_0 | Ri::Istore_1 | Ri::Istore_2 | Ri::Istore_3 |
        Ri::Astore_0 | Ri::Astore_1 | Ri::Astore_2 | Ri::Astore_3 => 1,
        Ri::Iload(_) | Ri::Aload(_) | Ri::Istore(_) | Ri::Astore(_) => 2,
        Ri::Ifeq(_) | Ri::Ifne(_) | Ri::If_icmpeq(_) | Ri::If_icmpne(_) |
        Ri::If_icmplt(_) | Ri::If_icmple(_) | Ri::If_icmpgt(_) | Ri::If_icmpge(_) |
        Ri::Goto(_) | Ri::Invokestatic(_) => 3,
        _ => 1,
    }
}

fn generate_inst(
    inst: &IrInstruction,
    locals: &HashMap<String, u16>,
    method_refs: &HashMap<String, u16>,
    string_consts: &HashMap<String, u16>,
) -> Vec<Instruction> {
    use ristretto_classfile::attributes::Instruction as Ri;
    let mut code = Vec::new();
    
    match inst.opcode {
        IrOpcode::Assign => {
            if let (Some(ref result), Some(ref operand)) = (&inst.result, inst.operands.first()) {
                emit_load(&mut code, operand, locals, string_consts);
                let slot = *locals.get(result).unwrap_or(&0);
                code.push(match operand.get_type() {
                    IrType::String => Ri::Astore(slot as u8),
                    _ => Ri::Istore(slot as u8),
                });
            }
        }
        IrOpcode::Add => emit_binop(&mut code, inst, locals, Ri::Iadd),
        IrOpcode::Sub => emit_binop(&mut code, inst, locals, Ri::Isub),
        IrOpcode::Mul => emit_binop(&mut code, inst, locals, Ri::Imul),
        IrOpcode::Div => emit_binop(&mut code, inst, locals, Ri::Idiv),
        IrOpcode::Mod => emit_binop(&mut code, inst, locals, Ri::Irem),
        IrOpcode::BitAnd => emit_binop(&mut code, inst, locals, Ri::Iand),
        IrOpcode::BitOr => emit_binop(&mut code, inst, locals, Ri::Ior),
        IrOpcode::Neg => {
            if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
                emit_load(&mut code, operand, locals, string_consts);
                code.push(Ri::Ineg);
                code.push(Ri::Istore(*locals.get(result).unwrap_or(&0) as u8));
            }
        }
        IrOpcode::Pos => {
            if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
                emit_load(&mut code, operand, locals, string_consts);
                code.push(Ri::Istore(*locals.get(result).unwrap_or(&0) as u8));
            }
        }
        IrOpcode::BitNot => {
            if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
                emit_load(&mut code, operand, locals, string_consts);
                code.push(Ri::Iconst_m1);
                code.push(Ri::Ixor);
                code.push(Ri::Istore(*locals.get(result).unwrap_or(&0) as u8));
            }
        }
        IrOpcode::And => {
            // Bitwise and
            if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
                emit_load(&mut code, left, locals, string_consts);
                emit_load(&mut code, right, locals, string_consts);
                code.push(Ri::Iand);
                code.push(Ri::Istore(*locals.get(result).unwrap_or(&0) as u8));
            }
        }
        IrOpcode::Or => {
            // Bitwise or
            if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
                emit_load(&mut code, left, locals, string_consts);
                emit_load(&mut code, right, locals, string_consts);
                code.push(Ri::Ior);
                code.push(Ri::Istore(*locals.get(result).unwrap_or(&0) as u8));
            }
        }
        IrOpcode::Not => {
            if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
                emit_load(&mut code, operand, locals, string_consts);
                code.push(Ri::Iconst_1);
                code.push(Ri::Ixor);
                code.push(Ri::Istore(*locals.get(result).unwrap_or(&0) as u8));
            }
        }
        IrOpcode::Eq => emit_cmp(&mut code, inst, locals, string_consts, CmpType::Eq),
        IrOpcode::Ne => emit_cmp(&mut code, inst, locals, string_consts, CmpType::Ne),
        IrOpcode::Lt => emit_cmp(&mut code, inst, locals, string_consts, CmpType::Lt),
        IrOpcode::Le => emit_cmp(&mut code, inst, locals, string_consts, CmpType::Le),
        IrOpcode::Gt => emit_cmp(&mut code, inst, locals, string_consts, CmpType::Gt),
        IrOpcode::Ge => emit_cmp(&mut code, inst, locals, string_consts, CmpType::Ge),
        IrOpcode::Call => {
            if let Some(ref target) = inst.jump_target {
                for operand in &inst.operands {
                    emit_load(&mut code, operand, locals, string_consts);
                }
                let method_idx = method_refs.get(target).copied().unwrap_or(1);
                code.push(Ri::Invokestatic(method_idx));
                if let Some(ref result) = inst.result {
                    code.push(Ri::Istore(*locals.get(result).unwrap_or(&0) as u8));
                }
            }
        }
        IrOpcode::Ret => {
            if let Some(operand) = inst.operands.first() {
                emit_load(&mut code, operand, locals, string_consts);
                code.push(Ri::Ireturn);
            } else {
                code.push(Ri::Return);
            }
        }
        IrOpcode::Jump => {
            // Placeholder - will be resolved later
            code.push(Ri::Goto(0));
        }
        IrOpcode::CondBr => {
            if let Some(operand) = inst.operands.first() {
                emit_load(&mut code, operand, locals, string_consts);
                code.push(Ri::Ifne(0));  // Placeholder
            }
        }
        IrOpcode::Load => {
            if let (Some(ref result), Some(array), Some(index)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
                emit_load(&mut code, array, locals, string_consts);
                emit_load(&mut code, index, locals, string_consts);
                code.push(Ri::Iaload);
                code.push(Ri::Istore(*locals.get(result).unwrap_or(&0) as u8));
            }
        }
        IrOpcode::Slice | IrOpcode::Alloca | IrOpcode::Store | IrOpcode::Cast => {}
    }
    
    code
}

enum CmpType { Eq, Ne, Lt, Le, Gt, Ge }

fn emit_cmp(
    code: &mut Vec<Instruction>,
    inst: &IrInstruction,
    locals: &HashMap<String, u16>,
    string_consts: &HashMap<String, u16>,
    cmp: CmpType,
) {
    use ristretto_classfile::attributes::Instruction as Ri;
    if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
        emit_load(code, left, locals, string_consts);
        emit_load(code, right, locals, string_consts);
        
        // if_cmpXX +3 -> iconst_1
        // iconst_0
        // goto +2
        // iconst_1
        let cmp_instr = match cmp {
            CmpType::Eq => Ri::If_icmpeq(3),
            CmpType::Ne => Ri::If_icmpne(3),
            CmpType::Lt => Ri::If_icmplt(3),
            CmpType::Le => Ri::If_icmple(3),
            CmpType::Gt => Ri::If_icmpgt(3),
            CmpType::Ge => Ri::If_icmpge(3),
        };
        code.push(cmp_instr);
        code.push(Ri::Iconst_0);
        code.push(Ri::Goto(2));
        code.push(Ri::Iconst_1);
        
        code.push(Ri::Istore(*locals.get(result).unwrap_or(&0) as u8));
    }
}

fn emit_binop(
    code: &mut Vec<Instruction>,
    inst: &IrInstruction,
    locals: &HashMap<String, u16>,
    op: Instruction,
) {
    if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
        emit_load(code, left, locals, &HashMap::new());
        emit_load(code, right, locals, &HashMap::new());
        code.push(op);
        code.push(ristretto_classfile::attributes::Instruction::Istore(*locals.get(result).unwrap_or(&0) as u8));
    }
}

fn emit_load(
    code: &mut Vec<Instruction>,
    operand: &IrOperand,
    locals: &HashMap<String, u16>,
    string_consts: &HashMap<String, u16>,
) {
    use ristretto_classfile::attributes::Instruction as Ri;
    match operand {
        IrOperand::Variable(name, ty) => {
            let slot = *locals.get(name).unwrap_or(&0);
            code.push(match ty {
                IrType::String => Ri::Aload(slot as u8),
                _ => Ri::Iload(slot as u8),
            });
        }
        IrOperand::Constant(c) => emit_const(code, c, string_consts),
    }
}

fn emit_const(
    code: &mut Vec<Instruction>,
    c: &crate::ir::Constant,
    string_consts: &HashMap<String, u16>,
) {
    use ristretto_classfile::attributes::Instruction as Ri;
    use crate::ir::Constant;
    match c {
        Constant::Int(n) => {
            match *n as i32 {
                -1 => code.push(Ri::Iconst_m1),
                0 => code.push(Ri::Iconst_0),
                1 => code.push(Ri::Iconst_1),
                2 => code.push(Ri::Iconst_2),
                3 => code.push(Ri::Iconst_3),
                4 => code.push(Ri::Iconst_4),
                5 => code.push(Ri::Iconst_5),
                n if n >= -128 && n <= 127 => code.push(Ri::Bipush(n as i8)),
                n if n >= -32768 && n <= 32767 => code.push(Ri::Sipush(n as i16)),
                _ => code.push(Ri::Iconst_0),
            }
        }
        Constant::Bool(true) => code.push(Ri::Iconst_1),
        Constant::Bool(false) => code.push(Ri::Iconst_0),
        Constant::String(s) => {
            let idx = string_consts.get(s).copied().unwrap_or(1);
            if idx <= u8::MAX as u16 {
                code.push(Ri::Ldc(idx as u8));
            } else {
                code.push(Ri::Ldc_w(idx));
            }
        }
        Constant::Char(c) => {
            let val = *c as i32;
            if val >= -128 && val <= 127 {
                code.push(Ri::Bipush(val as i8));
            } else {
                code.push(Ri::Sipush(val as i16));
            }
        }
    }
}

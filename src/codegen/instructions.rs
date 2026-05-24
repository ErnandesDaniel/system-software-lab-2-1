use crate::ir::types::*;

use super::AsmGenerator;

impl AsmGenerator {
    pub fn generate_instruction(&mut self, inst: &IrInstruction) {
        match inst.opcode {
            IrOpcode::Assign => self.generate_assign(inst),
            IrOpcode::Add => self.binary_op(inst, "add"),
            IrOpcode::Sub => self.binary_op(inst, "sub"),
            IrOpcode::Mul => self.binary_op(inst, "imul"),
            IrOpcode::Div => self.binary_op_div(inst),
            IrOpcode::Mod => self.binary_op_mod(inst),
            IrOpcode::Eq => self.compare_op(inst, "sete"),
            IrOpcode::Ne => self.compare_op(inst, "setne"),
            IrOpcode::Lt => self.compare_op(inst, "setl"),
            IrOpcode::Le => self.compare_op(inst, "setle"),
            IrOpcode::Gt => self.compare_op(inst, "setg"),
            IrOpcode::Ge => self.compare_op(inst, "setge"),
            IrOpcode::And => self.binary_op(inst, "and"),
            IrOpcode::Or => self.binary_op(inst, "or"),
            IrOpcode::Not => self.generate_not(inst),
            IrOpcode::Neg => self.generate_neg(inst),
            IrOpcode::Jump => self.generate_jump(inst),
            IrOpcode::Call => self.generate_call(inst),
            IrOpcode::Ret => self.generate_ret(inst),
            IrOpcode::CondBr => self.generate_cond_br(inst),
            IrOpcode::Load => self.generate_load(inst),
            IrOpcode::Slice => self.generate_slice(inst),
            IrOpcode::Alloca => {}
            IrOpcode::BitNot => self.generate_bitnot(inst),
            IrOpcode::BitAnd => self.binary_op(inst, "and"),
            IrOpcode::BitOr => self.binary_op(inst, "or"),
            IrOpcode::Pos => self.generate_pos(inst),
            IrOpcode::Store => self.generate_store(inst),
            IrOpcode::Cast => {}
            IrOpcode::CoroYield => self.generate_yield(inst),
        }
    }

    fn generate_assign(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                let is_pointer = operand.get_type().is_pointer();
                if is_pointer {
                    self.load_operand(operand, "rax", true);
                    self.store_variable(result, "rax", true);
                } else {
                    self.load_operand(operand, "eax", false);
                    self.store_variable(result, "eax", false);
                }
            }
        }
    }

    fn binary_op(&mut self, inst: &IrInstruction, op: &str) {
        if let (Some(result), Some(left), Some(right)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str(&format!("    {} eax, ebx\n", op));
            self.store_variable(result, "eax", false);
        }
    }

    fn binary_op_div(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cdq\n");
            self.output.push_str("    idiv ebx\n");
            self.store_variable(result, "eax", false);
        }
    }

    fn binary_op_mod(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cdq\n");
            self.output.push_str("    idiv ebx\n");
            self.store_variable(result, "edx", false);
        }
    }

    fn compare_op(&mut self, inst: &IrInstruction, set_op: &str) {
        if let (Some(result), Some(left), Some(right)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cmp eax, ebx\n");
            self.output.push_str(&format!("    {} al\n", set_op));
            self.output.push_str("    movzx eax, al\n");
            self.store_variable(result, "eax", false);
        }
    }

    fn generate_not(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.output.push_str("    test eax, eax\n");
                self.output.push_str("    setz al\n");
                self.output.push_str("    movzx eax, al\n");
                self.store_variable(result, "eax", false);
            }
        }
    }

    fn generate_neg(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.output.push_str("    neg eax\n");
                self.store_variable(result, "eax", false);
            }
        }
    }

    fn generate_jump(&mut self, inst: &IrInstruction) {
        if let Some(ref target) = inst.jump_target {
            let formatted = self.format_block_label(target);
            self.output.push_str(&format!("    jmp {}\n", formatted));
        }
    }

    fn generate_store(&mut self, inst: &IrInstruction) {
        if inst.operands.len() >= 3 {
            let base = &inst.operands[0];
            let offset = &inst.operands[1];
            let value = &inst.operands[2];

            if let IrOperand::Variable(name, _) = base {
                if let IrOperand::Constant(crate::ir::Constant::Int(off)) = offset {
                    if inst.operands.len() == 4 {
                        let index = &inst.operands[3];
                        self.load_operand(value, "eax", false);
                        self.load_operand(index, "ebx", false);
                        if *off == 0 {
                            self.output.push_str(&format!("    lea rcx, [rel {}]\n", name));
                        } else {
                            self.output.push_str(&format!("    lea rcx, [rel {} + {}]\n", name, off));
                        }
                        self.output.push_str("    mov [rcx + rbx * 4], eax\n");
                    } else {
                        self.load_operand(value, "eax", false);
                        if let Some(local_off) = self.locals.get(name) {
                            let addr = local_off + *off as i32;
                            self.output.push_str(&format!("    mov [rbp + {}], eax\n", addr));
                        } else if let Some(param_idx) = self.param_registers.iter().position(|r| r == name) {
                            let reg = self.get_param_register(param_idx, true);
                            if *off == 0 {
                                self.output.push_str(&format!("    mov [{}], eax\n", reg));
                            } else {
                                self.output.push_str(&format!("    mov [{} + {}], eax\n", reg, off));
                            }
                        } else if *off == 0 {
                            self.output.push_str(&format!("    mov [rel {}], eax\n", name));
                        } else {
                            self.output.push_str(&format!("    mov [rel {} + {}], eax\n", name, off));
                        }
                    }
                }
            }
        }
    }

    fn generate_load(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(_operand0)) = (&inst.result, inst.operands.get(0)) {
            if inst.operands.len() == 1 {
                if let IrOperand::Variable(name, _) = &inst.operands[0] {
                    self.output.push_str(&format!("    mov eax, [rel {}]\n", name));
                    self.store_variable(result, "eax", false);
                }
            } else if inst.operands.len() == 4 {
                // Array of structs: base + field_offset + index + elem_size
                if let IrOperand::Variable(name, _) = &inst.operands[0] {
                    if let (IrOperand::Constant(crate::ir::Constant::Int(field_off)),
                            IrOperand::Constant(crate::ir::Constant::Int(elem_sz))) = (&inst.operands[1], &inst.operands[3]) {
                        self.load_operand(&inst.operands[2], "ebx", false);
                        if *field_off == 0 {
                            self.output.push_str(&format!("    lea rax, [rel {}]\n", name));
                        } else {
                            self.output.push_str(&format!("    lea rax, [rel {} + {}]\n", name, field_off));
                        }
                        if *elem_sz != 1 {
                            self.output.push_str(&format!("    imul ebx, {}\n", elem_sz));
                        }
                        self.output.push_str("    add rax, rbx\n");
                        self.output.push_str("    mov eax, [rax]\n");
                        self.store_variable(result, "eax", false);
                    }
                }
            } else if inst.operands.len() == 3 {
                // Struct array field access: base + offset + index
                if let IrOperand::Variable(name, _) = &inst.operands[0] {
                    if let IrOperand::Constant(crate::ir::Constant::Int(off)) = &inst.operands[1] {
                        self.load_operand(&inst.operands[2], "ebx", false);
                        if *off == 0 {
                            self.output.push_str(&format!("    lea rax, [rel {}]\n", name));
                        } else {
                            self.output.push_str(&format!("    lea rax, [rel {} + {}]\n", name, off));
                        }
                        self.output.push_str("    mov eax, [rax + rbx * 4]\n");
                        self.store_variable(result, "eax", false);
                    }
                }
            } else if inst.operands.len() == 2 {
                let first = &inst.operands[0];
                let second = &inst.operands[1];
                match second {
                    IrOperand::Constant(c) => {
                        if let IrOperand::Variable(name, _) = first {
                            if let crate::ir::Constant::Int(offset) = c {
                                if let Some(local_off) = self.locals.get(name) {
                                    let addr = local_off + *offset as i32;
                                    self.output.push_str(&format!("    mov eax, [rbp + {}]\n", addr));
                                } else if let Some(param_idx) = self.param_registers.iter().position(|r| r == name) {
                                    let reg = self.get_param_register(param_idx, true);
                                    if *offset == 0 {
                                        self.output.push_str(&format!("    mov eax, [{}]\n", reg));
                                    } else {
                                        self.output.push_str(&format!("    mov eax, [{} + {}]\n", reg, offset));
                                    }
                                } else if *offset == 0 {
                                    self.output.push_str(&format!("    mov eax, [rel {}]\n", name));
                                } else {
                                    self.output.push_str(&format!("    mov eax, [rel {} + {}]\n", name, offset));
                                }
                                self.store_variable(result, "eax", false);
                            }
                        }
                    }
                    _ => {
                        // Array access: base + index
                        if let IrOperand::Variable(name, _) = first {
                            self.output.push_str(&format!("    lea rax, [rel {}]\n", name));
                        } else {
                            self.load_operand(first, "rax", true);
                        }
                        self.load_operand(second, "ebx", false);
                        self.output.push_str("    mov eax, [rax + rbx * 4]\n");
                        self.store_variable(result, "eax", false);
                    }
                }
            }
        }
    }

    fn generate_slice(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(array), Some(start)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            let arr_type = array.get_type();
            if let IrType::Array(elem_type, _size) = arr_type {
                self.load_operand(array, "rax", true);
                self.load_operand(start, "ebx", false);
                let elem_size = elem_type.size() as i32;
                self.output
                    .push_str(&format!("    imul ebx, ebx, {}\n", elem_size));
                self.output.push_str("    add rax, rbx\n");
                self.store_variable(result, "rax", true);
            }
        }
    }

    fn generate_cond_br(&mut self, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            self.load_operand(operand, "eax", false);
            self.output.push_str("    test eax, eax\n");

            if let (Some(ref true_t), Some(ref false_t)) = (&inst.true_target, &inst.false_target) {
                let formatted_true = self.format_block_label(true_t);
                let formatted_false = self.format_block_label(false_t);
                self.output
                    .push_str(&format!("    jne {}\n", formatted_true));
                self.output
                    .push_str(&format!("    jmp {}\n", formatted_false));
            }
        }
    }

    fn generate_bitnot(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.output.push_str("    not eax\n");
                self.store_variable(result, "eax", false);
            }
        }
    }

    fn generate_pos(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.store_variable(result, "eax", false);
            }
        }
    }
}

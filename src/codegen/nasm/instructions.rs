use crate::ir::types::{IrInstruction, IrOpcode, IrOperand, IrType};

use crate::codegen::nasm::AsmGenerator;

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
            IrOpcode::CallIndirect => self.generate_call_indirect(inst),
            IrOpcode::MakeClosure => self.generate_make_closure(inst),
            IrOpcode::CallClosure => self.generate_call_closure(inst),
            IrOpcode::LoadCaptured => self.generate_load_captured(inst),
            IrOpcode::StoreCaptured => self.generate_store_captured(inst),
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
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str(&format!("    {op} eax, ebx\n"));
            self.store_variable(result, "eax", false);
        }
    }

    fn binary_op_div(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cdq\n");
            self.output.push_str("    idiv ebx\n");
            self.store_variable(result, "eax", false);
        }
    }

    fn binary_op_mod(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cdq\n");
            self.output.push_str("    idiv ebx\n");
            self.store_variable(result, "edx", false);
        }
    }

    fn compare_op(&mut self, inst: &IrInstruction, set_op: &str) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cmp eax, ebx\n");
            self.output.push_str(&format!("    {set_op} al\n"));
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
            self.output.push_str(&format!("    jmp {formatted}\n"));
        }
    }

    fn store_reg_for_type(&self, ty: &IrType) -> &'static str {
        if ty.is_pointer() { "rax" } else { "eax" }
    }

    fn load_result_reg(&self, inst: &IrInstruction) -> &'static str {
        inst.result_type.as_ref().map_or("eax", |t| self.store_reg_for_type(t))
    }

    fn array_elem_stride(inst: &IrInstruction) -> i64 {
        if let Some(IrOperand::Variable(_, ty)) = inst.operands.first() {
            if let IrType::Array(elem_type, _) = ty {
                return elem_type.size() as i64;
            }
        }
        4
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
                        let elem_stride = Self::array_elem_stride(inst);
                        let val_is_ptr = value.get_type().is_pointer();
                        self.load_operand(value, if val_is_ptr { "rax" } else { "eax" }, val_is_ptr);
                        self.load_operand(index, "ebx", false);
                        if *off == 0 {
                            self.output.push_str(&format!("    lea rcx, [rel {name}]\n"));
                        } else {
                            self.output.push_str(&format!("    lea rcx, [rel {name} + {off}]\n"));
                        }
                        if elem_stride == 8 {
                            self.output.push_str("    mov [rcx + rbx * 8], rax\n");
                        } else {
                            self.output.push_str("    mov [rcx + rbx * 4], eax\n");
                        }
                    } else {
                        let val_is_ptr = value.get_type().is_pointer();
                        let val_reg = if val_is_ptr { "rax" } else { "eax" };
                        self.load_operand(value, val_reg, val_is_ptr);
                        if let Some(local_off) = self.locals.get(name) {
                            let addr = local_off + *off as i32;
                            self.output.push_str(&format!("    mov [rbp + {addr}], {val_reg}\n"));
                        } else if let Some(param_idx) = self.param_registers.iter().position(|r| r == name) {
                            let reg = self.get_param_register(param_idx, true);
                            if *off == 0 {
                                self.output.push_str(&format!("    mov [{reg}], {val_reg}\n"));
                            } else {
                                self.output.push_str(&format!("    mov [{reg} + {off}], {val_reg}\n"));
                            }
                        } else if *off == 0 {
                            self.output.push_str(&format!("    mov [rel {name}], {val_reg}\n"));
                        } else {
                            self.output.push_str(&format!("    mov [rel {name} + {off}], {val_reg}\n"));
                        }
                    }
                }
            }
        }
    }

    fn generate_load(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(_operand0)) = (&inst.result, inst.operands.first()) {
            let result_is_ptr = self.load_result_reg(inst) == "rax";
            let result_reg = if result_is_ptr { "rax" } else { "eax" };
            if inst.operands.len() == 1 {
                if let IrOperand::Variable(name, _) = &inst.operands[0] {
                    self.output.push_str(&format!("    mov {result_reg}, [rel {name}]\n"));
                    self.store_variable(result, result_reg, result_is_ptr);
                }
            } else if inst.operands.len() == 4 {
                // Array of structs: base + field_offset + index + elem_size
                if let IrOperand::Variable(name, _) = &inst.operands[0] {
                    if let (
                        IrOperand::Constant(crate::ir::Constant::Int(field_off)),
                        IrOperand::Constant(crate::ir::Constant::Int(elem_sz)),
                    ) = (&inst.operands[1], &inst.operands[3])
                    {
                        self.load_operand(&inst.operands[2], "ebx", false);
                        if *field_off == 0 {
                            self.output.push_str(&format!("    lea rax, [rel {name}]\n"));
                        } else {
                            self.output
                                .push_str(&format!("    lea rax, [rel {name} + {field_off}]\n"));
                        }
                        if *elem_sz != 1 {
                            self.output.push_str(&format!("    imul ebx, {elem_sz}\n"));
                        }
                        self.output.push_str("    add rax, rbx\n");
                        self.output.push_str(&format!("    mov {result_reg}, [rax]\n"));
                        self.store_variable(result, result_reg, result_is_ptr);
                    }
                }
            } else if inst.operands.len() == 3 {
                // Struct array field access: base + offset + index
                if let IrOperand::Variable(name, _) = &inst.operands[0] {
                    if let IrOperand::Constant(crate::ir::Constant::Int(off)) = &inst.operands[1] {
                        let elem_stride = Self::array_elem_stride(inst);
                        self.load_operand(&inst.operands[2], "ebx", false);
                        if *off == 0 {
                            self.output.push_str(&format!("    lea rax, [rel {name}]\n"));
                        } else {
                            self.output.push_str(&format!("    lea rax, [rel {name} + {off}]\n"));
                        }
                        if elem_stride == 8 {
                            self.output.push_str(&format!("    mov {result_reg}, [rax + rbx * 8]\n"));
                        } else {
                            self.output.push_str(&format!("    mov {result_reg}, [rax + rbx * 4]\n"));
                        }
                        self.store_variable(result, result_reg, result_is_ptr);
                    }
                }
            } else if inst.operands.len() == 2 {
                let first = &inst.operands[0];
                let second = &inst.operands[1];
                if let IrOperand::Constant(c) = second {
                    if let IrOperand::Variable(name, _) = first {
                        if let crate::ir::Constant::Int(offset) = c {
                            if let Some(local_off) = self.locals.get(name) {
                                let addr = local_off + *offset as i32;
                                self.output.push_str(&format!("    mov {result_reg}, [rbp + {addr}]\n"));
                            } else if let Some(param_idx) = self.param_registers.iter().position(|r| r == name) {
                                let reg = self.get_param_register(param_idx, true);
                                if *offset == 0 {
                                    self.output.push_str(&format!("    mov {result_reg}, [{reg}]\n"));
                                } else {
                                    self.output.push_str(&format!("    mov {result_reg}, [{reg} + {offset}]\n"));
                                }
                            } else if *offset == 0 {
                                self.output.push_str(&format!("    mov {result_reg}, [rel {name}]\n"));
                            } else {
                                self.output.push_str(&format!("    mov {result_reg}, [rel {name} + {offset}]\n"));
                            }
                            self.store_variable(result, result_reg, result_is_ptr);
                        }
                    }
                } else {
                    // Array access: base + index
                    let elem_stride = Self::array_elem_stride(inst);
                    if let IrOperand::Variable(name, _) = first {
                        self.output.push_str(&format!("    lea rax, [rel {name}]\n"));
                    } else {
                        self.load_operand(first, "rax", true);
                    }
                    self.load_operand(second, "ebx", false);
                    if elem_stride == 8 {
                        self.output.push_str(&format!("    mov {result_reg}, [rax + rbx * 8]\n"));
                    } else {
                        self.output.push_str(&format!("    mov {result_reg}, [rax + rbx * 4]\n"));
                    }
                    self.store_variable(result, result_reg, result_is_ptr);
                }
            }
        }
    }

    fn generate_slice(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(array), Some(start)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            let arr_type = array.get_type();
            if let IrType::Array(elem_type, _size) = arr_type {
                self.load_operand(array, "rax", true);
                self.load_operand(start, "ebx", false);
                let elem_size = elem_type.size() as i32;
                self.output.push_str(&format!("    imul ebx, ebx, {elem_size}\n"));
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
                self.output.push_str(&format!("    jne {formatted_true}\n"));
                self.output.push_str(&format!("    jmp {formatted_false}\n"));
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

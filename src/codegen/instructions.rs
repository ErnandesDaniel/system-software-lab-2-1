use crate::ir::*;

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
            IrOpcode::CreateThread => self.generate_create_thread(inst),
            IrOpcode::Yield => self.generate_yield(inst),
            IrOpcode::CoroutineCreate => self.generate_coroutine_create(inst),
            IrOpcode::CoroutineYield => self.generate_coroutine_yield(inst),
            IrOpcode::CoroutineResume => self.generate_coroutine_resume(inst),
            IrOpcode::Ret => self.generate_ret(inst),
            IrOpcode::CondBr => self.generate_cond_br(inst),
            IrOpcode::Load => self.generate_load(inst),
            IrOpcode::Slice => self.generate_slice(inst),
            IrOpcode::Alloca => {}
            IrOpcode::BitNot => self.generate_bitnot(inst),
            IrOpcode::BitAnd => self.binary_op(inst, "and"),
            IrOpcode::BitOr => self.binary_op(inst, "or"),
            IrOpcode::Pos => self.generate_pos(inst),
            IrOpcode::Store => {}
            IrOpcode::Cast => {}
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

    fn generate_load(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(array), Some(index)) =
            (&inst.result, inst.operands.get(0), inst.operands.get(1))
        {
            self.load_operand(array, "rax", true);
            self.load_operand(index, "ebx", false);
            self.output.push_str("    mov eax, [rax + rbx * 4]\n");
            self.store_variable(result, "eax", false);
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

    fn generate_coroutine_create(&mut self, inst: &IrInstruction) {
        // CoroutineCreate - создание корутины с отдельным стеком
        // Генерирует код для выделения стека и инициализации
        if let Some(IrOperand::Constant(Constant::String(name))) = inst.operands.first() {
            self.output
                .push_str(&format!("    ; CoroutineCreate: {}\n", name));
            // TODO: выделение стека для корутины
        }
    }

    fn generate_coroutine_yield(&mut self, inst: &IrInstruction) {
        // CoroutineYield - передача управления планировщику с сохранением контекста
        self.output
            .push_str("    ; CoroutineYield - save context and switch\n");
        // Сохраняем все регистры
        self.output.push_str("    push rax\n");
        self.output.push_str("    push rbx\n");
        self.output.push_str("    push rcx\n");
        self.output.push_str("    push rdx\n");
        self.output.push_str("    push rbp\n");
        self.output.push_str("    push rsi\n");
        self.output.push_str("    push rdi\n");
        self.output.push_str("    push r8\n");
        self.output.push_str("    push r9\n");
        self.output.push_str("    push r10\n");
        self.output.push_str("    push r11\n");
        self.output.push_str("    push r12\n");
        self.output.push_str("    push r13\n");
        self.output.push_str("    push r14\n");
        self.output.push_str("    push r15\n");

        // Сохраняем RSP в структуру текущей корутины
        self.output.push_str("    mov [current_sp], rsp\n");

        // Вызываем планировщик
        self.output.push_str("    call scheduler\n");

        // Восстанавливаем контекст новой корутины
        self.output.push_str("    mov rsp, [current_sp]\n");

        // Восстанавливаем регистры
        self.output.push_str("    pop r15\n");
        self.output.push_str("    pop r14\n");
        self.output.push_str("    pop r13\n");
        self.output.push_str("    pop r12\n");
        self.output.push_str("    pop r11\n");
        self.output.push_str("    pop r10\n");
        self.output.push_str("    pop r9\n");
        self.output.push_str("    pop r8\n");
        self.output.push_str("    pop rdi\n");
        self.output.push_str("    pop rsi\n");
        self.output.push_str("    pop rbp\n");
        self.output.push_str("    pop rdx\n");
        self.output.push_str("    pop rcx\n");
        self.output.push_str("    pop rbx\n");
        self.output.push_str("    pop rax\n");
    }

    fn generate_coroutine_resume(&mut self, inst: &IrInstruction) {
        // CoroutineResume - возобновление корутины
        if let Some(IrOperand::Constant(Constant::Int(id))) = inst.operands.first() {
            self.output
                .push_str(&format!("    ; CoroutineResume: coroutine {}\n", id));
        }
    }
}

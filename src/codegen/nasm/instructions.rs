use crate::ir::types::{IrInstruction, IrOpcode};

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
            IrOpcode::BitXor => self.binary_op(inst, "xor"),
            IrOpcode::Pos => self.generate_pos(inst),
            IrOpcode::StrGetByte => self.generate_str_get_byte(inst),
            IrOpcode::StrSetByte => self.generate_str_set_byte(inst),
            IrOpcode::Store => self.generate_store(inst),
            IrOpcode::Cast => {}
            IrOpcode::CoroYield => self.generate_yield(inst),
            IrOpcode::CallIndirect => self.generate_call_indirect(inst),
            IrOpcode::MakeClosure => self.generate_make_closure(inst),
            IrOpcode::CallClosure => self.generate_call_closure(inst),
            IrOpcode::LoadCaptured => self.generate_load_captured(inst),
            IrOpcode::StoreCaptured => self.generate_store_captured(inst),
            IrOpcode::AllocArray => {}
        }
    }

    pub fn generate_assign(&mut self, inst: &IrInstruction) {
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

    pub fn binary_op(&mut self, inst: &IrInstruction, op: &str) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str(&format!("    {op} eax, ebx\n"));
            self.store_variable(result, "eax", false);
        }
    }

    pub fn binary_op_div(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cdq\n");
            self.output.push_str("    idiv ebx\n");
            self.store_variable(result, "eax", false);
        }
    }

    pub fn binary_op_mod(&mut self, inst: &IrInstruction) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cdq\n");
            self.output.push_str("    idiv ebx\n");
            self.store_variable(result, "edx", false);
        }
    }

    pub fn compare_op(&mut self, inst: &IrInstruction, set_op: &str) {
        if let (Some(result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.load_operand(left, "eax", false);
            self.load_operand(right, "ebx", false);
            self.output.push_str("    cmp eax, ebx\n");
            self.output.push_str(&format!("    {set_op} al\n"));
            self.output.push_str("    movzx eax, al\n");
            self.store_variable(result, "eax", false);
        }
    }

    pub fn generate_not(&mut self, inst: &IrInstruction) {
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

    pub fn generate_neg(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.output.push_str("    neg eax\n");
                self.store_variable(result, "eax", false);
            }
        }
    }

    pub fn generate_bitnot(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.output.push_str("    not eax\n");
                self.store_variable(result, "eax", false);
            }
        }
    }

    pub fn generate_pos(&mut self, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(operand) = inst.operands.first() {
                self.load_operand(operand, "eax", false);
                self.store_variable(result, "eax", false);
            }
        }
    }
}

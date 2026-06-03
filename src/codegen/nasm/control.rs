use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::IrInstruction;

impl AsmGenerator {
    pub fn emit_jump(&mut self, inst: &IrInstruction) {
        if let Some(ref target) = inst.jump_target {
            let formatted = self.format_block_label(target);
            self.line(&format!("jmp {formatted}"));
        }
    }

    pub fn emit_cond_br(&mut self, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            let r = self.alloc_scratch(false);
            self.load_operand(operand, r);
            self.line(&format!("test {r}, {r}"));

            if let (Some(ref true_t), Some(ref false_t)) = (&inst.true_target, &inst.false_target) {
                let true_label = self.format_block_label(true_t);
                let false_label = self.format_block_label(false_t);
                self.line(&format!("jne {true_label}"));
                self.line(&format!("jmp {false_label}"));
            }
        }
    }
}

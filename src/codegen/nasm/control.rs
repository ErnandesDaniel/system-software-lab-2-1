use crate::ir::types::IrInstruction;
use crate::codegen::nasm::AsmGenerator;

impl AsmGenerator {
    pub fn generate_jump(&mut self, inst: &IrInstruction) {
        if let Some(ref target) = inst.jump_target {
            let formatted = self.format_block_label(target);
            self.output.push_str(&format!("    jmp {formatted}\n"));
        }
    }

    pub fn generate_cond_br(&mut self, inst: &IrInstruction) {
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
}

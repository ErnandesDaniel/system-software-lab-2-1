use crate::codegen::nasm::AsmGenerator;
use crate::ir::types::IrBlock;

impl AsmGenerator {
    pub fn generate_block(&mut self, block: &IrBlock) {
        let label = self.format_block_label(&block.id);
        self.line(&format!("{label}:"));
        for inst in &block.instructions {
            self.generate_instruction(inst);
        }
    }
}

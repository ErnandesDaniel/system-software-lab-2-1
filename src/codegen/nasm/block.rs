use crate::ir::types::IrBlock;
use crate::codegen::nasm::AsmGenerator;

impl AsmGenerator {
    pub fn generate_block(&mut self, block: &IrBlock) {
        let label = self.format_block_label(&block.id);
        self.output.push_str(&format!("{label}:\n"));

        for inst in &block.instructions {
            self.generate_instruction(inst);
        }
    }
}

pub mod format;

use crate::ir::*;
use format::format_instruction;

pub struct CfgMermaidGenerator {
    output: String,
}

impl CfgMermaidGenerator {
    pub fn new() -> Self {
        Self { output: String::new() }
    }

    pub fn generate(&mut self, program: &IrProgram) -> String {
        self.output.clear();
        self.output.push_str("graph TD\n");
        for func in &program.functions {
            self.generate_function(func);
        }
        std::mem::take(&mut self.output)
    }

    pub fn generate_function_only(&mut self, func: &IrFunction) -> String {
        self.output.clear();
        self.output.push_str("graph TD\n");
        self.generate_function(func);
        std::mem::take(&mut self.output)
    }

    fn generate_function(&mut self, func: &IrFunction) {
        for block in &func.blocks {
            self.generate_block(block);
        }
        for block in &func.blocks {
            for succ in &block.successors {
                self.output.push_str(&format!(
                    "{} --> {}\n",
                    format::format_block_id(&block.id),
                    format::format_block_id(succ)
                ));
            }
        }
    }

    fn generate_block(&mut self, block: &IrBlock) {
        let formatted_id = format::format_block_id(&block.id);
        let instructions = self.format_block_instructions(block);
        self.output
            .push_str(&format!("    {}[\"{}\\n{}\"]\n", formatted_id, block.id, instructions));
    }

    fn format_block_instructions(&self, block: &IrBlock) -> String {
        let raw = block
            .instructions
            .iter()
            .map(format_instruction)
            .collect::<Vec<_>>()
            .join("\n");
        raw.replace('\\', "\\\\")
            .replace('"', "\\\"")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('\t', "\\t")
            .replace('[', "\\[")
            .replace(']', "\\]")
            .replace('{', "\\{")
            .replace('}', "\\}")
    }
}

impl Default for CfgMermaidGenerator {
    fn default() -> Self {
        Self::new()
    }
}

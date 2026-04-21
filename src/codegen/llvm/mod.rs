mod instructions;
mod types;

use crate::ir::*;
use std::collections::HashSet;

pub struct LlvmGenerator {
    extern_decls: HashSet<String>,
    global_strings: Vec<(String, String)>,
    string_counter: usize,
    tmp_counter: usize,
}

impl LlvmGenerator {
    pub fn new() -> Self {
        Self {
            extern_decls: HashSet::new(),
            global_strings: Vec::new(),
            string_counter: 0,
            tmp_counter: 0,
        }
    }

    pub fn generate_program(&mut self, program: &IrProgram) -> String {
        let mut funcs_code = String::new();
        for func in &program.functions {
            funcs_code.push_str(&self.generate_function(func));
            funcs_code.push_str("\n");
        }

        let mut output = String::new();

        for (label, content) in &self.global_strings {
            let mut escaped = String::new();
            for c in content.chars() {
                match c {
                    '\n' => escaped.push_str("\\0A"),
                    '\r' => escaped.push_str("\\0D"),
                    '\t' => escaped.push_str("\\09"),
                    '"' => escaped.push_str("\\22"),
                    '\\' => escaped.push_str("\\5C"),
                    _ => escaped.push(c),
                }
            }
            output.push_str(&format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1\n",
                label,
                content.len() + 1,
                escaped
            ));
        }
        output.push_str("\n");

        for ext in &self.extern_decls {
            output.push_str(&self.get_extern_signature(ext));
        }
        output.push_str("\n");

        output.push_str(&funcs_code);
        output
    }

    pub fn generate_function(&mut self, func: &IrFunction) -> String {
        let mut output = String::new();
        self.tmp_counter = 0;

        let ret_type = self.ir_type_to_llvm(&func.return_type);
        let params: Vec<String> = func.parameters.iter()
            .map(|p| format!("{} %p.{}", self.ir_type_to_llvm(&p.ty), p.name))
            .collect();

        output.push_str(&format!("define {} @{}({}) {{\n", ret_type, func.name, params.join(", ")));
        output.push_str("entry:\n");

        for param in &func.parameters {
            let ty = self.ir_type_to_llvm(&param.ty);
            output.push_str(&format!("  %{} = alloca {}\n", param.name, ty));
            output.push_str(&format!("  store {} %p.{}, {}* %{}\n", ty, param.name, ty, param.name));
        }

        use crate::codegen::traits::OperandLoader;
        
        for local in &func.locals {
            // Use is_temp to properly identify temporaries (t0, t1, etc.), not just any name starting with 't'
            if !Self::is_temp(&local.name) && !func.parameters.iter().any(|p| p.name == local.name) {
                output.push_str(&format!("  %{} = alloca {}\n", local.name, self.ir_type_to_llvm(&local.ty)));
            }
        }

        output.push_str("  br label %bb_0\n");

        for (idx, block) in func.blocks.iter().enumerate() {
            let label = if idx == 0 {
                "bb_0"
            } else {
                &self.block_id_to_label(&block.id)
            };
            output.push_str(&format!("{}:\n", label));

            for inst in &block.instructions {
                output.push_str(&self.generate_instruction(inst));
            }
        }

        output.push_str("}\n");
        output
    }
}

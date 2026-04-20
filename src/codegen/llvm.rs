use crate::ir::*;

pub struct LlvmGenerator {
    extern_decls: Vec<String>,
    temp_counter: usize,
}

impl LlvmGenerator {
    pub fn new() -> Self {
        Self { 
            extern_decls: Vec::new(),
            temp_counter: 0,
        }
    }

    pub fn generate_function(&mut self, func: &IrFunction) -> String {
        let mut output = String::new();
        
        self.temp_counter = 0;
        
        let return_type = match func.return_type {
            IrType::Void => "void",
            IrType::Int | IrType::Bool => "i32",
            IrType::String => "i8*",
            _ => "i32",
        };
        
        let params: Vec<String> = func.parameters.iter()
            .map(|p| format!("i32 %{}", p.name))
            .collect();
        let params_str = params.join(", ");
        
        output.push_str(&format!(
            "define {} @{}({}) {{\n",
            return_type, func.name, params_str
        ));
        
        // Simple approach: just emit calls for external functions
        for block in &func.blocks {
            for inst in &block.instructions {
                let inst_ir = self.generate_instruction(inst, &func.return_type);
                if !inst_ir.is_empty() {
                    output.push_str(&inst_ir);
                }
            }
        }
        
        output.push_str("}\n");
        output
    }

    fn fresh_temp(&mut self) -> String {
        self.temp_counter += 1;
        format!("%t{}", self.temp_counter)
    }

    fn generate_instruction(&mut self, inst: &IrInstruction, return_type: &IrType) -> String {
        let mut output = String::new();
        
        match &inst.opcode {
            // External function calls - the main thing we need
            IrOpcode::Call => {
                if let Some(target) = &inst.jump_target {
                    self.add_extern_decl(target);
                    
                    // Only handle external calls for now
                    let has_result = matches!(
                        target.as_str(),
                        "printf" | "scanf" | "puts" | "getchar" | "putchar" | "rand" | "time" | "malloc"
                    );
                    
                    if has_result {
                        let result = self.fresh_temp();
                        output.push_str(&format!(
                            "  {} = call i32 @{}(i32)\n",
                            result, target
                        ));
                    } else {
                        output.push_str(&format!(
                            "  call i32 @{}(i32)\n",
                            target
                        ));
                    }
                }
            }
            
            // Just return 0 for return statements
            IrOpcode::Ret => {
                if matches!(return_type, IrType::Void) {
                    output.push_str("  ret void\n");
                } else {
                    output.push_str("  ret i32 0\n");
                }
            }
            
            // Skip other instructions for now
            _ => {}
        }
        
        output
    }

    fn add_extern_decl(&mut self, name: &str) {
        if !name.is_empty() && !self.extern_decls.contains(&name.to_string()) {
            self.extern_decls.push(name.to_string());
        }
    }

    pub fn get_extern_decls(&self) -> Vec<String> {
        self.extern_decls.clone()
    }
}

impl Default for LlvmGenerator {
    fn default() -> Self {
        Self::new()
    }
}
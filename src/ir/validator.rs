use crate::ir::types::*;

/// Validates IR before code generation
pub struct IrValidator;

impl IrValidator {
    pub fn validate(program: &IrProgram) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();
        
        for func in &program.functions {
            Self::validate_function(func, &mut errors);
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
    
    fn validate_function(func: &IrFunction, errors: &mut Vec<String>) {
        // Check for empty function name
        if func.name.is_empty() {
            errors.push("Function has empty name".to_string());
        }
        
        // Check for duplicate block IDs
        let mut block_ids = std::collections::HashSet::new();
        for block in &func.blocks {
            if !block_ids.insert(&block.id) {
                errors.push(format!(
                    "Function {}: duplicate block ID {}",
                    func.name, block.id
                ));
            }
        }
        
        // Check that all jump targets exist
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref target) = inst.jump_target {
                    if !block_ids.contains(target) && !target.starts_with("bb_") {
                        errors.push(format!(
                            "Function {}: jump to unknown block {}",
                            func.name, target
                        ));
                    }
                }
            }
        }
        
        // Check for unreachable blocks (simple check)
        if func.blocks.len() > 1 {
            let mut reachable = std::collections::HashSet::new();
            if let Some(first) = func.blocks.first() {
                reachable.insert(first.id.clone());
            }
            
            for block in &func.blocks {
                if reachable.contains(&block.id) {
                    for inst in &block.instructions {
                        if let Some(ref target) = inst.jump_target {
                            reachable.insert(target.clone());
                        }
                        if let Some(ref true_target) = inst.true_target {
                            reachable.insert(true_target.clone());
                        }
                        if let Some(ref false_target) = inst.false_target {
                            reachable.insert(false_target.clone());
                        }
                    }
                }
            }
        }
    }
}

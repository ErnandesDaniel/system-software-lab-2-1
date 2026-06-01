use crate::error::CompilerError;
use crate::ir::types::IrOpcode;
use crate::ir::{IrFunction, IrProgram};

pub struct IrValidator;

impl IrValidator {
    pub fn validate(program: &IrProgram) -> crate::Result<()> {
        let mut errors = Vec::new();

        Self::validate_globals(program, &mut errors);
        for func in &program.functions {
            Self::validate_function(func, &mut errors);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(CompilerError::Internal(errors.join("; ")))
        }
    }

    fn validate_globals(program: &IrProgram, errors: &mut Vec<String>) {
        for global in &program.globals {
            if global.name.is_empty() {
                errors.push("Global variable has empty name".to_string());
            }
        }
    }

    fn validate_function(func: &IrFunction, errors: &mut Vec<String>) {
        Self::validate_name_and_blocks(func, errors);
        Self::validate_block_ids(func, errors);
        Self::validate_jump_targets(func, errors);
        Self::validate_reachability(func, errors);
        Self::validate_terminators(func, errors);
        Self::validate_coroutine_consistency(func, errors);
    }

    fn validate_name_and_blocks(func: &IrFunction, errors: &mut Vec<String>) {
        if func.name.is_empty() {
            errors.push("Function has empty name".to_string());
        }
        if func.blocks.is_empty() {
            errors.push(format!("Function '{}' has no basic blocks", func.name));
        }
    }

    fn validate_block_ids(func: &IrFunction, errors: &mut Vec<String>) {
        let mut block_ids = std::collections::HashSet::new();
        for block in &func.blocks {
            if block.id.is_empty() {
                errors.push(format!("Function '{}': block with empty ID", func.name));
            }
            if !block_ids.insert(&block.id) {
                errors.push(format!(
                    "Function '{}': duplicate block ID '{}'",
                    func.name, block.id
                ));
            }
        }
    }

    fn validate_jump_targets(func: &IrFunction, errors: &mut Vec<String>) {
        let block_ids: std::collections::HashSet<&String> =
            func.blocks.iter().map(|b| &b.id).collect();

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref target) = inst.jump_target {
                    if matches!(
                        inst.opcode,
                        IrOpcode::Call | IrOpcode::MakeClosure
                    ) {
                        continue;
                    }
                    if !block_ids.contains(target) && !target.starts_with("bb_") {
                        errors.push(format!(
                            "Function '{}': jump to unknown block '{}'",
                            func.name, target
                        ));
                    }
                }
            }
        }
    }

    fn validate_reachability(func: &IrFunction, errors: &mut Vec<String>) {
        if func.blocks.len() <= 1 {
            return;
        }

        let mut reachable = std::collections::HashSet::new();
        if let Some(first) = func.blocks.first() {
            reachable.insert(first.id.clone());
        }

        let mut changed = true;
        while changed {
            changed = false;
            for block in &func.blocks {
                if !reachable.contains(&block.id) {
                    continue;
                }
                for inst in &block.instructions {
                    if matches!(
                        inst.opcode,
                        IrOpcode::Call | IrOpcode::MakeClosure
                    ) {
                        continue;
                    }
                    if let Some(ref target) = inst.jump_target {
                        if reachable.insert(target.clone()) {
                            changed = true;
                        }
                    }
                    if let Some(ref true_target) = inst.true_target {
                        if reachable.insert(true_target.clone()) {
                            changed = true;
                        }
                    }
                    if let Some(ref false_target) = inst.false_target {
                        if reachable.insert(false_target.clone()) {
                            changed = true;
                        }
                    }
                }
            }
        }

        for block in &func.blocks {
            if !reachable.contains(&block.id) {
                errors.push(format!(
                    "Function '{}': unreachable block '{}'",
                    func.name, block.id
                ));
            }
        }
    }

    fn validate_terminators(func: &IrFunction, errors: &mut Vec<String>) {
        for block in &func.blocks {
            if let Some(last) = block.instructions.last() {
                match last.opcode {
                    IrOpcode::Ret | IrOpcode::Jump | IrOpcode::CondBr | IrOpcode::CoroYield => {}
                    _ => {
                        if func.return_type == crate::ir::IrType::Void
                            || block.successors.is_empty()
                        {
                            if block.id == *func.blocks.last().map(|b| &b.id).unwrap_or(&String::new())
                            {
                                continue;
                            }
                            errors.push(format!(
                                "Function '{}', block '{}': last instruction {:?} is not a terminator",
                                func.name,
                                block.id,
                                last.opcode
                            ));
                        }
                    }
                }
            } else {
                if block.id == *func.blocks.last().map(|b| &b.id).unwrap_or(&String::new()) {
                    continue;
                }
                errors.push(format!(
                    "Function '{}', block '{}': empty block (no instructions)",
                    func.name, block.id
                ));
            }
        }
    }

    fn validate_coroutine_consistency(func: &IrFunction, errors: &mut Vec<String>) {
        if !func.is_coroutine {
            return;
        }

        if func.yield_count == 0 {
            errors.push(format!(
                "Function '{}': marked as coroutine but has 0 yield states",
                func.name
            ));
        }

        let has_yield = func.blocks.iter().any(|b| {
            b.instructions
                .iter()
                .any(|i| matches!(i.opcode, IrOpcode::CoroYield))
        });

        if !has_yield {
            errors.push(format!(
                "Function '{}': marked as coroutine but contains no yield instruction",
                func.name
            ));
        }
    }
}

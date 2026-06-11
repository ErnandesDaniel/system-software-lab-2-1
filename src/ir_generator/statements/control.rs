use super::IrGenerator;
use crate::ast::{IfStatement, LoopKeyword, LoopStatement, RepeatStatement};
use crate::ir::{IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_if_statement(&mut self, block: &mut IrBlock, stmt: &IfStatement) {
        let merge_id = self.generate_block_id();

        // Generate all else-if chains and else branch
        // Each else-if stores (entry_id, condbr_block_id, body_id).
        // entry_id is the entry point of the condition evaluation (for control flow targeting).
        // condbr_block_id is the block that contains the else-if's CondBr (for false_target chaining).
        // For simple conditions they are the same; for short-circuit `&&`/`||` they differ.
        let mut else_if_blocks: Vec<(String, String, String)> = Vec::new();
        for ei in &stmt.else_ifs {
            let ei_entry_id = self.generate_block_id();
            let ei_body_id = self.generate_block_id();

            let mut ei_cond_block = IrBlock::new(ei_entry_id.clone());
            let (ei_cond_temp, _) = self.visit_expr(&mut ei_cond_block, &ei.condition);
            // visit_expr may have swapped ei_cond_block (e.g. short-circuit && generates new blocks);
            // the current ei_cond_block.id is where the else-if CondBr will live.
            let ei_condbr_id = ei_cond_block.id.clone();
            else_if_blocks.push((ei_entry_id, ei_condbr_id, ei_body_id.clone()));

            ei_cond_block.instructions.push(IrInstruction {
                opcode: IrOpcode::CondBr,
                result: None,
                result_type: None,
                operands: vec![IrOperand::Variable(ei_cond_temp, IrType::Bool)],
                jump_target: None,
                true_target: Some(ei_body_id.clone()),
                false_target: None,
                span: ei.span,
            });
            ei_cond_block.successors.push(ei_body_id.clone());
            self.block_stack.push(ei_cond_block);

            let mut ei_body_block = IrBlock::new(ei_body_id);
            for s in &ei.body {
                self.visit_statement(&mut ei_body_block, s);
            }
            if !ends_with_control_flow(&ei_body_block) {
                ei_body_block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Jump,
                    result: None,
                    result_type: None,
                    operands: vec![],
                    jump_target: Some(merge_id.clone()),
                    true_target: None,
                    false_target: None,
                    span: ei.span,
                });
            }
            ei_body_block.successors.push(merge_id.clone());
            self.block_stack.push(ei_body_block);
        }

        let else_body_id = if let Some(ref else_body) = stmt.else_body {
            let eb_id = self.generate_block_id();
            let mut else_block = IrBlock::new(eb_id.clone());
            for s in else_body {
                self.visit_statement(&mut else_block, s);
            }
            if !ends_with_control_flow(&else_block) {
                else_block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Jump,
                    result: None,
                    result_type: None,
                    operands: vec![],
                    jump_target: Some(merge_id.clone()),
                    true_target: None,
                    false_target: None,
                    span: stmt.span,
                });
            }
            else_block.successors.push(merge_id.clone());
            self.block_stack.push(else_block);
            Some(eb_id)
        } else {
            None
        };

        // Determine false target of the main if condition
        // Use the entry_id (first element) of the first else-if to jump to the start of its condition evaluation.
        let main_false_id = if let Some((ref ei_entry_id, _, _)) = else_if_blocks.first() {
            ei_entry_id.clone()
        } else if let Some(ref eb_id) = else_body_id {
            eb_id.clone()
        } else {
            merge_id.clone()
        };

        // Main if condition
        let (cond_temp, _) = self.visit_expr(block, &stmt.condition);
        let then_id = self.generate_block_id();

        block.instructions.push(IrInstruction {
            opcode: IrOpcode::CondBr,
            result: None,
            result_type: None,
            operands: vec![IrOperand::Variable(cond_temp, IrType::Bool)],
            jump_target: None,
            true_target: Some(then_id.clone()),
            false_target: Some(main_false_id.clone()),
            span: stmt.span,
        });
        block.successors.push(then_id.clone());
        block.successors.push(main_false_id.clone());

        // Then block
        let mut then_block = IrBlock::new(then_id);
        for s in &stmt.body {
            self.visit_statement(&mut then_block, s);
        }
        if !ends_with_control_flow(&then_block) {
            then_block.instructions.push(IrInstruction {
                opcode: IrOpcode::Jump,
                result: None,
                result_type: None,
                operands: vec![],
                jump_target: Some(merge_id.clone()),
                true_target: None,
                false_target: None,
                span: stmt.span,
            });
        }
        then_block.successors.push(merge_id.clone());
        self.block_stack.push(then_block);

        // Chain false targets for else-if blocks
        // Use condbr_block_id (.1) to locate the block containing each else-if's CondBr,
        // and point its false_target to the next else-if's entry_id (.0), else_body, or merge.
        for (i, (_, ei_condbr_id, _)) in else_if_blocks.iter().enumerate() {
            let next_false = if i + 1 < else_if_blocks.len() {
                else_if_blocks[i + 1].0.clone()
            } else if let Some(ref eb_id) = else_body_id {
                eb_id.clone()
            } else {
                merge_id.clone()
            };
            if let Some(cond_block) = self
                .block_stack
                .iter_mut()
                .find(|b: &&mut IrBlock| b.id == *ei_condbr_id)
            {
                if let Some(last) = cond_block.instructions.last_mut() {
                    if last.opcode == IrOpcode::CondBr {
                        last.false_target = Some(next_false.clone());
                        cond_block.successors.push(next_false.clone());
                    }
                }
            }
        }

        // Swap current block to merge block
        let entry_block = std::mem::replace(block, IrBlock::new(merge_id));
        self.block_stack.push(entry_block);
    }

    pub fn visit_loop_statement(&mut self, block: &mut IrBlock, stmt: &LoopStatement) {
        let header_id = self.generate_block_id();
        let body_id = self.generate_block_id();
        let exit_id = self.generate_block_id();

        self.loop_depth += 1;
        self.loop_exit_stack.push(exit_id.clone());

        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Jump,
            result: None,
            result_type: None,
            operands: vec![],
            jump_target: Some(header_id.clone()),
            true_target: None,
            false_target: None,
            span: stmt.span,
        });
        block.successors.push(header_id.clone());

        let mut header_block = IrBlock::new(header_id.clone());
        let (cond_temp, _) = self.visit_expr(&mut header_block, &stmt.condition);

        let loop_keyword = matches!(stmt.keyword, LoopKeyword::While);
        let (true_target, false_target) = if loop_keyword {
            (body_id.clone(), exit_id.clone())
        } else {
            (exit_id.clone(), body_id.clone())
        };

        header_block.instructions.push(IrInstruction {
            opcode: IrOpcode::CondBr,
            result: None,
            result_type: None,
            operands: vec![IrOperand::Variable(cond_temp, IrType::Bool)],
            jump_target: None,
            true_target: Some(true_target.clone()),
            false_target: Some(false_target.clone()),
            span: stmt.span,
        });
        header_block.successors.push(true_target);
        header_block.successors.push(false_target);
        self.block_stack.push(header_block);

        let mut body_block = IrBlock::new(body_id.clone());
        for s in &stmt.body {
            self.visit_statement(&mut body_block, s);
        }
        body_block.instructions.push(IrInstruction {
            opcode: IrOpcode::Jump,
            result: None,
            result_type: None,
            operands: vec![],
            jump_target: Some(header_id.clone()),
            true_target: None,
            false_target: None,
            span: stmt.span,
        });
        body_block.successors.push(header_id.clone());
        self.block_stack.push(body_block);

        // Swap to exit block
        let entry_block = std::mem::replace(block, IrBlock::new(exit_id));
        self.block_stack.push(entry_block);

        self.loop_exit_stack.pop();
        self.loop_depth -= 1;
    }

    pub fn visit_repeat_statement(&mut self, block: &mut IrBlock, stmt: &RepeatStatement) {
        let body_id = self.generate_block_id();
        let header_id = self.generate_block_id();
        let exit_id = self.generate_block_id();

        self.loop_depth += 1;
        self.loop_exit_stack.push(exit_id.clone());

        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Jump,
            result: None,
            result_type: None,
            operands: vec![],
            jump_target: Some(body_id.clone()),
            true_target: None,
            false_target: None,
            span: stmt.span,
        });
        block.successors.push(body_id.clone());

        let mut body_block = IrBlock::new(body_id.clone());
        for s in &stmt.body {
            self.visit_statement(&mut body_block, s);
        }
        body_block.instructions.push(IrInstruction {
            opcode: IrOpcode::Jump,
            result: None,
            result_type: None,
            operands: vec![],
            jump_target: Some(header_id.clone()),
            true_target: None,
            false_target: None,
            span: stmt.span,
        });
        body_block.successors.push(header_id.clone());
        self.block_stack.push(body_block);

        let mut header_block = IrBlock::new(header_id.clone());
        let (cond_temp, _) = self.visit_expr(&mut header_block, &stmt.condition);

        let loop_keyword = matches!(stmt.keyword, LoopKeyword::While);
        let (true_target, false_target) = if loop_keyword {
            (body_id.clone(), exit_id.clone())
        } else {
            (exit_id.clone(), body_id.clone())
        };

        header_block.instructions.push(IrInstruction {
            opcode: IrOpcode::CondBr,
            result: None,
            result_type: None,
            operands: vec![IrOperand::Variable(cond_temp, IrType::Bool)],
            jump_target: None,
            true_target: Some(true_target.clone()),
            false_target: Some(false_target.clone()),
            span: stmt.span,
        });
        header_block.successors.push(true_target);
        header_block.successors.push(false_target);
        self.block_stack.push(header_block);

        let entry_block = std::mem::replace(block, IrBlock::new(exit_id));
        self.block_stack.push(entry_block);

        self.loop_exit_stack.pop();
        self.loop_depth -= 1;
    }
}

fn ends_with_control_flow(block: &IrBlock) -> bool {
    block
        .instructions
        .last()
        .is_some_and(|inst| matches!(inst.opcode, IrOpcode::Ret | IrOpcode::Jump | IrOpcode::CondBr))
}

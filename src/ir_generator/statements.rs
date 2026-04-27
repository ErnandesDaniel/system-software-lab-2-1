use super::IrGenerator;
use crate::ast::*;
use crate::ir::*;

impl IrGenerator {
    pub fn visit_statement(
        &mut self,
        block: &mut IrBlock,
        block_stack: &mut Vec<IrBlock>,
        stmt: &Statement,
    ) {
        match stmt {
            Statement::Return(ret) => {
                let operands = if let Some(ref expr) = ret.expr {
                    let (temp, _) = self.visit_expr(block, expr);
                    vec![IrOperand::Variable(temp, IrType::Int)]
                } else {
                    vec![]
                };

                let has_return_value = ret.expr.is_some();

                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Ret,
                    result: None,
                    result_type: if has_return_value { Some(IrType::Int) } else { None },
                    operands,
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: ret.span,
                });
            }
            Statement::If(if_stmt) => {
                self.visit_if_statement(block, block_stack, if_stmt);
            }
            Statement::Loop(loop_stmt) => {
                self.visit_loop_statement(block, block_stack, loop_stmt);
            }
            Statement::Repeat(repeat_stmt) => {
                self.visit_repeat_statement(block, block_stack, repeat_stmt);
            }
            Statement::Expression(expr_stmt) => {
                self.visit_expr(block, &expr_stmt.expr);
            }
            Statement::Block(block_stmt) => {
                for s in &block_stmt.body {
                    self.visit_statement(block, block_stack, s);
                }
            }
            Statement::Break(_) => {
                if let Some(exit_id) = self.loop_exit_stack.last() {
                    block.instructions.push(IrInstruction {
                        opcode: IrOpcode::Jump,
                        result: None,
                        result_type: None,
                        operands: vec![],
                        jump_target: Some(exit_id.clone()),
                        true_target: None,
                        false_target: None,
                        span: stmt.span(),
                    });
                    block.successors.push(exit_id.clone());
                }
            }
        }
    }

    pub fn visit_if_statement(
        &mut self,
        block: &mut IrBlock,
        block_stack: &mut Vec<IrBlock>,
        stmt: &IfStatement,
    ) {
        let (cond_temp, _) = self.visit_expr(block, &stmt.condition);

        let then_id = self.generate_block_id();
        let else_id = self.generate_block_id();
        let merge_id = self.generate_block_id();

        block.instructions.push(IrInstruction {
            opcode: IrOpcode::CondBr,
            result: None,
            result_type: None,
            operands: vec![IrOperand::Variable(cond_temp, IrType::Bool)],
            jump_target: None,
            true_target: Some(then_id.clone()),
            false_target: Some(else_id.clone()),
            span: stmt.span,
        });
        block.successors.push(then_id.clone());
        block.successors.push(else_id.clone());

        let mut then_block = IrBlock {
            id: then_id,
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        self.visit_statement(&mut then_block, block_stack, &stmt.consequence);
        then_block.successors.push(merge_id.clone());
        block_stack.push(then_block);

        let mut else_block = IrBlock {
            id: else_id,
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        if let Some(ref alt) = stmt.alternative {
            self.visit_statement(&mut else_block, block_stack, alt);
        }
        else_block.successors.push(merge_id.clone());
        block_stack.push(else_block);
        
        // Add merge block (code after if statement)
        block_stack.push(IrBlock {
            id: merge_id,
            instructions: Vec::new(),
            successors: Vec::new(),
        });
    }

    pub fn visit_loop_statement(
        &mut self,
        block: &mut IrBlock,
        block_stack: &mut Vec<IrBlock>,
        stmt: &LoopStatement,
    ) {
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

        let mut header_block = IrBlock {
            id: header_id.clone(),
            instructions: Vec::new(),
            successors: Vec::new(),
        };

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
        block_stack.push(header_block);

        let mut body_block = IrBlock {
            id: body_id.clone(),
            instructions: Vec::new(),
            successors: Vec::new(),
        };

        for s in &stmt.body {
            self.visit_statement(&mut body_block, block_stack, s);
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
        block_stack.push(body_block);

        // Save current block (entry with Jump) to stack
        let old_block = std::mem::replace(
            block,
            IrBlock {
                id: exit_id,
                instructions: Vec::new(),
                successors: Vec::new(),
            },
        );
        block_stack.push(old_block);  // Save entry block
        // block now is exit block for code after the loop

        self.loop_exit_stack.pop();
        self.loop_depth -= 1;
    }

    pub fn visit_repeat_statement(
        &mut self,
        block: &mut IrBlock,
        block_stack: &mut Vec<IrBlock>,
        stmt: &RepeatStatement,
    ) {
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

        let mut body_block = IrBlock {
            id: body_id.clone(),
            instructions: Vec::new(),
            successors: Vec::new(),
        };

        self.visit_statement(&mut body_block, block_stack, &*stmt.body);

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
        block_stack.push(body_block);

        let mut header_block = IrBlock {
            id: header_id.clone(),
            instructions: Vec::new(),
            successors: Vec::new(),
        };

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
        block_stack.push(header_block);

        let exit_block = IrBlock {
            id: exit_id.clone(),
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        block_stack.push(exit_block);

        self.loop_exit_stack.pop();
        self.loop_depth -= 1;
    }
}

use super::IrGenerator;
use crate::ast::{IfStatement, LoopKeyword, LoopStatement, RepeatStatement, Statement};
use crate::ir::{Constant, IrBlock, IrInstruction, IrLocal, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_statement(&mut self, block: &mut IrBlock, block_stack: &mut Vec<IrBlock>, stmt: &Statement) {
        match stmt {
            Statement::Return(ret) => {
                let (operands, result_type) = if let Some(ref expr) = ret.expr {
                    let (temp, expr_type) = self.visit_expr(block, expr);
                    (vec![IrOperand::Variable(temp, expr_type.clone())], Some(expr_type))
                } else {
                    (vec![], None)
                };

                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Ret,
                    result: None,
                    result_type,
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
            Statement::VarDecl(vd) => {
                if !self.declared_vars.contains(&vd.name.name) {
                    let ir_ty = self.convert_type(&vd.ty);
                    self.locals.insert(
                        vd.name.name.clone(),
                        IrLocal {
                            name: vd.name.name.clone(),
                            ty: ir_ty,
                            stack_offset: None,
                        },
                    );
                    self.declared_vars.insert(vd.name.name.clone());
                    if let crate::ast::TypeRef::Custom(id) = &vd.ty {
                        self.local_struct_types.insert(vd.name.name.clone(), id.name.clone());
                    }
                }
            }
            Statement::Yield(_) => {
                self.current_yield_state += 1;
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::CoroYield,
                    result: None,
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Int(self.current_yield_state as i64))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: crate::ast::Span::new(0, 0),
                });
                // Split block: start new block for next state
                let new_id = format!("BB{}", self.block_counter);
                self.coroutine_state_blocks.push(new_id.clone());
                self.block_counter += 1;
                let old_block = std::mem::replace(
                    block,
                    IrBlock {
                        id: new_id,
                        instructions: Vec::new(),
                        successors: Vec::new(),
                    },
                );
                block_stack.push(old_block);
            }
            Statement::FuncDef(fd) => {
                let mangled = format!("__lambda_{}", self.lambda_counter);
                self.lambda_counter += 1;

                let saved_locals = self.locals.clone();
                let saved_declared = self.declared_vars.clone();
                let saved_used = self.used_functions.clone();
                let saved_block = self.block_counter;

                let param_types: Vec<IrType> = fd
                    .signature
                    .parameters
                    .as_ref()
                    .map(|args| {
                        args.iter()
                            .map(|a| a.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t)))
                            .collect()
                    })
                    .unwrap_or_default();
                let ret_type = fd
                    .signature
                    .return_type
                    .as_ref()
                    .map_or(IrType::Void, |t| self.convert_type(t));

                // Scan for captures
                let captures = Self::scan_captures(&fd.body, &fd.signature.parameters, &saved_locals);
                let has_captures = !captures.is_empty();

                let mut inner_def = fd.clone();
                inner_def.signature.name.name = mangled.clone();

                if has_captures {
                    // Add __env as hidden first parameter
                    let env_param = crate::ast::Arg {
                        name: crate::ast::Identifier {
                            name: "__env".to_string(),
                            span: crate::ast::Span::new(0, 0),
                        },
                        ty: None,
                        span: crate::ast::Span::new(0, 0),
                    };
                    let mut new_params = vec![env_param];
                    if let Some(ref args) = inner_def.signature.parameters {
                        new_params.extend(args.clone());
                    }
                    inner_def.signature.parameters = Some(new_params);
                }

                self.captured_vars = captures.iter().cloned().collect();
                let ir_func = self.generate_function(&inner_def);
                self.pending_functions.push(ir_func);
                self.captured_vars.clear();

                self.locals = saved_locals;
                self.declared_vars = saved_declared;
                self.used_functions = saved_used;
                self.block_counter = saved_block;

                let func_type = IrType::Function(param_types, Box::new(ret_type));

                if !self.declared_vars.contains(&fd.signature.name.name) {
                    self.locals.insert(
                        fd.signature.name.name.clone(),
                        IrLocal {
                            name: fd.signature.name.name.clone(),
                            ty: func_type.clone(),
                            stack_offset: None,
                        },
                    );
                    self.declared_vars.insert(fd.signature.name.name.clone());
                }

                let tmp = self.generate_temp();
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(tmp.clone()),
                    result_type: Some(func_type.clone()),
                    operands: vec![IrOperand::FuncRef(mangled.clone())],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: fd.span,
                });

                if has_captures {
                    let env_tmp = self.generate_temp();
                    let mut env_operands: Vec<IrOperand> = captures
                        .iter()
                        .map(|(name, _)| IrOperand::Variable(name.clone(), IrType::Int))
                        .collect();
                    env_operands.insert(0, IrOperand::FuncRef(mangled.clone()));
                    // Add env slot locals so the frame size accounts for them
                    for (i, _) in captures.iter().enumerate() {
                        let slot_name = format!("__env_slot_{i}");
                        self.locals.insert(
                            slot_name.clone(),
                            IrLocal {
                                name: slot_name,
                                ty: IrType::Int,
                                stack_offset: None,
                            },
                        );
                    }
                    block.instructions.push(IrInstruction {
                        opcode: IrOpcode::MakeClosure,
                        result: Some(env_tmp.clone()),
                        result_type: Some(IrType::Int),
                        operands: env_operands,
                        jump_target: Some(mangled.clone()),
                        true_target: None,
                        false_target: None,
                        span: fd.span,
                    });
                    self.closure_envs.insert(tmp.clone(), env_tmp.clone());
                    self.closure_envs.insert(fd.signature.name.name.clone(), env_tmp);
                }

                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(fd.signature.name.name.clone()),
                    result_type: Some(func_type.clone()),
                    operands: vec![IrOperand::Variable(tmp, func_type.clone())],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: fd.span,
                });
            }
        }
    }

    pub fn visit_if_statement(&mut self, block: &mut IrBlock, block_stack: &mut Vec<IrBlock>, stmt: &IfStatement) {
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
        block_stack.push(then_block);

        let mut else_block = IrBlock {
            id: else_id,
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        if let Some(ref alt) = stmt.alternative {
            self.visit_statement(&mut else_block, block_stack, alt);
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
        block_stack.push(else_block);

        let entry_block = std::mem::replace(
            block,
            IrBlock {
                id: merge_id,
                instructions: Vec::new(),
                successors: Vec::new(),
            },
        );
        block_stack.push(entry_block);
    }

    pub fn visit_loop_statement(&mut self, block: &mut IrBlock, block_stack: &mut Vec<IrBlock>, stmt: &LoopStatement) {
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

        // Save entry block (with Jump) to stack, replace current_block with exit
        let entry_block = std::mem::replace(
            block,
            IrBlock {
                id: exit_id,
                instructions: Vec::new(),
                successors: Vec::new(),
            },
        );
        // Add blocks in correct order: entry, header, body, exit (current_block)
        block_stack.push(entry_block);

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

        self.visit_statement(&mut body_block, block_stack, &stmt.body);

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

fn ends_with_control_flow(block: &IrBlock) -> bool {
    block.instructions.last().is_some_and(|inst| {
        matches!(
            inst.opcode,
            IrOpcode::Ret | IrOpcode::Jump | IrOpcode::CondBr | IrOpcode::CoroYield
        )
    })
}

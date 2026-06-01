mod control;

use super::IrGenerator;
use crate::ast::Statement;
use crate::ir::{Constant, IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

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
                if !self.symbols.is_declared(&vd.name.name) {
                    let ir_ty = self.convert_type(&vd.ty);
                    self.symbols.define_local(&vd.name.name, ir_ty.clone());
                    if let crate::ast::TypeRef::Custom(id) = &vd.ty {
                        self.symbols.local_struct_types.insert(vd.name.name.clone(), id.name.clone());
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

                let saved_symbols = self.symbols.clone();
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

                let captures = Self::scan_captures(&fd.body, &fd.signature.parameters, &saved_symbols);
                let has_captures = !captures.is_empty();

                let mut inner_def = fd.clone();
                inner_def.signature.name.name = mangled.clone();

                if has_captures {
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

                self.symbols = saved_symbols;
                self.used_functions = saved_used;
                self.block_counter = saved_block;

                let func_type = IrType::Function(param_types, Box::new(ret_type));

                if !self.symbols.is_declared(&fd.signature.name.name) {
                    self.symbols.define_local(&fd.signature.name.name, func_type.clone());
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
                    for (i, _) in captures.iter().enumerate() {
                        let slot_name = format!("__env_slot_{i}");
                        self.symbols.define_local(&slot_name, IrType::Int);
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
}

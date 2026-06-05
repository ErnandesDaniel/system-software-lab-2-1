mod control;

use super::IrGenerator;
use crate::ast::{Arg, FuncDefinition, Identifier, Span, Statement};
use crate::ir::{Constant, IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_statement(&mut self, block: &mut IrBlock, stmt: &Statement) {
        match stmt {
            Statement::Return(ret) => self.visit_return(block, ret),
            Statement::If(if_stmt) => self.visit_if_statement(block, if_stmt),
            Statement::Loop(loop_stmt) => self.visit_loop_statement(block, loop_stmt),
            Statement::Repeat(repeat_stmt) => self.visit_repeat_statement(block, repeat_stmt),
            Statement::Expression(expr_stmt) => {
                self.visit_expr(block, &expr_stmt.expr);
            }
            Statement::Block(block_stmt) => {
                self.symbols.push_scope();
                for s in &block_stmt.body {
                    self.visit_statement(block, s);
                }
                self.symbols.pop_scope();
            }
            Statement::Break(_) => self.visit_break(block, stmt),
            Statement::VarDecl(vd) => self.visit_var_decl(vd),
            Statement::Yield(y) => self.handle_yield(block, y.span),
            Statement::FuncDef(fd) => self.handle_func_def(block, fd),
        }
    }

    fn visit_return(&mut self, block: &mut IrBlock, ret: &crate::ast::ReturnStatement) {
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

    fn visit_break(&mut self, block: &mut IrBlock, stmt: &Statement) {
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

    fn visit_var_decl(&mut self, vd: &crate::ast::VarDeclStatement) {
        let ir_ty = self.convert_type(&vd.ty);
        self.symbols.define_local(&vd.name.name, ir_ty.clone());
        match &vd.ty {
            crate::ast::TypeRef::Custom(id) => {
                self.symbols
                    .local_struct_types
                    .insert(vd.name.name.clone(), id.name.clone());
            }
            crate::ast::TypeRef::Array { element_type, .. } => {
                if let crate::ast::TypeRef::Custom(id) = element_type.as_ref() {
                    self.symbols
                        .local_struct_types
                        .insert(vd.name.name.clone(), id.name.clone());
                }
            }
            _ => {}
        }
    }

    fn handle_yield(&mut self, block: &mut IrBlock, span: Span) {
        self.current_yield_state += 1;
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::CoroYield,
            result: None,
            result_type: Some(IrType::Int),
            operands: vec![IrOperand::Constant(Constant::Int(self.current_yield_state as i64))],
            jump_target: None,
            true_target: None,
            false_target: None,
            span,
        });
        let new_id = format!("BB{}", self.block_counter);
        self.coroutine_state_blocks.push(new_id.clone());
        self.block_counter += 1;
        let old_block = std::mem::replace(block, IrBlock::new(new_id));
        self.block_stack.push(old_block);
    }

    fn handle_func_def(&mut self, block: &mut IrBlock, fd: &FuncDefinition) {
        let mangled = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        let saved_symbols = self.symbols.clone();
        let saved_used = self.used_functions.clone();
        let saved_block_counter = self.block_counter;
        let saved_loop_exit = self.loop_exit_stack.clone();
        let saved_loop_depth = self.loop_depth;

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

        let ir_func = self.generate_nested_func(fd, &mangled, &captures);
        self.pending_functions.push(ir_func);

        self.symbols = saved_symbols;
        self.used_functions = saved_used;
        self.block_counter = saved_block_counter;
        self.loop_exit_stack = saved_loop_exit;
        self.loop_depth = saved_loop_depth;

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
            self.emit_make_closure(block, fd, &mangled, &captures, &tmp, &func_type);
        }

        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Assign,
            result: Some(fd.signature.name.name.clone()),
            result_type: Some(func_type.clone()),
            operands: vec![IrOperand::Variable(tmp, func_type)],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: fd.span,
        });
    }

    fn generate_nested_func(
        &mut self,
        fd: &FuncDefinition,
        mangled: &str,
        captures: &[(String, usize)],
    ) -> crate::ir::IrFunction {
        self.captured_vars = captures.iter().cloned().collect();
        let mut inner_def = fd.clone();
        inner_def.signature.name.name = mangled.to_string();

        if !captures.is_empty() {
            let env_param = Arg {
                name: Identifier {
                    name: "__env".to_string(),
                    span: fd.span,
                },
                ty: None,
                span: fd.span,
            };
            let mut new_params = vec![env_param];
            if let Some(ref args) = inner_def.signature.parameters {
                new_params.extend(args.clone());
            }
            inner_def.signature.parameters = Some(new_params);
        }

        let ir_func = self.generate_function(&inner_def);
        self.captured_vars.clear();
        ir_func
    }

    fn emit_make_closure(
        &mut self,
        block: &mut IrBlock,
        fd: &FuncDefinition,
        mangled: &str,
        captures: &[(String, usize)],
        tmp: &str,
        _func_type: &IrType,
    ) {
        let env_tmp = self.generate_temp();
        let mut env_operands: Vec<IrOperand> = captures
            .iter()
            .map(|(name, _)| IrOperand::Variable(name.clone(), IrType::Int))
            .collect();
        env_operands.insert(0, IrOperand::FuncRef(mangled.to_string()));
        for (i, _) in captures.iter().enumerate() {
            self.symbols.define_local(&format!("__env_slot_{i}"), IrType::Int);
        }
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::MakeClosure,
            result: Some(env_tmp.clone()),
            result_type: Some(IrType::Int),
            operands: env_operands,
            jump_target: Some(mangled.to_string()),
            true_target: None,
            false_target: None,
            span: fd.span,
        });
        self.closure_envs.insert(tmp.to_string(), env_tmp.clone());
        self.closure_envs.insert(fd.signature.name.name.clone(), env_tmp);
    }
}

use super::IrGenerator;
use crate::ast::*;
use crate::ir::*;

impl IrGenerator {
    pub fn visit_expr(&mut self, block: &mut IrBlock, expr: &Expr) -> (String, IrType) {
        match expr {
            Expr::Binary(bin) => self.visit_binary_expr(block, bin),
            Expr::Unary(un) => self.visit_unary_expr(block, un),
            Expr::Parenthesized(inner) => self.visit_expr(block, inner),
            Expr::Call(call) => self.visit_call_expr(block, call),
            Expr::Slice(slice) => self.visit_slice_expr(block, slice),
            Expr::Identifier(id) => {
                // Check if this is a captured variable (closure env)
                if let Some(slot) = self.captured_vars.get(&id.name).copied() {
                    let tmp = self.generate_temp();
                    block.instructions.push(IrInstruction {
                        opcode: IrOpcode::LoadCaptured,
                        result: Some(tmp.clone()),
                        result_type: Some(IrType::Int),
                        operands: vec![
                            IrOperand::Variable("__env".to_string(), IrType::Int),
                            IrOperand::Constant(Constant::Int(slot as i64)),
                        ],
                        jump_target: None, true_target: None, false_target: None,
                        span: crate::ast::Span::new(0, 0),
                    });
                    return (tmp, IrType::Int);
                }
                if self.global_names.contains(&id.name) {
                    let ir_type = self.global_types
                        .get(&id.name)
                        .cloned()
                        .unwrap_or(IrType::Int);
                    if matches!(ir_type, IrType::Array(..)) {
                        // For array globals, return the name directly — Slice will use it as base
                        (id.name.clone(), ir_type)
                    } else {
                        let tmp = self.generate_temp();
                        block.instructions.push(IrInstruction {
                            opcode: IrOpcode::Load,
                            result: Some(tmp.clone()),
                            result_type: Some(ir_type.clone()),
                            operands: vec![IrOperand::Variable(id.name.clone(), ir_type.clone())],
                            jump_target: None,
                            true_target: None,
                            false_target: None,
                            span: crate::ast::Span::new(0, 0),
                        });
                        (tmp, ir_type)
                    }
                } else {
                    (id.name.clone(), self.get_ident_type(id))
                }
            }
            Expr::Literal(lit) => self.visit_literal_expr(block, lit),
            Expr::ArrayLiteral(elements) => {
                for elem in elements {
                    self.visit_expr(block, elem);
                }
                let tmp = self.generate_temp();
                (tmp, IrType::Int)
            }
            Expr::FuncLiteral(f) => {
                let mangled = format!("__lambda_{}", self.lambda_counter);
                self.lambda_counter += 1;

                // Save state
                let saved_locals = self.locals.clone();
                let saved_declared = self.declared_vars.clone();
                let saved_used = self.used_functions.clone();
                let saved_block = self.block_counter;

                // Build function type for this literal
                let param_types: Vec<IrType> = f.signature.parameters.as_ref().map(|args| {
                    args.iter().map(|a| a.ty.as_ref().map(|t| self.convert_type(t)).unwrap_or(IrType::Int)).collect()
                }).unwrap_or_default();
                let ret_type = f.signature.return_type.as_ref().map(|t| self.convert_type(t)).unwrap_or(IrType::Void);

                // Scan for captures before generating inner function
                let captures = self.scan_captures(&f.body, &f.signature.parameters, &saved_locals);
                let has_captures = !captures.is_empty();

                let mut inner_def = f.clone();
                inner_def.signature.name.name = mangled.clone();

                if has_captures {
                    // Add __env as hidden first parameter
                    let env_param = crate::ast::Arg {
                        name: crate::ast::Identifier { name: "__env".to_string(), span: crate::ast::Span::new(0, 0) },
                        ty: None,
                        span: crate::ast::Span::new(0, 0),
                    };
                    let mut new_params = vec![env_param];
                    if let Some(ref args) = inner_def.signature.parameters {
                        new_params.extend(args.clone());
                    }
                    inner_def.signature.parameters = Some(new_params);
                }

                // Set captured vars context for inner function generation
                self.captured_vars = captures.iter().cloned().collect();
                let ir_func = self.generate_function(&inner_def);
                self.pending_functions.push(ir_func);
                self.captured_vars.clear();

                // Restore state
                self.locals = saved_locals;
                self.declared_vars = saved_declared;
                self.used_functions = saved_used;
                self.block_counter = saved_block;

                if has_captures {
                    // Closure path: allocate env + store func ptr
                    let func_type = IrType::Function(param_types, Box::new(ret_type));
                    let func_tmp = self.generate_temp();
                    let env_tmp = self.generate_temp();

                    // Store func ptr
                    block.instructions.push(IrInstruction {
                        opcode: IrOpcode::Assign,
                        result: Some(func_tmp.clone()),
                        result_type: Some(func_type.clone()),
                        operands: vec![IrOperand::FuncRef(mangled.clone())],
                        jump_target: None, true_target: None, false_target: None,
                        span: f.span,
                    });

                    // Create closure env: stores addresses of captured vars
                    let mut env_operands: Vec<IrOperand> = captures.iter().map(|(name, _)| {
                        IrOperand::Variable(name.clone(), IrType::Int)
                    }).collect();
                    env_operands.insert(0, IrOperand::FuncRef(mangled.clone()));
                    // Add env slot locals so the frame size accounts for them
                    for (i, _) in captures.iter().enumerate() {
                        let slot_name = format!("__env_slot_{}", i);
                        self.locals.insert(slot_name.clone(), IrLocal {
                            name: slot_name,
                            ty: IrType::Int,
                            stack_offset: None,
                        });
                    }
                    block.instructions.push(IrInstruction {
                        opcode: IrOpcode::MakeClosure,
                        result: Some(env_tmp.clone()),
                        result_type: Some(IrType::Int),
                        operands: env_operands,
                        jump_target: Some(mangled.clone()),
                        true_target: None, false_target: None,
                        span: f.span,
                    });

                    // Track closure: func_tmp -> env_tmp mapping
                    self.closure_envs.insert(func_tmp.clone(), env_tmp.clone());

                    (func_tmp, func_type)
                } else {
                    // Non-closure path: just store func pointer (existing)
                    let func_type = IrType::Function(param_types, Box::new(ret_type));
                    let tmp = self.generate_temp();
                    block.instructions.push(IrInstruction {
                        opcode: IrOpcode::Assign,
                        result: Some(tmp.clone()),
                        result_type: Some(func_type.clone()),
                        operands: vec![IrOperand::FuncRef(mangled)],
                        jump_target: None, true_target: None, false_target: None,
                        span: f.span,
                    });
                    (tmp, func_type)
                }
            }
            Expr::FieldAccess(base, field) => {
                if let Expr::Slice(slice) = base.as_ref() {
                    // Array of structs: sched[i].field
                    let (arr_name, _) = self.visit_expr(block, &slice.array);
                    if let Some(range) = slice.ranges.first() {
                        let (idx_temp, _) = self.visit_expr(block, &range.start);
                        let field_offset = self.find_field_offset_for_array(&arr_name, &field.name);
                        let elem_size = self.struct_size_for_var(&arr_name);
                        let tmp = self.generate_temp();
                        block.instructions.push(IrInstruction {
                            opcode: IrOpcode::Load,
                            result: Some(tmp.clone()),
                            result_type: Some(IrType::Int),
                            operands: vec![
                                IrOperand::Variable(arr_name, IrType::Int),
                                IrOperand::Constant(Constant::Int(field_offset as i64)),
                                IrOperand::Variable(idx_temp, IrType::Int),
                                IrOperand::Constant(Constant::Int(elem_size as i64)),
                            ],
                            jump_target: None, true_target: None, false_target: None,
                            span: crate::ast::Span::new(0, 0),
                        });
                        return (tmp, IrType::Int);
                    }
                }
                let (base_name, total_offset) = self.resolve_field_chain(expr);
                let tmp = self.generate_temp();
                // Use resolved chain — base name + cumulative offset
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Load,
                    result: Some(tmp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![
                        IrOperand::Variable(base_name, IrType::Int),
                        IrOperand::Constant(Constant::Int(total_offset as i64)),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: crate::ast::Span::new(0, 0),
                });
                (tmp, IrType::Int)
            }
        }
    }

    pub fn visit_binary_expr(
        &mut self,
        block: &mut IrBlock,
        expr: &BinaryExpr,
    ) -> (String, IrType) {
        let left_temp = self.visit_expr(block, &expr.left).0;
        let (right_temp, right_type) = self.visit_expr(block, &expr.right);

        let result_temp = self.generate_temp();

        let (opcode, result_type) = match expr.operator {
            BinaryOp::Multiply => (IrOpcode::Mul, IrType::Int),
            BinaryOp::Divide => (IrOpcode::Div, IrType::Int),
            BinaryOp::Modulo => (IrOpcode::Mod, IrType::Int),
            BinaryOp::Add => (IrOpcode::Add, IrType::Int),
            BinaryOp::Subtract => (IrOpcode::Sub, IrType::Int),
            BinaryOp::Equal => (IrOpcode::Eq, IrType::Bool),
            BinaryOp::NotEqual => (IrOpcode::Ne, IrType::Bool),
            BinaryOp::Less => (IrOpcode::Lt, IrType::Bool),
            BinaryOp::Greater => (IrOpcode::Gt, IrType::Bool),
            BinaryOp::LessOrEqual => (IrOpcode::Le, IrType::Bool),
            BinaryOp::GreaterOrEqual => (IrOpcode::Ge, IrType::Bool),
            BinaryOp::And => (IrOpcode::And, IrType::Bool),
            BinaryOp::Or => (IrOpcode::Or, IrType::Bool),
            BinaryOp::Assign => {
                match expr.left.as_ref() {
                    Expr::FieldAccess(_, _) => {
                        let (base_name, total_offset) = self.resolve_field_chain(expr.left.as_ref());
                        block.instructions.push(IrInstruction {
                            opcode: IrOpcode::Store,
                            result: None,
                            result_type: None,
                            operands: vec![
                                IrOperand::Variable(base_name, IrType::Int),
                                IrOperand::Constant(Constant::Int(total_offset as i64)),
                                IrOperand::Variable(right_temp.clone(), right_type.clone()),
                            ],
                            jump_target: None,
                            true_target: None,
                            false_target: None,
                            span: expr.span,
                        });
                        return (right_temp, right_type);
                    }
                    Expr::Slice(slice) => {
                        // Struct array field assignment: scheduler.coroutines[i] = value
                        if let Expr::FieldAccess(_, _) = slice.array.as_ref() {
                            let (base_name, total_offset) = self.resolve_field_chain(slice.array.as_ref());
                            if let Some(range) = slice.ranges.first() {
                                let (idx, _) = self.visit_expr(block, &range.start);
                                block.instructions.push(IrInstruction {
                                    opcode: IrOpcode::Store,
                                    result: None,
                                    result_type: None,
                                    operands: vec![
                                        IrOperand::Variable(base_name, IrType::Int),
                                        IrOperand::Constant(Constant::Int(total_offset as i64)),
                                        IrOperand::Variable(right_temp.clone(), right_type.clone()),
                                        IrOperand::Variable(idx, IrType::Int),
                                    ],
                                    jump_target: None, true_target: None, false_target: None,
                                    span: expr.span,
                                });
                                return (right_temp, right_type);
                            }
                        }
                        let target_name = left_temp.clone();
                        let right_type = right_type.clone();
                        block.instructions.push(IrInstruction {
                            opcode: IrOpcode::Assign,
                            result: Some(target_name.clone()),
                            result_type: Some(right_type.clone()),
                            operands: vec![IrOperand::Variable(right_temp.clone(), right_type.clone())],
                            jump_target: None, true_target: None, false_target: None,
                            span: expr.span,
                        });
                        return (right_temp, right_type);
                    }
                    _ => {
                        let target_name = match expr.left.as_ref() {
                            Expr::Identifier(id) => id.name.clone(),
                            _ => left_temp.clone(),
                        };

                        let right_type = right_type.clone();

                        // Check if target is a captured variable (closure write)
                        if let Some(slot) = self.captured_vars.get(&target_name) {
                            block.instructions.push(IrInstruction {
                                opcode: IrOpcode::StoreCaptured,
                                result: None,
                                result_type: None,
                                operands: vec![
                                    IrOperand::Variable("__env".to_string(), IrType::Int),
                                    IrOperand::Constant(Constant::Int(*slot as i64)),
                                    IrOperand::Variable(right_temp.clone(), right_type.clone()),
                                ],
                                jump_target: None, true_target: None, false_target: None,
                                span: expr.span,
                            });
                            return (right_temp, right_type);
                        }

                        block.instructions.push(IrInstruction {
                            opcode: IrOpcode::Assign,
                            result: Some(target_name.clone()),
                            result_type: Some(right_type.clone()),
                            operands: vec![IrOperand::Variable(right_temp.clone(), right_type.clone())],
                            jump_target: None,
                            true_target: None,
                            false_target: None,
                            span: expr.span,
                        });

                        if !self.declared_vars.contains(&target_name) {
                            self.locals.insert(
                                target_name.clone(),
                                IrLocal {
                                    name: target_name.clone(),
                                    ty: right_type.clone(),
                                    stack_offset: None,
                                },
                            );
                            self.declared_vars.insert(target_name.clone());
                        }
                        // Propagate closure env from right temp to target variable
                        if self.closure_envs.contains_key(&right_temp) {
                            let env_tmp = self.closure_envs[&right_temp].clone();
                            self.closure_envs.insert(target_name.clone(), env_tmp);
                        }
                        return (right_temp, right_type);
                    }
                }
            }
        };

        block.instructions.push(IrInstruction {
            opcode,
            result: Some(result_temp.clone()),
            result_type: Some(result_type.clone()),
            operands: vec![
                IrOperand::Variable(left_temp, IrType::Int),
                IrOperand::Variable(right_temp, IrType::Int),
            ],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });

        (result_temp, result_type)
    }

    pub fn visit_unary_expr(&mut self, block: &mut IrBlock, expr: &UnaryExpr) -> (String, IrType) {
        let (operand_temp, _) = self.visit_expr(block, &expr.operand);

        let result_temp = self.generate_temp();

        let (opcode, result_type) = match expr.operator {
            UnaryOp::Negate => (IrOpcode::Neg, IrType::Int),
            UnaryOp::Not => (IrOpcode::Not, IrType::Bool),
            UnaryOp::BitNot => (IrOpcode::BitNot, IrType::Int),
        };

        block.instructions.push(IrInstruction {
            opcode,
            result: Some(result_temp.clone()),
            result_type: Some(result_type.clone()),
            operands: vec![IrOperand::Variable(operand_temp, IrType::Int)],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });

        (result_temp, result_type)
    }

    pub fn visit_call_expr(&mut self, block: &mut IrBlock, expr: &CallExpr) -> (String, IrType) {
        let func_name = match *expr.function.clone() {
            Expr::Identifier(ref id) => id.name.clone(),
            _ => String::new(),
        };

        // If calling a known function by name, use direct Call
        let is_direct = !func_name.is_empty()
            && (self.function_return_types.contains_key(&func_name)
                || self.is_external_function(&func_name));

        if is_direct {
            let mut args = Vec::new();
            for arg in &expr.arguments {
                let (temp, arg_type) = self.visit_expr(block, arg);
                args.push(IrOperand::Variable(temp, arg_type));
            }

            let result_return_type = self.function_return_types
                .get(&func_name)
                .cloned()
                .unwrap_or(IrType::Int);

            let is_void = matches!(result_return_type, IrType::Void);
            let result_temp = if is_void { String::new() } else { self.generate_temp() };

            block.instructions.push(IrInstruction {
                opcode: IrOpcode::Call,
                result: if is_void { None } else { Some(result_temp.clone()) },
                result_type: Some(result_return_type.clone()),
                operands: args,
                jump_target: Some(func_name.clone()),
                true_target: None,
                false_target: None,
                span: expr.span,
            });

            self.used_functions.push(func_name);
            (result_temp, result_return_type)
        } else {
            // Indirect call through function pointer or closure
            let (func_temp, func_ty) = self.visit_expr(block, &expr.function);

            // Check if this is a closure (has an associated env)
            if let Some(env_tmp) = self.closure_envs.get(&func_temp) {
                let mut return_type = IrType::Int;
                if let IrType::Function(_, ret) = &func_ty {
                    return_type = *ret.clone();
                }
                let mut operands = vec![
                    IrOperand::Variable(func_temp.clone(), func_ty.clone()),
                    IrOperand::Variable(env_tmp.clone(), IrType::Int),
                ];
                for arg in &expr.arguments {
                    let (temp, arg_type) = self.visit_expr(block, arg);
                    operands.push(IrOperand::Variable(temp, arg_type));
                }
                let is_void = matches!(return_type, IrType::Void);
                let result_temp = if is_void { String::new() } else { self.generate_temp() };
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::CallClosure,
                    result: if is_void { None } else { Some(result_temp.clone()) },
                    result_type: Some(return_type.clone()),
                    operands,
                    jump_target: None, true_target: None, false_target: None,
                    span: expr.span,
                });
                (result_temp, return_type)
            } else {
                // Regular indirect call
                let mut return_type = IrType::Int;
                if let IrType::Function(_, ret) = &func_ty {
                    return_type = *ret.clone();
                }
                let mut operands = vec![IrOperand::Variable(func_temp, func_ty)];

                for arg in &expr.arguments {
                    let (temp, arg_type) = self.visit_expr(block, arg);
                    operands.push(IrOperand::Variable(temp, arg_type));
                }

                let is_void = matches!(return_type, IrType::Void);
                let result_temp = if is_void { String::new() } else { self.generate_temp() };

                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::CallIndirect,
                    result: if is_void { None } else { Some(result_temp.clone()) },
                    result_type: Some(return_type.clone()),
                    operands,
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });

                (result_temp, return_type)
            }
        }
    }

    pub fn visit_slice_expr(&mut self, block: &mut IrBlock, expr: &SliceExpr) -> (String, IrType) {
        // Handle struct field array access: scheduler.coroutines[i]
        if let crate::ast::Expr::FieldAccess(_, _) = expr.array.as_ref() {
            let (base_name, total_offset) = self.resolve_field_chain(expr.array.as_ref());
            if let Some(range) = expr.ranges.first() {
                let (index_temp, _) = self.visit_expr(block, &range.start);
                let result_temp = self.generate_temp();
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Load,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![
                        IrOperand::Variable(base_name, IrType::Int),
                        IrOperand::Constant(Constant::Int(total_offset as i64)),
                        IrOperand::Variable(index_temp, IrType::Int),
                    ],
                    jump_target: None, true_target: None, false_target: None,
                    span: expr.span,
                });
                return (result_temp, IrType::Int);
            }
        }

        let (array_temp, array_type) = self.visit_expr(block, &expr.array);

        let element_type = match &array_type {
            IrType::Array(ref elem, _) => *elem.clone(),
            _ => IrType::Int,
        };

        if let Some(range) = expr.ranges.first() {
            let (start_temp, _) = self.visit_expr(block, &range.start);

            let result_temp = self.generate_temp();

            if range.end.is_some() {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Slice,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Array(Box::new(element_type.clone()), 0)),
                    operands: vec![
                        IrOperand::Variable(array_temp, array_type.clone()),
                        IrOperand::Variable(start_temp, IrType::Int),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
            } else {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Load,
                    result: Some(result_temp.clone()),
                    result_type: Some(element_type.clone()),
                    operands: vec![
                        IrOperand::Variable(array_temp, array_type.clone()),
                        IrOperand::Variable(start_temp, IrType::Int),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
            }

            return (result_temp, element_type);
        }

        (String::new(), IrType::Int)
    }

    pub fn visit_literal_expr(&mut self, block: &mut IrBlock, lit: &Literal) -> (String, IrType) {
        let result_temp = self.generate_temp();

        match lit {
            Literal::Bool(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Bool),
                    operands: vec![IrOperand::Constant(Constant::Bool(*v))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Bool)
            }
            Literal::Dec(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Int(*v as i64))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Int)
            }
            Literal::Hex(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Int(*v as i64))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Int)
            }
            Literal::Bits(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Int(*v as i64))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Int)
            }
            Literal::Char(c) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Char(*c as u8))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Int)
            }
            Literal::Str(s) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::String),
                    operands: vec![IrOperand::Constant(Constant::String(s.clone()))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::String)
            }
        }
    }
}

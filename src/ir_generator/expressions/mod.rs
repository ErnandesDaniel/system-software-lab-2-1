mod binary;
mod call;
pub(crate) mod literal;
mod slice;

use super::IrGenerator;
use crate::ast::{Expr, FuncDefinition, Identifier, Span, UnaryExpr, UnaryOp};
use crate::ir::{Constant, IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_expr(&mut self, block: &mut IrBlock, expr: &Expr) -> (String, IrType) {
        match expr {
            Expr::Binary(bin) => self.visit_binary_expr(block, bin),
            Expr::Unary(un) => self.visit_unary_expr(block, un),
            Expr::Parenthesized(inner) => self.visit_expr(block, inner),
            Expr::Call(call) => self.visit_call_expr(block, call),
            Expr::Slice(slice) => self.visit_slice_expr(block, slice),
            Expr::Identifier(id) => self.visit_identifier(block, id),
            Expr::Literal(lit, s) => self.visit_literal_expr(block, lit, *s),
            Expr::ArrayLiteral(elements, _s) => self.visit_array_literal(block, elements),
            Expr::FuncLiteral(f) => self.visit_func_literal(block, f),
            Expr::FieldAccess(base, field, _) => self.visit_field_access(block, base, field),
        }
    }

    pub fn visit_unary_expr(&mut self, block: &mut IrBlock, expr: &UnaryExpr) -> (String, IrType) {
        let (operand_temp, operand_type) = self.visit_expr(block, &expr.operand);

        let result_temp = self.generate_temp();
        let (opcode, result_type) = match expr.operator {
            UnaryOp::Negate => (IrOpcode::Neg, operand_type.clone()),
            UnaryOp::Not => (IrOpcode::Not, IrType::Bool),
            UnaryOp::BitNot => (IrOpcode::BitNot, operand_type.clone()),
        };

        block.instructions.push(IrInstruction {
            opcode,
            result: Some(result_temp.clone()),
            result_type: Some(result_type.clone()),
            operands: vec![IrOperand::Variable(operand_temp, operand_type)],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });

        (result_temp, result_type)
    }

    fn visit_identifier(&mut self, block: &mut IrBlock, id: &Identifier) -> (String, IrType) {
        let name = &id.name;

        // Captured variable → load from closure env
        if let Some(slot) = self.captured_vars.get(name).copied() {
            let captured_type = self.symbols.get_type(name);
            let tmp = self.generate_temp();
            block.instructions.push(IrInstruction {
                opcode: IrOpcode::LoadCaptured,
                result: Some(tmp.clone()),
                result_type: Some(captured_type.clone()),
                operands: vec![
                    IrOperand::Variable("__env".to_string(), IrType::Int),
                    IrOperand::Constant(Constant::Int(slot as i64)),
                ],
                jump_target: None,
                true_target: None,
                false_target: None,
                span: id.span,
            });
            return (tmp, captured_type);
        }

        // Global variable
        if let Some(g_type) = self.symbols.global_types.get(name) {
            let ir_type = g_type.clone();
            // Array-like globals (arrays, structs) → return address (no Load)
            if matches!(ir_type, IrType::Array(..) | IrType::Struct { .. }) {
                return (name.clone(), ir_type);
            }
            // Scalar globals → Load the value
            let tmp = self.generate_temp();
            block.instructions.push(IrInstruction::load(
                tmp.clone(),
                ir_type.clone(),
                IrOperand::Variable(name.clone(), ir_type.clone()),
                id.span,
            ));
            return (tmp, ir_type);
        }

        // Local variable or parameter
        let local_type = self.symbols.get_type(name);
        (name.clone(), local_type)
    }

    fn visit_array_literal(&mut self, block: &mut IrBlock, elements: &[Expr]) -> (String, IrType) {
        if elements.is_empty() {
            let t = self.generate_temp();
            let arr_type = IrType::Array(Box::new(IrType::Int), 0);
            self.symbols.define_local(&t, arr_type.clone());
            block.instructions.push(IrInstruction {
                opcode: IrOpcode::AllocArray,
                result: Some(t.clone()),
                result_type: Some(arr_type.clone()),
                operands: vec![],
                jump_target: None,
                true_target: None,
                false_target: None,
                span: Span::new(0, 0),
            });
            return (t, arr_type);
        }

        let arr_tmp = self.generate_temp();
        let mut elem_results: Vec<(String, IrType)> = Vec::new();
        for elem in elements {
            elem_results.push(self.visit_expr(block, elem));
        }

        let elem_type = elem_results[0].1.clone();
        let arr_type = IrType::Array(Box::new(elem_type.clone()), elements.len());
        self.symbols.define_local(&arr_tmp, arr_type.clone());

        let span = elements.first().map(|e| e.span()).unwrap_or(Span::new(0, 0));

        block.instructions.push(IrInstruction {
            opcode: IrOpcode::AllocArray,
            result: Some(arr_tmp.clone()),
            result_type: Some(arr_type.clone()),
            operands: vec![],
            jump_target: None,
            true_target: None,
            false_target: None,
            span,
        });

        for (i, (elem_temp, ty)) in elem_results.iter().enumerate() {
            let store_span = elements.get(i).map(|e| e.span()).unwrap_or(span);
            block.instructions.push(IrInstruction {
                opcode: IrOpcode::Store,
                result: None,
                result_type: None,
                operands: vec![
                    IrOperand::Variable(arr_tmp.clone(), arr_type.clone()),
                    IrOperand::Constant(Constant::Int(0)),
                    IrOperand::Variable(elem_temp.clone(), ty.clone()),
                    IrOperand::Constant(Constant::Int(i as i64)),
                ],
                jump_target: None,
                true_target: None,
                false_target: None,
                span: store_span,
            });
        }

        (arr_tmp, arr_type)
    }

    fn visit_func_literal(&mut self, block: &mut IrBlock, f: &FuncDefinition) -> (String, IrType) {
        let mangled = format!("__lambda_{}", self.lambda_counter);
        self.lambda_counter += 1;

        let saved_symbols = self.symbols.clone();
        let saved_used = self.used_functions.clone();
        let saved_block_counter = self.block_counter;
        let saved_loop_exit = self.loop_exit_stack.clone();
        let saved_loop_depth = self.loop_depth;

        let param_types: Vec<IrType> = f
            .signature
            .parameters
            .as_ref()
            .map(|args| {
                args.iter()
                    .map(|a| a.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t)))
                    .collect()
            })
            .unwrap_or_default();
        let ret_type = f
            .signature
            .return_type
            .as_ref()
            .map_or(IrType::Void, |t| self.convert_type(t));

        let captures = Self::scan_captures(&f.body, &f.signature.parameters, &saved_symbols);
        let has_captures = !captures.is_empty();

        let mut inner_def = f.clone();
        inner_def.signature.name.name = mangled.clone();

        if has_captures {
            let env_param = crate::ast::Arg {
                name: Identifier {
                    name: "__env".to_string(),
                    span: f.span,
                },
                ty: None,
                span: f.span,
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
        self.block_counter = saved_block_counter;
        self.loop_exit_stack = saved_loop_exit;
        self.loop_depth = saved_loop_depth;

        let func_type = IrType::Function(param_types, Box::new(ret_type));

        if has_captures {
            let func_tmp = self.generate_temp();
            let env_tmp = self.generate_temp();

            block.instructions.push(IrInstruction {
                opcode: IrOpcode::Assign,
                result: Some(func_tmp.clone()),
                result_type: Some(func_type.clone()),
                operands: vec![IrOperand::FuncRef(mangled.clone())],
                jump_target: None,
                true_target: None,
                false_target: None,
                span: f.span,
            });

            let mut env_operands: Vec<IrOperand> = captures
                .iter()
                .map(|(name, _)| IrOperand::Variable(name.clone(), IrType::Int))
                .collect();
            env_operands.insert(0, IrOperand::FuncRef(mangled.clone()));

            block.instructions.push(IrInstruction {
                opcode: IrOpcode::MakeClosure,
                result: Some(env_tmp.clone()),
                result_type: Some(IrType::Int),
                operands: env_operands,
                jump_target: Some(mangled.clone()),
                true_target: None,
                false_target: None,
                span: f.span,
            });

            self.closure_envs.insert(func_tmp.clone(), env_tmp.clone());
            (func_tmp, func_type)
        } else {
            let tmp = self.generate_temp();
            block.instructions.push(IrInstruction {
                opcode: IrOpcode::Assign,
                result: Some(tmp.clone()),
                result_type: Some(func_type.clone()),
                operands: vec![IrOperand::FuncRef(mangled)],
                jump_target: None,
                true_target: None,
                false_target: None,
                span: f.span,
            });
            (tmp, func_type)
        }
    }

    fn visit_field_access(&mut self, block: &mut IrBlock, base: &Expr, field: &Identifier) -> (String, IrType) {
        // Array-of-structs indexed access: arr[i].field
        if let Expr::Slice(slice) = base {
            if let Some(range) = slice.ranges.first() {
                let (idx_temp, _) = self.visit_expr(block, &range.start);
                let (base_name, base_type, total_offset, field_type, elem_size) =
                    self.resolve_indexed_field(&slice.array, &field.name);
                let tmp = self.generate_temp();
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Load,
                    result: Some(tmp.clone()),
                    result_type: Some(field_type.clone()),
                    operands: vec![
                        IrOperand::Variable(base_name, base_type),
                        IrOperand::Constant(Constant::Int(total_offset as i64)),
                        IrOperand::Variable(idx_temp, IrType::Int),
                        IrOperand::Constant(Constant::Int(elem_size)),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: field.span,
                });
                return (tmp, field_type);
            }
        }

        // Field access: base.field  (or a.b.c via resolve_field_chain)
        let (base_name, base_offset) = self.resolve_field_chain(base);
        let (base_type, total_offset, field_type) = self.resolve_field_info(&base_name, base, field, base_offset);

        let tmp = self.generate_temp();
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Load,
            result: Some(tmp.clone()),
            result_type: Some(field_type.clone()),
            operands: vec![
                IrOperand::Variable(base_name, base_type),
                IrOperand::Constant(Constant::Int(total_offset as i64)),
            ],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: field.span,
        });
        (tmp, field_type)
    }
}

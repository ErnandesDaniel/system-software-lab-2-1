mod binary;
mod call;
mod literal;
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
            Expr::FieldAccess(base, field) => self.visit_field_access(block, base, field),
        }
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

    fn visit_identifier(&mut self, block: &mut IrBlock, id: &Identifier) -> (String, IrType) {
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
                jump_target: None,
                true_target: None,
                false_target: None,
                span: id.span,
            });
            return (tmp, IrType::Int);
        }
        if self.symbols.global_types.contains_key(&id.name) {
            let ir_type = self.symbols.global_types.get(&id.name).cloned().unwrap_or(IrType::Int);
            if matches!(ir_type, IrType::Array(..)) {
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
                    span: id.span,
                });
                (tmp, ir_type)
            }
        } else {
            (id.name.clone(), self.get_ident_type(id))
        }
    }

    fn visit_array_literal(&mut self, block: &mut IrBlock, elements: &[Expr]) -> (String, IrType) {
        let arr_tmp = self.generate_temp();
        let mut elem_results: Vec<(String, IrType)> = Vec::new();
        for elem in elements {
            let res = self.visit_expr(block, elem);
            elem_results.push(res);
        }
        let elem_type = elem_results.first().map(|(_, t)| t.clone()).unwrap_or(IrType::Int);
        let arr_type = IrType::Array(Box::new(elem_type.clone()), elements.len());
        self.symbols.define_local(&arr_tmp, arr_type.clone());
        let arr_span = elements.first().map(|e| e.span()).unwrap_or(Span::new(0, 0));
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::AllocArray,
            result: Some(arr_tmp.clone()),
            result_type: Some(arr_type.clone()),
            operands: vec![],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: arr_span,
        });
        for (i, (elem_temp, ty)) in elem_results.iter().enumerate() {
            let store_span = elements.get(i).map(|e| e.span()).unwrap_or(arr_span);
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
        let saved_block = self.block_counter;

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
                name: Identifier { name: "__env".to_string(), span: f.span },
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
        self.block_counter = saved_block;

        if has_captures {
            let func_type = IrType::Function(param_types, Box::new(ret_type));
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
                span: f.span,
            });

            self.closure_envs.insert(func_tmp.clone(), env_tmp.clone());
            (func_tmp, func_type)
        } else {
            let func_type = IrType::Function(param_types, Box::new(ret_type));
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
        if let Expr::Slice(slice) = base {
            let (arr_name, _) = self.visit_expr(block, &slice.array);
            if let Some(range) = slice.ranges.first() {
                let (idx_temp, _) = self.visit_expr(block, &range.start);
                let field_offset = self.find_field_offset_for_array(&arr_name, &field.name);
                let field_type = self.find_field_type_for_var(&arr_name, &field.name);
                let elem_size = self.struct_size_for_var(&arr_name);
                let tmp = self.generate_temp();
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Load,
                    result: Some(tmp.clone()),
                    result_type: Some(field_type.clone()),
                    operands: vec![
                        IrOperand::Variable(arr_name, IrType::Int),
                        IrOperand::Constant(Constant::Int(field_offset as i64)),
                        IrOperand::Variable(idx_temp, IrType::Int),
                        IrOperand::Constant(Constant::Int(elem_size as i64)),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: field.span,
                });
                return (tmp, field_type);
            }
        }
        let (base_name, base_offset) = self.resolve_field_chain(base);
        let struct_name = self
            .symbols
            .local_struct_types
            .get(&base_name)
            .map(String::as_str)
            .or_else(|| self.symbols.global_struct_type_names.get(&base_name).map(String::as_str))
            .unwrap_or(&base_name);
        let field_offset = self
            .symbols
            .struct_fields
            .get(struct_name)
            .and_then(|fields| fields.iter().find(|(n, _, _)| n == &field.name))
            .map_or(0, |(_, _, o)| *o);
        let total_offset = base_offset + field_offset;
        let field_type = self.find_field_type_for_var(&base_name, &field.name);
        let tmp = self.generate_temp();
        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Load,
            result: Some(tmp.clone()),
            result_type: Some(field_type.clone()),
            operands: vec![
                IrOperand::Variable(base_name, IrType::Int),
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

use super::IrGenerator;
use crate::ast::CallExpr;
use crate::ir::{IrBlock, IrInstruction, IrOpcode, IrOperand, IrType};

impl IrGenerator {
    pub fn visit_call_expr(&mut self, block: &mut IrBlock, expr: &CallExpr) -> (String, IrType) {
        let func_name = match *expr.function.clone() {
            crate::ast::Expr::Identifier(ref id) => id.name.clone(),
            _ => String::new(),
        };

        let is_direct = !func_name.is_empty()
            && (self.symbols.function_return_types.contains_key(&func_name) || self.is_external_function(&func_name));

        if is_direct {
            let mut args = Vec::new();
            for arg in &expr.arguments {
                let (temp, arg_type) = self.visit_expr(block, arg);
                let is_fn = self.symbols.function_return_types.contains_key(&temp);
                if is_fn {
                    args.push(IrOperand::FuncRef(temp));
                } else {
                    args.push(IrOperand::Variable(temp, arg_type));
                }
            }

            let result_return_type = self
                .symbols
                .function_return_types
                .get(&func_name)
                .cloned()
                .or_else(|| {
                    crate::stdlib::StdLib::get_signature(&func_name).and_then(|(_, ret)| match ret {
                        "string" => Some(IrType::String),
                        "int" => Some(IrType::Int),
                        "" => Some(IrType::Void),
                        _ => None,
                    })
                })
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
            let (func_temp, func_ty) = self.visit_expr(block, &expr.function);

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
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
                (result_temp, return_type)
            } else {
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
}

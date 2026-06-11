use crate::ast::{BinaryExpr, BinaryOp, CallExpr, Expr, Identifier, SliceExpr, UnaryExpr, UnaryOp};
use crate::ir::IrType;
use crate::semantics::analysis::SemanticsAnalyzer;
use crate::semantics::types::SymbolTable;

impl SemanticsAnalyzer {
    pub fn check_expression(&mut self, scope: &mut SymbolTable, expr: &Expr) -> crate::Result<IrType> {
        match expr {
            Expr::Binary(bin) => self.check_binary_expr(scope, bin),
            Expr::Unary(un) => self.check_unary_expr(scope, un),
            Expr::Parenthesized(inner) => self.check_expression(scope, inner),
            Expr::Call(call) => self.check_call_expr(scope, call),
            Expr::Slice(slice) => self.check_slice_expr(scope, slice),
            Expr::Identifier(id) => Ok(self.check_identifier(scope, id)),
            Expr::Literal(lit, _) => Ok(Self::literal_type(lit)),
            Expr::ArrayLiteral(elems, _) => {
                let elem_type = if let Some(first) = elems.first() {
                    self.check_expression(scope, first)?
                } else {
                    IrType::Int
                };
                Ok(IrType::Array(Box::new(elem_type), elems.len()))
            }
            Expr::FieldAccess(base, field, _) => {
                let _base_type = self.check_expression(scope, base)?;
                let field_type = self.resolve_field_type(scope, base, &field.name);
                Ok(field_type)
            }
            Expr::FuncLiteral(f) => {
                let params = f
                    .signature
                    .parameters
                    .as_ref()
                    .map(|args| {
                        args.iter()
                            .map(|a| a.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t)))
                            .collect()
                    })
                    .unwrap_or_default();
                let ret = f
                    .signature
                    .return_type
                    .as_ref()
                    .map_or(IrType::Void, |t| self.convert_type(t));
                let mut inner_scope = scope.clone();
                inner_scope.push_scope();
                if let Some(ref args) = f.signature.parameters {
                    for arg in args {
                        let pty = arg.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t));
                        if let Err(e) = inner_scope.add(arg.name.name.clone(), pty) {
                            self.add_error(e.to_string(), arg.span);
                        }
                    }
                }
                let saved_return_type = self.current_return_type.take();
                self.current_return_type = Some(ret.clone());
                let saved_loop_depth = self.loop_depth;
                self.loop_depth = 0;
                for s in &f.body {
                    if let Err(e) = self.check_statement(&mut inner_scope, s) {
                        self.add_error(e.to_string(), s.span());
                    }
                }
                self.loop_depth = saved_loop_depth;
                self.current_return_type = saved_return_type;
                Ok(IrType::Function(params, Box::new(ret)))
            }
        }
    }

    fn check_binary_expr(&mut self, scope: &mut SymbolTable, bin: &BinaryExpr) -> crate::Result<IrType> {
        // For assignment, check if the left identifier needs auto-declaration
        let mut auto_declared = false;
        if matches!(bin.operator, BinaryOp::Assign) {
            if let Expr::Identifier(id) = &*bin.left {
                if self.get_global_symbol(&id.name).is_some() {
                    // Global variable — don't auto-declare locally
                } else if !scope.is_declared(&id.name) {
                    scope.add(id.name.clone(), IrType::Int)?;
                    auto_declared = true;
                }
            }
        }
        let left_type = self.check_expression(scope, &bin.left)?;
        let right_type = self.check_expression(scope, &bin.right)?;

        match bin.operator {
            BinaryOp::Assign => {
                if let Expr::Identifier(id) = &*bin.left {
                    if !auto_declared {
                        if let Some(existing) = scope.lookup(&id.name) {
                            if existing.ty != right_type {
                                self.add_error(
                                    format!(
                                        "Type mismatch: cannot assign {:?} to variable '{}' of type {:?}",
                                        right_type, id.name, existing.ty
                                    ),
                                    bin.span,
                                );
                            }
                        }
                    }
                    scope.upsert(id.name.clone(), right_type.clone());
                }
                Ok(right_type)
            }
            BinaryOp::Add => {
                if (left_type == IrType::String && right_type.is_int_like())
                    || (left_type.is_int_like() && right_type == IrType::String)
                {
                    Ok(IrType::String)
                } else if !left_type.is_int_like() || !right_type.is_int_like() {
                    self.add_error("Arithmetic operations require numeric operands".to_string(), bin.span);
                    Ok(IrType::Int)
                } else {
                    Ok(IrType::Int)
                }
            }
            BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor => {
                if !left_type.is_int_like() || !right_type.is_int_like() {
                    let msg = if matches!(bin.operator, BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor) {
                        "Bitwise operations require numeric operands"
                    } else {
                        "Arithmetic operations require numeric operands"
                    };
                    self.add_error(msg.to_string(), bin.span);
                }
                Ok(IrType::Int)
            }
            BinaryOp::Equal | BinaryOp::NotEqual => {
                if !left_type.is_int_like() && !left_type.is_bool() && left_type != IrType::String {
                    self.add_error(
                        "Equality comparison requires numeric, bool, or string operands".to_string(),
                        bin.span,
                    );
                }
                if !right_type.is_int_like() && !right_type.is_bool() && right_type != IrType::String {
                    self.add_error(
                        "Equality comparison requires numeric, bool, or string operands".to_string(),
                        bin.span,
                    );
                }
                Ok(IrType::Bool)
            }
            BinaryOp::Less | BinaryOp::Greater | BinaryOp::LessOrEqual | BinaryOp::GreaterOrEqual => {
                if !left_type.is_int_like() || !right_type.is_int_like() {
                    self.add_error("Comparison requires numeric operands".to_string(), bin.span);
                }
                Ok(IrType::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                if !left_type.is_bool() || !right_type.is_bool() {
                    self.add_error("Logical operations require bool operands".to_string(), bin.span);
                }
                Ok(IrType::Bool)
            }
        }
    }

    fn check_unary_expr(&mut self, scope: &mut SymbolTable, un: &UnaryExpr) -> crate::Result<IrType> {
        let operand_type = self.check_expression(scope, &un.operand)?;
        match un.operator {
            UnaryOp::Not => {
                if !operand_type.is_bool() {
                    self.add_error("Not operator requires bool operand".to_string(), un.span);
                }
                Ok(IrType::Bool)
            }
            UnaryOp::Negate | UnaryOp::BitNot => {
                if !operand_type.is_int_like() {
                    self.add_error(
                        "Unary arithmetic operators require numeric operand".to_string(),
                        un.span,
                    );
                }
                Ok(IrType::Int)
            }
        }
    }

    fn check_call_expr(&mut self, scope: &mut SymbolTable, call: &CallExpr) -> crate::Result<IrType> {
        if let Expr::Identifier(id) = call.function.as_ref() {
            let builtin_funcs = [
                "println", "putchar", "getchar", "rand", "time", "srand", "puts", "printf",
            ];
            if builtin_funcs.contains(&id.name.as_str()) {
                return Ok(IrType::Int);
            }

            if let Some(sig) = self.get_function_sig(&id.name).cloned() {
                let expected = sig.parameters.len();
                let actual = call.arguments.len();
                if expected != actual {
                    self.add_error(
                        format!("Function '{}' expected {} arguments, got {}", id.name, expected, actual),
                        call.span,
                    );
                }
                // Check argument types for named function calls
                for (i, arg) in call.arguments.iter().enumerate() {
                    if i >= sig.parameters.len() {
                        break;
                    }
                    let arg_type = self.check_expression(scope, arg)?;
                    let param_type = &sig.parameters[i].1;
                    let compatible = arg_type == *param_type
                        || (matches!(param_type, crate::ir::IrType::Int)
                            && matches!(arg_type, crate::ir::IrType::Function(_, _)));
                    if !compatible {
                        self.add_error(
                            format!(
                                "Argument {} of function '{}': expected type {:?}, got {:?}",
                                i + 1,
                                id.name,
                                param_type,
                                arg_type
                            ),
                            call.span,
                        );
                    }
                }
                return Ok(sig.return_type);
            }
        }

        let func_type = self.check_expression(scope, &call.function)?;

        if let IrType::Function(ref params, ref ret) = func_type {
            let expected = params.len();
            let actual = call.arguments.len();
            if expected != actual {
                self.add_error(
                    format!("Function expected {expected} arguments, got {actual}"),
                    call.span,
                );
            }
            for (i, arg) in call.arguments.iter().enumerate() {
                if i >= params.len() {
                    break;
                }
                let arg_type = self.check_expression(scope, arg)?;
                if arg_type != params[i] {
                    self.add_error(
                        format!(
                            "Argument {} type mismatch: expected {:?}, got {:?}",
                            i, params[i], arg_type
                        ),
                        call.span,
                    );
                }
            }
            return Ok(*ret.clone());
        }

        if let Expr::Identifier(id) = call.function.as_ref() {
            self.add_error(format!("Call to undefined function '{}'", id.name), call.span);
            return Ok(IrType::Int);
        }

        self.add_error("Call target is not a function".to_string(), call.span);
        Ok(IrType::Int)
    }

    fn check_slice_expr(&mut self, scope: &mut SymbolTable, slice: &SliceExpr) -> crate::Result<IrType> {
        let array_type = self.check_expression(scope, &slice.array)?;
        if let IrType::Array(elem, _) = array_type {
            if let Some(range) = slice.ranges.first() {
                let start_ty = self.check_expression(scope, &range.start)?;
                if !start_ty.is_int_like() {
                    self.add_error("Array index must be an integer".to_string(), range.span);
                }
                if let Some(ref end) = range.end {
                    let end_ty = self.check_expression(scope, end)?;
                    if !end_ty.is_int_like() {
                        self.add_error("Slice end must be an integer".to_string(), range.span);
                    }
                    return Ok(IrType::Array(Box::new(*elem.clone()), 0));
                }
            }
            return Ok(*elem);
        }
        if array_type == IrType::String {
            if let Some(range) = slice.ranges.first() {
                let start_ty = self.check_expression(scope, &range.start)?;
                if !start_ty.is_int_like() {
                    self.add_error("String index must be an integer".to_string(), range.span);
                }
            }
            return Ok(IrType::Int);
        }
        self.add_error("Cannot index non-array type".to_string(), slice.span);
        Ok(IrType::Int)
    }

    fn check_identifier(&mut self, scope: &mut SymbolTable, id: &Identifier) -> IrType {
        if let Some(symbol) = scope.lookup(&id.name) {
            return symbol.ty.clone();
        }
        if let Some(symbol) = self.get_global_symbol(&id.name) {
            return symbol.ty.clone();
        }
        self.add_error(format!("Undeclared identifier '{}'", id.name), id.span);
        IrType::Int
    }
}

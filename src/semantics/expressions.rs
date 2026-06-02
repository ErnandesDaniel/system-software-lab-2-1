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
            Expr::FieldAccess(base, field) => {
                let _base_type = self.check_expression(scope, base)?;
                let field_type = self.resolve_field_type(base, &field.name);
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
                Ok(IrType::Function(params, Box::new(ret)))
            }
        }
    }

    fn check_binary_expr(&mut self, scope: &mut SymbolTable, bin: &BinaryExpr) -> crate::Result<IrType> {
        if matches!(bin.operator, BinaryOp::Assign) {
            if let Expr::Identifier(id) = &*bin.left {
                scope.add(id.name.clone(), IrType::Int).ok();
            }
        }
        let left_type = self.check_expression(scope, &bin.left)?;
        let right_type = self.check_expression(scope, &bin.right)?;

        match bin.operator {
            BinaryOp::Assign => {
                if let Expr::Identifier(id) = &*bin.left {
                    scope.upsert(id.name.clone(), right_type.clone());
                }
                Ok(right_type)
            }
            BinaryOp::Add
            | BinaryOp::Subtract
            | BinaryOp::Multiply
            | BinaryOp::Divide
            | BinaryOp::Modulo
            | BinaryOp::BitAnd
            | BinaryOp::BitOr
            | BinaryOp::BitXor => {
                if !left_type.is_int_like() || !right_type.is_int_like() {
                    self.add_error(
                        if matches!(bin.operator, BinaryOp::BitAnd | BinaryOp::BitOr | BinaryOp::BitXor) {
                            "Bitwise operations require numeric operands"
                        } else {
                            "Arithmetic operations require numeric operands"
                        }.to_string(),
                    );
                }
                Ok(IrType::Int)
            }
            BinaryOp::Equal | BinaryOp::NotEqual => {
                if !left_type.is_int_like() && !left_type.is_bool() && left_type != IrType::String {
                    self.add_error("Equality comparison requires numeric, bool, or string operands".to_string());
                }
                if !right_type.is_int_like() && !right_type.is_bool() && right_type != IrType::String {
                    self.add_error("Equality comparison requires numeric, bool, or string operands".to_string());
                }
                Ok(IrType::Bool)
            }
            BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessOrEqual
            | BinaryOp::GreaterOrEqual => {
                if !left_type.is_int_like() || !right_type.is_int_like() {
                    self.add_error("Comparison requires numeric operands".to_string());
                }
                Ok(IrType::Bool)
            }
            BinaryOp::And | BinaryOp::Or => {
                if !left_type.is_bool() || !right_type.is_bool() {
                    self.add_error("Logical operations require bool operands".to_string());
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
                    self.add_error("Not operator requires bool operand".to_string());
                }
                Ok(IrType::Bool)
            }
            UnaryOp::Negate | UnaryOp::BitNot => {
                if !operand_type.is_int_like() {
                    self.add_error("Unary arithmetic operators require numeric operand".to_string());
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
                    self.add_error(format!(
                        "Function '{}' expected {} arguments, got {}",
                        id.name, expected, actual
                    ));
                }
                return Ok(sig.return_type);
            }
        }

        let func_type = self.check_expression(scope, &call.function)?;

        if let IrType::Function(ref params, ref ret) = func_type {
            let expected = params.len();
            let actual = call.arguments.len();
            if expected != actual {
                self.add_error(format!("Function expected {expected} arguments, got {actual}"));
            }
            for (i, arg) in call.arguments.iter().enumerate() {
                if i >= params.len() {
                    break;
                }
                let arg_type = self.check_expression(scope, arg)?;
                if arg_type != params[i] {
                    self.add_error(format!(
                        "Argument {} type mismatch: expected {:?}, got {:?}",
                        i, params[i], arg_type
                    ));
                }
            }
            return Ok(*ret.clone());
        }

        if let Expr::Identifier(id) = call.function.as_ref() {
            self.add_error(format!("Call to undefined function '{}'", id.name));
            return Ok(IrType::Int);
        }

        self.add_error("Call target is not a function".to_string());
        Ok(IrType::Int)
    }

    fn check_slice_expr(&mut self, scope: &mut SymbolTable, slice: &SliceExpr) -> crate::Result<IrType> {
        let array_type = self.check_expression(scope, &slice.array)?;
        if let IrType::Array(elem, _) = array_type {
            if let Some(range) = slice.ranges.first() {
                if range.end.is_some() {
                    return Ok(IrType::Array(Box::new(*elem.clone()), 0));
                }
            }
            return Ok(*elem);
        }
        Ok(IrType::Int)
    }

    fn check_identifier(&mut self, scope: &mut SymbolTable, id: &Identifier) -> IrType {
        if let Some(symbol) = scope.lookup(&id.name) {
            return symbol.ty.clone();
        }
        if let Some(symbol) = self.get_global_symbol(&id.name) {
            return symbol.ty.clone();
        }
        self.add_error(format!("Undeclared identifier '{}'", id.name));
        IrType::Int
    }
}

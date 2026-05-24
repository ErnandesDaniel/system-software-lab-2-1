use crate::ast::*;
use crate::semantics::analysis::SemanticsAnalyzer;
use crate::semantics::types::{SemanticType, SymbolTable};

impl SemanticsAnalyzer {
    pub fn check_expression(
        &mut self,
        scope: &mut SymbolTable,
        expr: &Expr,
    ) -> Result<SemanticType, Vec<String>> {
        match expr {
            Expr::Binary(bin) => self.check_binary_expr(scope, bin),
            Expr::Unary(un) => self.check_unary_expr(scope, un),
            Expr::Parenthesized(inner) => self.check_expression(scope, inner),
            Expr::Call(call) => self.check_call_expr(scope, call),
            Expr::Slice(slice) => self.check_slice_expr(scope, slice),
            Expr::Identifier(id) => self.check_identifier(scope, id),
            Expr::Literal(lit) => Ok(self.literal_type(lit)),
            Expr::ArrayLiteral(_) => Ok(SemanticType::Array(Box::new(SemanticType::Int), 0)),
            Expr::FieldAccess(_, _) => Ok(SemanticType::Int),
            Expr::FuncLiteral(f) => {
                let params = f.signature.parameters.as_ref().map(|args| {
                    args.iter().map(|a| a.ty.as_ref().map(|t| self.convert_type(t)).unwrap_or(SemanticType::Int)).collect()
                }).unwrap_or_default();
                let ret = f.signature.return_type.as_ref().map(|t| self.convert_type(t)).unwrap_or(SemanticType::Void);
                Ok(SemanticType::Function(params, Box::new(ret)))
            }
        }
    }

    fn check_binary_expr(
        &mut self,
        scope: &mut SymbolTable,
        bin: &BinaryExpr,
    ) -> Result<SemanticType, Vec<String>> {
        if matches!(bin.operator, BinaryOp::Assign) {
            if let Expr::Identifier(id) = &*bin.left {
                scope.add(id.name.clone(), SemanticType::Int).ok();
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
            | BinaryOp::Modulo => {
                if left_type != SemanticType::Int || right_type != SemanticType::Int {
                    self.add_error("Arithmetic operations require int operands".to_string());
                }
                Ok(SemanticType::Int)
            }
            BinaryOp::Equal
            | BinaryOp::NotEqual
            | BinaryOp::Less
            | BinaryOp::Greater
            | BinaryOp::LessOrEqual
            | BinaryOp::GreaterOrEqual => Ok(SemanticType::Bool),
            BinaryOp::And | BinaryOp::Or => {
                if left_type != SemanticType::Bool || right_type != SemanticType::Bool {
                    self.add_error("Logical operations require bool operands".to_string());
                }
                Ok(SemanticType::Bool)
            }
        }
    }

    fn check_unary_expr(
        &mut self,
        scope: &mut SymbolTable,
        un: &UnaryExpr,
    ) -> Result<SemanticType, Vec<String>> {
        let operand_type = self.check_expression(scope, &un.operand)?;
        match un.operator {
            UnaryOp::Not => {
                if operand_type != SemanticType::Bool {
                    self.add_error("Not operator requires bool operand".to_string());
                }
                Ok(SemanticType::Bool)
            }
            UnaryOp::Negate | UnaryOp::BitNot => {
                if operand_type != SemanticType::Int {
                    self.add_error("Unary arithmetic operators require int operand".to_string());
                }
                Ok(SemanticType::Int)
            }
        }
    }

    fn check_call_expr(
        &mut self,
        scope: &mut SymbolTable,
        call: &CallExpr,
    ) -> Result<SemanticType, Vec<String>> {
        if let Expr::Identifier(id) = call.function.as_ref() {
            let builtin_funcs = [
                "println", "putchar", "getchar", "rand", "time", "srand", "puts", "printf",
            ];
            if builtin_funcs.contains(&id.name.as_str()) {
                return Ok(SemanticType::Int);
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

        if let SemanticType::Function(ref params, ref ret) = func_type {
            let expected = params.len();
            let actual = call.arguments.len();
            if expected != actual {
                self.add_error(format!(
                    "Function expected {} arguments, got {}", expected, actual
                ));
            }
            for (i, arg) in call.arguments.iter().enumerate() {
                if i >= params.len() { break; }
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
            return Ok(SemanticType::Int);
        }

        self.add_error("Call target is not a function".to_string());
        Ok(SemanticType::Int)
    }

    fn check_slice_expr(
        &mut self,
        scope: &mut SymbolTable,
        slice: &SliceExpr,
    ) -> Result<SemanticType, Vec<String>> {
        let array_type = self.check_expression(scope, &slice.array)?;
        if let SemanticType::Array(elem, _) = array_type {
            if let Some(range) = slice.ranges.first() {
                if range.end.is_some() {
                    return Ok(SemanticType::Array(Box::new(*elem.clone()), 0));
                }
            }
            return Ok(*elem);
        }
        Ok(SemanticType::Int)
    }

    fn check_identifier(
        &mut self,
        scope: &mut SymbolTable,
        id: &Identifier,
    ) -> Result<SemanticType, Vec<String>> {
        if let Some(symbol) = scope.get(&id.name) {
            return Ok(symbol.ty.clone());
        }
        if let Some(symbol) = self.get_global_symbol(&id.name) {
            return Ok(symbol.ty.clone());
        }
        self.add_error(format!("Undeclared identifier '{}'", id.name));
        Ok(SemanticType::Int)
    }
}

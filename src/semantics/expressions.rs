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
        }
    }

    fn check_binary_expr(
        &mut self,
        scope: &mut SymbolTable,
        bin: &BinaryExpr,
    ) -> Result<SemanticType, Vec<String>> {
        let left_type = self.check_expression(scope, &bin.left)?;
        let right_type = self.check_expression(scope, &bin.right)?;

        match bin.operator {
            BinaryOp::Assign => {
                if let Expr::Identifier(id) = &*bin.left {
                    scope.insert(id.name.clone(), right_type.clone()).ok();
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
            UnaryOp::Negate | UnaryOp::Plus | UnaryOp::BitNot => {
                if operand_type != SemanticType::Int {
                    self.add_error("Unary arithmetic operators require int operand".to_string());
                }
                Ok(SemanticType::Int)
            }
        }
    }

    fn check_call_expr(
        &mut self,
        _scope: &mut SymbolTable,
        call: &CallExpr,
    ) -> Result<SemanticType, Vec<String>> {
        let func_name = match *call.function.clone() {
            Expr::Identifier(id) => id.name,
            _ => return Ok(SemanticType::Int),
        };

        let builtin_funcs = [
            "println", "putchar", "getchar", "rand", "time", "srand", "puts", "printf",
        ];
        if builtin_funcs.contains(&func_name.as_str()) {
            return Ok(SemanticType::Int);
        }

        let sig = self.get_function_sig(&func_name);
        if sig.is_none() {
            self.add_error(format!("Call to undefined function '{}'", func_name));
            return Ok(SemanticType::Int);
        }

        let sig = sig.unwrap();
        let expected = sig.parameters.len();
        let actual = call.arguments.len();
        let return_type = sig.return_type.clone();
        if expected != actual {
            self.add_error(format!(
                "Function '{}' expected {} arguments, got {}",
                func_name, expected, actual
            ));
        }

        Ok(return_type)
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
        Ok(SemanticType::Int)
    }
}

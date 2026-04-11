use crate::ast::*;
use crate::semantics::types::{SemanticType, SymbolTable};
use crate::stdlib::StdLib;

#[derive(Debug)]
pub struct FunctionSig {
    pub name: String,
    pub return_type: SemanticType,
    pub parameters: Vec<(String, SemanticType)>,
}

pub struct SemanticsAnalyzer {
    global_scope: SymbolTable,
    functions: Vec<FunctionSig>,
    errors: Vec<String>,
}

impl SemanticsAnalyzer {
    pub fn new() -> Self {
        Self {
            global_scope: SymbolTable::new(),
            functions: Vec::new(),
            errors: Vec::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> Result<(), Vec<String>> {
        self.collect_functions(program)?;
        self.check_functions(program)?;

        if !self.errors.is_empty() {
            Err(std::mem::take(&mut self.errors))
        } else {
            Ok(())
        }
    }

    fn collect_functions(&mut self, program: &Program) -> Result<(), Vec<String>> {
        for item in &program.items {
            match item {
                SourceItem::FuncDefinition(def) => {
                    let return_type = def
                        .signature
                        .return_type
                        .as_ref()
                        .map(|ty| self.convert_type(ty))
                        .unwrap_or(SemanticType::Void);
                    let mut params = Vec::new();

                    if let Some(ref args) = def.signature.parameters {
                        for arg in args {
                            let param_type = arg
                                .ty
                                .as_ref()
                                .map(|t| self.convert_type(t))
                                .unwrap_or(SemanticType::Int);
                            params.push((arg.name.name.clone(), param_type));
                        }
                    }

                    self.global_scope
                        .insert(def.signature.name.name.clone(), SemanticType::Unknown)
                        .ok();

                    self.functions.push(FunctionSig {
                        name: def.signature.name.name.clone(),
                        return_type,
                        parameters: params,
                    });
                }
                SourceItem::FuncDeclaration(decl) => {
                    let func_name = decl.signature.name.name.clone();

                    // Check if the function is in the standard library
                    if !StdLib::is_stdlib(&func_name) {
                        self.errors.push(format!(
                            "Error: '{}' is not a standard library function. Only C standard library functions can be declared as extern.",
                            func_name
                        ));
                    }

                    // If short form (no parameters/return type specified), get from stdlib
                    let (return_type, params) = if decl.signature.parameters.is_none()
                        && decl.signature.return_type.is_none()
                    {
                        if let Some((params_str, return_str)) = StdLib::get_signature(&func_name) {
                            let params = Self::parse_stdlib_params(params_str);
                            let return_type = match return_str {
                                "int" => SemanticType::Int,
                                "string" => SemanticType::String,
                                "" => SemanticType::Void,
                                _ => SemanticType::Int,
                            };
                            (return_type, params)
                        } else {
                            (SemanticType::Void, Vec::new())
                        }
                    } else {
                        let return_type = decl
                            .signature
                            .return_type
                            .as_ref()
                            .map(|ty| self.convert_type(ty))
                            .unwrap_or(SemanticType::Void);
                        let mut params = Vec::new();

                        if let Some(ref args) = decl.signature.parameters {
                            for arg in args {
                                let param_type = arg
                                    .ty
                                    .as_ref()
                                    .map(|t| self.convert_type(t))
                                    .unwrap_or(SemanticType::Int);
                                params.push((arg.name.name.clone(), param_type));
                            }
                        }
                        (return_type, params)
                    };

                    self.functions.push(FunctionSig {
                        name: decl.signature.name.name.clone(),
                        return_type,
                        parameters: params,
                    });
                }
            }
        }
        Ok(())
    }

    fn check_functions(&mut self, program: &Program) -> Result<(), Vec<String>> {
        for item in &program.items {
            if let SourceItem::FuncDefinition(def) = item {
                self.check_function(def)?;
            }
        }
        Ok(())
    }

    fn check_function(&mut self, def: &FuncDefinition) -> Result<(), Vec<String>> {
        let mut local_scope = SymbolTable::new();

        if let Some(ref params) = def.signature.parameters {
            for arg in params {
                let param_type = arg
                    .ty
                    .as_ref()
                    .map(|t| self.convert_type(t))
                    .unwrap_or(SemanticType::Int);
                local_scope.insert(arg.name.name.clone(), param_type).ok();
            }
        }

        for stmt in &def.body {
            self.check_statement(&mut local_scope, stmt)?;
        }

        Ok(())
    }

    fn check_statement(
        &mut self,
        scope: &mut SymbolTable,
        stmt: &Statement,
    ) -> Result<(), Vec<String>> {
        match stmt {
            Statement::Return(ret) => {
                if let Some(ref expr) = ret.expr {
                    self.check_expression(scope, expr)?;
                }
            }
            Statement::If(if_stmt) => {
                let cond_type = self.check_expression(scope, &if_stmt.condition)?;
                if cond_type != SemanticType::Bool {
                    self.errors
                        .push(format!("If condition must be bool, got {:?}", cond_type));
                }
                self.check_statement(scope, &if_stmt.consequence)?;
                if let Some(ref alt) = if_stmt.alternative {
                    self.check_statement(scope, alt)?;
                }
            }
            Statement::Loop(loop_stmt) => {
                let cond_type = self.check_expression(scope, &loop_stmt.condition)?;
                if cond_type != SemanticType::Bool {
                    self.errors
                        .push(format!("Loop condition must be bool, got {:?}", cond_type));
                }
                for s in &loop_stmt.body {
                    self.check_statement(scope, s)?;
                }
            }
            Statement::Repeat(repeat_stmt) => {
                let cond_type = self.check_expression(scope, &repeat_stmt.condition)?;
                if cond_type != SemanticType::Bool {
                    self.errors.push(format!(
                        "Repeat condition must be bool, got {:?}",
                        cond_type
                    ));
                }
                self.check_statement(scope, &repeat_stmt.body)?;
            }
            Statement::Expression(expr_stmt) => {
                self.check_expression(scope, &expr_stmt.expr)?;
            }
            Statement::Block(block_stmt) => {
                let mut inner_scope = scope.clone();
                for s in &block_stmt.body {
                    self.check_statement(&mut inner_scope, s)?;
                }
            }
            Statement::Break(_) => {}
        }
        Ok(())
    }

    fn check_expression(
        &mut self,
        scope: &mut SymbolTable,
        expr: &Expr,
    ) -> Result<SemanticType, Vec<String>> {
        match expr {
            Expr::Binary(bin) => {
                let left_type = self.check_expression(scope, &bin.left)?;
                let right_type = self.check_expression(scope, &bin.right)?;

                match bin.operator {
                    BinaryOp::Assign => {
                        if let Expr::Identifier(id) = &*bin.left {
                            scope.insert(id.name.clone(), right_type.clone()).ok();
                        }
                        return Ok(right_type);
                    }
                    BinaryOp::Add
                    | BinaryOp::Subtract
                    | BinaryOp::Multiply
                    | BinaryOp::Divide
                    | BinaryOp::Modulo => {
                        if left_type != SemanticType::Int || right_type != SemanticType::Int {
                            self.errors
                                .push("Arithmetic operations require int operands".to_string());
                        }
                        return Ok(SemanticType::Int);
                    }
                    BinaryOp::Equal
                    | BinaryOp::NotEqual
                    | BinaryOp::Less
                    | BinaryOp::Greater
                    | BinaryOp::LessOrEqual
                    | BinaryOp::GreaterOrEqual => {
                        return Ok(SemanticType::Bool);
                    }
                    BinaryOp::And | BinaryOp::Or => {
                        if left_type != SemanticType::Bool || right_type != SemanticType::Bool {
                            self.errors
                                .push("Logical operations require bool operands".to_string());
                        }
                        return Ok(SemanticType::Bool);
                    }
                }
            }
            Expr::Unary(un) => {
                let operand_type = self.check_expression(scope, &un.operand)?;
                match un.operator {
                    UnaryOp::Not => {
                        if operand_type != SemanticType::Bool {
                            self.errors
                                .push("Not operator requires bool operand".to_string());
                        }
                        return Ok(SemanticType::Bool);
                    }
                    UnaryOp::Negate | UnaryOp::Plus | UnaryOp::BitNot => {
                        if operand_type != SemanticType::Int {
                            self.errors
                                .push("Unary arithmetic operators require int operand".to_string());
                        }
                        return Ok(SemanticType::Int);
                    }
                }
            }
            Expr::Parenthesized(inner) => {
                return self.check_expression(scope, inner);
            }
            Expr::Call(call) => {
                let func_name = match *call.function.clone() {
                    Expr::Identifier(id) => id.name,
                    _ => return Ok(SemanticType::Int),
                };

                let sig = self.functions.iter().find(|f| f.name == func_name);
                if sig.is_none() {
                    self.errors
                        .push(format!("Call to undefined function '{}'", func_name));
                    return Ok(SemanticType::Int);
                }

                let sig = sig.unwrap();
                if sig.parameters.len() != call.arguments.len() {
                    self.errors.push(format!(
                        "Function '{}' expected {} arguments, got {}",
                        func_name,
                        sig.parameters.len(),
                        call.arguments.len()
                    ));
                }

                return Ok(sig.return_type.clone());
            }
            Expr::CreateThread(_) => {
                return Ok(SemanticType::Void);
            }
            Expr::Slice(slice) => {
                let array_type = self.check_expression(scope, &slice.array)?;
                if let SemanticType::Array(elem, _) = array_type {
                    if let Some(range) = slice.ranges.first() {
                        if range.end.is_some() {
                            return Ok(SemanticType::Array(Box::new(*elem.clone()), 0));
                        }
                    }
                    return Ok(*elem);
                }
                return Ok(SemanticType::Int);
            }
            Expr::Identifier(id) => {
                if let Some(symbol) = scope.get(&id.name) {
                    return Ok(symbol.ty.clone());
                }
                if let Some(symbol) = self.global_scope.get(&id.name) {
                    return Ok(symbol.ty.clone());
                }
                return Ok(SemanticType::Int);
            }
            Expr::Literal(lit) => {
                return Ok(self.literal_type(lit));
            }
        }
    }

    fn convert_type(&self, ty: &TypeRef) -> SemanticType {
        match ty {
            TypeRef::BuiltinType(bt) => match bt {
                BuiltinType::Bool => SemanticType::Bool,
                BuiltinType::String => SemanticType::String,
                BuiltinType::Byte
                | BuiltinType::Int
                | BuiltinType::Uint
                | BuiltinType::Long
                | BuiltinType::Ulong
                | BuiltinType::Char => SemanticType::Int,
            },
            TypeRef::Custom(_) => SemanticType::Int,
            TypeRef::Array {
                element_type, size, ..
            } => SemanticType::Array(Box::new(self.convert_type(element_type)), *size as usize),
        }
    }

    fn literal_type(&self, lit: &Literal) -> SemanticType {
        match lit {
            Literal::Bool(_) => SemanticType::Bool,
            Literal::Dec(_) | Literal::Hex(_) | Literal::Bits(_) | Literal::Char(_) => {
                SemanticType::Int
            }
            Literal::Str(_) => SemanticType::String,
        }
    }

    fn parse_stdlib_params(params_str: &str) -> Vec<(String, SemanticType)> {
        if params_str.is_empty() {
            return Vec::new();
        }
        params_str
            .split(", ")
            .map(|param| {
                let parts: Vec<&str> = param.split(": ").collect();
                let name = parts[0].to_string();
                let ty = match parts.get(1) {
                    Some(&"int") => SemanticType::Int,
                    Some(&"string") => SemanticType::String,
                    _ => SemanticType::Int,
                };
                (name, ty)
            })
            .collect()
    }
}

impl Default for SemanticsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

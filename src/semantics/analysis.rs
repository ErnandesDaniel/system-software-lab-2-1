use crate::ast::{BuiltinType, Expr, Literal, Program, Span, TypeRef};
use crate::error::CompilerError;
use crate::ir::IrType;
use crate::semantics::types::SymbolTable;

#[derive(Debug, Clone)]
pub struct FunctionSig {
    pub name: String,
    pub return_type: IrType,
    pub parameters: Vec<(String, IrType)>,
}

pub struct SemanticsAnalyzer {
    pub(crate) global_scope: SymbolTable,
    pub(crate) functions: Vec<FunctionSig>,
    pub(crate) errors: Vec<(String, Span)>,
    pub(crate) current_return_type: Option<IrType>,
    pub(crate) loop_depth: usize,
    pub(crate) struct_fields: std::collections::HashMap<String, Vec<(String, IrType)>>,
}

impl SemanticsAnalyzer {
    pub fn new() -> Self {
        Self {
            global_scope: SymbolTable::new(),
            functions: Vec::new(),
            errors: Vec::new(),
            current_return_type: None,
            loop_depth: 0,
            struct_fields: std::collections::HashMap::new(),
        }
    }

    pub fn analyze(&mut self, program: &Program) -> crate::Result<()> {
        self.collect_functions(program);
        self.check_functions(program)?;

        if self.errors.is_empty() {
            Ok(())
        } else {
            let msgs: Vec<String> = self
                .errors
                .iter()
                .map(|(msg, span)| format!("{} at {}..{}", msg, span.start, span.end))
                .collect();
            Err(CompilerError::Semantic(msgs.join("; ")))
        }
    }

    pub fn convert_type(&mut self, ty: &TypeRef) -> IrType {
        match ty {
            TypeRef::BuiltinType(bt) => match bt {
                BuiltinType::Bool => IrType::Bool,
                BuiltinType::Byte => IrType::Byte,
                BuiltinType::Int => IrType::Int,
                BuiltinType::Uint => IrType::Uint,
                BuiltinType::Long => IrType::Long,
                BuiltinType::Ulong => IrType::Ulong,
                BuiltinType::Char => IrType::Char,
                BuiltinType::String => IrType::String,
            },
            TypeRef::Custom(id) => {
                if let Some(fields) = self.struct_fields.get(&id.name) {
                    let total_size = fields.iter().map(|(_, t)| t.size() as usize).sum();
                    let typed_fields: Vec<(String, IrType)> =
                        fields.iter().map(|(n, t)| (n.clone(), t.clone())).collect();
                    IrType::Struct {
                        name: id.name.clone(),
                        fields: typed_fields,
                        size: total_size,
                    }
                } else {
                    self.add_error(format!("Undeclared type '{}'", id.name), id.span);
                    IrType::Int
                }
            }
            TypeRef::Array { element_type, size, .. } => {
                IrType::Array(Box::new(self.convert_type(element_type)), *size as usize)
            }
            TypeRef::Function {
                params, return_type, ..
            } => {
                let p: Vec<IrType> = params.iter().map(|t| self.convert_type(t)).collect();
                IrType::Function(p, Box::new(self.convert_type(return_type)))
            }
        }
    }

    pub fn literal_type(lit: &Literal) -> IrType {
        match lit {
            Literal::Bool(_) => IrType::Bool,
            Literal::Dec(_) | Literal::Hex(_) | Literal::Bits(_) => IrType::Int,
            Literal::Char(_) => IrType::Char,
            Literal::Str(_) => IrType::String,
        }
    }

    pub fn add_error(&mut self, msg: String, span: Span) {
        self.errors.push((msg, span));
    }

    pub fn get_function_sig(&self, name: &str) -> Option<&FunctionSig> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn get_global_symbol(&self, name: &str) -> Option<&crate::ir::IrLocal> {
        self.global_scope.lookup(name)
    }

    /// Resolve the type of a field access by looking up the base expression's type
    /// in the local or global scope, then finding the struct field definition.
    pub fn resolve_field_type(&mut self, scope: &SymbolTable, base_expr: &Expr, field: &str) -> IrType {
        let base_type = self.infer_expr_type(scope, base_expr);
        self.resolve_field_type_by_ir_type(&base_type, field, base_expr.span())
    }

    pub fn resolve_field_type_by_ir_type(&mut self, base_type: &IrType, field: &str, span: Span) -> IrType {
        if let IrType::Struct { name: _, fields, .. } = base_type {
            for (fname, fty) in fields {
                if fname == field {
                    return fty.clone();
                }
            }
        }
        self.add_error(format!("Field '{}' not found on type {:?}", field, base_type), span);
        IrType::Int
    }

    fn infer_expr_type(&mut self, scope: &SymbolTable, expr: &Expr) -> IrType {
        match expr {
            Expr::Identifier(id) => {
                if let Some(sym) = scope.lookup(&id.name) {
                    return sym.ty.clone();
                }
                if let Some(sym) = self.get_global_symbol(&id.name) {
                    return sym.ty.clone();
                }
                IrType::Int
            }
            Expr::FieldAccess(base, field, _) => {
                let base_ty = self.infer_expr_type(scope, base);
                self.resolve_field_type_by_ir_type(&base_ty, &field.name, field.span)
            }
            Expr::Binary(bin) => {
                let _left = self.infer_expr_type(scope, &bin.left);
                let _right = self.infer_expr_type(scope, &bin.right);
                match bin.operator {
                    crate::ast::BinaryOp::Assign => _right,
                    crate::ast::BinaryOp::Add
                    | crate::ast::BinaryOp::Subtract
                    | crate::ast::BinaryOp::Multiply
                    | crate::ast::BinaryOp::Divide
                    | crate::ast::BinaryOp::Modulo
                    | crate::ast::BinaryOp::BitAnd
                    | crate::ast::BinaryOp::BitOr
                    | crate::ast::BinaryOp::BitXor => IrType::Int,
                    crate::ast::BinaryOp::Equal
                    | crate::ast::BinaryOp::NotEqual
                    | crate::ast::BinaryOp::Less
                    | crate::ast::BinaryOp::Greater
                    | crate::ast::BinaryOp::LessOrEqual
                    | crate::ast::BinaryOp::GreaterOrEqual
                    | crate::ast::BinaryOp::And
                    | crate::ast::BinaryOp::Or => IrType::Bool,
                }
            }
            Expr::Unary(un) => match un.operator {
                crate::ast::UnaryOp::Not => IrType::Bool,
                crate::ast::UnaryOp::Negate | crate::ast::UnaryOp::BitNot => IrType::Int,
            },
            Expr::Parenthesized(inner) => self.infer_expr_type(scope, inner),
            Expr::Call(call) => {
                if let Expr::Identifier(id) = call.function.as_ref() {
                    if let Some(sig) = self.get_function_sig(&id.name) {
                        return sig.return_type.clone();
                    }
                }
                if let Expr::Identifier(id) = call.function.as_ref() {
                    let builtin = [
                        "println", "putchar", "getchar", "rand", "time", "srand", "puts", "printf",
                    ];
                    if builtin.contains(&id.name.as_str()) {
                        return IrType::Int;
                    }
                }
                let func_ty = self.infer_expr_type(scope, &call.function);
                if let IrType::Function(_, ret) = func_ty {
                    return *ret;
                }
                IrType::Int
            }
            Expr::Slice(slice) => {
                let arr_ty = self.infer_expr_type(scope, &slice.array);
                if let IrType::Array(elem, _) = arr_ty {
                    if let Some(range) = slice.ranges.first() {
                        if range.end.is_some() {
                            return IrType::Array(elem, 0);
                        }
                    }
                    return *elem;
                }
                IrType::Int
            }
            Expr::Literal(lit, _) => Self::literal_type(lit),
            Expr::ArrayLiteral(elems, _) => {
                let elem_ty = elems.first().map_or(IrType::Int, |e| self.infer_expr_type(scope, e));
                IrType::Array(Box::new(elem_ty), elems.len())
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
                IrType::Function(params, Box::new(ret))
            }
        }
    }
}

impl Default for SemanticsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

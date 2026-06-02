use crate::ast::{BuiltinType, Literal, Program, TypeRef};
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
    pub(crate) errors: Vec<String>,
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
        self.collect_functions(program)?;
        self.check_functions(program)?;

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(CompilerError::Semantic(std::mem::take(&mut self.errors).join("; ")))
        }
    }

    pub fn convert_type(&self, ty: &TypeRef) -> IrType {
        match ty {
            TypeRef::BuiltinType(bt) => match bt {
                BuiltinType::Bool => IrType::Bool,
                BuiltinType::String => IrType::String,
                BuiltinType::Byte | BuiltinType::Int
                | BuiltinType::Uint | BuiltinType::Long
                | BuiltinType::Ulong | BuiltinType::Char => IrType::Int,
            },
            TypeRef::Custom(id) => {
                if self.struct_fields.contains_key(&id.name) {
                    IrType::Int
                } else {
                    IrType::Int
                }
            }
            TypeRef::Array { element_type, size, .. } => {
                IrType::Array(Box::new(self.convert_type(element_type)), *size as usize)
            }
            TypeRef::Function { params, return_type, .. } => {
                let p: Vec<IrType> = params.iter().map(|t| self.convert_type(t)).collect();
                IrType::Function(p, Box::new(self.convert_type(return_type)))
            }
        }
    }

    pub fn literal_type(lit: &Literal) -> IrType {
        match lit {
            Literal::Bool(_) => IrType::Bool,
            Literal::Dec(_) | Literal::Hex(_) | Literal::Bits(_) => IrType::Int,
            Literal::Char(_) => IrType::Int,
            Literal::Str(_) => IrType::String,
        }
    }

    pub fn add_error(&mut self, msg: String) {
        self.errors.push(msg);
    }

    pub fn get_function_sig(&self, name: &str) -> Option<&FunctionSig> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn get_global_symbol(&self, name: &str) -> Option<&crate::semantics::types::Symbol> {
        self.global_scope.get(name)
    }

    pub fn resolve_field_type(&self, base_expr: &crate::ast::Expr, field: &str) -> IrType {
        let base_name = match base_expr {
            crate::ast::Expr::Identifier(id) => Some(id.name.as_str()),
            crate::ast::Expr::FieldAccess(inner, _) => {
                match inner.as_ref() {
                    crate::ast::Expr::Identifier(id) => Some(id.name.as_str()),
                    _ => None,
                }
            }
            _ => None,
        };

        if let Some(name) = base_name {
            for (sname, fields) in &self.struct_fields {
                if name == sname || name == sname.as_str() {
                    for (fname, fty) in fields {
                        if fname == field {
                            return fty.clone();
                        }
                    }
                }
            }
        }
        IrType::Int
    }
}

impl Default for SemanticsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

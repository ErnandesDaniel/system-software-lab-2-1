use crate::ast::*;
use crate::semantics::types::{SemanticType, SymbolTable};

#[derive(Debug)]
pub struct FunctionSig {
    pub name: String,
    pub return_type: SemanticType,
    pub parameters: Vec<(String, SemanticType)>,
}

pub struct SemanticsAnalyzer {
    pub(crate) global_scope: SymbolTable,
    pub(crate) functions: Vec<FunctionSig>,
    pub(crate) errors: Vec<String>,
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

    pub fn convert_type(&self, ty: &TypeRef) -> SemanticType {
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

    pub fn literal_type(&self, lit: &Literal) -> SemanticType {
        match lit {
            Literal::Bool(_) => SemanticType::Bool,
            Literal::Dec(_) | Literal::Hex(_) | Literal::Bits(_) | Literal::Char(_) => {
                SemanticType::Int
            }
            Literal::Str(_) => SemanticType::String,
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
}

impl Default for SemanticsAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

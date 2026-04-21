use crate::ast::*;
use crate::semantics::analysis::{FunctionSig, SemanticsAnalyzer};
use crate::semantics::types::{SemanticType, SymbolTable};
use crate::stdlib::StdLib;

impl SemanticsAnalyzer {
    pub fn collect_functions(&mut self, program: &Program) -> Result<(), Vec<String>> {
        for item in &program.items {
            match item {
                SourceItem::FuncDefinition(def) => {
                    self.collect_func_definition(def)?;
                }
                SourceItem::FuncDeclaration(decl) => {
                    self.collect_func_declaration(decl)?;
                }
            }
        }
        Ok(())
    }

    fn collect_func_definition(&mut self, def: &FuncDefinition) -> Result<(), Vec<String>> {
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
        Ok(())
    }

    fn collect_func_declaration(&mut self, decl: &FuncDeclaration) -> Result<(), Vec<String>> {
        let func_name = decl.signature.name.name.clone();

        // Check if the function is in the standard library
        if !StdLib::is_stdlib(&func_name) {
            self.add_error(format!(
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
        Ok(())
    }

    pub fn check_functions(&mut self, program: &Program) -> Result<(), Vec<String>> {
        for item in &program.items {
            if let SourceItem::FuncDefinition(def) = item {
                self.check_function(def)?;
            }
        }
        Ok(())
    }

    pub fn check_function(&mut self, def: &FuncDefinition) -> Result<(), Vec<String>> {
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

    pub fn parse_stdlib_params(params_str: &str) -> Vec<(String, SemanticType)> {
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

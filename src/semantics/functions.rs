use crate::ast::{FuncDeclaration, FuncDefinition, Program, SourceItem};
use crate::ir::IrType;
use crate::semantics::analysis::{FunctionSig, SemanticsAnalyzer};
use crate::semantics::types::SymbolTable;
use crate::stdlib::StdLib;

impl SemanticsAnalyzer {
    pub fn collect_functions(&mut self, program: &Program) -> crate::Result<()> {
        for item in &program.items {
            match item {
                SourceItem::FuncDefinition(def) => {
                    self.collect_func_definition(def);
                }
                SourceItem::FuncDeclaration(decl) => {
                    self.collect_func_declaration(decl);
                }
                SourceItem::GlobalDecl(global) => {
                    let ty = self.convert_type(&global.ty);
                    if let Err(e) = self.global_scope.add(global.name.name.clone(), ty) {
                        self.add_error(e.to_string(), global.span);
                    }
                }
                SourceItem::StructDef(s) => {
                    let mut fields = Vec::new();
                    for f in &s.fields {
                        let fty = self.convert_type(&f.ty);
                        fields.push((f.name.name.clone(), fty));
                    }
                    self.struct_fields.insert(s.name.name.clone(), fields);
                }
                SourceItem::CoroutineDef(_) => {}
            }
        }
        Ok(())
    }

    fn collect_func_definition(&mut self, def: &FuncDefinition) {
        let return_type = def
            .signature
            .return_type
            .as_ref()
            .map_or(IrType::Void, |ty| self.convert_type(ty));
        let mut params = Vec::new();

        if let Some(ref args) = def.signature.parameters {
            for arg in args {
                let param_type = arg.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t));
                params.push((arg.name.name.clone(), param_type));
            }
        }

        let sem_params: Vec<IrType> = params.iter().map(|(_, t)| t.clone()).collect();
        let func_name = def.signature.name.name.clone();
        if let Err(e) = self.global_scope.add(
            func_name.clone(),
            IrType::Function(sem_params, Box::new(return_type.clone())),
        ) {
            self.add_error(e.to_string(), def.span);
            return;
        }

        self.functions.push(FunctionSig {
            name: func_name,
            return_type,
            parameters: params,
        });
    }

    fn collect_func_declaration(&mut self, decl: &FuncDeclaration) {
        let func_name = decl.signature.name.name.clone();

        if !StdLib::is_stdlib(&func_name) {
            self.add_error(format!(
                "Error: '{func_name}' is not a standard library function. Only C standard library functions can be declared as extern."
            ), decl.span);
        }

        let (return_type, params) = if decl.signature.parameters.is_none() && decl.signature.return_type.is_none() {
            if let Some((params_str, return_str)) = StdLib::get_signature(&func_name) {
                let params = Self::parse_stdlib_params(params_str);
                let return_type = match return_str {
                    "string" => IrType::String,
                    _ => IrType::Int,
                };
                (return_type, params)
            } else {
                (IrType::Void, Vec::new())
            }
        } else {
            let return_type = decl
                .signature
                .return_type
                .as_ref()
                .map_or(IrType::Void, |ty| self.convert_type(ty));
            let mut params = Vec::new();

            if let Some(ref args) = decl.signature.parameters {
                for arg in args {
                    let param_type = arg.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t));
                    params.push((arg.name.name.clone(), param_type));
                }
            }
            (return_type, params)
        };

        let sem_params: Vec<IrType> = params.iter().map(|(_, t)| t.clone()).collect();
        self.functions.push(FunctionSig {
            name: decl.signature.name.name.clone(),
            return_type: return_type.clone(),
            parameters: params,
        });

        if let Err(e) = self.global_scope.add(
            decl.signature.name.name.clone(),
            IrType::Function(sem_params, Box::new(return_type)),
        ) {
            self.add_error(e.to_string(), decl.span);
        }
    }

    pub fn check_functions(&mut self, program: &Program) -> crate::Result<()> {
        for item in &program.items {
            match item {
                SourceItem::FuncDefinition(def) => {
                    self.in_coroutine = false;
                    self.check_function(def)?;
                }
                SourceItem::CoroutineDef(coro) => {
                    self.in_coroutine = true;
                    self.check_coroutine(coro)?;
                    self.in_coroutine = false;
                }
                _ => {}
            }
        }
        Ok(())
    }

    pub fn check_function(&mut self, def: &FuncDefinition) -> crate::Result<()> {
        let mut local_scope = SymbolTable::new();

        if let Some(ref params) = def.signature.parameters {
            for arg in params {
                let param_type = arg.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t));
                local_scope.add(arg.name.name.clone(), param_type)?;
            }
        }

        let ret_type = def
            .signature
            .return_type
            .as_ref()
            .map_or(IrType::Void, |t| self.convert_type(t));
        self.current_return_type = Some(ret_type);

        for stmt in &def.body {
            self.check_statement(&mut local_scope, stmt)?;
        }

        self.current_return_type = None;

        Ok(())
    }

    pub fn check_coroutine(&mut self, def: &crate::ast::CoroutineDefinition) -> crate::Result<()> {
        let mut local_scope = SymbolTable::new();

        if let Some(ref params) = def.signature.parameters {
            for arg in params {
                let param_type = arg.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t));
                local_scope.add(arg.name.name.clone(), param_type)?;
            }
        }

        let ret_type = def
            .signature
            .return_type
            .as_ref()
            .map_or(IrType::Void, |t| self.convert_type(t));
        self.current_return_type = Some(ret_type);

        for stmt in &def.body {
            self.check_statement(&mut local_scope, stmt)?;
        }

        self.current_return_type = None;
        Ok(())
    }

    pub fn parse_stdlib_params(params_str: &str) -> Vec<(String, IrType)> {
        if params_str.is_empty() {
            return Vec::new();
        }
        params_str
            .split(", ")
            .map(|param| {
                let parts: Vec<&str> = param.split(": ").collect();
                let name = parts[0].to_string();
                let ty = match parts.get(1) {
                    Some(&"string") => IrType::String,
                    _ => IrType::Int,
                };
                (name, ty)
            })
            .collect()
    }
}

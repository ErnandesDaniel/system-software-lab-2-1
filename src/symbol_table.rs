use crate::ir::{IrLocal, IrType};
use std::collections::{HashMap, HashSet};

/// Represents a single lexical scope with local variable bindings
#[derive(Debug, Clone)]
struct Scope {
    locals: HashMap<String, IrLocal>,
    declared: HashSet<String>,
}

impl Scope {
    fn new() -> Self {
        Self {
            locals: HashMap::new(),
            declared: HashSet::new(),
        }
    }
}

/// Full signature of a function
#[derive(Debug, Clone)]
pub struct FunctionSignature {
    pub name: String,
    pub return_type: IrType,
    pub param_names: Vec<String>,
    pub param_types: Vec<IrType>,
    pub is_extern: bool,
}

/// Unified symbol table used by:
/// - Semantic analyser (type checking)
/// - IR generator (variable resolution, struct layout)
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,

    /// Struct field definitions: struct_name -> Vec<(field_name, field_type, byte_offset)>
    pub struct_fields: HashMap<String, Vec<(String, IrType, usize)>>,

    /// Function signatures indexed by name
    pub function_sigs: HashMap<String, FunctionSignature>,

    /// Return types of functions (convenience access)
    pub function_return_types: HashMap<String, IrType>,

    /// Global variable types
    pub global_types: HashMap<String, IrType>,

    /// Map from global variable name to its struct type name (if any)
    pub global_struct_type_names: HashMap<String, String>,

    /// Map from local variable name to its struct type name (if any)
    pub local_struct_types: HashMap<String, String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            struct_fields: HashMap::new(),
            function_sigs: HashMap::new(),
            function_return_types: HashMap::new(),
            global_types: HashMap::new(),
            global_struct_type_names: HashMap::new(),
            local_struct_types: HashMap::new(),
        }
    }

    // ── Scope management ──────────────────────────────────────────

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    // ── Locals ────────────────────────────────────────────────────

    pub fn add(&mut self, name: String, ty: IrType) -> crate::Result<()> {
        let scope = self
            .scopes
            .last_mut()
            .ok_or_else(|| crate::error::CompilerError::Semantic("No active scope".to_string()))?;
        if scope.declared.contains(&name) {
            return Err(crate::error::CompilerError::Semantic(format!(
                "Symbol '{name}' already exists in this scope"
            )));
        }
        let n = name.clone();
        scope.declared.insert(n);
        scope.locals.insert(
            name.clone(),
            IrLocal {
                name,
                ty,
                stack_offset: None,
            },
        );
        Ok(())
    }

    pub fn upsert(&mut self, name: String, ty: IrType) {
        // Search from innermost to outermost for existing variable
        for scope in self.scopes.iter_mut().rev() {
            if scope.declared.contains(&name) {
                scope.locals.insert(
                    name.clone(),
                    IrLocal {
                        name,
                        ty,
                        stack_offset: None,
                    },
                );
                return;
            }
        }
        // Not found in any scope — add to innermost
        if let Some(scope) = self.scopes.last_mut() {
            let n = name.clone();
            scope.declared.insert(n);
            scope.locals.insert(
                name.clone(),
                IrLocal {
                    name,
                    ty,
                    stack_offset: None,
                },
            );
        }
    }

    pub fn define_local(&mut self, name: &str, ty: IrType) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.declared.insert(name.to_string());
            scope.locals.insert(
                name.to_string(),
                IrLocal {
                    name: name.to_string(),
                    ty,
                    stack_offset: None,
                },
            );
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&IrLocal> {
        for scope in self.scopes.iter().rev() {
            if let Some(local) = scope.locals.get(name) {
                return Some(local);
            }
        }
        None
    }

    pub fn get_type(&self, name: &str) -> IrType {
        self.lookup(name).map_or(IrType::Int, |l| l.ty.clone())
    }

    pub fn is_declared(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.declared.contains(name))
    }

    pub fn reset_locals(&mut self) {
        self.scopes = vec![Scope::new()];
        self.local_struct_types.clear();
    }

    pub fn all_locals(&self) -> Vec<IrLocal> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        for scope in self.scopes.iter().rev() {
            for (name, local) in &scope.locals {
                if seen.insert(name.clone()) {
                    result.push(local.clone());
                }
            }
        }
        result
    }

    // ── Function signatures ───────────────────────────────────────

    pub fn register_function(
        &mut self,
        name: &str,
        return_type: IrType,
        param_names: Vec<String>,
        param_types: Vec<IrType>,
        is_extern: bool,
    ) {
        let sig = FunctionSignature {
            name: name.to_string(),
            return_type: return_type.clone(),
            param_names,
            param_types: param_types.clone(),
            is_extern,
        };
        self.function_sigs.insert(name.to_string(), sig);
        self.function_return_types.insert(name.to_string(), return_type);
    }

    pub fn get_function_sig(&self, name: &str) -> Option<&FunctionSignature> {
        self.function_sigs.get(name)
    }

    // ── Struct layout ─────────────────────────────────────────────

    pub fn register_struct(&mut self, name: &str, fields: Vec<(String, IrType)>) {
        let mut field_data = Vec::new();
        let mut offset: usize = 0;
        for (fname, fty) in &fields {
            let size = fty.size() as usize;
            field_data.push((fname.clone(), fty.clone(), offset));
            offset += size;
        }
        self.struct_fields.insert(name.to_string(), field_data);
    }

    pub fn get_struct_fields(&self, name: &str) -> Option<&[(String, IrType, usize)]> {
        self.struct_fields.get(name).map(|v| v.as_slice())
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

use crate::ir::{IrLocal, IrType};
use std::collections::{HashMap, HashSet};

/// A single scope frame in the symbol table.
#[derive(Debug, Clone)]
struct Scope {
    locals: HashMap<String, IrLocal>,
    declared: HashSet<String>,
}

/// Hierarchical symbol table with scope chaining.
///
/// Professional compilers maintain a stack of scopes so that
/// variable shadowing in nested blocks works correctly and
/// each scope can be entered/left independently.
#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    /// Struct definitions: struct_name → [(field_name, field_type, byte_offset)]
    pub struct_fields: HashMap<String, Vec<(String, IrType, usize)>>,
    /// Function return types keyed by function name
    pub function_return_types: HashMap<String, IrType>,
    /// Global variable names and their types
    pub global_types: HashMap<String, IrType>,
    /// For globals that are arrays of structs: global_name → struct_type_name
    pub global_struct_type_names: HashMap<String, String>,
    /// Local variables that hold structs: var_name → struct_type_name
    pub local_struct_types: HashMap<String, String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            scopes: vec![Scope::new()],
            struct_fields: HashMap::new(),
            function_return_types: HashMap::new(),
            global_types: HashMap::new(),
            global_struct_type_names: HashMap::new(),
            local_struct_types: HashMap::new(),
        }
    }

    // ── Scope management ──

    pub fn enter_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn leave_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    // ── Variable registration & lookup ──

    pub fn declare_local(&mut self, name: &str, ty: IrType) {
        let scope = self.scopes.last_mut().expect("no scope");
        if !scope.declared.contains(name) {
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

    pub fn define_local(&mut self, name: &str, ty: IrType) {
        let scope = self.scopes.last_mut().expect("no scope");
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

    /// Look up a variable in all scopes (current first, then parent).
    pub fn lookup(&self, name: &str) -> Option<IrLocal> {
        for scope in self.scopes.iter().rev() {
            if let Some(local) = scope.locals.get(name) {
                return Some(local.clone());
            }
        }
        None
    }

    pub fn is_declared(&self, name: &str) -> bool {
        self.scopes.iter().rev().any(|s| s.declared.contains(name))
    }

    pub fn get_type(&self, name: &str) -> IrType {
        self.lookup(name).map_or(IrType::Int, |l| l.ty)
    }

    pub fn is_global(&self, name: &str) -> bool {
        self.global_types.contains_key(name)
    }

    pub fn get_global_type(&self, name: &str) -> IrType {
        self.global_types.get(name).cloned().unwrap_or(IrType::Int)
    }

    // ── Struct support ──

    pub fn register_struct(&mut self, name: &str, fields: Vec<(String, IrType, usize)>) {
        self.struct_fields.insert(name.to_string(), fields);
    }

    pub fn get_struct_fields(&self, name: &str) -> &[(String, IrType, usize)] {
        self.struct_fields
            .get(name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    pub fn find_field_offset(&self, struct_name: &str, field: &str) -> usize {
        self.struct_fields
            .get(struct_name)
            .and_then(|fields| fields.iter().find(|(n, _, _)| n == field))
            .map_or(0, |(_, _, o)| *o)
    }

    pub fn find_field_type(&self, struct_name: &str, field: &str) -> IrType {
        self.struct_fields
            .get(struct_name)
            .and_then(|fields| fields.iter().find(|(n, _, _)| n == field))
            .map_or(IrType::Int, |(_, t, _)| t.clone())
    }

    pub fn struct_size(&self, struct_name: &str) -> usize {
        self.struct_fields
            .get(struct_name)
            .and_then(|fields| fields.last())
            .map_or(4, |(_, last_type, last_offset)| {
                last_offset + last_type.size() as usize
            })
    }

    /// Resolve a field access like `a.b.c` to the base variable name and total byte offset.
    pub fn resolve_field_chain(&self, base_var: &str, field_name: &str) -> (String, usize) {
        let struct_name = self
            .local_struct_types
            .get(base_var)
            .map(String::as_str)
            .or_else(|| self.global_struct_type_names.get(base_var).map(String::as_str));

        if let Some(sname) = struct_name {
            let offset = self.find_field_offset(sname, field_name);
            (base_var.to_string(), offset)
        } else {
            (base_var.to_string(), 0)
        }
    }

    /// Collect all declared locals from all scopes (for function frame setup).
    pub fn all_locals(&self) -> Vec<IrLocal> {
        let mut result = Vec::new();
        let mut seen = HashSet::new();
        for scope in &self.scopes {
            for (name, local) in &scope.locals {
                if seen.insert(name.clone()) {
                    result.push(local.clone());
                }
            }
        }
        result
    }
}

impl Scope {
    fn new() -> Self {
        Self {
            locals: HashMap::new(),
            declared: HashSet::new(),
        }
    }
}

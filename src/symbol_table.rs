use crate::ir::{IrLocal, IrType};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone)]
struct Scope {
    locals: HashMap<String, IrLocal>,
    declared: HashSet<String>,
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<Scope>,
    pub struct_fields: HashMap<String, Vec<(String, IrType, usize)>>,
    pub function_return_types: HashMap<String, IrType>,
    pub global_types: HashMap<String, IrType>,
    pub global_struct_type_names: HashMap<String, String>,
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

    pub fn push_scope(&mut self) {
        self.scopes.push(Scope::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn add(&mut self, name: String, ty: IrType) -> crate::Result<()> {
        let scope = self.scopes.last_mut().expect("no scope");
        if scope.declared.contains(&name) {
            return Err(crate::error::CompilerError::Semantic(
                format!("Symbol '{name}' already exists in this scope"),
            ));
        }
        let n = name.clone();
        scope.declared.insert(n);
        scope.locals.insert(name.clone(), IrLocal { name, ty, stack_offset: None });
        Ok(())
    }

    pub fn upsert(&mut self, name: String, ty: IrType) {
        let scope = self.scopes.last_mut().expect("no scope");
        let n = name.clone();
        scope.declared.insert(n);
        scope.locals.insert(name.clone(), IrLocal { name, ty, stack_offset: None });
    }

    pub fn define_local(&mut self, name: &str, ty: IrType) {
        let scope = self.scopes.last_mut().expect("no scope");
        scope.declared.insert(name.to_string());
        scope.locals.insert(
            name.to_string(),
            IrLocal { name: name.to_string(), ty, stack_offset: None },
        );
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

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

impl Scope {
    fn new() -> Self {
        Self { locals: HashMap::new(), declared: HashSet::new() }
    }
}

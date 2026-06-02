use crate::ir::IrType;
use std::collections::HashMap;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: IrType,
    pub stack_offset: Option<i32>,
}

impl Symbol {
    pub fn new(name: String, ty: IrType) -> Self {
        Self { name, ty, stack_offset: None }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self { scopes: vec![HashMap::new()] }
    }

    #[allow(dead_code)]
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    #[allow(dead_code)]
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn add(&mut self, name: String, ty: IrType) -> crate::Result<()> {
        let scope = self.scopes.last_mut().expect("no scope");
        if scope.contains_key(&name) {
            return Err(crate::error::CompilerError::Semantic(
                format!("Symbol '{name}' already exists in this scope"),
            ));
        }
        scope.insert(name.clone(), Symbol::new(name, ty));
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.lookup(name)
    }

    pub fn upsert(&mut self, name: String, ty: IrType) {
        let scope = self.scopes.last_mut().expect("no scope");
        scope.insert(name.clone(), Symbol::new(name, ty));
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

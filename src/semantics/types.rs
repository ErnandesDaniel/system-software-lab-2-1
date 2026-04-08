pub use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticType {
    Int,
    Bool,
    String,
    Array(Box<SemanticType>, usize),
    Void,
    Unknown,
}

impl SemanticType {
    pub fn is_array(&self) -> bool {
        matches!(self, SemanticType::Array(_, _))
    }

    pub fn element_type(&self) -> Option<SemanticType> {
        match self {
            SemanticType::Array(elem, _) => Some(*elem.clone()),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub ty: SemanticType,
    pub stack_offset: Option<i32>,
}

impl Symbol {
    pub fn new(name: String, ty: SemanticType) -> Self {
        Self {
            name,
            ty,
            stack_offset: None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SymbolTable {
    symbols: HashMap<String, Symbol>,
}

impl SymbolTable {
    pub fn new() -> Self {
        Self {
            symbols: HashMap::new(),
        }
    }

    pub fn insert(&mut self, name: String, ty: SemanticType) -> Result<(), String> {
        if self.symbols.contains_key(&name) {
            return Err(format!("Symbol '{}' already exists", name));
        }
        self.symbols.insert(name.clone(), Symbol::new(name, ty));
        Ok(())
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        self.symbols.get(name)
    }

    pub fn get(&self, name: &str) -> Option<&Symbol> {
        self.lookup(name)
    }

    pub fn contains(&self, name: &str) -> bool {
        self.symbols.contains_key(name)
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

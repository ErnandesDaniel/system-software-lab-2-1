pub use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum SemanticType {
    Int,
    Uint,
    Long,
    Ulong,
    Byte,
    Char,
    Bool,
    String,
    Array(Box<SemanticType>, usize),
    Function(Vec<SemanticType>, Box<SemanticType>),
    Void,
}

impl SemanticType {
    pub fn is_int_like(&self) -> bool {
        matches!(
            self,
            SemanticType::Int
                | SemanticType::Uint
                | SemanticType::Long
                | SemanticType::Ulong
                | SemanticType::Byte
                | SemanticType::Char
        )
    }
}

#[derive(Debug, Clone)]
pub struct Symbol {
    #[allow(dead_code)]
    pub name: String,
    pub ty: SemanticType,
    #[allow(dead_code)]
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

    pub fn add(&mut self, name: String, ty: SemanticType) -> crate::Result<()> {
        if self.symbols.contains_key(&name) {
            return Err(crate::error::CompilerError::Semantic(format!("Symbol '{name}' already exists")));
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

    pub fn upsert(&mut self, name: String, ty: SemanticType) {
        self.symbols.insert(name.clone(), Symbol::new(name, ty));
    }
}

impl Default for SymbolTable {
    fn default() -> Self {
        Self::new()
    }
}

use crate::ast::*;
use crate::ir::*;
use crate::stdlib::StdLib;
use std::collections::{HashMap, HashSet};

mod expressions;
mod statements;

pub struct IrGenerator {
    pub temp_counter: usize,
    pub block_counter: usize,
    pub locals: HashMap<String, IrLocal>,
    pub declared_vars: HashSet<String>,
    pub used_functions: Vec<String>,
    pub loop_exit_stack: Vec<String>,
    pub loop_depth: usize,
    pub external_functions: HashSet<String>,
}

impl IrGenerator {
    pub fn new() -> Self {
        Self {
            temp_counter: 0,
            block_counter: 0,
            locals: HashMap::new(),
            declared_vars: HashSet::new(),
            used_functions: Vec::new(),
            loop_exit_stack: Vec::new(),
            loop_depth: 0,
            external_functions: HashSet::new(),
        }
    }

    #[allow(dead_code)]
    pub fn is_external_function(&self, name: &str) -> bool {
        self.external_functions.contains(name) || StdLib::is_stdlib(name)
    }

    pub fn generate_temp(&mut self) -> String {
        let temp = format!("t{}", self.temp_counter);
        self.temp_counter += 1;
        temp
    }

    pub fn generate_block_id(&mut self) -> String {
        let id = format!("BB{}", self.block_counter);
        self.block_counter += 1;
        id
    }

    pub fn generate(&mut self, program: &Program) -> IrProgram {
        let mut functions = Vec::new();

        for item in &program.items {
            if let SourceItem::FuncDeclaration(decl) = item {
                self.external_functions
                    .insert(decl.signature.name.name.clone());
            }
        }

        for item in &program.items {
            if let SourceItem::FuncDefinition(def) = item {
                self.block_counter = 0;
                self.declared_vars.clear();
                let ir_func = self.generate_function(def);
                functions.push(ir_func);
            }
        }

        IrProgram { functions }
    }

    pub fn generate_function(&mut self, def: &FuncDefinition) -> IrFunction {
        let return_type = match &def.signature.return_type {
            Some(ty) => self.convert_type(ty),
            None => IrType::Void,
        };

        let mut parameters = Vec::new();
        let mut locals = HashMap::new();

        if let Some(ref params) = def.signature.parameters {
            for (_i, arg) in params.iter().enumerate() {
                let ty = match &arg.ty {
                    Some(t) => self.convert_type(t),
                    None => IrType::Int,
                };

                parameters.push(IrParameter {
                    name: arg.name.name.clone(),
                    ty: ty.clone(),
                });

                locals.insert(
                    arg.name.name.clone(),
                    IrLocal {
                        name: arg.name.name.clone(),
                        ty,
                        stack_offset: None,
                    },
                );

                self.declared_vars.insert(arg.name.name.clone());
            }
        }

        let mut blocks = Vec::new();
        let mut block_stack: Vec<IrBlock> = Vec::new();

        let entry_id = self.generate_block_id();
        let mut current_block = IrBlock {
            id: entry_id,
            instructions: Vec::new(),
            successors: Vec::new(),
        };

        self.locals = locals.clone();

        for stmt in &def.body {
            self.visit_statement(&mut current_block, &mut block_stack, stmt);
        }

        // Collect all blocks in order
        // For functions: entry block first, then stack blocks
        // For if/loop statements, blocks in stack are already in correct order
        blocks.push(current_block);
        
        for block in block_stack.drain(..) {
            blocks.push(block);
        }

        let mut func = IrFunction {
            name: def.signature.name.name.clone(),
            return_type,
            parameters,
            blocks,
            locals: self.locals.clone().into_values().collect(),
            used_functions: self.used_functions.clone(),
        };

        func.used_functions = self.used_functions.clone();
        func
    }

    pub fn get_ident_type(&self, id: &Identifier) -> IrType {
        self.locals
            .get(&id.name)
            .map(|l| l.ty.clone())
            .unwrap_or(IrType::Int)
    }

    pub fn convert_type(&self, ty: &TypeRef) -> IrType {
        match ty {
            TypeRef::BuiltinType(bt) => match bt {
                BuiltinType::Bool => IrType::Bool,
                BuiltinType::Byte
                | BuiltinType::Int
                | BuiltinType::Uint
                | BuiltinType::Long
                | BuiltinType::Ulong
                | BuiltinType::Char => IrType::Int,
                BuiltinType::String => IrType::String,
            },
            TypeRef::Custom(_) => IrType::Int,
            TypeRef::Array {
                element_type, size, ..
            } => IrType::Array(Box::new(self.convert_type(element_type)), *size as usize),
        }
    }

}

#[allow(dead_code)]
impl Span {
    fn span(&self) -> Span {
        *self
    }
}

impl Statement {
    fn span(&self) -> Span {
        match self {
            Statement::Return(s) => s.span,
            Statement::If(s) => s.span,
            Statement::Loop(s) => s.span,
            Statement::Repeat(s) => s.span,
            Statement::Break(s) => s.span,
            Statement::Expression(s) => s.span,
            Statement::Block(s) => s.span,
        }
    }
}

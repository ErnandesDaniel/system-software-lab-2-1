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

    pub fn is_external_function(&self, name: &str) -> bool {
        // Function is external if:
        // 1. It's declared as extern def in the source code
        // 2. It's a stdlib function (C runtime)
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

        // First, collect all external function declarations
        for item in &program.items {
            if let SourceItem::FuncDeclaration(decl) = item {
                self.external_functions
                    .insert(decl.signature.name.name.clone());
            }
        }

        for item in &program.items {
            if let SourceItem::FuncDefinition(def) = item {
                // Reset block counter and declared vars for each function
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

                // Track declared variables
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

        // After processing all statements:
        // - current_block contains post-loop code
        // - block_stack contains: [header, body, init_jmp, exit] (in that order from visit_loop_statement)
        //
        // We want final order: init_jmp (BB_0), header (BB_1), body (BB_2), post_loop (BB_3)
        // Exit block is empty and gets merged with post-loop code
        //
        // block_stack[0] = header (contains CondBr)
        // block_stack[1] = body (contains body code + Jump to header)
        // block_stack[2] = init_jmp (old current_block with Jump to header)
        // block_stack[3] = exit (empty - merge with post-loop)

        // Add init_jmp block first (BB_0) - it's at index 2 in stack
        if block_stack.len() >= 3 {
            let init_jmp_block = block_stack.remove(2);
            blocks.push(init_jmp_block);
        }

        // Add header block (BB_1)
        if !block_stack.is_empty() {
            let header_block = block_stack.remove(0);
            blocks.push(header_block);
        }

        // Add body block (BB_2)
        if !block_stack.is_empty() {
            let body_block = block_stack.remove(0);
            blocks.push(body_block);
        }

        // Exit block is at index 0 now - just remove it (empty), don't add as separate block
        if !block_stack.is_empty() {
            block_stack.remove(0); // discard empty exit block
        }

        // Add post-loop code with fixed ID (next available is BB_3)
        current_block.id = format!("BB{}", blocks.len());
        blocks.push(current_block);

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

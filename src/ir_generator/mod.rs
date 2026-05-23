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
    pub function_return_types: HashMap<String, IrType>,
    pub global_names: HashSet<String>,
    pub global_types: HashMap<String, IrType>,
    pub struct_fields: HashMap<String, Vec<(String, IrType, usize)>>,
    pub local_struct_types: HashMap<String, String>, // name → [(field, type, offset)]
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
            function_return_types: HashMap::new(),
            global_names: HashSet::new(),
            global_types: HashMap::new(),
            struct_fields: HashMap::new(),
            local_struct_types: HashMap::new(),
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

        // First pass: collect all function signatures (extern + user-defined)
        for item in &program.items {
            match item {
                SourceItem::FuncDeclaration(decl) => {
                    self.external_functions
                        .insert(decl.signature.name.name.clone());
                    let ret_type = decl.signature.return_type.as_ref()
                        .map(|t| self.convert_type(t))
                        .unwrap_or(IrType::Int);
                    self.function_return_types
                        .insert(decl.signature.name.name.clone(), ret_type);
                }
                SourceItem::FuncDefinition(def) => {
                    let ret_type = def.signature.return_type.as_ref()
                        .map(|t| self.convert_type(t))
                        .unwrap_or(IrType::Void);
                    self.function_return_types
                        .insert(def.signature.name.name.clone(), ret_type);
                }
                SourceItem::GlobalDecl(global) => {
                    self.global_names.insert(global.name.name.clone());
                    let ir_ty = self.convert_type(&global.ty);
                    self.global_types.insert(global.name.name.clone(), ir_ty);
                }
                SourceItem::StructDef(s) => {
                    let mut fields = Vec::new();
                    let mut offset: usize = 0;
                    for f in &s.fields {
                        let fty = self.convert_type(&f.ty);
                        let size = fty.size() as usize;
                        fields.push((f.name.name.clone(), fty, offset));
                        offset += size;
                    }
                    self.struct_fields.insert(s.name.name.clone(), fields);
                }
            }
        }

        // Second pass: collect globals and generate IR for each function definition
        let mut globals = Vec::new();
        for item in &program.items {
            match item {
                SourceItem::GlobalDecl(global) => {
                    let ir_ty = self.convert_type(&global.ty);
                    let init = global.initializer.as_ref()
                        .and_then(|e| self.expr_to_constant(e));
                    globals.push(crate::ir::IrGlobal {
                        name: global.name.name.clone(),
                        ty: ir_ty,
                        initializer: init,
                    });
                }
                _ => {}
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

        IrProgram { functions, globals }
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
        // For if/functions: current_block is entry (first), stack has then/else
        // For loops: we need special handling - see visit_loop_statement
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
            TypeRef::Custom(id) => {
                if let Some(fields) = self.struct_fields.get(&id.name) {
                    if let Some(last) = fields.last() {
                        let size = last.2 + last.1.size() as usize;
                        IrType::Array(Box::new(IrType::Int), size)
                    } else {
                        IrType::Int
                    }
                } else {
                    IrType::Int
                }
            }
            TypeRef::Array {
                element_type, size, ..
            } => IrType::Array(Box::new(self.convert_type(element_type)), *size as usize),
        }
    }

    pub fn expr_to_constant(&self, expr: &crate::ast::Expr) -> Option<crate::ir::Constant> {
        match expr {
            crate::ast::Expr::Literal(lit) => match lit {
                crate::ast::Literal::Dec(v) => Some(crate::ir::Constant::Int(*v as i64)),
                crate::ast::Literal::Str(s) => Some(crate::ir::Constant::String(s.clone())),
                crate::ast::Literal::Char(c) => Some(crate::ir::Constant::Char(*c as u8)),
                crate::ast::Literal::Bool(b) => Some(crate::ir::Constant::Int(if *b { 1 } else { 0 })),
                _ => None,
            },
            crate::ast::Expr::ArrayLiteral(elements) => {
                let constants: Vec<crate::ir::Constant> = elements
                    .iter()
                    .map(|e| self.expr_to_constant(e))
                    .collect::<Option<_>>()?;
                Some(crate::ir::Constant::Array(constants))
            }
            _ => None,
        }
    }

    pub fn find_field_offset_for_array(&self, base: &str, field: &str) -> usize {
        for (_, fields) in &self.struct_fields {
            for (fname, _, offset) in fields {
                if fname == field {
                    return *offset;
                }
            }
        }
        0
    }

    pub fn struct_size_for_var(&self, _base: &str) -> usize {
        4
    }

    pub fn resolve_field_chain(&self, expr: &crate::ast::Expr) -> (String, usize) {
        match expr {
            crate::ast::Expr::FieldAccess(base, field) => {
                let (base_name, base_offset) = self.resolve_field_chain(base);
                // Find which struct the base variable belongs to
                let struct_name = self.local_struct_types.get(&base_name)
                    .map(|s| s.as_str())
                    .or_else(|| {
                        // Check if base is a global with a struct type
                        self.global_types.get(&base_name).and_then(|t| {
                            if let IrType::Array(..) = t { None } else { Some(base_name.as_str()) }
                        })
                    })
                    .unwrap_or(&base_name);
                let field_offset = self.struct_fields.get(struct_name)
                    .and_then(|fields| fields.iter().find(|(n,_,_)| n == &field.name))
                    .map(|(_,_,o)| *o)
                    .unwrap_or(0);
                (base_name, base_offset + field_offset)
            }
            crate::ast::Expr::Identifier(id) => {
                (id.name.clone(), 0)
            }
            _ => (String::new(), 0),
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
            Statement::VarDecl(s) => s.span,
        }
    }
}

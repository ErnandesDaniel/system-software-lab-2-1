use crate::ast::{BuiltinType, Identifier, Program, SourceItem, Span, Statement, TypeRef};
use crate::ir::{IrFunction, IrProgram, IrType};
use crate::stdlib::StdLib;
use std::collections::{HashMap, HashSet};

mod expressions;
mod functions;
mod statements;
mod symbols;

pub struct IrGenerator {
    pub temp_counter: usize,
    pub block_counter: usize,
    pub symbols: symbols::SymbolTable,
    pub used_functions: Vec<String>,
    pub loop_exit_stack: Vec<String>,
    pub loop_depth: usize,
    pub external_functions: HashSet<String>,
    pub current_yield_state: usize,
    pub coroutine_state_blocks: Vec<String>,
    pub pending_functions: Vec<IrFunction>,
    pub lambda_counter: usize,
    pub captured_vars: HashMap<String, usize>,
    pub closure_envs: HashMap<String, String>,
}

impl IrGenerator {
    pub fn new() -> Self {
        Self {
            temp_counter: 0,
            block_counter: 0,
            symbols: symbols::SymbolTable::new(),
            used_functions: Vec::new(),
            loop_exit_stack: Vec::new(),
            loop_depth: 0,
            external_functions: HashSet::new(),
            current_yield_state: 0,
            coroutine_state_blocks: Vec::new(),
            pending_functions: Vec::new(),
            lambda_counter: 0,
            captured_vars: HashMap::new(),
            closure_envs: HashMap::new(),
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
                    let func_name = decl.signature.name.name.clone();
                    self.external_functions.insert(func_name.clone());
                    let ret_type = decl
                        .signature
                        .return_type
                        .as_ref()
                        .map(|t| self.convert_type(t))
                        .or_else(|| {
                            crate::stdlib::StdLib::get_signature(&func_name)
                                .and_then(|(_, ret)| match ret {
                                    "string" => Some(IrType::String),
                                    "int" => Some(IrType::Int),
                                    "" => Some(IrType::Void),
                                    _ => None,
                                })
                        })
                        .unwrap_or(IrType::Int);
                    self.symbols.function_return_types.insert(func_name, ret_type);
                }
                SourceItem::FuncDefinition(def) => {
                    let ret_type = def
                        .signature
                        .return_type
                        .as_ref()
                        .map_or(IrType::Void, |t| self.convert_type(t));
                    self.symbols
                        .function_return_types
                        .insert(def.signature.name.name.clone(), ret_type);
                }
                SourceItem::GlobalDecl(global) => {
                    let ir_ty = self.convert_type(&global.ty);
                    self.symbols.global_types.insert(global.name.name.clone(), ir_ty);
                    if let crate::ast::TypeRef::Custom(ref id) = global.ty {
                        if self.symbols.struct_fields.contains_key(&id.name) {
                            self.symbols
                                .global_struct_type_names
                                .insert(global.name.name.clone(), id.name.clone());
                        }
                    }
                    if let crate::ast::TypeRef::Array { element_type, .. } = &global.ty {
                        if let crate::ast::TypeRef::Custom(ref id) = element_type.as_ref() {
                            if self.symbols.struct_fields.contains_key(&id.name) {
                                self.symbols
                                    .global_struct_type_names
                                    .insert(global.name.name.clone(), id.name.clone());
                            }
                        }
                    }
                }
                SourceItem::CoroutineDef(_) => {}
                SourceItem::StructDef(s) => {
                    let mut fields = Vec::new();
                    let mut offset: usize = 0;
                    for f in &s.fields {
                        let fty = self.convert_type(&f.ty);
                        let size = fty.size() as usize;
                        fields.push((f.name.name.clone(), fty, offset));
                        offset += size;
                    }
                    self.symbols.struct_fields.insert(s.name.name.clone(), fields);
                }
            }
        }

        // Second pass: collect globals and generate IR for each function definition
        let mut globals = Vec::new();
        for item in &program.items {
            if let SourceItem::GlobalDecl(global) = item {
                let ir_ty = self.convert_type(&global.ty);
                let init = global.initializer.as_ref().and_then(|e| self.expr_to_constant(e));
                globals.push(crate::ir::IrGlobal {
                    name: global.name.name.clone(),
                    ty: ir_ty,
                    initializer: init,
                });
            }
        }
        for item in &program.items {
            if let SourceItem::FuncDefinition(def) = item {
                self.block_counter = 0;
                self.symbols = symbols::SymbolTable::new();
                self.used_functions.clear();
                self.current_yield_state = 0;
                let ir_func = self.generate_function(def);
                functions.push(ir_func);
                while !self.pending_functions.is_empty() {
                    let pf = self.pending_functions.remove(0);
                    functions.push(pf);
                }
            }
            if let SourceItem::CoroutineDef(coroutine) = item {
                self.block_counter = 0;
                self.symbols = symbols::SymbolTable::new();
                self.used_functions.clear();
                self.current_yield_state = 0;
                let ir_func = self.generate_coroutine_function(coroutine);
                functions.push(ir_func);
            }
        }

        IrProgram { functions, globals }
    }

    pub fn get_ident_type(&self, id: &Identifier) -> IrType {
        self.symbols.get_type(&id.name)
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
                if let Some(fields) = self.symbols.struct_fields.get(&id.name) {
                    if let Some(last) = fields.last() {
                        let size = last.2 + last.1.size() as usize;
                        IrType::Array(Box::new(IrType::Int), size / 4)
                    } else {
                        IrType::Int
                    }
                } else {
                    IrType::Int
                }
            }
            TypeRef::Array { element_type, size, .. } => {
                IrType::Array(Box::new(self.convert_type(element_type)), *size as usize)
            }
            TypeRef::Function {
                params, return_type, ..
            } => {
                let p: Vec<IrType> = params.iter().map(|t| self.convert_type(t)).collect();
                IrType::Function(p, Box::new(self.convert_type(return_type)))
            }
        }
    }

    pub fn expr_to_constant(&self, expr: &crate::ast::Expr) -> Option<crate::ir::Constant> {
        match expr {
            crate::ast::Expr::Literal(lit) => match lit {
                crate::ast::Literal::Dec(v) => Some(crate::ir::Constant::Int(*v as i64)),
                crate::ast::Literal::Str(s) => Some(crate::ir::Constant::String(s.clone())),
                crate::ast::Literal::Char(c) => Some(crate::ir::Constant::Char(*c as u8)),
                crate::ast::Literal::Bool(b) => Some(crate::ir::Constant::Int(i64::from(*b))),
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

    pub fn find_field_offset_for_array(&self, _base: &str, field: &str) -> usize {
        for fields in self.symbols.struct_fields.values() {
            for (fname, _, offset) in fields {
                if fname == field {
                    return *offset;
                }
            }
        }
        0
    }

    pub fn struct_size_for_var(&self, base: &str) -> usize {
        let struct_name = self
            .symbols
            .global_struct_type_names
            .get(base)
            .or_else(|| self.symbols.local_struct_types.get(base));
        if let Some(struct_name) = struct_name {
            if let Some(fields) = self.symbols.struct_fields.get(struct_name) {
                if let Some((_, last_type, last_offset)) = fields.last() {
                    return last_offset + last_type.size() as usize;
                }
            }
        }
        4
    }

    pub fn find_field_type_for_var(&self, base: &str, field: &str) -> IrType {
        let struct_name = self
            .symbols
            .global_struct_type_names
            .get(base)
            .or_else(|| self.symbols.local_struct_types.get(base));
        if let Some(struct_name) = struct_name {
            if let Some(fields) = self.symbols.struct_fields.get(struct_name) {
                for (fname, ftype, _) in fields {
                    if fname == field {
                        return ftype.clone();
                    }
                }
            }
        }
        IrType::Int
    }

    pub fn resolve_field_chain(&self, expr: &crate::ast::Expr) -> (String, usize) {
        match expr {
            crate::ast::Expr::FieldAccess(base, field) => {
                let (base_name, base_offset) = self.resolve_field_chain(base);
                let struct_name = self
                    .symbols
                    .local_struct_types
                    .get(&base_name)
                    .map(String::as_str)
                    .or_else(|| {
                        self.symbols
                            .global_struct_type_names
                            .get(&base_name)
                            .map(String::as_str)
                    })
                    .unwrap_or(&base_name);
                let field_offset = self
                    .symbols
                    .struct_fields
                    .get(struct_name)
                    .and_then(|fields| fields.iter().find(|(n, _, _)| n == &field.name))
                    .map_or(0, |(_, _, o)| *o);
                (base_name, base_offset + field_offset)
            }
            crate::ast::Expr::Identifier(id) => (id.name.clone(), 0),
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
            Statement::Yield(s) => s.span,
            Statement::FuncDef(s) => s.span,
        }
    }
}

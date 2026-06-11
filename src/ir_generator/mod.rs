use crate::ast::{Program, SourceItem, Span};
use crate::ir::{IrBlock, IrFunction, IrProgram, IrType};
use crate::stdlib::StdLib;
use std::collections::{HashMap, HashSet};

mod expressions;
mod functions;
mod statements;
mod symbols;
mod types;

pub(crate) use expressions::literal::unescape_string;

pub struct IrGenerator {
    pub temp_counter: usize,
    pub block_counter: usize,
    pub symbols: symbols::SymbolTable,
    pub used_functions: Vec<String>,
    pub loop_exit_stack: Vec<String>,
    pub loop_depth: usize,
    pub external_functions: HashSet<String>,
    pub pending_functions: Vec<IrFunction>,
    pub lambda_counter: usize,
    pub captured_vars: HashMap<String, usize>,
    pub block_stack: Vec<IrBlock>,
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
            pending_functions: Vec::new(),
            lambda_counter: 0,
            captured_vars: HashMap::new(),
            block_stack: Vec::new(),
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

    pub fn generate(&mut self, program: &Program) -> crate::ir::IrProgram {
        self.try_generate(program)
    }

    pub fn try_generate(&mut self, program: &Program) -> crate::ir::IrProgram {
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
                            crate::stdlib::StdLib::get_signature(&func_name).and_then(|(_, ret)| match ret {
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
                self.symbols.reset_locals();
                self.used_functions.clear();
                let ir_func = self.generate_function(def);
                functions.push(ir_func);
                while !self.pending_functions.is_empty() {
                    let pf = self.pending_functions.remove(0);
                    functions.push(pf);
                }
            }
        }

        let mut layout_db = crate::struct_layout::LayoutDatabase::new();
        for (struct_name, fields) in &self.symbols.struct_fields {
            let ir_fields: Vec<(String, crate::ir::IrType)> =
                fields.iter().map(|(name, ty, _)| (name.clone(), ty.clone())).collect();
            layout_db.register_struct(struct_name, &ir_fields);
        }

        IrProgram {
            functions,
            globals,
            struct_layouts: layout_db,
        }
    }
}

#[allow(dead_code)]
impl Span {
    fn span(&self) -> Span {
        *self
    }
}

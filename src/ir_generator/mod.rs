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
    pub local_struct_types: HashMap<String, String>,
    pub current_yield_state: usize,
    pub pending_functions: Vec<IrFunction>,
    pub lambda_counter: usize,
    pub captured_vars: std::collections::HashMap<String, usize>,
    pub closure_envs: std::collections::HashMap<String, String>,
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
            current_yield_state: 0,
            pending_functions: Vec::new(),
            lambda_counter: 0,
            captured_vars: std::collections::HashMap::new(),
            closure_envs: std::collections::HashMap::new(),
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
                SourceItem::CoroutineDef(_) => {
                    // Coroutines are parsed like functions from FuncDefinition items
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
                self.locals.clear();
                self.used_functions.clear();
                self.current_yield_state = 0;
                let ir_func = self.generate_function(def);
                functions.push(ir_func);
                // Collect any pending functions generated by func literals
                while !self.pending_functions.is_empty() {
                    let pf = self.pending_functions.remove(0);
                    functions.push(pf);
                }
            }
            if let SourceItem::CoroutineDef(coroutine) = item {
                self.block_counter = 0;
                self.declared_vars.clear();
                self.locals.clear();
                self.used_functions.clear();
                self.current_yield_state = 0;
                let ir_func = self.generate_coroutine_function(coroutine);
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
        let mut params = Vec::new();
        if let Some(ref args) = def.signature.parameters {
            for arg in args {
                let param_type = arg.ty.as_ref()
                    .map(|t| self.convert_type(t))
                    .unwrap_or(IrType::Int);
                params.push(IrParameter { name: arg.name.name.clone(), ty: param_type });
            }
        }

        self.locals.clear();
        for param in &params {
            self.locals.insert(param.name.clone(), IrLocal {
                name: param.name.clone(),
                ty: param.ty.clone(),
                stack_offset: None,
            });
        }
        self.used_functions.clear();
        let mut block_stack = Vec::new();
        let mut current_block = IrBlock {
            id: format!("BB{}", self.block_counter),
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        self.block_counter += 1;

        for stmt in &def.body {
            self.visit_statement(&mut current_block, &mut block_stack, stmt);
        }

        // Add implicit Ret for void functions (e.g. closures without explicit return)
        if matches!(return_type, IrType::Void) {
            current_block.instructions.push(IrInstruction {
                opcode: IrOpcode::Ret,
                result: None,
                result_type: None,
                operands: vec![],
                jump_target: None,
                true_target: None,
                false_target: None,
                span: crate::ast::Span::new(0, 0),
            });
        }

        let mut blocks = Vec::new();
        blocks.push(current_block);
        for block in block_stack.drain(..) {
            blocks.push(block);
        }

        let mut used = Vec::new();
        std::mem::swap(&mut used, &mut self.used_functions);

        IrFunction {
            name: def.signature.name.name.clone(),
            return_type,
            parameters: params,
            blocks,
            locals: self.locals.values().cloned().collect(),
            used_functions: used,
            yield_count: 0,
        }
    }

    pub fn generate_coroutine_function(&mut self, def: &CoroutineDefinition) -> IrFunction {
        let return_type = match &def.signature.return_type {
            Some(ty) => self.convert_type(ty),
            None => IrType::Void,
        };
        let mut params = Vec::new();
        if let Some(ref args) = def.signature.parameters {
            for arg in args {
                let param_type = arg.ty.as_ref()
                    .map(|t| self.convert_type(t))
                    .unwrap_or(IrType::Int);
                params.push(IrParameter { name: arg.name.name.clone(), ty: param_type });
            }
        }

        self.locals.clear();
        self.used_functions.clear();
        let mut block_stack = Vec::new();
        let mut current_block = IrBlock {
            id: format!("BB{}", self.block_counter),
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        self.block_counter += 1;

        // Generate body тАФ yield statements will set current_yield_state
        for stmt in &def.body {
            self.visit_statement(&mut current_block, &mut block_stack, stmt);
        }

        // Collect blocks
        let mut blocks = Vec::new();
        blocks.push(current_block);
        for block in block_stack.drain(..) {
            blocks.push(block);
        }

        let mut used = Vec::new();
        std::mem::swap(&mut used, &mut self.used_functions);

        IrFunction {
            name: def.signature.name.name.clone(),
            return_type,
            parameters: params,
            blocks,
            locals: self.locals.values().cloned().collect(),
            used_functions: used,
            yield_count: self.current_yield_state,
        }
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
            TypeRef::Function { params, return_type, .. } => {
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

    /// Scan a function body AST for variables captured from the outer scope.
    /// Returns a list of (var_name, env_slot_index) for variables that are
    /// used in the body but belong to an outer (saved) locals table.
    pub fn scan_captures(
        &self,
        body: &[Statement],
        params: &Option<Vec<Arg>>,
        outer_locals: &std::collections::HashMap<String, IrLocal>,
    ) -> Vec<(String, usize)> {
        use std::collections::HashSet;
        let param_names: HashSet<String> = params.as_ref().map(|args| {
            args.iter().map(|a| a.name.name.clone()).collect()
        }).unwrap_or_default();
        let mut found: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        Self::scan_exprs_in_stmts(body, outer_locals, &param_names, &mut found, &mut seen);
        found.iter().enumerate().map(|(i, name)| (name.clone(), i)).collect()
    }

    fn scan_exprs_in_stmts(
        stmts: &[Statement],
        outer_locals: &std::collections::HashMap<String, IrLocal>,
        param_names: &std::collections::HashSet<String>,
        found: &mut Vec<String>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        for stmt in stmts {
            Self::scan_exprs_in_stmt(stmt, outer_locals, param_names, found, seen);
        }
    }

    fn scan_exprs_in_stmt(
        stmt: &Statement,
        outer_locals: &std::collections::HashMap<String, IrLocal>,
        param_names: &std::collections::HashSet<String>,
        found: &mut Vec<String>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        match stmt {
            Statement::Return(r) => {
                if let Some(ref e) = r.expr {
                    Self::scan_expr(e, outer_locals, param_names, found, seen);
                }
            }
            Statement::If(i) => {
                Self::scan_expr(&i.condition, outer_locals, param_names, found, seen);
                Self::scan_exprs_in_stmt(&i.consequence, outer_locals, param_names, found, seen);
                if let Some(ref a) = i.alternative {
                    Self::scan_exprs_in_stmt(a, outer_locals, param_names, found, seen);
                }
            }
            Statement::Loop(l) => {
                Self::scan_expr(&l.condition, outer_locals, param_names, found, seen);
                Self::scan_exprs_in_stmts(&l.body, outer_locals, param_names, found, seen);
            }
            Statement::Repeat(r) => {
                Self::scan_expr(&r.condition, outer_locals, param_names, found, seen);
                Self::scan_exprs_in_stmt(&r.body, outer_locals, param_names, found, seen);
            }
            Statement::Expression(e) => {
                Self::scan_expr(&e.expr, outer_locals, param_names, found, seen);
            }
            Statement::Block(b) => {
                Self::scan_exprs_in_stmts(&b.body, outer_locals, param_names, found, seen);
            }
            Statement::VarDecl(vd) => {
                // var decls inside the inner function body create new locals,
                // not captured vars
            }
            Statement::Break(_) | Statement::Yield(_) => {}
            Statement::FuncDef(fd) => {
                Self::scan_exprs_in_stmts(&fd.body, outer_locals, param_names, found, seen);
            }
        }
    }

    fn scan_expr(
        expr: &Expr,
        outer_locals: &std::collections::HashMap<String, IrLocal>,
        param_names: &std::collections::HashSet<String>,
        found: &mut Vec<String>,
        seen: &mut std::collections::HashSet<String>,
    ) {
        match expr {
            Expr::Binary(b) => {
                Self::scan_expr(&b.left, outer_locals, param_names, found, seen);
                Self::scan_expr(&b.right, outer_locals, param_names, found, seen);
            }
            Expr::Unary(u) => {
                Self::scan_expr(&u.operand, outer_locals, param_names, found, seen);
            }
            Expr::Parenthesized(inner) => {
                Self::scan_expr(inner, outer_locals, param_names, found, seen);
            }
            Expr::Call(c) => {
                Self::scan_expr(&c.function, outer_locals, param_names, found, seen);
                for a in &c.arguments {
                    Self::scan_expr(a, outer_locals, param_names, found, seen);
                }
            }
            Expr::Slice(s) => {
                Self::scan_expr(&s.array, outer_locals, param_names, found, seen);
                for r in &s.ranges {
                    Self::scan_expr(&r.start, outer_locals, param_names, found, seen);
                }
            }
            Expr::Identifier(id) => {
                if outer_locals.contains_key(&id.name)
                    && !param_names.contains(&id.name)
                    && !seen.contains(&id.name)
                {
                    seen.insert(id.name.clone());
                    found.push(id.name.clone());
                }
            }
            Expr::FieldAccess(base, _) => {
                Self::scan_expr(base, outer_locals, param_names, found, seen);
            }
            Expr::FuncLiteral(f) => {
                Self::scan_exprs_in_stmts(&f.body, outer_locals, param_names, found, seen);
            }
            Expr::Literal(_) | Expr::ArrayLiteral(_) => {}
        }
    }

    pub fn find_field_offset_for_array(&self, _base: &str, field: &str) -> usize {
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
            Statement::Yield(s) => s.span,
            Statement::FuncDef(s) => s.span,
        }
    }
}

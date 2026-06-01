use super::IrGenerator;
use crate::ast::{Arg, CoroutineDefinition, Expr, FuncDefinition, Span, Statement};
use crate::ir_generator::symbols::SymbolTable;
use crate::ir::{IrBlock, IrFunction, IrInstruction, IrOpcode, IrParameter, IrType};
use std::collections::HashSet;

impl IrGenerator {
    pub fn generate_function(&mut self, def: &FuncDefinition) -> IrFunction {
        let return_type = match &def.signature.return_type {
            Some(ty) => self.convert_type(ty),
            None => IrType::Void,
        };
        let mut params = Vec::new();
        if let Some(ref args) = def.signature.parameters {
            for arg in args {
                let param_type = arg.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t));
                params.push(IrParameter {
                    name: arg.name.name.clone(),
                    ty: param_type,
                });
            }
        }

        for param in &params {
            self.symbols.define_local(&param.name, param.ty.clone());
        }
        self.used_functions.clear();
        let mut block_stack = Vec::new();
        let entry_id = format!("BB{}", self.block_counter);
        let mut current_block = IrBlock {
            id: entry_id.clone(),
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        self.block_counter += 1;

        for stmt in &def.body {
            self.visit_statement(&mut current_block, &mut block_stack, stmt);
        }

        if matches!(return_type, IrType::Void) {
            current_block.instructions.push(IrInstruction {
                opcode: IrOpcode::Ret,
                result: None,
                result_type: None,
                operands: vec![],
                jump_target: None,
                true_target: None,
                false_target: None,
                span: Span::new(0, 0),
            });
        }

        let mut blocks = Vec::new();
        for block in block_stack.drain(..) {
            blocks.push(block);
        }
        blocks.push(current_block);
        if let Some(pos) = blocks.iter().position(|b| b.id == entry_id) {
            if pos != 0 {
                let entry = blocks.remove(pos);
                blocks.insert(0, entry);
            }
        }

        let mut used = Vec::new();
        std::mem::swap(&mut used, &mut self.used_functions);

        IrFunction {
            name: def.signature.name.name.clone(),
            return_type,
            parameters: params,
            blocks,
            locals: self.symbols.all_locals(),
            used_functions: used,
            yield_count: 0,
            coroutine_blocks: vec![],
            is_coroutine: false,
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
                let param_type = arg.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t));
                params.push(IrParameter {
                    name: arg.name.name.clone(),
                    ty: param_type,
                });
            }
        }

        for param in &params {
            self.symbols.define_local(&param.name, param.ty.clone());
        }
        self.used_functions.clear();
        let mut block_stack = Vec::new();
        let entry_id = format!("BB{}", self.block_counter);
        let mut current_block = IrBlock {
            id: entry_id.clone(),
            instructions: Vec::new(),
            successors: Vec::new(),
        };
        self.block_counter += 1;

        self.coroutine_state_blocks = vec![entry_id.clone()];

        for stmt in &def.body {
            self.visit_statement(&mut current_block, &mut block_stack, stmt);
        }

        let mut blocks = Vec::new();
        for block in block_stack.drain(..) {
            blocks.push(block);
        }
        blocks.push(current_block);
        if let Some(pos) = blocks.iter().position(|b| b.id == entry_id) {
            if pos != 0 {
                let entry = blocks.remove(pos);
                blocks.insert(0, entry);
            }
        }

        let mut used = Vec::new();
        std::mem::swap(&mut used, &mut self.used_functions);
        let state_blocks = std::mem::take(&mut self.coroutine_state_blocks);

        IrFunction {
            name: def.signature.name.name.clone(),
            return_type,
            parameters: params,
            blocks,
            locals: self.symbols.all_locals(),
            used_functions: used,
            yield_count: self.current_yield_state,
            coroutine_blocks: state_blocks,
            is_coroutine: true,
        }
    }

    pub fn scan_captures(
        body: &[Statement],
        params: &Option<Vec<Arg>>,
        outer_symbols: &SymbolTable,
    ) -> Vec<(String, usize)> {
        let param_names: HashSet<String> = params
            .as_ref()
            .map(|args| args.iter().map(|a| a.name.name.clone()).collect())
            .unwrap_or_default();
        let mut found: Vec<String> = Vec::new();
        let mut seen: HashSet<String> = HashSet::new();
        Self::scan_exprs_in_stmts(body, outer_symbols, &param_names, &mut found, &mut seen);
        found.iter().enumerate().map(|(i, name)| (name.clone(), i)).collect()
    }

    fn scan_exprs_in_stmts(
        stmts: &[Statement],
        outer_symbols: &SymbolTable,
        param_names: &HashSet<String>,
        found: &mut Vec<String>,
        seen: &mut HashSet<String>,
    ) {
        for stmt in stmts {
            Self::scan_exprs_in_stmt(stmt, outer_symbols, param_names, found, seen);
        }
    }

    fn scan_exprs_in_stmt(
        stmt: &Statement,
        outer_symbols: &SymbolTable,
        param_names: &HashSet<String>,
        found: &mut Vec<String>,
        seen: &mut HashSet<String>,
    ) {
        match stmt {
            Statement::Return(r) => {
                if let Some(ref e) = r.expr {
                    Self::scan_expr(e, outer_symbols, param_names, found, seen);
                }
            }
            Statement::If(i) => {
                Self::scan_expr(&i.condition, outer_symbols, param_names, found, seen);
                Self::scan_exprs_in_stmt(&i.consequence, outer_symbols, param_names, found, seen);
                if let Some(ref a) = i.alternative {
                    Self::scan_exprs_in_stmt(a, outer_symbols, param_names, found, seen);
                }
            }
            Statement::Loop(l) => {
                Self::scan_expr(&l.condition, outer_symbols, param_names, found, seen);
                Self::scan_exprs_in_stmts(&l.body, outer_symbols, param_names, found, seen);
            }
            Statement::Expression(e) => {
                Self::scan_expr(&e.expr, outer_symbols, param_names, found, seen);
            }
            Statement::Block(block) => {
                Self::scan_exprs_in_stmts(&block.body, outer_symbols, param_names, found, seen);
            }
            Statement::Repeat(r) => {
                Self::scan_expr(&r.condition, outer_symbols, param_names, found, seen);
                Self::scan_exprs_in_stmt(&r.body, outer_symbols, param_names, found, seen);
            }
            Statement::VarDecl(_) => {}
            Statement::Break(_) | Statement::Yield(_) => {}
            Statement::FuncDef(fd) => {
                Self::scan_exprs_in_stmts(&fd.body, outer_symbols, param_names, found, seen);
            }
        }
    }

    fn scan_expr(
        expr: &Expr,
        outer_symbols: &SymbolTable,
        param_names: &HashSet<String>,
        found: &mut Vec<String>,
        seen: &mut HashSet<String>,
    ) {
        match expr {
            Expr::Binary(b) => {
                Self::scan_expr(&b.left, outer_symbols, param_names, found, seen);
                Self::scan_expr(&b.right, outer_symbols, param_names, found, seen);
            }
            Expr::Unary(u) => {
                Self::scan_expr(&u.operand, outer_symbols, param_names, found, seen);
            }
            Expr::Parenthesized(inner) => {
                Self::scan_expr(inner, outer_symbols, param_names, found, seen);
            }
            Expr::Call(c) => {
                Self::scan_expr(&c.function, outer_symbols, param_names, found, seen);
                for a in &c.arguments {
                    Self::scan_expr(a, outer_symbols, param_names, found, seen);
                }
            }
            Expr::Slice(s) => {
                Self::scan_expr(&s.array, outer_symbols, param_names, found, seen);
                for r in &s.ranges {
                    Self::scan_expr(&r.start, outer_symbols, param_names, found, seen);
                }
            }
            Expr::Identifier(id) => {
                if outer_symbols.lookup(&id.name).is_some() && !param_names.contains(&id.name) && !seen.contains(&id.name) {
                    seen.insert(id.name.clone());
                    found.push(id.name.clone());
                }
            }
            Expr::FieldAccess(base, _) => {
                Self::scan_expr(base, outer_symbols, param_names, found, seen);
            }
            Expr::FuncLiteral(f) => {
                Self::scan_exprs_in_stmts(&f.body, outer_symbols, param_names, found, seen);
            }
            Expr::Literal(_) | Expr::ArrayLiteral(_) => {}
        }
    }
}

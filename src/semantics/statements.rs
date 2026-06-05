use crate::ast::Statement;
use crate::ir::IrType;
use crate::semantics::analysis::SemanticsAnalyzer;
use crate::semantics::types::SymbolTable;

impl SemanticsAnalyzer {
    pub fn check_statement(&mut self, scope: &mut SymbolTable, stmt: &Statement) -> crate::Result<()> {
        match stmt {
            Statement::Return(ret) => {
                let expr_type = if let Some(ref expr) = ret.expr {
                    Some(self.check_expression(scope, expr)?)
                } else {
                    None
                };
                if let Some(ref expected) = self.current_return_type {
                    if *expected == IrType::Void {
                        if expr_type.is_some() {
                            self.add_error("Void function should not return a value".to_string(), ret.span);
                        }
                    } else if let Some(ref actual) = expr_type {
                        if actual != expected {
                            self.add_error(
                                format!("Return type mismatch: expected {expected:?}, got {actual:?}"),
                                ret.span,
                            );
                        }
                    } else {
                        self.add_error(
                            format!("Function expected return value of type {expected:?}, got none"),
                            ret.span,
                        );
                    }
                }
            }
            Statement::If(if_stmt) => {
                let cond_type = self.check_expression(scope, &if_stmt.condition)?;
                if !cond_type.is_bool() {
                    self.add_error(
                        format!("If condition must be bool, got {cond_type:?}"),
                        if_stmt.condition.span(),
                    );
                }
                let mut inner_scope = scope.clone();
                for s in &if_stmt.body {
                    self.check_statement(&mut inner_scope, s)?;
                }
                for ei in &if_stmt.else_ifs {
                    let ei_cond_type = self.check_expression(scope, &ei.condition)?;
                    if !ei_cond_type.is_bool() {
                        self.add_error(
                            format!("Else-if condition must be bool, got {ei_cond_type:?}"),
                            ei.condition.span(),
                        );
                    }
                    let mut ei_scope = scope.clone();
                    for s in &ei.body {
                        self.check_statement(&mut ei_scope, s)?;
                    }
                }
                if let Some(ref eb) = if_stmt.else_body {
                    let mut else_scope = scope.clone();
                    for s in eb {
                        self.check_statement(&mut else_scope, s)?;
                    }
                }
            }
            Statement::Loop(loop_stmt) => {
                let cond_type = self.check_expression(scope, &loop_stmt.condition)?;
                if !cond_type.is_bool() {
                    self.add_error(
                        format!("Loop condition must be bool, got {cond_type:?}"),
                        loop_stmt.condition.span(),
                    );
                }
                self.loop_depth += 1;
                let mut inner_scope = scope.clone();
                for s in &loop_stmt.body {
                    self.check_statement(&mut inner_scope, s)?;
                }
                self.loop_depth -= 1;
            }
            Statement::Repeat(repeat_stmt) => {
                let cond_type = self.check_expression(scope, &repeat_stmt.condition)?;
                if !cond_type.is_bool() {
                    self.add_error(
                        format!("Repeat condition must be bool, got {cond_type:?}"),
                        repeat_stmt.condition.span(),
                    );
                }
                self.loop_depth += 1;
                let mut inner_scope = scope.clone();
                for s in &repeat_stmt.body {
                    self.check_statement(&mut inner_scope, s)?;
                }
                self.loop_depth -= 1;
            }
            Statement::Expression(expr_stmt) => {
                self.check_expression(scope, &expr_stmt.expr)?;
            }
            Statement::Block(block_stmt) => {
                let mut inner_scope = scope.clone();
                for s in &block_stmt.body {
                    self.check_statement(&mut inner_scope, s)?;
                }
            }
            Statement::Break(b) => {
                if self.loop_depth == 0 {
                    self.add_error("'break' outside loop".to_string(), b.span);
                }
            }
            Statement::VarDecl(vd) => {
                let ty = self.convert_type(&vd.ty);
                if let Err(e) = scope.add(vd.name.name.clone(), ty) {
                    self.add_error(e.to_string(), vd.span);
                }
            }
            Statement::Yield(y) => {
                if !self.in_coroutine {
                    self.add_error("'yield' outside coroutine".to_string(), y.span);
                }
            }
            Statement::FuncDef(fd) => {
                let param_types: Vec<IrType> = fd
                    .signature
                    .parameters
                    .as_ref()
                    .map(|args| {
                        args.iter()
                            .map(|a| a.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t)))
                            .collect()
                    })
                    .unwrap_or_default();
                let ret_type = fd
                    .signature
                    .return_type
                    .as_ref()
                    .map_or(IrType::Void, |t| self.convert_type(t));
                let func_type = IrType::Function(param_types, Box::new(ret_type.clone()));
                if let Err(e) = scope.add(fd.signature.name.name.clone(), func_type) {
                    self.add_error(e.to_string(), fd.span);
                }
                let mut inner_scope = scope.clone();
                inner_scope.push_scope();
                if let Some(ref args) = fd.signature.parameters {
                    for arg in args {
                        let pty = arg.ty.as_ref().map_or(IrType::Int, |t| self.convert_type(t));
                        if let Err(e) = inner_scope.add(arg.name.name.clone(), pty) {
                            self.add_error(e.to_string(), arg.span);
                        }
                    }
                }
                let saved_return_type = self.current_return_type.take();
                self.current_return_type = Some(ret_type);
                let saved_loop_depth = self.loop_depth;
                let saved_coroutine = self.in_coroutine;
                self.loop_depth = 0;
                self.in_coroutine = false;
                for s in &fd.body {
                    if let Err(e) = self.check_statement(&mut inner_scope, s) {
                        self.add_error(e.to_string(), s.span());
                    }
                }
                self.in_coroutine = saved_coroutine;
                self.loop_depth = saved_loop_depth;
                self.current_return_type = saved_return_type;
            }
        }
        Ok(())
    }
}

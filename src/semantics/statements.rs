use crate::ast::Statement;
use crate::semantics::analysis::SemanticsAnalyzer;
use crate::semantics::types::{SemanticType, SymbolTable};

impl SemanticsAnalyzer {
    pub fn check_statement(&mut self, scope: &mut SymbolTable, stmt: &Statement) -> Result<(), Vec<String>> {
        match stmt {
            Statement::Return(ret) => {
                if let Some(ref expr) = ret.expr {
                    self.check_expression(scope, expr)?;
                }
            }
            Statement::If(if_stmt) => {
                let cond_type = self.check_expression(scope, &if_stmt.condition)?;
                if cond_type != SemanticType::Bool {
                    self.add_error(format!("If condition must be bool, got {cond_type:?}"));
                }
                self.check_statement(scope, &if_stmt.consequence)?;
                if let Some(ref alt) = if_stmt.alternative {
                    self.check_statement(scope, alt)?;
                }
            }
            Statement::Loop(loop_stmt) => {
                let cond_type = self.check_expression(scope, &loop_stmt.condition)?;
                if cond_type != SemanticType::Bool {
                    self.add_error(format!("Loop condition must be bool, got {cond_type:?}"));
                }
                for s in &loop_stmt.body {
                    self.check_statement(scope, s)?;
                }
            }
            Statement::Repeat(repeat_stmt) => {
                let cond_type = self.check_expression(scope, &repeat_stmt.condition)?;
                if cond_type != SemanticType::Bool {
                    self.add_error(format!("Repeat condition must be bool, got {cond_type:?}"));
                }
                self.check_statement(scope, &repeat_stmt.body)?;
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
            Statement::Break(_) | Statement::VarDecl(_) | Statement::Yield(_) => {}
            Statement::FuncDef(fd) => {
                let param_types: Vec<SemanticType> = fd
                    .signature
                    .parameters
                    .as_ref()
                    .map(|args| {
                        args.iter()
                            .map(|a| a.ty.as_ref().map_or(SemanticType::Int, |t| self.convert_type(t)))
                            .collect()
                    })
                    .unwrap_or_default();
                let ret_type = fd
                    .signature
                    .return_type
                    .as_ref()
                    .map_or(SemanticType::Void, |t| self.convert_type(t));
                let func_type = SemanticType::Function(param_types, Box::new(ret_type));
                let _ = scope.add(fd.signature.name.name.clone(), func_type);
                let mut inner_scope = scope.clone();
                if let Some(ref args) = fd.signature.parameters {
                    for arg in args {
                        let pty = arg.ty.as_ref().map_or(SemanticType::Int, |t| self.convert_type(t));
                        let _ = inner_scope.add(arg.name.name.clone(), pty);
                    }
                }
                for s in &fd.body {
                    self.check_statement(&mut inner_scope, s)?;
                }
            }
        }
        Ok(())
    }
}

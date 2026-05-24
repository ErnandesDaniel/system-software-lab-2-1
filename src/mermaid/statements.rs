use super::MermaidGenerator;
use crate::ast::{ExpressionStatement, LoopKeyword, LoopStatement, RepeatStatement, Statement};

impl MermaidGenerator {
    pub fn generate_statement(&mut self, stmt: &Statement, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{id}[\"statement\"]\n"));

        if let Some(p) = parent_id {
            output.push_str(&format!("{p} --> N{id}\n"));
        }

        match stmt {
            Statement::Return(r) => {
                let stmt_id = self.next_id();
                output.push_str(&format!("N{stmt_id}[\"return_statement\"]\n"));
                output.push_str(&format!("N{id} --> N{stmt_id}\n"));
                if let Some(expr) = &r.expr {
                    self.generate_expr(expr, output, Some(&format!("N{stmt_id}")));
                }
            }
            Statement::If(i) => {
                let stmt_id = self.next_id();
                output.push_str(&format!("N{stmt_id}[\"if_statement\"]\n"));
                output.push_str(&format!("N{id} --> N{stmt_id}\n"));

                let keyword_id = self.next_id();
                output.push_str(&format!("N{keyword_id}[\"if\"]\n"));
                output.push_str(&format!("N{stmt_id} --> N{keyword_id}\n"));

                self.generate_expr(&i.condition, output, Some(&format!("N{stmt_id}")));
                self.generate_statement(&i.consequence, output, Some(&format!("N{stmt_id}")));
                if let Some(alt) = &i.alternative {
                    self.generate_statement(alt, output, Some(&format!("N{stmt_id}")));
                }
            }
            Statement::Loop(l) => {
                self.generate_loop(l, output, Some(&format!("N{id}")));
            }
            Statement::Repeat(r) => {
                self.generate_repeat(r, output, Some(&format!("N{id}")));
            }
            Statement::Break(_) => {
                let stmt_id = self.next_id();
                output.push_str(&format!("N{stmt_id}[\"break_statement\"]\n"));
                output.push_str(&format!("N{id} --> N{stmt_id}\n"));

                let semi_id = self.next_id();
                output.push_str(&format!("N{semi_id}[\";\"]\n"));
                output.push_str(&format!("N{stmt_id} --> N{semi_id}\n"));
            }
            Statement::Expression(e) => {
                self.generate_expression_statement(e, output, Some(&format!("N{id}")));
            }
            Statement::Block(b) => {
                let stmt_id = self.next_id();
                output.push_str(&format!("N{stmt_id}[\"block_statement\"]\n"));
                output.push_str(&format!("N{id} --> N{stmt_id}\n"));

                let begin_id = self.next_id();
                output.push_str(&format!("N{begin_id}[\"begin\"]\n"));
                output.push_str(&format!("N{stmt_id} --> N{begin_id}\n"));

                for s in &b.body {
                    self.generate_statement(s, output, Some(&format!("N{stmt_id}")));
                }

                let end_id = self.next_id();
                output.push_str(&format!("N{end_id}[\"end\"]\n"));
                output.push_str(&format!("N{stmt_id} --> N{end_id}\n"));
            }
            Statement::Yield(_) => {
                let id = self.next_id();
                output.push_str(&format!("N{id}[\"yield\"]\n"));
                if let Some(p) = parent_id {
                    output.push_str(&format!("{p} --> N{id}\n"));
                }
            }
            Statement::VarDecl(vd) => {
                let id = self.next_id();
                output.push_str(&format!("N{}[\"var_decl {}\"]\n", id, vd.name.name));
                if let Some(p) = parent_id {
                    output.push_str(&format!("{p} --> N{id}\n"));
                }
            }
            Statement::FuncDef(fd) => {
                let stmt_id = self.next_id();
                output.push_str(&format!("N{}[\"local_func {}()\"]\n", stmt_id, fd.signature.name.name));
                output.push_str(&format!("N{id} --> N{stmt_id}\n"));
            }
        }
    }

    fn generate_loop(&mut self, l: &LoopStatement, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{id}[\"loop_statement\"]\n"));

        if let Some(p) = parent_id {
            output.push_str(&format!("{p} --> N{id}\n"));
        }

        let keyword_id = self.next_id();
        let keyword = match l.keyword {
            LoopKeyword::While => "while",
            LoopKeyword::Until => "until",
        };
        output.push_str(&format!("N{keyword_id}[\"{keyword}\"]\n"));
        output.push_str(&format!("N{id} --> N{keyword_id}\n"));

        self.generate_expr(&l.condition, output, Some(&format!("N{id}")));

        for b in &l.body {
            self.generate_statement(b, output, Some(&format!("N{id}")));
        }

        let end_id = self.next_id();
        output.push_str(&format!("N{end_id}[\"end\"]\n"));
        output.push_str(&format!("N{id} --> N{end_id}\n"));
    }

    fn generate_repeat(&mut self, r: &RepeatStatement, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{id}[\"repeat_statement\"]\n"));

        if let Some(p) = parent_id {
            output.push_str(&format!("{p} --> N{id}\n"));
        }

        self.generate_statement(&r.body, output, Some(&format!("N{id}")));

        let keyword_id = self.next_id();
        let keyword = match r.keyword {
            LoopKeyword::While => "while",
            LoopKeyword::Until => "until",
        };
        output.push_str(&format!("N{keyword_id}[\"{keyword}\"]\n"));
        output.push_str(&format!("N{id} --> N{keyword_id}\n"));

        self.generate_expr(&r.condition, output, Some(&format!("N{id}")));

        let semi_id = self.next_id();
        output.push_str(&format!("N{semi_id}[\";\"]\n"));
        output.push_str(&format!("N{id} --> N{semi_id}\n"));
    }

    fn generate_expression_statement(&mut self, e: &ExpressionStatement, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{id}[\"expression_statement\"]\n"));

        if let Some(p) = parent_id {
            output.push_str(&format!("{p} --> N{id}\n"));
        }

        self.generate_expr(&e.expr, output, Some(&format!("N{id}")));

        let semi_id = self.next_id();
        output.push_str(&format!("N{semi_id}[\";\"]\n"));
        output.push_str(&format!("N{id} --> N{semi_id}\n"));
    }
}

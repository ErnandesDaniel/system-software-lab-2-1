use super::MermaidGenerator;
use crate::ast::*;

impl MermaidGenerator {
    pub fn generate_statement(
        &mut self,
        stmt: &Statement,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"statement\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        match stmt {
            Statement::Return(r) => {
                let stmt_id = self.next_id();
                output.push_str(&format!("N{}[\"return_statement\"]\n", stmt_id));
                output.push_str(&format!("N{} --> N{}\n", id, stmt_id));
                if let Some(expr) = &r.expr {
                    self.generate_expr(expr, output, Some(&format!("N{}", stmt_id)));
                }
            }
            Statement::If(i) => {
                let stmt_id = self.next_id();
                output.push_str(&format!("N{}[\"if_statement\"]\n", stmt_id));
                output.push_str(&format!("N{} --> N{}\n", id, stmt_id));

                let keyword_id = self.next_id();
                output.push_str(&format!("N{}[\"if\"]\n", keyword_id));
                output.push_str(&format!("N{} --> N{}\n", stmt_id, keyword_id));

                self.generate_expr(&i.condition, output, Some(&format!("N{}", stmt_id)));
                self.generate_statement(&i.consequence, output, Some(&format!("N{}", stmt_id)));
                if let Some(alt) = &i.alternative {
                    self.generate_statement(alt, output, Some(&format!("N{}", stmt_id)));
                }
            }
            Statement::Loop(l) => {
                self.generate_loop(l, output, Some(&format!("N{}", id)));
            }
            Statement::Repeat(r) => {
                self.generate_repeat(r, output, Some(&format!("N{}", id)));
            }
            Statement::Break(_) => {
                let stmt_id = self.next_id();
                output.push_str(&format!("N{}[\"break_statement\"]\n", stmt_id));
                output.push_str(&format!("N{} --> N{}\n", id, stmt_id));

                let semi_id = self.next_id();
                output.push_str(&format!("N{}[\";\"]\n", semi_id));
                output.push_str(&format!("N{} --> N{}\n", stmt_id, semi_id));
            }
            Statement::Expression(e) => {
                self.generate_expression_statement(e, output, Some(&format!("N{}", id)));
            }
            Statement::Block(b) => {
                let stmt_id = self.next_id();
                output.push_str(&format!("N{}[\"block_statement\"]\n", stmt_id));
                output.push_str(&format!("N{} --> N{}\n", id, stmt_id));

                let begin_id = self.next_id();
                output.push_str(&format!("N{}[\"begin\"]\n", begin_id));
                output.push_str(&format!("N{} --> N{}\n", stmt_id, begin_id));

                for s in &b.body {
                    self.generate_statement(s, output, Some(&format!("N{}", stmt_id)));
                }

                let end_id = self.next_id();
                output.push_str(&format!("N{}[\"end\"]\n", end_id));
                output.push_str(&format!("N{} --> N{}\n", stmt_id, end_id));
            }
        }
    }

    fn generate_loop(&mut self, l: &LoopStatement, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"loop_statement\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        let keyword_id = self.next_id();
        let keyword = match l.keyword {
            LoopKeyword::While => "while",
            LoopKeyword::Until => "until",
        };
        output.push_str(&format!("N{}[\"{}\"]\n", keyword_id, keyword));
        output.push_str(&format!("N{} --> N{}\n", id, keyword_id));

        self.generate_expr(&l.condition, output, Some(&format!("N{}", id)));

        for b in &l.body {
            self.generate_statement(b, output, Some(&format!("N{}", id)));
        }

        let end_id = self.next_id();
        output.push_str(&format!("N{}[\"end\"]\n", end_id));
        output.push_str(&format!("N{} --> N{}\n", id, end_id));
    }

    fn generate_repeat(
        &mut self,
        r: &RepeatStatement,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"repeat_statement\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_statement(&r.body, output, Some(&format!("N{}", id)));

        let keyword_id = self.next_id();
        let keyword = match r.keyword {
            LoopKeyword::While => "while",
            LoopKeyword::Until => "until",
        };
        output.push_str(&format!("N{}[\"{}\"]\n", keyword_id, keyword));
        output.push_str(&format!("N{} --> N{}\n", id, keyword_id));

        self.generate_expr(&r.condition, output, Some(&format!("N{}", id)));

        let semi_id = self.next_id();
        output.push_str(&format!("N{}[\";\"]\n", semi_id));
        output.push_str(&format!("N{} --> N{}\n", id, semi_id));
    }

    fn generate_expression_statement(
        &mut self,
        e: &ExpressionStatement,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"expression_statement\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_expr(&e.expr, output, Some(&format!("N{}", id)));

        let semi_id = self.next_id();
        output.push_str(&format!("N{}[\";\"]\n", semi_id));
        output.push_str(&format!("N{} --> N{}\n", id, semi_id));
    }
}

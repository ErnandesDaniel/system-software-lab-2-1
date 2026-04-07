use crate::ast::*;

pub struct MermaidGenerator {
    id_counter: usize,
}

impl MermaidGenerator {
    pub fn new() -> Self {
        Self { id_counter: 0 }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        let mut output = String::from("graph TD;\n");
        self.generate_program(program, &mut output, None);
        output
    }

    fn generate_program(
        &mut self,
        program: &Program,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"source\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        for item in &program.items {
            self.generate_source_item(item, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_source_item(
        &mut self,
        item: &SourceItem,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        match item {
            SourceItem::FuncDeclaration(f) => {
                self.generate_func_declaration(f, output, parent_id);
            }
            SourceItem::FuncDefinition(f) => {
                self.generate_func_definition(f, output, parent_id);
            }
        }
    }

    fn generate_func_declaration(
        &mut self,
        f: &FuncDeclaration,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"func_declaration\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_func_signature(&f.signature, output, Some(&format!("N{}", id)));
    }

    fn generate_func_definition(
        &mut self,
        f: &FuncDefinition,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"func_definition\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_func_signature(&f.signature, output, Some(&format!("N{}", id)));

        for stmt in &f.body {
            self.generate_statement(stmt, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_func_signature(
        &mut self,
        sig: &FuncSignature,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"func_signature\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_identifier(&sig.name, output, Some(&format!("N{}", id)));

        if let Some(params) = &sig.parameters {
            for arg in params {
                self.generate_arg(arg, output, Some(&format!("N{}", id)));
            }
        }

        if let Some(ret) = &sig.return_type {
            self.generate_type_ref(ret, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_arg(&mut self, arg: &Arg, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"arg\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_identifier(&arg.name, output, Some(&format!("N{}", id)));

        if let Some(ty) = &arg.ty {
            self.generate_type_ref(ty, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_type_ref(&mut self, ty: &TypeRef, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        let label = match ty {
            TypeRef::BuiltinType(b) => format!("{:?}", b),
            TypeRef::Custom(id) => id.name.clone(),
            TypeRef::Array {
                element_type: _,
                size,
                ..
            } => format!("array[{}]", size),
        };
        output.push_str(&format!("N{}[\"type_ref: {}\"]\n", id, label));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        if let TypeRef::Array { element_type, .. } = ty {
            self.generate_type_ref(element_type, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_statement(
        &mut self,
        stmt: &Statement,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let id = self.next_id();
        let label = match stmt {
            Statement::Return(_) => "return",
            Statement::If(_) => "if_statement",
            Statement::Loop(_) => "loop_statement",
            Statement::Repeat(_) => "repeat_statement",
            Statement::Break(_) => "break_statement",
            Statement::Expression(_) => "expression_statement",
            Statement::Block(_) => "block_statement",
        };
        output.push_str(&format!("N{}[\"{}\"]\n", id, label));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        match stmt {
            Statement::Return(r) => {
                if let Some(expr) = &r.expr {
                    self.generate_expr(expr, output, Some(&format!("N{}", id)));
                }
            }
            Statement::If(i) => {
                self.generate_expr(&i.condition, output, Some(&format!("N{}", id)));
                self.generate_statement(&i.consequence, output, Some(&format!("N{}", id)));
                if let Some(alt) = &i.alternative {
                    self.generate_statement(alt, output, Some(&format!("N{}", id)));
                }
            }
            Statement::Loop(l) => {
                self.generate_expr(&l.condition, output, Some(&format!("N{}", id)));
                for b in &l.body {
                    self.generate_statement(b, output, Some(&format!("N{}", id)));
                }
            }
            Statement::Repeat(r) => {
                self.generate_statement(&r.body, output, Some(&format!("N{}", id)));
                self.generate_expr(&r.condition, output, Some(&format!("N{}", id)));
            }
            Statement::Break(_) => {}
            Statement::Expression(e) => {
                self.generate_expr(&e.expr, output, Some(&format!("N{}", id)));
            }
            Statement::Block(b) => {
                for s in &b.body {
                    self.generate_statement(s, output, Some(&format!("N{}", id)));
                }
            }
        }
    }

    fn generate_expr(&mut self, expr: &Expr, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        let label = match expr {
            Expr::Binary(b) => format!("{:?}", b.operator),
            Expr::Unary(u) => format!("{:?}", u.operator),
            Expr::Call(_) => "call_expr".to_string(),
            Expr::Slice(_) => "slice_expr".to_string(),
            Expr::Identifier(i) => i.name.clone(),
            Expr::Literal(l) => match l {
                Literal::Bool(b) => {
                    if *b {
                        "true".to_string()
                    } else {
                        "false".to_string()
                    }
                }
                Literal::Str(s) => s.clone(),
                Literal::Char(c) => c.to_string(),
                Literal::Hex(n) => n.to_string(),
                Literal::Bits(n) => n.to_string(),
                Literal::Dec(n) => n.to_string(),
            },
            Expr::Parenthesized(_) => "parenthesized_expr".to_string(),
        };

        output.push_str(&format!("N{}[\"{}\"]\n", id, label));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        match expr {
            Expr::Binary(b) => {
                self.generate_expr(&b.left, output, Some(&format!("N{}", id)));
                self.generate_expr(&b.right, output, Some(&format!("N{}", id)));
            }
            Expr::Unary(u) => {
                self.generate_expr(&u.operand, output, Some(&format!("N{}", id)));
            }
            Expr::Call(c) => {
                self.generate_expr(&c.function, output, Some(&format!("N{}", id)));
                for arg in &c.arguments {
                    self.generate_expr(arg, output, Some(&format!("N{}", id)));
                }
            }
            Expr::Slice(s) => {
                self.generate_expr(&s.array, output, Some(&format!("N{}", id)));
                for r in &s.ranges {
                    self.generate_range(r, output, Some(&format!("N{}", id)));
                }
            }
            Expr::Parenthesized(e) => {
                self.generate_expr(e, output, Some(&format!("N{}", id)));
            }
            Expr::Identifier(_) | Expr::Literal(_) => {}
        }
    }

    fn generate_range(&mut self, range: &Range, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"range\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_expr(&range.start, output, Some(&format!("N{}", id)));

        if let Some(end) = &range.end {
            self.generate_expr(end, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_identifier(
        &mut self,
        id: &Identifier,
        output: &mut String,
        parent_id: Option<&str>,
    ) {
        let node_id = self.next_id();
        output.push_str(&format!("N{}[\"identifier: {}\"]\n", node_id, id.name));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, node_id));
        }
    }

    fn next_id(&mut self) -> usize {
        let id = self.id_counter;
        self.id_counter += 1;
        id
    }
}

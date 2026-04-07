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
        let id = self.next_id();
        output.push_str(&format!("N{}[\"source_item\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        match item {
            SourceItem::FuncDeclaration(f) => {
                self.generate_func_declaration(f, output, Some(&format!("N{}", id)));
            }
            SourceItem::FuncDefinition(f) => {
                self.generate_func_definition(f, output, Some(&format!("N{}", id)));
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
        output.push_str(&format!("N{}[\"def\"]\n", id));

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
        output.push_str(&format!("N{}[\"def\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_func_signature(&f.signature, output, Some(&format!("N{}", id)));

        for stmt in &f.body {
            self.generate_statement(stmt, output, Some(&format!("N{}", id)));
        }

        let end_id = self.next_id();
        output.push_str(&format!("N{}[\"end\"]\n", end_id));
        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, end_id));
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

        let lparen_id = self.next_id();
        output.push_str(&format!("N{}[\"(\"]\n", lparen_id));
        output.push_str(&format!("N{} --> N{}\n", id, lparen_id));

        if let Some(params) = &sig.parameters {
            let mut first = true;
            for arg in params {
                if first {
                    first = false;
                } else {
                    let comma_id = self.next_id();
                    output.push_str(&format!("N{}[\",\"]\n", comma_id));
                    output.push_str(&format!("N{} --> N{}\n", id, comma_id));
                }
                self.generate_arg(arg, output, Some(&format!("N{}", id)));
            }
        }

        let rparen_id = self.next_id();
        output.push_str(&format!("N{}[\")\"]\n", rparen_id));
        output.push_str(&format!("N{} --> N{}\n", id, rparen_id));

        if let Some(ret) = &sig.return_type {
            self.generate_type_ref(ret, output, Some(&format!("N{}", id)));
        }
    }

    fn generate_arg(&mut self, arg: &Arg, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"identifier\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        self.generate_identifier(&arg.name, output, Some(&format!("N{}", id)));

        if let Some(ty) = &arg.ty {
            let of_id = self.next_id();
            output.push_str(&format!("N{}[\"of\"]\n", of_id));
            output.push_str(&format!("N{} --> N{}\n", id, of_id));
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

    fn generate_expr(&mut self, expr: &Expr, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{}[\"expr\"]\n", id));

        if let Some(p) = parent_id {
            output.push_str(&format!("{} --> N{}\n", p, id));
        }

        match expr {
            Expr::Binary(b) => {
                let bin_id = self.next_id();
                let op_str = match b.operator {
                    BinaryOp::Multiply => "*",
                    BinaryOp::Divide => "/",
                    BinaryOp::Modulo => "%",
                    BinaryOp::Add => "+",
                    BinaryOp::Subtract => "-",
                    BinaryOp::Less => "<",
                    BinaryOp::Greater => ">",
                    BinaryOp::Equal => "==",
                    BinaryOp::NotEqual => "!=",
                    BinaryOp::LessOrEqual => "<=",
                    BinaryOp::GreaterOrEqual => ">=",
                    BinaryOp::And => "&&",
                    BinaryOp::Or => "||",
                    BinaryOp::Assign => "=",
                };
                output.push_str(&format!("N{}[\"binary_expr\"]\n", bin_id));
                output.push_str(&format!("N{} --> N{}\n", id, bin_id));

                self.generate_expr(&b.left, output, Some(&format!("N{}", bin_id)));

                let op_id = self.next_id();
                output.push_str(&format!("N{}[\"{}\"]\n", op_id, op_str));
                output.push_str(&format!("N{} --> N{}\n", bin_id, op_id));

                self.generate_expr(&b.right, output, Some(&format!("N{}", bin_id)));
            }
            Expr::Unary(u) => {
                let un_id = self.next_id();
                let op_str = match u.operator {
                    UnaryOp::Negate => "-",
                    UnaryOp::Plus => "+",
                    UnaryOp::Not => "!",
                    UnaryOp::BitNot => "~",
                };
                output.push_str(&format!("N{}[\"unary_expr\"]\n", un_id));
                output.push_str(&format!("N{} --> N{}\n", id, un_id));

                let op_id = self.next_id();
                output.push_str(&format!("N{}[\"{}\"]\n", op_id, op_str));
                output.push_str(&format!("N{} --> N{}\n", un_id, op_id));

                self.generate_expr(&u.operand, output, Some(&format!("N{}", un_id)));
            }
            Expr::Call(c) => {
                let call_id = self.next_id();
                output.push_str(&format!("N{}[\"call_expr\"]\n", call_id));
                output.push_str(&format!("N{} --> N{}\n", id, call_id));

                self.generate_expr(&c.function, output, Some(&format!("N{}", call_id)));

                for arg in &c.arguments {
                    self.generate_expr(arg, output, Some(&format!("N{}", call_id)));
                }
            }
            Expr::Slice(s) => {
                let slice_id = self.next_id();
                output.push_str(&format!("N{}[\"slice_expr\"]\n", slice_id));
                output.push_str(&format!("N{} --> N{}\n", id, slice_id));

                self.generate_expr(&s.array, output, Some(&format!("N{}", slice_id)));

                for r in &s.ranges {
                    self.generate_range(r, output, Some(&format!("N{}", slice_id)));
                }
            }
            Expr::Identifier(i) => {
                let ident_id = self.next_id();
                output.push_str(&format!("N{}[\"identifier: {}\"]\n", ident_id, i.name));
                output.push_str(&format!("N{} --> N{}\n", id, ident_id));
            }
            Expr::Literal(l) => {
                let lit_id = self.next_id();
                let (type_label, value_label): (&str, String) = match l {
                    Literal::Bool(b) => (
                        "bool",
                        if *b {
                            "true".to_string()
                        } else {
                            "false".to_string()
                        },
                    ),
                    Literal::Str(s) => ("str", format!("\"{}\"", s)),
                    Literal::Char(c) => ("char", c.to_string()),
                    Literal::Hex(n) => ("hex", format!("0x{:x}", n)),
                    Literal::Bits(n) => ("bits", format!("0b{:b}", n)),
                    Literal::Dec(n) => ("dec", n.to_string()),
                };
                output.push_str(&format!("N{}[\"literal\"]\n", lit_id));
                output.push_str(&format!("N{} --> N{}\n", id, lit_id));

                let val_id = self.next_id();
                output.push_str(&format!(
                    "N{}[\"{}: {}\"]\n",
                    val_id, type_label, value_label
                ));
                output.push_str(&format!("N{} --> N{}\n", lit_id, val_id));
            }
            Expr::Parenthesized(e) => {
                let paren_id = self.next_id();
                output.push_str(&format!("N{}[\"parenthesized_expr\"]\n", paren_id));
                output.push_str(&format!("N{} --> N{}\n", id, paren_id));

                self.generate_expr(e, output, Some(&format!("N{}", paren_id)));
            }
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
            let dotdot_id = self.next_id();
            output.push_str(&format!("N{}[\"..\"]\n", dotdot_id));
            output.push_str(&format!("N{} --> N{}\n", id, dotdot_id));

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

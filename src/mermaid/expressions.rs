use super::MermaidGenerator;
use crate::ast::{BinaryOp, Expr, Literal, Range, UnaryOp};

impl MermaidGenerator {
    pub fn generate_expr(&mut self, expr: &Expr, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{id}[\"expr\"]\n"));

        if let Some(p) = parent_id {
            output.push_str(&format!("{p} --> N{id}\n"));
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
                    BinaryOp::BitAnd => "&",
                    BinaryOp::BitOr => "|",
                    BinaryOp::BitXor => "^",
                    BinaryOp::Assign => "=",
                };
                output.push_str(&format!("N{bin_id}[\"binary_expr\"]\n"));
                output.push_str(&format!("N{id} --> N{bin_id}\n"));

                self.generate_expr(&b.left, output, Some(&format!("N{bin_id}")));

                let op_id = self.next_id();
                output.push_str(&format!("N{op_id}[\"{op_str}\"]\n"));
                output.push_str(&format!("N{bin_id} --> N{op_id}\n"));

                self.generate_expr(&b.right, output, Some(&format!("N{bin_id}")));
            }
            Expr::Unary(u) => {
                let un_id = self.next_id();
                let op_str = match u.operator {
                    UnaryOp::Negate => "-",
                    UnaryOp::Not => "!",
                    UnaryOp::BitNot => "~",
                };
                output.push_str(&format!("N{un_id}[\"unary_expr\"]\n"));
                output.push_str(&format!("N{id} --> N{un_id}\n"));

                let op_id = self.next_id();
                output.push_str(&format!("N{op_id}[\"{op_str}\"]\n"));
                output.push_str(&format!("N{un_id} --> N{op_id}\n"));

                self.generate_expr(&u.operand, output, Some(&format!("N{un_id}")));
            }
            Expr::Call(c) => {
                let call_id = self.next_id();
                output.push_str(&format!("N{call_id}[\"call_expr\"]\n"));
                output.push_str(&format!("N{id} --> N{call_id}\n"));

                self.generate_expr(&c.function, output, Some(&format!("N{call_id}")));

                for arg in &c.arguments {
                    self.generate_expr(arg, output, Some(&format!("N{call_id}")));
                }
            }
            Expr::Slice(s) => {
                let slice_id = self.next_id();
                output.push_str(&format!("N{slice_id}[\"slice_expr\"]\n"));
                output.push_str(&format!("N{id} --> N{slice_id}\n"));

                self.generate_expr(&s.array, output, Some(&format!("N{slice_id}")));

                for r in &s.ranges {
                    self.generate_range(r, output, Some(&format!("N{slice_id}")));
                }
            }
            Expr::Identifier(i) => {
                let ident_id = self.next_id();
                output.push_str(&format!("N{}[\"identifier: {}\"]\n", ident_id, i.name));
                output.push_str(&format!("N{id} --> N{ident_id}\n"));
            }
            Expr::Literal(l, _) => {
                let lit_id = self.next_id();
                let (type_label, value_label): (&str, String) = match l {
                    Literal::Bool(b) => ("bool", if *b { "true".to_string() } else { "false".to_string() }),
                    Literal::Str(s) => ("str", format!("\"{s}\"")),
                    Literal::Char(c) => ("char", c.to_string()),
                    Literal::Hex(n) => ("hex", format!("0x{n:x}")),
                    Literal::Bits(n) => ("bits", format!("0b{n:b}")),
                    Literal::Dec(n) => ("dec", n.to_string()),
                };
                output.push_str(&format!("N{lit_id}[\"literal\"]\n"));
                output.push_str(&format!("N{id} --> N{lit_id}\n"));

                let val_id = self.next_id();
                output.push_str(&format!("N{val_id}[\"{type_label}: {value_label}\"]\n"));
                output.push_str(&format!("N{lit_id} --> N{val_id}\n"));
            }
            Expr::Parenthesized(e) => {
                let paren_id = self.next_id();
                output.push_str(&format!("N{paren_id}[\"parenthesized_expr\"]\n"));
                output.push_str(&format!("N{id} --> N{paren_id}\n"));

                self.generate_expr(e, output, Some(&format!("N{paren_id}")));
            }
            Expr::ArrayLiteral(elements, _) => {
                let arr_id = self.next_id();
                output.push_str(&format!("N{}[\"array_literal [{}]\"]\n", arr_id, elements.len()));
                output.push_str(&format!("N{id} --> N{arr_id}\n"));
                for elem in elements {
                    self.generate_expr(elem, output, Some(&format!("N{arr_id}")));
                }
            }
            Expr::FieldAccess(base, field, _) => {
                let fa_id = self.next_id();
                output.push_str(&format!("N{}[\"field_access .{}\"]\n", fa_id, field.name));
                output.push_str(&format!("N{id} --> N{fa_id}\n"));
                self.generate_expr(base, output, Some(&format!("N{fa_id}")));
            }
            Expr::FuncLiteral(f) => {
                let fl_id = self.next_id();
                output.push_str(&format!("N{}[\"func_literal {}\"]\n", fl_id, f.signature.name.name));
                output.push_str(&format!("N{id} --> N{fl_id}\n"));
            }
        }
    }

    fn generate_range(&mut self, range: &Range, output: &mut String, parent_id: Option<&str>) {
        let id = self.next_id();
        output.push_str(&format!("N{id}[\"range\"]\n"));

        if let Some(p) = parent_id {
            output.push_str(&format!("{p} --> N{id}\n"));
        }

        self.generate_expr(&range.start, output, Some(&format!("N{id}")));

        if let Some(end) = &range.end {
            let dotdot_id = self.next_id();
            output.push_str(&format!("N{dotdot_id}[\"..\"]\n"));
            output.push_str(&format!("N{id} --> N{dotdot_id}\n"));

            self.generate_expr(end, output, Some(&format!("N{id}")));
        }
    }
}

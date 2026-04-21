use super::MermaidGenerator;
use crate::ast::*;

impl MermaidGenerator {
    pub fn generate_expr(&mut self, expr: &Expr, output: &mut String, parent_id: Option<&str>) {
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
}

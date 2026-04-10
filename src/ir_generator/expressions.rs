use super::IrGenerator;
use crate::ast::*;
use crate::ir::*;

impl IrGenerator {
    pub fn visit_expr(&mut self, block: &mut IrBlock, expr: &Expr) -> (String, IrType) {
        match expr {
            Expr::Binary(bin) => self.visit_binary_expr(block, bin),
            Expr::Unary(un) => self.visit_unary_expr(block, un),
            Expr::Parenthesized(inner) => self.visit_expr(block, inner),
            Expr::Call(call) => self.visit_call_expr(block, call),
            Expr::Slice(slice) => self.visit_slice_expr(block, slice),
            Expr::Identifier(id) => (id.name.clone(), self.get_ident_type(id)),
            Expr::Literal(lit) => self.visit_literal_expr(block, lit),
        }
    }

    pub fn visit_binary_expr(
        &mut self,
        block: &mut IrBlock,
        expr: &BinaryExpr,
    ) -> (String, IrType) {
        let (left_temp, _) = self.visit_expr(block, &expr.left);
        let (right_temp, _) = self.visit_expr(block, &expr.right);

        let result_temp = self.generate_temp();

        let (opcode, result_type) = match expr.operator {
            BinaryOp::Multiply => (IrOpcode::Mul, IrType::Int),
            BinaryOp::Divide => (IrOpcode::Div, IrType::Int),
            BinaryOp::Modulo => (IrOpcode::Mod, IrType::Int),
            BinaryOp::Add => (IrOpcode::Add, IrType::Int),
            BinaryOp::Subtract => (IrOpcode::Sub, IrType::Int),
            BinaryOp::Equal => (IrOpcode::Eq, IrType::Bool),
            BinaryOp::NotEqual => (IrOpcode::Ne, IrType::Bool),
            BinaryOp::Less => (IrOpcode::Lt, IrType::Bool),
            BinaryOp::Greater => (IrOpcode::Gt, IrType::Bool),
            BinaryOp::LessOrEqual => (IrOpcode::Le, IrType::Bool),
            BinaryOp::GreaterOrEqual => (IrOpcode::Ge, IrType::Bool),
            BinaryOp::And => (IrOpcode::And, IrType::Bool),
            BinaryOp::Or => (IrOpcode::Or, IrType::Bool),
            BinaryOp::Assign => {
                // For assignment, we need to track the actual variable name, not convert to temp
                let target_name = match expr.left.as_ref() {
                    Expr::Identifier(id) => id.name.clone(),
                    _ => left_temp.clone(),
                };

                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(target_name.clone()),
                    result_type: None,
                    operands: vec![IrOperand::Variable(right_temp.clone(), IrType::Int)],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });

                // Add to locals only if it's a new variable (implicit declaration)
                // Only add if not already declared (as parameter or earlier assignment)
                if !self.declared_vars.contains(&target_name) {
                    self.locals.insert(
                        target_name.clone(),
                        IrLocal {
                            name: target_name.clone(),
                            ty: IrType::Int,
                            stack_offset: None,
                        },
                    );
                    self.declared_vars.insert(target_name);
                }
                return (right_temp, IrType::Int);
            }
        };

        block.instructions.push(IrInstruction {
            opcode,
            result: Some(result_temp.clone()),
            result_type: Some(result_type.clone()),
            operands: vec![
                IrOperand::Variable(left_temp, IrType::Int),
                IrOperand::Variable(right_temp, IrType::Int),
            ],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });

        (result_temp, result_type)
    }

    pub fn visit_unary_expr(&mut self, block: &mut IrBlock, expr: &UnaryExpr) -> (String, IrType) {
        let (operand_temp, _) = self.visit_expr(block, &expr.operand);

        let result_temp = self.generate_temp();

        let (opcode, result_type) = match expr.operator {
            UnaryOp::Negate => (IrOpcode::Neg, IrType::Int),
            UnaryOp::Plus => (IrOpcode::Pos, IrType::Int),
            UnaryOp::Not => (IrOpcode::Not, IrType::Bool),
            UnaryOp::BitNot => (IrOpcode::BitNot, IrType::Int),
        };

        block.instructions.push(IrInstruction {
            opcode,
            result: Some(result_temp.clone()),
            result_type: Some(result_type.clone()),
            operands: vec![IrOperand::Variable(operand_temp, IrType::Int)],
            jump_target: None,
            true_target: None,
            false_target: None,
            span: expr.span,
        });

        (result_temp, result_type)
    }

    pub fn visit_call_expr(&mut self, block: &mut IrBlock, expr: &CallExpr) -> (String, IrType) {
        let func_name = match *expr.function.clone() {
            Expr::Identifier(id) => id.name,
            _ => String::new(),
        };

        let mut args = Vec::new();
        for arg in &expr.arguments {
            let (temp, _) = self.visit_expr(block, arg);
            args.push(IrOperand::Variable(temp, IrType::Int));
        }

        let result_temp = self.generate_temp();

        block.instructions.push(IrInstruction {
            opcode: IrOpcode::Call,
            result: Some(result_temp.clone()),
            result_type: Some(IrType::Int),
            operands: args,
            jump_target: Some(func_name.clone()),
            true_target: None,
            false_target: None,
            span: expr.span,
        });

        self.used_functions.push(func_name);

        (result_temp, IrType::Int)
    }

    pub fn visit_slice_expr(&mut self, block: &mut IrBlock, expr: &SliceExpr) -> (String, IrType) {
        let (array_temp, array_type) = self.visit_expr(block, &expr.array);

        let element_type = match &array_type {
            IrType::Array(ref elem, _) => *elem.clone(),
            _ => IrType::Int,
        };

        if let Some(range) = expr.ranges.first() {
            let (start_temp, _) = self.visit_expr(block, &range.start);

            let result_temp = self.generate_temp();

            if range.end.is_some() {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Slice,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Array(Box::new(element_type.clone()), 0)),
                    operands: vec![
                        IrOperand::Variable(array_temp, array_type.clone()),
                        IrOperand::Variable(start_temp, IrType::Int),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
            } else {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Load,
                    result: Some(result_temp.clone()),
                    result_type: Some(element_type.clone()),
                    operands: vec![
                        IrOperand::Variable(array_temp, array_type.clone()),
                        IrOperand::Variable(start_temp, IrType::Int),
                    ],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: expr.span,
                });
            }

            return (result_temp, element_type);
        }

        (String::new(), IrType::Int)
    }

    pub fn visit_literal_expr(&mut self, block: &mut IrBlock, lit: &Literal) -> (String, IrType) {
        let result_temp = self.generate_temp();

        match lit {
            Literal::Bool(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Bool),
                    operands: vec![IrOperand::Constant(Constant::Bool(*v))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Bool)
            }
            Literal::Dec(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Int(*v as i64))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Int)
            }
            Literal::Hex(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Int(*v as i64))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Int)
            }
            Literal::Bits(v) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Int(*v as i64))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Int)
            }
            Literal::Char(c) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::Int),
                    operands: vec![IrOperand::Constant(Constant::Char(*c as u8))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::Int)
            }
            Literal::Str(s) => {
                block.instructions.push(IrInstruction {
                    opcode: IrOpcode::Assign,
                    result: Some(result_temp.clone()),
                    result_type: Some(IrType::String),
                    operands: vec![IrOperand::Constant(Constant::String(s.clone()))],
                    jump_target: None,
                    true_target: None,
                    false_target: None,
                    span: Span::new(0, 0),
                });
                (result_temp, IrType::String)
            }
        }
    }
}

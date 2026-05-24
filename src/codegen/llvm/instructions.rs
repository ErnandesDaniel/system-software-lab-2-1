use super::LlvmGenerator;
use crate::ir::{Constant, IrInstruction, IrOpcode, IrOperand};

impl LlvmGenerator {
    pub(crate) fn generate_instruction(&mut self, inst: &IrInstruction) -> String {
        let mut out = String::new();

        match inst.opcode {
            IrOpcode::Assign => {
                use crate::codegen::traits::OperandLoader;

                let res_name = inst.result.as_ref().unwrap();
                let (pre, val) = self.load_op(&inst.operands[0]);
                out.push_str(&pre);
                let ty = self.ir_type_to_llvm(inst.result_type.as_ref().unwrap());
                let is_result_temp = Self::is_temp(res_name);

                if is_result_temp && !ty.contains('*') {
                    // Integer SSA temp: use add 0, x pattern
                    out.push_str(&format!("  %{res_name} = add {ty} 0, {val}\n"));
                } else if is_result_temp {
                    // Pointer SSA temp: use alloca + store (not pure SSA but works)
                    out.push_str(&format!("  %{res_name} = alloca {ty}\n"));
                    out.push_str(&format!("  store {ty} {val}, {ty}* %{res_name}\n"));
                } else {
                    // User variable - store to alloca
                    out.push_str(&format!("  store {ty} {val}, {ty}* %{res_name}\n"));
                }
            }

            IrOpcode::Add | IrOpcode::Sub | IrOpcode::Mul | IrOpcode::Div | IrOpcode::Mod => {
                let res = inst.result.as_ref().unwrap();
                let (p1, v1) = self.load_op(&inst.operands[0]);
                let (p2, v2) = self.load_op(&inst.operands[1]);
                out.push_str(&p1);
                out.push_str(&p2);

                let op = match inst.opcode {
                    IrOpcode::Add => "add",
                    IrOpcode::Sub => "sub",
                    IrOpcode::Mul => "mul",
                    IrOpcode::Div => "sdiv",
                    IrOpcode::Mod => "srem",
                    _ => unreachable!(),
                };
                let ty = self.ir_type_to_llvm(inst.result_type.as_ref().unwrap());
                out.push_str(&format!("  %{res} = {op} {ty} {v1}, {v2}\n"));
            }

            IrOpcode::And | IrOpcode::Or => {
                let res = inst.result.as_ref().unwrap();
                let (p1, v1) = self.load_op(&inst.operands[0]);
                let (p2, v2) = self.load_op(&inst.operands[1]);
                out.push_str(&p1);
                out.push_str(&p2);

                let op = match inst.opcode {
                    IrOpcode::And => "and",
                    IrOpcode::Or => "or",
                    _ => unreachable!(),
                };
                out.push_str(&format!("  %{res} = {op} i32 {v1}, {v2}\n"));
            }

            IrOpcode::Eq | IrOpcode::Lt | IrOpcode::Gt | IrOpcode::Le | IrOpcode::Ge | IrOpcode::Ne => {
                let res = inst.result.as_ref().unwrap();
                let (p1, v1) = self.load_op(&inst.operands[0]);
                let (p2, v2) = self.load_op(&inst.operands[1]);
                out.push_str(&p1);
                out.push_str(&p2);

                let cond = match inst.opcode {
                    IrOpcode::Eq => "eq",
                    IrOpcode::Ne => "ne",
                    IrOpcode::Lt => "slt",
                    IrOpcode::Le => "sle",
                    IrOpcode::Gt => "sgt",
                    IrOpcode::Ge => "sge",
                    _ => unreachable!(),
                };

                let tmp = self.fresh_tmp();
                out.push_str(&format!("  %{tmp} = icmp {cond} i32 {v1}, {v2}\n"));
                out.push_str(&format!("  %{res} = zext i1 %{tmp} to i32\n"));
            }

            IrOpcode::Call => {
                let target = inst.jump_target.as_ref().unwrap();
                self.extern_decls.insert(target.clone());

                let mut args = Vec::new();
                for op in &inst.operands {
                    let (pre, val) = self.load_op(op);
                    out.push_str(&pre);
                    args.push(format!("{} {}", self.ir_type_to_llvm(&op.get_type()), val));
                }

                let ret_ty = if Self::is_void_func(target) { "void" } else { "i32" };

                if let Some(res) = &inst.result {
                    out.push_str(&format!(
                        "  %{} = call {} @{}({})\n",
                        res,
                        ret_ty,
                        target,
                        args.join(", ")
                    ));
                } else {
                    out.push_str(&format!("  call {} @{}({})\n", ret_ty, target, args.join(", ")));
                }
            }

            IrOpcode::CondBr => {
                let (pre, val) = self.load_op(&inst.operands[0]);
                out.push_str(&pre);
                let tmp = self.fresh_tmp();
                out.push_str(&format!("  %{tmp} = icmp ne i32 {val}, 0\n"));
                out.push_str(&format!(
                    "  br i1 %{}, label %{}, label %{}\n",
                    tmp,
                    Self::block_id_to_label(inst.true_target.as_ref().unwrap()),
                    Self::block_id_to_label(inst.false_target.as_ref().unwrap())
                ));
            }

            IrOpcode::Jump => {
                out.push_str(&format!(
                    "  br label %{}\n",
                    Self::block_id_to_label(inst.jump_target.as_ref().unwrap())
                ));
            }

            IrOpcode::Ret => {
                if inst.operands.is_empty() {
                    out.push_str("  ret void\n");
                } else {
                    let (pre, val) = self.load_op(&inst.operands[0]);
                    out.push_str(&pre);
                    out.push_str(&format!("  ret i32 {val}\n"));
                }
            }

            _ => out.push_str(&format!("  ; Opcode {:?} not implemented\n", inst.opcode)),
        }

        out
    }

    pub(crate) fn load_op(&mut self, op: &IrOperand) -> (String, String) {
        use crate::codegen::traits::OperandLoader;

        match op {
            IrOperand::Constant(c) => (String::new(), self.const_to_str(c)),
            IrOperand::Variable(name, ty) => {
                let llvm_ty = self.ir_type_to_llvm(ty);
                // Use trait method to check if temp
                if Self::is_temp(name) {
                    if llvm_ty.contains('*') {
                        let tmp = self.fresh_tmp();
                        (
                            format!("  %{tmp} = load {llvm_ty}, {llvm_ty}* %{name}\n"),
                            format!("%{tmp}"),
                        )
                    } else {
                        (String::new(), format!("%{name}"))
                    }
                } else {
                    let tmp = self.fresh_tmp();
                    (
                        format!("  %{tmp} = load {llvm_ty}, {llvm_ty}* %{name}\n"),
                        format!("%{tmp}"),
                    )
                }
            }
            IrOperand::FuncRef(name) => (String::new(), format!("@{name}")),
        }
    }

    fn const_to_str(&mut self, c: &Constant) -> String {
        match c {
            Constant::Int(v) => v.to_string(),
            Constant::Bool(b) => {
                if *b {
                    "1".to_string()
                } else {
                    "0".to_string()
                }
            }
            Constant::String(s) => {
                let label = format!("str.{}", self.string_counter);
                self.global_strings.push((label.clone(), s.clone()));
                self.string_counter += 1;
                format!(
                    "getelementptr inbounds ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)",
                    s.len() + 1,
                    s.len() + 1,
                    label
                )
            }
            Constant::Char(c) => (*c as i8).to_string(),
            Constant::Array(_) => "0".to_string(),
        }
    }

    pub(crate) fn fresh_tmp(&mut self) -> String {
        self.tmp_counter += 1;
        format!("tmp.{}", self.tmp_counter)
    }

    fn is_void_func(name: &str) -> bool {
        matches!(name, "puts" | "printf" | "srand")
    }
}

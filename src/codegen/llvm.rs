use crate::ir::*;
use std::collections::HashSet;

pub struct LlvmGenerator {
    extern_decls: HashSet<String>,
    global_strings: Vec<(String, String)>,
    string_counter: usize,
    tmp_counter: usize,
}

impl LlvmGenerator {
    pub fn new() -> Self {
        Self {
            extern_decls: HashSet::new(),
            global_strings: Vec::new(),
            string_counter: 0,
            tmp_counter: 0,
        }
    }

    pub fn generate_program(&mut self, program: &IrProgram) -> String {
        let mut funcs_code = String::new();
        for func in &program.functions {
            funcs_code.push_str(&self.generate_function(func));
            funcs_code.push_str("\n");
        }

        let mut output = String::new();

        for (label, content) in &self.global_strings {
            // Escape special characters for LLVM IR
            let mut escaped = String::new();
            for c in content.chars() {
                match c {
                    '\n' => escaped.push_str("\\0A"),
                    '\r' => escaped.push_str("\\0D"),
                    '\t' => escaped.push_str("\\09"),
                    '"' => escaped.push_str("\\22"),
                    '\\' => escaped.push_str("\\5C"),
                    _ => escaped.push(c),
                }
            }
            // Array size is content.len() + 1 (for null terminator)
            // LLVM IR escapes like \0A represent 1 byte each
            output.push_str(&format!(
                "@{} = private unnamed_addr constant [{} x i8] c\"{}\\00\", align 1\n",
                label,
                content.len() + 1,
                escaped
            ));
        }
        output.push_str("\n");

        for ext in &self.extern_decls {
            output.push_str(&self.get_extern_signature(ext));
        }
        output.push_str("\n");

        output.push_str(&funcs_code);
        output
    }

    pub fn generate_function(&mut self, func: &IrFunction) -> String {
        let mut output = String::new();
        self.tmp_counter = 0;

        let ret_type = self.ir_type_to_llvm(&func.return_type);
        let params: Vec<String> = func.parameters.iter()
            .map(|p| format!("{} %p.{}", self.ir_type_to_llvm(&p.ty), p.name))
            .collect();

        output.push_str(&format!("define {} @{}({}) {{\n", ret_type, func.name, params.join(", ")));
        output.push_str("entry:\n");

        for param in &func.parameters {
            let ty = self.ir_type_to_llvm(&param.ty);
            output.push_str(&format!("  %{} = alloca {}\n", param.name, ty));
            output.push_str(&format!("  store {} %p.{}, {}* %{}\n", ty, param.name, ty, param.name));
        }

        for local in &func.locals {
            if !local.name.starts_with('t') && !func.parameters.iter().any(|p| p.name == local.name) {
                output.push_str(&format!("  %{} = alloca {}\n", local.name, self.ir_type_to_llvm(&local.ty)));
            }
        }

        output.push_str("  br label %bb_0\n");

        for (idx, block) in func.blocks.iter().enumerate() {
            let label = if idx == 0 {
                "bb_0"
            } else {
                &self.block_id_to_label(&block.id)
            };
            output.push_str(&format!("{}:\n", label));

            for inst in &block.instructions {
                output.push_str(&self.generate_instruction(inst));
            }
        }

        output.push_str("}\n");
        output
    }

    fn generate_instruction(&mut self, inst: &IrInstruction) -> String {
        let mut out = String::new();

        match inst.opcode {
            IrOpcode::Assign => {
                let res_name = inst.result.as_ref().unwrap();
                let (pre, val) = self.load_op(&inst.operands[0]);
                out.push_str(&pre);
                let ty = self.ir_type_to_llvm(inst.result_type.as_ref().unwrap());

                if res_name.starts_with('t') && !ty.contains('*') {
                    // Integer SSA temp: use add 0, x pattern
                    out.push_str(&format!("  %{} = add {} 0, {}\n", res_name, ty, val));
                } else if res_name.starts_with('t') {
                    // Pointer SSA temp: use alloca + store (not pure SSA but works)
                    out.push_str(&format!("  %{} = alloca {}\n", res_name, ty));
                    out.push_str(&format!("  store {} {}, {}* %{}\n", ty, val, ty, res_name));
                } else {
                    // User variable
                    out.push_str(&format!("  store {} {}, {}* %{}\n", ty, val, ty, res_name));
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
                out.push_str(&format!("  %{} = {} {} {}, {}\n", res, op, ty, v1, v2));
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
                out.push_str(&format!("  %{} = {} i32 {}, {}\n", res, op, v1, v2));
            }

            IrOpcode::Eq | IrOpcode::Lt | IrOpcode::Gt | IrOpcode::Le | IrOpcode::Ge | IrOpcode::Ne => {
                let res = inst.result.as_ref().unwrap();
                let (p1, v1) = self.load_op(&inst.operands[0]);
                let (p2, v2) = self.load_op(&inst.operands[1]);
                out.push_str(&p1);
                out.push_str(&p2);

                let cond = match inst.opcode {
                    IrOpcode::Eq => "eq", IrOpcode::Ne => "ne",
                    IrOpcode::Lt => "slt", IrOpcode::Le => "sle",
                    IrOpcode::Gt => "sgt", IrOpcode::Ge => "sge",
                    _ => unreachable!(),
                };

                let tmp = self.fresh_tmp();
                out.push_str(&format!("  %{} = icmp {} i32 {}, {}\n", tmp, cond, v1, v2));
                out.push_str(&format!("  %{} = zext i1 %{} to i32\n", res, tmp));
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

                let ret_ty = if self.is_void_func(target) { "void" } else { "i32" };

                if let Some(res) = &inst.result {
                    out.push_str(&format!("  %{} = call {} @{}({})\n", res, ret_ty, target, args.join(", ")));
                } else {
                    out.push_str(&format!("  call {} @{}({})\n", ret_ty, target, args.join(", ")));
                }
            }

            IrOpcode::CondBr => {
                let (pre, val) = self.load_op(&inst.operands[0]);
                out.push_str(&pre);
                let tmp = self.fresh_tmp();
                out.push_str(&format!("  %{} = icmp ne i32 {}, 0\n", tmp, val));
                out.push_str(&format!(
                    "  br i1 %{}, label %{}, label %{}\n",
                    tmp,
                    self.block_id_to_label(inst.true_target.as_ref().unwrap()),
                    self.block_id_to_label(inst.false_target.as_ref().unwrap())
                ));
            }

            IrOpcode::Jump => {
                out.push_str(&format!("  br label %{}\n", self.block_id_to_label(inst.jump_target.as_ref().unwrap())));
            }

            IrOpcode::Ret => {
                if inst.operands.is_empty() {
                    out.push_str("  ret void\n");
                } else {
                    let (pre, val) = self.load_op(&inst.operands[0]);
                    out.push_str(&pre);
                    out.push_str(&format!("  ret i32 {}\n", val));
                }
            }

            _ => out.push_str(&format!("  ; Opcode {:?} not implemented\n", inst.opcode)),
        }

        out
    }

    fn load_op(&mut self, op: &IrOperand) -> (String, String) {
        match op {
            IrOperand::Constant(c) => (String::new(), self.const_to_str(c)),
            IrOperand::Variable(name, ty) => {
                let llvm_ty = self.ir_type_to_llvm(ty);
                if name.starts_with('t') {
                    if llvm_ty.contains('*') {
                        // Pointer temp stored in alloca, need to load
                        let tmp = self.fresh_tmp();
                        (
                            format!("  %{} = load {}, {}* %{}\n", tmp, llvm_ty, llvm_ty, name),
                            format!("%{}", tmp)
                        )
                    } else {
                        // Integer SSA temp
                        (String::new(), format!("%{}", name))
                    }
                } else {
                    // User variable - load from alloca
                    let tmp = self.fresh_tmp();
                    (
                        format!("  %{} = load {}, {}* %{}\n", tmp, llvm_ty, llvm_ty, name),
                        format!("%{}", tmp)
                    )
                }
            }
        }
    }

    fn const_to_str(&mut self, c: &Constant) -> String {
        match c {
            Constant::Int(v) => v.to_string(),
            Constant::Bool(b) => if *b { "1".to_string() } else { "0".to_string() },
            Constant::String(s) => {
                let label = format!("str.{}", self.string_counter);
                self.global_strings.push((label.clone(), s.clone()));
                self.string_counter += 1;
                format!("getelementptr inbounds ([{} x i8], [{} x i8]* @{}, i64 0, i64 0)", s.len() + 1, s.len() + 1, label)
            }
            Constant::Char(c) => (*c as i8).to_string(),
        }
    }

    fn ir_type_to_llvm(&self, ty: &IrType) -> String {
        match ty {
            IrType::Int | IrType::Bool => "i32".to_string(),
            IrType::Void => "void".to_string(),
            IrType::String => "i8*".to_string(),
            IrType::Array(t, sz) => format!("[{} x {}]", sz, self.ir_type_to_llvm(t)),
        }
    }

    fn block_id_to_label(&self, id: &str) -> String {
        format!("bb_{}", id.trim_start_matches("BB"))
    }

    fn fresh_tmp(&mut self) -> String {
        self.tmp_counter += 1;
        format!("tmp.{}", self.tmp_counter)
    }

    fn is_void_func(&self, name: &str) -> bool {
        matches!(name, "puts" | "printf" | "yieldThread" | "srand")
    }

    fn get_extern_signature(&self, name: &str) -> String {
        match name {
            "puts" => "declare i32 @puts(i8*)\n".to_string(),
            "printf" => "declare i32 @printf(i8*, ...)\n".to_string(),
            "srand" => "declare void @srand(i32)\n".to_string(),
            "rand" => "declare i32 @rand()\n".to_string(),
            "time" => "declare i64 @time(i64)\n".to_string(),
            "getchar" => "declare i32 @getchar()\n".to_string(),
            "putchar" => "declare i32 @putchar(i32)\n".to_string(),
            "scanf" => "declare i32 @scanf(i8*, ...)\n".to_string(),
            _ => format!("declare i32 @{}(i32)\n", name),
        }
    }
}

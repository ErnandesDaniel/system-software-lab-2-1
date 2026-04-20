use crate::ir::*;

pub struct LlvmGenerator {
    local_names: Vec<String>,
    param_names: Vec<String>,
    temp_counter: usize,
    extern_decls: Vec<String>,
}

impl LlvmGenerator {
    pub fn new() -> Self {
        Self {
            local_names: Vec::new(),
            param_names: Vec::new(),
            temp_counter: 0,
            extern_decls: Vec::new(),
        }
    }

    pub fn generate_function(&mut self, func: &IrFunction) -> String {
        let mut output = String::new();
        
        self.temp_counter = 0;
        self.local_names.clear();
        self.param_names.clear();
        
        for param in &func.parameters {
            self.param_names.push(param.name.clone());
        }
        for local in &func.locals {
            self.local_names.push(local.name.clone());
        }
        
        let return_type = match func.return_type {
            IrType::Void => "void",
            IrType::Int | IrType::Bool => "i32",
            IrType::String => "i8*",
            _ => "i32",
        };
        
        let params: Vec<String> = func.parameters.iter()
            .map(|p| format!("i32 %{}", p.name))
            .collect();
        let params_str = params.join(", ");
        
        output.push_str(&format!(
            "define {} @{}({}) {{\n",
            return_type, func.name, params_str
        ));
        
        // Allocate locals
        for local in &func.locals {
            let is_param = func.parameters.iter().any(|p| p.name == local.name);
            if !is_param {
                output.push_str(&format!(
                    "  %{} = alloca i32, align 4\n",
                    local.name
                ));
            }
        }
        
        // Store params
        for param in &func.parameters {
            output.push_str(&format!(
                "  store i32 %{}, i32* %{}, align 4\n",
                param.name, param.name
            ));
        }
        
        // Generate instructions
        for block in &func.blocks {
            for inst in &block.instructions {
                let inst_ir = self.generate_instruction(inst, &func.return_type);
                if !inst_ir.is_empty() {
                    output.push_str(&inst_ir);
                }
            }
        }
        
        output.push_str("}\n");
        output
    }

    fn is_local(&self, name: &str) -> bool {
        self.local_names.contains(&name.to_string()) || 
        self.param_names.contains(&name.to_string())
    }

    fn fresh_temp(&mut self) -> String {
        self.temp_counter += 1;
        format!("%t{}", self.temp_counter)
    }

    fn generate_instruction(&mut self, inst: &IrInstruction, return_type: &IrType) -> String {
        let mut output = String::new();
        
        match &inst.opcode {
            IrOpcode::Assign => {
                if let Some(result) = &inst.result {
                    let result_is_local = self.is_local(result);
                    
                    if let Some(IrOperand::Constant(Constant::Int(n))) = inst.operands.first() {
                        let temp = self.fresh_temp();
                        output.push_str(&format!(
                            "  {} = add i32 0, {}\n",
                            temp, n
                        ));
                        if result_is_local {
                            output.push_str(&format!(
                                "  store i32 {}, i32* %{}, align 4\n",
                                temp, result
                            ));
                        }
                    } else if let Some(IrOperand::Variable(name, _)) = inst.operands.first() {
                        let src = self.get_value_ref(name);
                        if self.is_local(name) {
                            let load_temp = self.fresh_temp();
                            output.push_str(&format!(
                                "  {} = load i32, i32* %{}, align 4\n",
                                load_temp, name
                            ));
                            if result_is_local {
                                output.push_str(&format!(
                                    "  store i32 {}, i32* %{}, align 4\n",
                                    load_temp, result
                                ));
                            }
                        }
                    }
                }
            }
            
            IrOpcode::Add | IrOpcode::Sub | IrOpcode::Mul | IrOpcode::Div | IrOpcode::Mod |
            IrOpcode::And | IrOpcode::Or => {
                if let Some(result) = &inst.result {
                    let (preamble, left, right) = self.get_binary_ops(inst);
                    output.push_str(&preamble);
                    
                    let op = match inst.opcode {
                        IrOpcode::Add => "add",
                        IrOpcode::Sub => "sub",
                        IrOpcode::Mul => "mul",
                        IrOpcode::Div => "sdiv",
                        IrOpcode::Mod => "srem",
                        IrOpcode::And => "and",
                        IrOpcode::Or => "or",
                        _ => "add",
                    };
                    
                    let res = self.fresh_temp();
                    output.push_str(&format!(
                        "  {} = {} i32 {}, {}\n",
                        res, op, left, right
                    ));
                    
                    if self.is_local(result) {
                        output.push_str(&format!(
                            "  store i32 {}, i32* %{}, align 4\n",
                            res, result
                        ));
                    }
                }
            }
            
            IrOpcode::Lt | IrOpcode::Le | IrOpcode::Gt | IrOpcode::Ge | IrOpcode::Eq | IrOpcode::Ne => {
                if let Some(result) = &inst.result {
                    let (preamble, left, right) = self.get_binary_ops(inst);
                    output.push_str(&preamble);
                    
                    let pred = match inst.opcode {
                        IrOpcode::Lt => "slt",
                        IrOpcode::Le => "sle",
                        IrOpcode::Gt => "sgt",
                        IrOpcode::Ge => "sge",
                        IrOpcode::Eq => "eq",
                        IrOpcode::Ne => "ne",
                        _ => "eq",
                    };
                    
                    let cmp_temp = self.fresh_temp();
                    output.push_str(&format!(
                        "  {} = icmp {} i32 {}, {}\n",
                        cmp_temp, pred, left, right
                    ));
                    
                    // Convert i1 to i32
                    let res = self.fresh_temp();
                    output.push_str(&format!(
                        "  {} = zext i1 {} to i32\n",
                        res, cmp_temp
                    ));
                    
                    if self.is_local(result) {
                        output.push_str(&format!(
                            "  store i32 {}, i32* %{}, align 4\n",
                            res, result
                        ));
                    }
                }
            }
            
            IrOpcode::Call => {
                if let Some(target) = &inst.jump_target {
                    self.add_extern(target);
                    
                    let has_result = matches!(
                        target.as_str(),
                        "printf" | "scanf" | "puts" | "getchar" | "putchar" | "rand" | "time" | "malloc"
                    );
                    
                    if let Some(result) = &inst.result {
                        if has_result {
                            output.push_str(&format!(
                                "  %{} = call i32 @{}(i32)\n",
                                result, target
                            ));
                        } else {
                            output.push_str(&format!(
                                "  %{} = call i32 @{}(i32)\n",
                                result, target
                            ));
                        }
                        if self.is_local(result) {
                            output.push_str(&format!(
                                "  store i32 %{}, i32* %{}, align 4\n",
                                result, result
                            ));
                        }
                    } else {
                        output.push_str(&format!(
                            "  call i32 @{}(i32)\n",
                            target
                        ));
                    }
                }
            }
            
            IrOpcode::Ret => {
                if let Some(IrOperand::Variable(name, _)) = inst.operands.first() {
                    if self.is_local(name) {
                        let load_temp = self.fresh_temp();
                        output.push_str(&format!(
                            "  {} = load i32, i32* %{}, align 4\n",
                            load_temp, name
                        ));
                        output.push_str(&format!(
                            "  ret i32 {}\n",
                            load_temp
                        ));
                    } else {
                        output.push_str(&format!(
                            "  ret i32 %{}\n",
                            name
                        ));
                    }
                } else if let Some(IrOperand::Constant(Constant::Int(n))) = inst.operands.first() {
                    output.push_str(&format!("  ret i32 {}\n", n));
                } else if matches!(return_type, IrType::Void) {
                    output.push_str("  ret void\n");
                } else {
                    output.push_str("  ret i32 0\n");
                }
            }
            
            IrOpcode::Jump => {
                if let Some(target) = &inst.jump_target {
                    let label = format!("bb_{}", target.trim_start_matches("BB").trim_start_matches("bb_"));
                    output.push_str(&format!("  br label %{}\n", label));
                }
            }
            
            IrOpcode::CondBr => {
                if let Some(IrOperand::Variable(name, _)) = inst.operands.first() {
                    let val;
                    if self.is_local(name) {
                        let load_temp = self.fresh_temp();
                        output.push_str(&format!(
                            "  {} = load i32, i32* %{}, align 4\n",
                            load_temp, name
                        ));
                        val = load_temp;
                    } else {
                        val = format!("%{}", name);
                    }
                    
                    if let (Some(true_target), Some(false_target)) = 
                        (inst.true_target.as_ref(), inst.false_target.as_ref()) 
                    {
                        let true_label = format!("bb_{}", true_target.trim_start_matches("BB").trim_start_matches("bb_"));
                        let false_label = format!("bb_{}", false_target.trim_start_matches("BB").trim_start_matches("bb_"));
                        output.push_str(&format!(
                            "  br i1 {}, label %{}, label %{}\n",
                            val, true_label, false_label
                        ));
                    }
                } else if let Some(IrOperand::Constant(Constant::Int(n))) = inst.operands.first() {
                    if let (Some(true_target), Some(false_target)) = 
                        (inst.true_target.as_ref(), inst.false_target.as_ref()) 
                    {
                        let true_label = format!("bb_{}", true_target.trim_start_matches("BB").trim_start_matches("bb_"));
                        let false_label = format!("bb_{}", false_target.trim_start_matches("BB").trim_start_matches("bb_"));
                        output.push_str(&format!(
                            "  br i1 {}, label %{}, label %{}\n",
                            n, true_label, false_label
                        ));
                    }
                }
            }
            
            IrOpcode::Load => {
                if let Some(result) = &inst.result {
                    if let Some(IrOperand::Variable(src, _)) = inst.operands.first() {
                        if self.is_local(src) {
                            let val = self.get_value_ref(src);
                            if self.is_local(result) {
                                output.push_str(&format!(
                                    "  store i32 {}, i32* %{}, align 4\n",
                                    val, result
                                ));
                            }
                        }
                    }
                }
            }
            
            IrOpcode::Store => {
                if let Some(IrOperand::Variable(dest, _)) = inst.operands.first() {
                    if let Some(IrOperand::Variable(src, _)) = inst.operands.get(1) {
                        let val = self.get_value_ref(src);
                        output.push_str(&format!(
                            "  store i32 {}, i32* %{}, align 4\n",
                            val, dest
                        ));
                    } else if let Some(IrOperand::Constant(Constant::Int(n))) = inst.operands.get(1) {
                        output.push_str(&format!(
                            "  store i32 {}, i32* %{}, align 4\n",
                            n, dest
                        ));
                    }
                }
            }
            
            IrOpcode::Alloca => {}
            
            IrOpcode::CreateThread => {
                if let Some(IrOperand::Constant(Constant::String(name))) = inst.operands.first() {
                    output.push_str(&format!("  call void @{}(i32)\n", name));
                }
            }
            
            IrOpcode::Yield => {
                output.push_str("  call void @yieldThread()\n");
                self.add_extern("yieldThread");
            }
            
            IrOpcode::Neg => {
                if let Some(result) = &inst.result {
                    if let Some(IrOperand::Variable(name, _)) = inst.operands.first() {
                        let (preamble, left, _) = self.get_binary_ops(inst);
                        output.push_str(&preamble);
                        let res = self.fresh_temp();
                        output.push_str(&format!("  {} = sub i32 0, {}\n", res, left));
                        if self.is_local(result) {
                            output.push_str(&format!(
                                "  store i32 {}, i32* %{}, align 4\n",
                                res, result
                            ));
                        }
                    }
                }
            }
            
            _ => {}
        }
        
        output
    }

    fn get_value_ref(&self, name: &str) -> String {
        format!("%{}", name)
    }

    fn get_binary_ops(&mut self, inst: &IrInstruction) -> (String, String, String) {
        let mut preamble = String::new();
        
        let left = if let Some(IrOperand::Variable(name, _)) = inst.operands.first() {
            if self.is_local(name) {
                let temp = self.fresh_temp();
                preamble.push_str(&format!(
                    "  {} = load i32, i32* %{}, align 4\n",
                    temp, name
                ));
                temp
            } else {
                format!("%{}", name)
            }
        } else if let Some(IrOperand::Constant(Constant::Int(n))) = inst.operands.first() {
            n.to_string()
        } else {
            "0".to_string()
        };
        
        let right = if let Some(IrOperand::Variable(name, _)) = inst.operands.get(1) {
            if self.is_local(name) {
                let temp = self.fresh_temp();
                preamble.push_str(&format!(
                    "  {} = load i32, i32* %{}, align 4\n",
                    temp, name
                ));
                temp
            } else {
                format!("%{}", name)
            }
        } else if let Some(IrOperand::Constant(Constant::Int(n))) = inst.operands.get(1) {
            n.to_string()
        } else {
            "0".to_string()
        };
        
        (preamble, left, right)
    }

    fn add_extern(&mut self, name: &str) {
        if !name.is_empty() && !self.extern_decls.contains(&name.to_string()) {
            self.extern_decls.push(name.to_string());
        }
    }

    pub fn get_extern_decls(&self) -> Vec<String> {
        self.extern_decls.clone()
    }
}

impl Default for LlvmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

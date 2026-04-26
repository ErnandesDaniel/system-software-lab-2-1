use crate::codegen::traits::OperandLoader;
use crate::ir::types::*;
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, ConstantPool, Method, MethodAccessFlags};
use std::collections::HashMap;

pub struct JvmGenerator {
    locals: HashMap<String, u16>,
    next_local_slot: u16,
    constant_pool: ConstantPool<'static>,
    method_refs: HashMap<String, u16>,
    string_consts: HashMap<String, u16>,
    current_function_name: String,
    current_params: Vec<IrParameter>,
    current_return_type: IrType,
}

impl JvmGenerator {
    pub fn new() -> Self {
        Self {
            locals: HashMap::new(),
            next_local_slot: 0,
            constant_pool: ConstantPool::default(),
            method_refs: HashMap::new(),
            string_consts: HashMap::new(),
            current_function_name: String::new(),
            current_params: Vec::new(),
            current_return_type: IrType::Void,
        }
    }

    pub fn generate_program(&mut self, program: &IrProgram) -> Vec<(String, Vec<u8>)> {
        let mut classes = Vec::new();

        for func in &program.functions {
            let class_name = if func.name == "main" {
                "Main".to_string()
            } else {
                capitalize_first(&func.name)
            };

            let class_bytes = self.generate_function_class(func, &class_name);
            classes.push((class_name, class_bytes));
        }

        classes
    }

    fn generate_function_class(&mut self, func: &IrFunction, class_name: &str) -> Vec<u8> {
        self.reset_state();
        self.current_function_name = func.name.clone();
        self.current_params = func.parameters.clone();
        self.current_return_type = func.return_type.clone();

        self.setup_local_variables(func);
        
        // First, populate method refs for external calls
        self.collect_external_calls(func);
        
        let code = self.generate_bytecode(func);
        self.build_class_file(class_name, func, code)
    }
    
    fn collect_external_calls(&mut self, func: &IrFunction) {
        // Add RuntimeStub class to constant pool
        let runtime_stub_class = self.constant_pool.add_class("RuntimeStub").unwrap();
        
        for block in &func.blocks {
            for inst in &block.instructions {
                // Collect strings from operands
                for operand in &inst.operands {
                    if let IrOperand::Constant(crate::ir::Constant::String(s)) = operand {
                        if !self.string_consts.contains_key(s) {
                            if let Ok(idx) = self.constant_pool.add_string(s) {
                                self.string_consts.insert(s.clone(), idx);
                            }
                        }
                    }
                }
                
                // Collect method calls
                if let IrOpcode::Call = inst.opcode {
                    if let Some(ref target) = inst.jump_target {
                        // Skip if already added
                        if self.method_refs.contains_key(target) {
                            continue;
                        }
                        
                        // Build descriptor from actual arguments
                        let param_types: Vec<IrType> = inst.operands.iter()
                            .map(|op| op.get_type())
                            .collect();
                        let return_type = inst.result_type.clone();
                        
                        // Determine if this is an external (RuntimeStub) function or user function
                        let (class_idx, method_name, descriptor) = if self.is_external_function(target) {
                            // External function in RuntimeStub
                            let desc = self.get_method_descriptor(target, &param_types, return_type.as_ref());
                            (runtime_stub_class, target.clone(), desc)
                        } else {
                            // User-defined function in its own class
                            let class_name = capitalize_first(target);
                            let user_class = self.constant_pool.add_class(&class_name).unwrap();
                            let desc = self.build_user_method_descriptor(&param_types, return_type.as_ref());
                            (user_class, "call".to_string(), desc)
                        };
                        
                        // Add method ref to constant pool
                        let method_idx = self.constant_pool
                            .add_method_ref(class_idx, &method_name, &descriptor)
                            .unwrap();
                        
                        self.method_refs.insert(target.clone(), method_idx);
                    }
                }
            }
        }
    }
    
    fn is_external_function(&self, name: &str) -> bool {
        matches!(name, "puts" | "putchar" | "getchar" | "printf" | "rand" | "srand" | "time" | "Sleep")
    }
    
    fn build_user_method_descriptor(&self, param_types: &[IrType], return_type: Option<&IrType>) -> String {
        let param_desc: String = param_types.iter()
            .map(|t| ir_type_to_jvm_descriptor(t))
            .collect();
        let ret_desc = return_type.as_ref()
            .map(|t| ir_type_to_jvm_descriptor(t))
            .unwrap_or_else(|| "I".to_string());
        format!("({}){}", param_desc, ret_desc)
    }

    fn reset_state(&mut self) {
        self.locals.clear();
        self.next_local_slot = 0;
        self.constant_pool = ConstantPool::default();
        self.method_refs.clear();
        self.string_consts.clear();
    }

    fn setup_local_variables(&mut self, func: &IrFunction) {
        for param in &func.parameters {
            self.locals.insert(param.name.clone(), self.next_local_slot);
            self.next_local_slot += 1;
        }

        for local in &func.locals {
            if !Self::is_temp(&local.name) && !self.locals.contains_key(&local.name) {
                self.locals.insert(local.name.clone(), self.next_local_slot);
                self.next_local_slot += 1;
            }
        }

        let mut temps_used: Vec<String> = Vec::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref result) = inst.result {
                    if Self::is_temp(result) && !temps_used.contains(result) {
                        temps_used.push(result.clone());
                    }
                }
            }
        }

        for temp in temps_used {
            if !self.locals.contains_key(&temp) {
                self.locals.insert(temp, self.next_local_slot);
                self.next_local_slot += 1;
            }
        }
    }

    fn generate_bytecode(&self, func: &IrFunction) -> Vec<Instruction> {
        let mut code = Vec::new();

        for block in &func.blocks {
            for inst in &block.instructions {
                self.generate_instruction(&mut code, inst);
            }
        }

        code
    }

    fn generate_instruction(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        match inst.opcode {
            IrOpcode::Assign => self.generate_assign(code, inst),
            IrOpcode::Add => self.generate_binary_op(code, inst, BinaryOp::Add),
            IrOpcode::Sub => self.generate_binary_op(code, inst, BinaryOp::Sub),
            IrOpcode::Mul => self.generate_binary_op(code, inst, BinaryOp::Mul),
            IrOpcode::Div => self.generate_binary_op(code, inst, BinaryOp::Div),
            IrOpcode::Mod => self.generate_binary_op(code, inst, BinaryOp::Mod),
            IrOpcode::Neg => self.generate_neg(code, inst),
            IrOpcode::Pos => self.generate_pos(code, inst),
            IrOpcode::And => self.generate_logical_and(code, inst),
            IrOpcode::Or => self.generate_logical_or(code, inst),
            IrOpcode::Not => self.generate_logical_not(code, inst),
            IrOpcode::BitAnd => self.generate_binary_op(code, inst, BinaryOp::BitAnd),
            IrOpcode::BitOr => self.generate_binary_op(code, inst, BinaryOp::BitOr),
            IrOpcode::BitNot => self.generate_bit_not(code, inst),
            IrOpcode::Eq => self.generate_comparison(code, inst, ComparisonOp::Eq),
            IrOpcode::Ne => self.generate_comparison(code, inst, ComparisonOp::Ne),
            IrOpcode::Lt => self.generate_comparison(code, inst, ComparisonOp::Lt),
            IrOpcode::Le => self.generate_comparison(code, inst, ComparisonOp::Le),
            IrOpcode::Gt => self.generate_comparison(code, inst, ComparisonOp::Gt),
            IrOpcode::Ge => self.generate_comparison(code, inst, ComparisonOp::Ge),
            IrOpcode::Call => self.generate_call(code, inst),
            IrOpcode::Ret => self.generate_return(code, inst),
            IrOpcode::Jump => self.generate_jump(code, inst),
            IrOpcode::CondBr => self.generate_conditional_branch(code, inst),
            IrOpcode::Load => self.generate_array_load(code, inst),
            IrOpcode::Slice => {}
            IrOpcode::Alloca => {}
            IrOpcode::Store => {}
            IrOpcode::Cast => {}
        }
    }

    fn generate_assign(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(ref operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            let slot = self.get_local_slot(result);
            
            // Choose store instruction based on type
            match operand.get_type() {
                IrType::String => code.push(Instruction::Astore(slot as u8)),
                _ => code.push(Instruction::Istore(slot as u8)),
            }
        }
    }

    fn generate_binary_op(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, op: BinaryOp) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            self.emit_load_operand(code, right);

            let instr = match op {
                BinaryOp::Add => Instruction::Iadd,
                BinaryOp::Sub => Instruction::Isub,
                BinaryOp::Mul => Instruction::Imul,
                BinaryOp::Div => Instruction::Idiv,
                BinaryOp::Mod => Instruction::Irem,
                BinaryOp::BitAnd => Instruction::Iand,
                BinaryOp::BitOr => Instruction::Ior,
            };
            code.push(instr);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_neg(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Ineg);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_pos(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_bit_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Iconst_m1);
            code.push(Instruction::Ixor);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_logical_and(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            let false_label = 0u16;
            let end_label = 0u16;
            code.push(Instruction::Ifeq(false_label));

            self.emit_load_operand(code, right);
            code.push(Instruction::Ifeq(false_label));

            code.push(Instruction::Iconst_1);
            code.push(Instruction::Goto(end_label));

            code.push(Instruction::Iconst_0);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_logical_or(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            let true_label = 0u16;
            let end_label = 0u16;
            code.push(Instruction::Ifne(true_label));

            self.emit_load_operand(code, right);
            code.push(Instruction::Ifne(true_label));

            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(end_label));

            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_logical_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            let true_label = 0u16;
            let end_label = 0u16;
            code.push(Instruction::Ifeq(true_label));
            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(end_label));
            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_comparison(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, op: ComparisonOp) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            self.emit_load_operand(code, right);

            let true_label = 0u16;

            let branch_instr = match op {
                ComparisonOp::Eq => Instruction::If_icmpeq(true_label),
                ComparisonOp::Ne => Instruction::If_icmpne(true_label),
                ComparisonOp::Lt => Instruction::If_icmplt(true_label),
                ComparisonOp::Le => Instruction::If_icmple(true_label),
                ComparisonOp::Gt => Instruction::If_icmpgt(true_label),
                ComparisonOp::Ge => Instruction::If_icmpge(true_label),
            };
            code.push(branch_instr);

            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(0));

            code.push(Instruction::Iconst_1);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_call(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref target) = inst.jump_target {
            for operand in &inst.operands {
                self.emit_load_operand(code, operand);
            }

            // Get method index from cache
            let method_idx = self.method_refs.get(target).copied().unwrap_or(1);
            code.push(Instruction::Invokestatic(method_idx));

            if let Some(ref result) = inst.result {
                let slot = self.get_local_slot(result);
                code.push(Instruction::Istore(slot as u8));
            }
        }
    }

    fn generate_return(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Ireturn);
        } else {
            code.push(Instruction::Return);
        }
    }

    fn generate_jump(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref _target) = inst.jump_target {
            code.push(Instruction::Goto(0));
        }
    }

    fn generate_conditional_branch(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Ifne(0));
        }
    }

    fn generate_array_load(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(array), Some(index)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, array);
            self.emit_load_operand(code, index);
            code.push(Instruction::Iaload);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn emit_load_operand(&self, code: &mut Vec<Instruction>, operand: &IrOperand) {
        match operand {
            IrOperand::Variable(name, ty) => {
                let slot = self.get_local_slot(name);
                let instr = match ty {
                    IrType::String => {
                        match slot {
                            0 => Instruction::Aload_0,
                            1 => Instruction::Aload_1,
                            2 => Instruction::Aload_2,
                            3 => Instruction::Aload_3,
                            _ => Instruction::Aload(slot as u8),
                        }
                    }
                    _ => {
                        match slot {
                            0 => Instruction::Iload_0,
                            1 => Instruction::Iload_1,
                            2 => Instruction::Iload_2,
                            3 => Instruction::Iload_3,
                            _ => Instruction::Iload(slot as u8),
                        }
                    }
                };
                code.push(instr);
            }
            IrOperand::Constant(c) => self.emit_load_constant(code, c),
        }
    }

    fn emit_load_constant(&self, code: &mut Vec<Instruction>, c: &crate::ir::Constant) {
        use crate::ir::Constant;
        match c {
            Constant::Int(n) => {
                match *n {
                    -1 => code.push(Instruction::Iconst_m1),
                    0 => code.push(Instruction::Iconst_0),
                    1 => code.push(Instruction::Iconst_1),
                    2 => code.push(Instruction::Iconst_2),
                    3 => code.push(Instruction::Iconst_3),
                    4 => code.push(Instruction::Iconst_4),
                    5 => code.push(Instruction::Iconst_5),
                    n if n >= -128 && n <= 127 => code.push(Instruction::Bipush(n as i8)),
                    n if n >= -32768 && n <= 32767 => code.push(Instruction::Sipush(n as i16)),
                    _ => code.push(Instruction::Iconst_0), // Large constants not yet supported
                }
            }
            Constant::Bool(true) => code.push(Instruction::Iconst_1),
            Constant::Bool(false) => code.push(Instruction::Iconst_0),
            Constant::String(s) => {
                // Use pre-collected string index
                let idx = self.string_consts.get(s).copied().unwrap_or(1);
                if idx <= u8::MAX as u16 {
                    code.push(Instruction::Ldc(idx as u8));
                } else {
                    code.push(Instruction::Ldc_w(idx));
                }
            }
            Constant::Char(c) => {
                let val = *c as i32;
                if val >= -128 && val <= 127 {
                    code.push(Instruction::Bipush(val as i8));
                } else {
                    code.push(Instruction::Sipush(val as i16));
                }
            }
        }
    }

    fn get_local_slot(&self, name: &str) -> u16 {
        *self.locals.get(name).unwrap_or(&0)
    }

    fn build_class_file(&mut self, class_name: &str, func: &IrFunction, code: Vec<Instruction>) -> Vec<u8> {
        let this_class = self.constant_pool.add_class(class_name).unwrap();
        let super_class = self.constant_pool.add_class("java/lang/Object").unwrap();

        let code_attr_name_idx = self.constant_pool.add_utf8("Code").unwrap();
        let max_locals = self.next_local_slot;
        let max_stack = self.estimate_max_stack(&code);

        let mut methods = Vec::new();

        // Build method descriptor for the actual function
        let param_types: String = func.parameters.iter()
            .map(|p| ir_type_to_jvm_descriptor(&p.ty))
            .collect();
        let return_type = ir_type_to_jvm_descriptor(&func.return_type);
        let call_desc = format!("({}){}", param_types, return_type);

        // For main function, also create main(String[]) method
        if func.name == "main" {
            // Create call() method with actual implementation
            let call_name_idx = self.constant_pool.add_utf8("call").unwrap();
            let call_desc_idx = self.constant_pool.add_utf8(&call_desc).unwrap();
            
            let call_code_attr = Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack,
                max_locals,
                code,
                exception_table: vec![],
                attributes: vec![],
            };

            let call_method = Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: call_name_idx,
                descriptor_index: call_desc_idx,
                attributes: vec![call_code_attr],
            };
            methods.push(call_method);

            // Create main(String[]) method that calls call()
            let main_name_idx = self.constant_pool.add_utf8("main").unwrap();
            let main_desc = "([Ljava/lang/String;)V".to_string();
            let main_desc_idx = self.constant_pool.add_utf8(&main_desc).unwrap();

            // Add method reference for call() to constant pool
            let call_ref_idx = self.constant_pool
                .add_method_ref(this_class, "call", &call_desc)
                .unwrap();

            // Bytecode for: public static void main(String[] args) { call(); }
            let main_code = vec![
                Instruction::Invokestatic(call_ref_idx),
                Instruction::Return,
            ];

            let main_code_attr = Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack: 1,
                max_locals: 1,
                code: main_code,
                exception_table: vec![],
                attributes: vec![],
            };

            let main_method = Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: main_name_idx,
                descriptor_index: main_desc_idx,
                attributes: vec![main_code_attr],
            };
            methods.push(main_method);
        } else {
            // For non-main functions, create call() method
            let call_name_idx = self.constant_pool.add_utf8("call").unwrap();
            let call_desc_idx = self.constant_pool.add_utf8(&call_desc).unwrap();

            let call_code_attr = Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack,
                max_locals,
                code,
                exception_table: vec![],
                attributes: vec![],
            };

            let call_method = Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: call_name_idx,
                descriptor_index: call_desc_idx,
                attributes: vec![call_code_attr],
            };
            methods.push(call_method);
        }

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_21,
            constant_pool: self.constant_pool.clone(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class,
            super_class,
            interfaces: vec![],
            fields: vec![],
            methods,
            attributes: vec![],
            code_source_url: None,
        };

        let mut buffer = Vec::new();
        class_file.to_bytes(&mut buffer).unwrap();
        buffer
    }

    /// Get JVM method descriptor for external functions
    fn get_method_descriptor(&self, target: &str, param_types: &[IrType], _return_type: Option<&IrType>) -> String {
        match target {
            "puts" => "(Ljava/lang/String;)I".to_string(),
            "putchar" => "(I)I".to_string(),
            "getchar" => "()I".to_string(),
            "printf" => "(Ljava/lang/String;I)I".to_string(),
            "rand" => "()I".to_string(),
            "srand" => "(I)V".to_string(),
            "time" => "(I)I".to_string(),
            "Sleep" => "(I)V".to_string(),
            _ => {
                // Build from IR types for unknown functions
                let param_desc: String = param_types.iter()
                    .map(|t| ir_type_to_jvm_descriptor(t))
                    .collect();
                format!("({})I", param_desc)
            }
        }
    }

    fn estimate_max_stack(&self, code: &[Instruction]) -> u16 {
        let pushes = code.iter().filter(|i| self.instr_pushes(i)).count() as u16;
        (pushes / 2 + 2).max(4)
    }

    fn instr_pushes(&self, instr: &Instruction) -> bool {
        matches!(instr,
            Instruction::Iconst_m1 | Instruction::Iconst_0 |
            Instruction::Iconst_1 | Instruction::Iconst_2 |
            Instruction::Iconst_3 | Instruction::Iconst_4 |
            Instruction::Iconst_5 | Instruction::Bipush(_) |
            Instruction::Sipush(_) | Instruction::Ldc(_) |
            Instruction::Ldc_w(_) | Instruction::Iload(_) |
            Instruction::Iload_0 | Instruction::Iload_1 |
            Instruction::Iload_2 | Instruction::Iload_3 |
            Instruction::Iaload | Instruction::Invokestatic(_)
        )
    }
}

impl Default for JvmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    BitAnd,
    BitOr,
}

enum ComparisonOp {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

fn ir_type_to_jvm_descriptor(ty: &IrType) -> String {
    match ty {
        IrType::Void => "V".to_string(),
        IrType::Bool => "Z".to_string(),
        IrType::Int => "I".to_string(),
        IrType::String => "Ljava/lang/String;".to_string(),
        IrType::Array(elem, _) => format!("[{}]", ir_type_to_jvm_descriptor(elem)),
    }
}

fn capitalize_first(s: &str) -> String {
    if s.is_empty() {
        return s.to_string();
    }
    let mut chars = s.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => s.to_string(),
    }
}

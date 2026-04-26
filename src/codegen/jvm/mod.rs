use crate::codegen::traits::OperandLoader;
use crate::codegen::jvm::types::{capitalize_first, get_method_descriptor, ir_type_to_jvm_descriptor};
use crate::codegen::jvm::bytecode::{JvmInstruction, resolve_instructions, instruction_size};
use crate::ir::types::*;
use ristretto_classfile::attributes::Instruction;
use ristretto_classfile::{ConstantPool};
use std::collections::HashMap;

mod types;
mod bytecode;
mod classfile;

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
        self.collect_external_calls(func);
        let code = self.generate_bytecode(func);
        self.build_class_file(class_name, func, code)
    }
    
    fn collect_external_calls(&mut self, func: &IrFunction) {
        let runtime_stub_class = self.constant_pool.add_class("RuntimeStub").unwrap();
        
        for block in &func.blocks {
            for inst in &block.instructions {
                for operand in &inst.operands {
                    if let IrOperand::Constant(crate::ir::Constant::String(s)) = operand {
                        if !self.string_consts.contains_key(s) {
                            if let Ok(idx) = self.constant_pool.add_string(s) {
                                self.string_consts.insert(s.clone(), idx);
                            }
                        }
                    }
                }
                
                if let IrOpcode::Call = inst.opcode {
                    if let Some(ref target) = inst.jump_target {
                        if self.method_refs.contains_key(target) {
                            continue;
                        }
                        
                        let param_types: Vec<IrType> = inst.operands.iter()
                            .map(|op| op.get_type())
                            .collect();
                        let return_type = inst.result_type.clone();
                        
                        let (class_idx, method_name, descriptor) = if self.is_external_function(target) {
                            let desc = get_method_descriptor(target);
                            (runtime_stub_class, target.clone(), desc)
                        } else {
                            let class_name = capitalize_first(target);
                            let user_class = self.constant_pool.add_class(&class_name).unwrap();
                            let desc = self.build_user_method_descriptor(&param_types, return_type.as_ref());
                            (user_class, "call".to_string(), desc)
                        };
                        
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
        // Two-pass generation:
        // 1. Generate intermediate instructions with labels
        // 2. Compute block positions
        // 3. Resolve labels to offsets
        
        let mut instructions: Vec<JvmInstruction> = Vec::new();
        let mut block_positions: HashMap<String, usize> = HashMap::new();
        
        // First pass: collect instructions and compute block positions
        let mut current_pos = 0usize;
        for block in &func.blocks {
            block_positions.insert(block.id.clone(), current_pos);
            
            for inst in &block.instructions {
                let insts = self.generate_intermediate_instruction(inst);
                for i in &insts {
                    current_pos += instruction_size(i);
                }
                instructions.extend(insts);
            }
        }
        
        // Second pass: resolve labels to actual offsets
        resolve_instructions(&instructions, &block_positions, &self.method_refs, &self.string_consts)
    }
    
    fn generate_intermediate_instruction(&self, inst: &IrInstruction) -> Vec<JvmInstruction> {
        use JvmInstruction::*;
        let mut code = Vec::new();
        
        match inst.opcode {
            IrOpcode::Assign => {
                if let (Some(ref result), Some(ref operand)) = (&inst.result, inst.operands.first()) {
                    self.emit_load_operand(&mut code, operand);
                    let slot = self.get_local_slot(result);
                    code.push(match operand.get_type() {
                        IrType::String => Astore(slot),
                        _ => Istore(slot),
                    });
                }
            }
            IrOpcode::Add => self.emit_binary_op(&mut code, inst, Iadd),
            IrOpcode::Sub => self.emit_binary_op(&mut code, inst, Isub),
            IrOpcode::Mul => self.emit_binary_op(&mut code, inst, Imul),
            IrOpcode::Div => self.emit_binary_op(&mut code, inst, Idiv),
            IrOpcode::Mod => self.emit_binary_op(&mut code, inst, Irem),
            IrOpcode::BitAnd => self.emit_binary_op(&mut code, inst, Iand),
            IrOpcode::BitOr => self.emit_binary_op(&mut code, inst, Ior),
            IrOpcode::Neg => {
                if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
                    self.emit_load_operand(&mut code, operand);
                    code.push(Ineg);
                    code.push(Istore(self.get_local_slot(result)));
                }
            }
            IrOpcode::Pos => {
                if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
                    self.emit_load_operand(&mut code, operand);
                    code.push(Istore(self.get_local_slot(result)));
                }
            }
            IrOpcode::BitNot => {
                if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
                    self.emit_load_operand(&mut code, operand);
                    code.push(Iconst(-1));
                    code.push(Ixor);
                    code.push(Istore(self.get_local_slot(result)));
                }
            }
            IrOpcode::And => {
                // Simple bitwise and - no short-circuit
                if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
                    self.emit_load_operand(&mut code, left);
                    self.emit_load_operand(&mut code, right);
                    code.push(Iand);
                    code.push(Istore(self.get_local_slot(result)));
                }
            }
            IrOpcode::Or => {
                // Simple bitwise or - no short-circuit
                if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
                    self.emit_load_operand(&mut code, left);
                    self.emit_load_operand(&mut code, right);
                    code.push(Ior);
                    code.push(Istore(self.get_local_slot(result)));
                }
            }
            IrOpcode::Not => {
                // Logical not: x == 0 ? 1 : 0
                if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
                    self.emit_load_operand(&mut code, operand);
                    code.push(Iconst(1));
                    code.push(Ixor);  // x ^ 1 = !x for booleans
                    code.push(Istore(self.get_local_slot(result)));
                }
            }
            IrOpcode::Eq => self.emit_comparison(&mut code, inst, |offset| If_icmpeq(offset)),
            IrOpcode::Ne => self.emit_comparison(&mut code, inst, |offset| If_icmpne(offset)),
            IrOpcode::Lt => self.emit_comparison(&mut code, inst, |offset| If_icmplt(offset)),
            IrOpcode::Le => self.emit_comparison(&mut code, inst, |offset| If_icmple(offset)),
            IrOpcode::Gt => self.emit_comparison(&mut code, inst, |offset| If_icmpgt(offset)),
            IrOpcode::Ge => self.emit_comparison(&mut code, inst, |offset| If_icmpge(offset)),
            IrOpcode::Call => {
                if let Some(ref target) = inst.jump_target {
                    for operand in &inst.operands {
                        self.emit_load_operand(&mut code, operand);
                    }
                    let method_idx = self.method_refs.get(target).copied().unwrap_or(1);
                    code.push(Invokestatic(method_idx));
                    if let Some(ref result) = inst.result {
                        code.push(Istore(self.get_local_slot(result)));
                    }
                }
            }
            IrOpcode::Ret => {
                if let Some(operand) = inst.operands.first() {
                    self.emit_load_operand(&mut code, operand);
                    code.push(Ireturn);
                } else {
                    code.push(Return);
                }
            }
            IrOpcode::Jump => {
                if let Some(ref target) = inst.jump_target {
                    code.push(GotoLabel(target.clone()));
                }
            }
            IrOpcode::CondBr => {
                if let Some(operand) = inst.operands.first() {
                    if let Some(ref target) = inst.jump_target {
                        self.emit_load_operand(&mut code, operand);
                        code.push(JvmInstruction::IfneLabel(target.clone()));
                    }
                }
            }
            IrOpcode::Load => {
                if let (Some(ref result), Some(array), Some(index)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
                    self.emit_load_operand(&mut code, array);
                    self.emit_load_operand(&mut code, index);
                    code.push(Iaload);
                    code.push(Istore(self.get_local_slot(result)));
                }
            }
            IrOpcode::Slice | IrOpcode::Alloca | IrOpcode::Store | IrOpcode::Cast => {}
        }
        
        code
    }
    
    fn emit_binary_op(&self, code: &mut Vec<JvmInstruction>, inst: &IrInstruction, op: JvmInstruction) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            self.emit_load_operand(code, right);
            code.push(op);
            code.push(JvmInstruction::Istore(self.get_local_slot(result)));
        }
    }
    
    fn emit_comparison<F>(&self, code: &mut Vec<JvmInstruction>, inst: &IrInstruction, make_br: F)
    where F: Fn(i16) -> JvmInstruction {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            self.emit_load_operand(code, right);
            
            // Layout: if_icmpXX +3 -> iconst_1
            //         iconst_0
            //         goto +2
            //         iconst_1
            // Total: 3 + 1 + 3 + 1 = 8 bytes
            code.push(make_br(3));  // jump to iconst_1 (skip iconst_0 + goto)
            code.push(JvmInstruction::Iconst(0));
            code.push(JvmInstruction::Goto(2));  // jump past iconst_1
            code.push(JvmInstruction::Iconst(1));
            
            code.push(JvmInstruction::Istore(self.get_local_slot(result)));
        }
    }
    
    fn emit_load_operand(&self, code: &mut Vec<JvmInstruction>, operand: &IrOperand) {
        use JvmInstruction::*;
        match operand {
            IrOperand::Variable(name, ty) => {
                let slot = self.get_local_slot(name);
                code.push(match ty {
                    IrType::String => Aload(slot),
                    _ => Iload(slot),
                });
            }
            IrOperand::Constant(c) => self.emit_load_constant(code, c),
        }
    }
    
    fn emit_load_constant(&self, code: &mut Vec<JvmInstruction>, c: &crate::ir::Constant) {
        use crate::ir::Constant;
        use JvmInstruction::*;
        match c {
            Constant::Int(n) => code.push(Iconst(*n as i32)),
            Constant::Bool(true) => code.push(Iconst(1)),
            Constant::Bool(false) => code.push(Iconst(0)),
            Constant::String(s) => code.push(LdcString(s.clone())),
            Constant::Char(c) => code.push(Iconst(*c as i32)),
        }
    }

    pub fn get_local_slot(&self, name: &str) -> u16 {
        *self.locals.get(name).unwrap_or(&0)
    }
}

impl Default for JvmGenerator {
    fn default() -> Self {
        Self::new()
    }
}

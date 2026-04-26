use crate::codegen::traits::OperandLoader;
use crate::codegen::jvm::types::{capitalize_first, get_method_descriptor, ir_type_to_jvm_descriptor};
use crate::ir::types::*;
use ristretto_classfile::attributes::Instruction;
use ristretto_classfile::ConstantPool;
use std::collections::HashMap;

mod types;
mod bytecode;
mod classfile;
mod instructions;
mod loaders;
mod logical;

/// Represents a placeholder for jump instructions that need label resolution
#[derive(Debug, Clone)]
enum JumpPlaceholder {
    Goto { block_id: String },
    Ifne { block_id: String },
    Ifeq { block_id: String },
}

/// Extended instruction type that can hold either a real JVM instruction or a placeholder
#[derive(Debug, Clone)]
enum JvmInst {
    Real(Instruction),
    Placeholder(JumpPlaceholder),
}

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
        // 1. Generate instructions with placeholders for jumps
        // 2. Compute block positions
        // 3. Resolve placeholders to actual offsets
        
        let mut instructions: Vec<JvmInst> = Vec::new();
        let mut block_positions: HashMap<String, usize> = HashMap::new();
        
        // First pass: collect instructions and compute block positions
        let mut current_pos = 0usize;
        for block in &func.blocks {
            block_positions.insert(block.id.clone(), current_pos);
            
            for inst in &block.instructions {
                let insts = self.generate_instruction_with_placeholders(inst);
                for i in &insts {
                    current_pos += self.instruction_size(i);
                }
                instructions.extend(insts);
            }
        }
        
        // Second pass: resolve placeholders to actual offsets
        let mut result = Vec::new();
        current_pos = 0;
        
        for inst in instructions {
            match inst {
                JvmInst::Real(i) => {
                    current_pos += self.instruction_size(&JvmInst::Real(i.clone()));
                    result.push(i);
                }
                JvmInst::Placeholder(placeholder) => {
                    let (target_block, is_conditional, is_ifne) = match &placeholder {
                        JumpPlaceholder::Goto { block_id } => (block_id, false, false),
                        JumpPlaceholder::Ifne { block_id } => (block_id, true, true),
                        JumpPlaceholder::Ifeq { block_id } => (block_id, true, false),
                    };
                    
                    if let Some(&target_pos) = block_positions.get(target_block) {
                        // ristretto_classfile expects ABSOLUTE byte position, not relative offset!
                        // The write_offset function calculates the relative offset internally
                        let target_u16 = target_pos as u16;
                        
                        let resolved = if is_conditional {
                            if is_ifne {
                                Instruction::Ifne(target_u16)
                            } else {
                                Instruction::Ifeq(target_u16)
                            }
                        } else {
                            Instruction::Goto(target_u16)
                        };
                        
                        current_pos += 3; // Size of branch instruction
                        result.push(resolved);
                    } else {
                        // Fallback: jump to end (current position)
                        let fallback_pos = current_pos as u16;
                        let resolved = if is_conditional {
                            if is_ifne {
                                Instruction::Ifne(fallback_pos)
                            } else {
                                Instruction::Ifeq(fallback_pos)
                            }
                        } else {
                            Instruction::Goto(fallback_pos)
                        };
                        current_pos += 3;
                        result.push(resolved);
                    }
                }
            }
        }
        
        result
    }
    
    fn generate_instruction_with_placeholders(&self, inst: &IrInstruction) -> Vec<JvmInst> {
        let mut code: Vec<Instruction> = Vec::new();
        
        // Use the generate_instruction method from instructions.rs
        self.generate_instruction(&mut code, inst);
        
        // Convert to JvmInst, but detect and replace jump placeholders
        // We need to intercept Jump and CondBr to create proper placeholders
        match inst.opcode {
            IrOpcode::Jump => {
                if let Some(ref target) = inst.jump_target {
                    vec![JvmInst::Placeholder(JumpPlaceholder::Goto { 
                        block_id: target.clone() 
                    })]
                } else {
                    vec![JvmInst::Real(Instruction::Nop)]
                }
            }
            IrOpcode::CondBr => {
                if let Some(operand) = inst.operands.first() {
                    self.emit_load_operand(&mut code, operand);
                    if let Some(ref target) = inst.jump_target {
                        // Remove the placeholder Ifne(0) that generate_instruction added
                        code.pop();
                        code.into_iter().map(JvmInst::Real).chain(
                            vec![JvmInst::Placeholder(JumpPlaceholder::Ifne { 
                                block_id: target.clone() 
                            })]
                        ).collect()
                    } else {
                        code.into_iter().map(JvmInst::Real).collect()
                    }
                } else {
                    code.into_iter().map(JvmInst::Real).collect()
                }
            }
            _ => {
                code.into_iter().map(JvmInst::Real).collect()
            }
        }
    }
    
    fn instruction_size(&self, inst: &JvmInst) -> usize {
        use ristretto_classfile::attributes::Instruction as Ri;
        
        match inst {
            JvmInst::Placeholder(_) => 3, // Branch instructions are 3 bytes
            JvmInst::Real(instr) => match instr {
                Ri::Nop => 1,
                Ri::Aconst_null => 1,
                Ri::Iconst_m1 | Ri::Iconst_0 | Ri::Iconst_1 | Ri::Iconst_2 |
                Ri::Iconst_3 | Ri::Iconst_4 | Ri::Iconst_5 => 1,
                Ri::Lconst_0 | Ri::Lconst_1 => 1,
                Ri::Fconst_0 | Ri::Fconst_1 | Ri::Fconst_2 => 1,
                Ri::Dconst_0 | Ri::Dconst_1 => 1,
                Ri::Bipush(_) => 2,
                Ri::Sipush(_) => 3,
                Ri::Ldc(_) => 2,
                Ri::Ldc_w(_) | Ri::Ldc2_w(_) => 3,
                Ri::Iload_0 | Ri::Iload_1 | Ri::Iload_2 | Ri::Iload_3 => 1,
                Ri::Lload_0 | Ri::Lload_1 | Ri::Lload_2 | Ri::Lload_3 => 1,
                Ri::Fload_0 | Ri::Fload_1 | Ri::Fload_2 | Ri::Fload_3 => 1,
                Ri::Dload_0 | Ri::Dload_1 | Ri::Dload_2 | Ri::Dload_3 => 1,
                Ri::Aload_0 | Ri::Aload_1 | Ri::Aload_2 | Ri::Aload_3 => 1,
                Ri::Iload(_) | Ri::Lload(_) | Ri::Fload(_) | Ri::Dload(_) | Ri::Aload(_) => 2,
                Ri::Istore_0 | Ri::Istore_1 | Ri::Istore_2 | Ri::Istore_3 => 1,
                Ri::Lstore_0 | Ri::Lstore_1 | Ri::Lstore_2 | Ri::Lstore_3 => 1,
                Ri::Fstore_0 | Ri::Fstore_1 | Ri::Fstore_2 | Ri::Fstore_3 => 1,
                Ri::Dstore_0 | Ri::Dstore_1 | Ri::Dstore_2 | Ri::Dstore_3 => 1,
                Ri::Astore_0 | Ri::Astore_1 | Ri::Astore_2 | Ri::Astore_3 => 1,
                Ri::Istore(_) | Ri::Lstore(_) | Ri::Fstore(_) | Ri::Dstore(_) | Ri::Astore(_) => 2,
                Ri::Pop | Ri::Pop2 => 1,
                Ri::Dup | Ri::Dup_x1 | Ri::Dup_x2 | Ri::Dup2 | Ri::Dup2_x1 | Ri::Dup2_x2 => 1,
                Ri::Swap => 1,
                Ri::Iadd | Ri::Ladd | Ri::Fadd | Ri::Dadd => 1,
                Ri::Isub | Ri::Lsub | Ri::Fsub | Ri::Dsub => 1,
                Ri::Imul | Ri::Lmul | Ri::Fmul | Ri::Dmul => 1,
                Ri::Idiv | Ri::Ldiv | Ri::Fdiv | Ri::Ddiv => 1,
                Ri::Irem | Ri::Lrem | Ri::Frem | Ri::Drem => 1,
                Ri::Ineg | Ri::Lneg | Ri::Fneg | Ri::Dneg => 1,
                Ri::Ishl | Ri::Lshl => 1,
                Ri::Ishr | Ri::Lshr => 1,
                Ri::Iushr | Ri::Lushr => 1,
                Ri::Iand | Ri::Land => 1,
                Ri::Ior | Ri::Lor => 1,
                Ri::Ixor | Ri::Lxor => 1,
                Ri::Iinc(_, _) => 3,
                Ri::I2l | Ri::I2f | Ri::I2d | Ri::L2i | Ri::L2f | Ri::L2d | Ri::F2i | Ri::F2l | Ri::F2d | Ri::D2i | Ri::D2l | Ri::D2f => 1,
                Ri::I2b | Ri::I2c | Ri::I2s => 1,
                Ri::Lcmp => 1,
                Ri::Fcmpl | Ri::Fcmpg | Ri::Dcmpl | Ri::Dcmpg => 1,
                Ri::Ifeq(_) | Ri::Ifne(_) | Ri::Iflt(_) | Ri::Ifge(_) | Ri::Ifgt(_) | Ri::Ifle(_) => 3,
                Ri::If_icmpeq(_) | Ri::If_icmpne(_) | Ri::If_icmplt(_) | Ri::If_icmpge(_) | Ri::If_icmpgt(_) | Ri::If_icmple(_) => 3,
                Ri::If_acmpeq(_) | Ri::If_acmpne(_) => 3,
                Ri::Goto(_) => 3,
                Ri::Jsr(_) => 3,
                Ri::Ret(_) => 2,
                Ri::Tableswitch { .. } => 1, // Variable size, simplified
                Ri::Lookupswitch { .. } => 1, // Variable size, simplified
                Ri::Ireturn | Ri::Lreturn | Ri::Freturn | Ri::Dreturn | Ri::Areturn | Ri::Return => 1,
                Ri::Getstatic(_) => 3,
                Ri::Putstatic(_) => 3,
                Ri::Getfield(_) => 3,
                Ri::Putfield(_) => 3,
                Ri::Invokevirtual(_) | Ri::Invokespecial(_) | Ri::Invokestatic(_) => 3,
                Ri::Invokeinterface(_, _) => 5,
                Ri::Invokedynamic(_) => 5,
                Ri::New(_) => 3,
                Ri::Newarray(_) => 2,
                Ri::Anewarray(_) => 3,
                Ri::Arraylength => 1,
                Ri::Athrow => 1,
                Ri::Checkcast(_) => 3,
                Ri::Instanceof(_) => 3,
                Ri::Monitorenter | Ri::Monitorexit => 1,
                Ri::Wide => 1,
                Ri::Ifnull(_) | Ri::Ifnonnull(_) => 3,
                Ri::Goto_w(_) | Ri::Jsr_w(_) => 5,
                _ => 1,
            }
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

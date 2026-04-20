use crate::ir::*;
use std::collections::HashMap;

use super::types::{JasmFunction, JasmInstruction};

pub struct JasmGenerator {
    pub functions: Vec<JasmFunction>,
    pub locals: HashMap<String, usize>,
    pub temps: HashMap<String, usize>,
    pub local_counter: usize,
    pub current_local_types: HashMap<String, IrType>,
    pub max_stack: usize,
}

impl JasmGenerator {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            locals: HashMap::new(),
            temps: HashMap::new(),
            local_counter: 0,
            current_local_types: HashMap::new(),
            max_stack: 4,
        }
    }

    pub fn generate(&mut self, program: &IrProgram) -> String {
        self.functions.clear();
        for func in &program.functions {
            self.generate_function(func);
        }
        self.build_jasm_source()
    }

    fn generate_function(&mut self, func: &IrFunction) {
        let mut instructions = Vec::new();
        self.locals.clear();
        self.temps.clear();
        self.local_counter = 0;
        let mut local_types = HashMap::new();

        for param in &func.parameters {
            let idx = self.local_counter;
            self.locals.insert(param.name.clone(), idx);
            local_types.insert(param.name.clone(), param.ty.clone());
            self.local_counter += 1;
        }

        for local in &func.locals {
            if !self.locals.contains_key(&local.name) {
                let idx = self.local_counter;
                self.locals.insert(local.name.clone(), idx);
                local_types.insert(local.name.clone(), local.ty.clone());
                self.local_counter += 1;
            }
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                if let Some(ref result) = inst.result {
                    if result.starts_with("Call_") {
                        continue;
                    }
                    if inst.opcode == IrOpcode::Call {
                        let is_void = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::Void));
                        if is_void {
                            continue;
                        }
                    }
                    if result.starts_with('t') && !self.temps.contains_key(result) {
                        let is_void = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::Void));
                        if !is_void {
                            let idx = self.local_counter;
                            self.temps.insert(result.clone(), idx);
                            local_types.insert(result.clone(), inst.result_type.clone().unwrap_or(IrType::Int));
                            self.local_counter += 1;
                        }
                    } else if !self.locals.contains_key(result) && !result.starts_with('t') {
                        let is_void = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::Void));
                        if !is_void {
                            let idx = self.local_counter;
                            self.locals.insert(result.clone(), idx);
                            if let Some(ty) = &inst.result_type {
                                local_types.insert(result.clone(), ty.clone());
                            }
                            self.local_counter += 1;
                        }
                    }
                }
                for operand in &inst.operands {
                    if let IrOperand::Variable(name, ty) = operand {
                        if name.starts_with('t') && !self.temps.contains_key(name) {
                            let idx = self.local_counter;
                            self.temps.insert(name.clone(), idx);
                            local_types.insert(name.clone(), ty.clone());
                            self.local_counter += 1;
                        } else if !self.locals.contains_key(name) && !name.starts_with('t') {
                            let idx = self.local_counter;
                            self.locals.insert(name.clone(), idx);
                            local_types.insert(name.clone(), ty.clone());
                            self.local_counter += 1;
                        }
                    }
                }
            }
        }

        self.current_local_types = local_types.clone();

        for block in &func.blocks {
            instructions.push(JasmInstruction {
                opcode: "label".to_string(),
                operands: vec![block.id.clone()],
            });
            for inst in &block.instructions {
                self.translate_instruction(inst, func, &mut instructions);
            }
        }

        let func_info = JasmFunction {
            name: func.name.clone(),
            params: func.parameters.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
            return_type: func.return_type.clone(),
            instructions,
            local_types,
        };

        self.functions.push(func_info);
    }

    pub fn get_local_index(&self, name: &str) -> Option<usize> {
        self.locals.get(name).or(self.temps.get(name)).copied()
    }

    pub fn translate_instruction(&mut self, inst: &IrInstruction, func: &IrFunction, instructions: &mut Vec<JasmInstruction>) {
        super::instructions::translate_instruction(self, inst, func, instructions)
    }

    pub fn push_value(&self, operand: &IrOperand, instructions: &mut Vec<JasmInstruction>) {
        match operand {
            IrOperand::Variable(name, ty) => {
                let is_string = matches!(ty, IrType::String);
                if let Some(idx) = self.locals.get(name).or(self.temps.get(name)) {
                    if is_string {
                        instructions.push(JasmInstruction { opcode: "aload".to_string(), operands: vec![idx.to_string()] });
                    } else {
                        instructions.push(JasmInstruction { opcode: "iload".to_string(), operands: vec![idx.to_string()] });
                    }
                }
            }
            IrOperand::Constant(c) => match c {
                Constant::Int(v) => {
                    if *v >= -128 && *v <= 127 {
                        instructions.push(JasmInstruction { opcode: "bipush".to_string(), operands: vec![v.to_string()] });
                    } else if *v >= -32768 && *v <= 32767 {
                        instructions.push(JasmInstruction { opcode: "sipush".to_string(), operands: vec![v.to_string()] });
                    } else {
                        instructions.push(JasmInstruction { opcode: "ldc".to_string(), operands: vec![v.to_string()] });
                    }
                }
                Constant::Bool(b) => {
                    instructions.push(JasmInstruction { opcode: "bipush".to_string(), operands: vec![if *b { "1".to_string() } else { "0".to_string() }] });
                }
                Constant::Char(ch) => {
                    instructions.push(JasmInstruction { opcode: "bipush".to_string(), operands: vec![(*ch as i64).to_string()] });
                }
                Constant::String(s) => {
                    if !s.is_empty() && !s.starts_with('%') {
                        instructions.push(JasmInstruction { opcode: "ldc".to_string(), operands: vec![format!("\"{}\"", s)] });
                    }
                }
            },
        }
    }

    pub fn build_jasm_source(&self) -> String {
        let mut source = String::new();
        source.push_str("public class MyLang {\n");
        if self.functions.is_empty() {
            source.push_str("    public static main([java/lang/String)V {\n        return\n    }\n");
        } else {
            for func in &self.functions {
                if func.name == "main" {
                    source.push_str("    public static main([java/lang/String)V {\n");
                    for inst in &func.instructions {
                        if inst.opcode == "label" {
                            source.push_str(&format!("        {}:\n", inst.operands.first().unwrap_or(&String::new())));
                        } else if inst.operands.is_empty() {
                            source.push_str(&format!("        {}\n", inst.opcode));
                        } else {
                            source.push_str(&format!("        {} {}\n", inst.opcode, inst.operands.join(" ")));
                        }
                    }
                    source.push_str("    }\n");
                }
            }
        }
        source.push_str("}\n");
        source
    }
}
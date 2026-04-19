use crate::ir::*;
use std::collections::HashMap;

pub struct JasmGenerator {
    functions: Vec<JasmFunction>,
    locals: HashMap<String, usize>,
    temps: HashMap<String, usize>,
    local_counter: usize,
    current_local_types: HashMap<String, IrType>,
    max_stack: usize,
}

#[derive(Clone)]
struct JasmFunction {
    name: String,
    params: Vec<(String, IrType)>,
    return_type: IrType,
    instructions: Vec<JasmInstruction>,
    local_types: HashMap<String, IrType>,
}

#[derive(Clone)]
struct JasmInstruction {
    opcode: String,
    operands: Vec<String>,
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
                    if result.starts_with('t') && !self.temps.contains_key(result) {
                        let is_void = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::Void));
                        if !is_void {
                            let idx = self.local_counter;
                            self.temps.insert(result.clone(), idx);
                            local_types.insert(result.clone(), inst.result_type.clone().unwrap_or(IrType::Int));
                            self.local_counter += 1;
                        }
                    } else if !self.locals.contains_key(result) && !result.starts_with('t') {
                        let idx = self.local_counter;
                        self.locals.insert(result.clone(), idx);
                        if let Some(ty) = &inst.result_type {
                            local_types.insert(result.clone(), ty.clone());
                        }
                        self.local_counter += 1;
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

    fn get_local_index(&self, name: &str) -> Option<usize> {
        self.locals.get(name).or(self.temps.get(name)).copied()
    }

    fn is_string_type(&self, name: &str, local_types: &HashMap<String, IrType>) -> bool {
        if let Some(ty) = local_types.get(name) {
            return matches!(ty, IrType::String);
        }
        false
    }

    fn translate_instruction(&mut self, inst: &IrInstruction, func: &IrFunction, instructions: &mut Vec<JasmInstruction>) {
        match inst.opcode {
            IrOpcode::Assign => {
                if let Some(result) = &inst.result {
                    if let Some(operand) = inst.operands.first() {
                        let is_void = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::Void));
                        if is_void {
                            return;
                        }
                        if let IrOperand::Variable(ref _name, ref ty) = operand {
                            if matches!(ty, IrType::Void) {
                                return;
                            }
                        }
                        let local_idx = self.get_local_index(result);
                        self.push_value(operand, instructions, &self.current_local_types);
                        if let Some(idx) = local_idx {
                            let is_string = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::String));
                            let store_opcode = if is_string { "astore" } else { "istore" };
                            instructions.push(JasmInstruction {
                                opcode: store_opcode.to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            IrOpcode::Add => {
                if let Some(result) = &inst.result {
                    if let (Some(left), Some(right)) = (inst.operands.get(0), inst.operands.get(1)) {
                        let local_idx = self.get_local_index(result);
                        self.push_value(left, instructions, &&self.current_local_types);
                        self.push_value(right, instructions, &&self.current_local_types);
                        instructions.push(JasmInstruction {
                            opcode: "iadd".to_string(),
                            operands: vec![],
                        });
                        if let Some(idx) = local_idx {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            IrOpcode::Sub => {
                if let Some(result) = &inst.result {
                    if let (Some(left), Some(right)) = (inst.operands.get(0), inst.operands.get(1)) {
                        let local_idx = self.get_local_index(result);
                        self.push_value(left, instructions, &&self.current_local_types);
                        self.push_value(right, instructions, &&self.current_local_types);
                        instructions.push(JasmInstruction {
                            opcode: "isub".to_string(),
                            operands: vec![],
                        });
                        if let Some(idx) = local_idx {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            IrOpcode::Mul => {
                if let Some(result) = &inst.result {
                    if let (Some(left), Some(right)) = (inst.operands.get(0), inst.operands.get(1)) {
                        let local_idx = self.get_local_index(result);
                        self.push_value(left, instructions, &&self.current_local_types);
                        self.push_value(right, instructions, &&self.current_local_types);
                        instructions.push(JasmInstruction {
                            opcode: "imul".to_string(),
                            operands: vec![],
                        });
                        if let Some(idx) = local_idx {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            IrOpcode::Div => {
                if let Some(result) = &inst.result {
                    if let (Some(left), Some(right)) = (inst.operands.get(0), inst.operands.get(1)) {
                        let local_idx = self.get_local_index(result);
                        self.push_value(left, instructions, &&self.current_local_types);
                        self.push_value(right, instructions, &&self.current_local_types);
                        instructions.push(JasmInstruction {
                            opcode: "idiv".to_string(),
                            operands: vec![],
                        });
                        if let Some(idx) = local_idx {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            IrOpcode::Mod => {
                if let Some(result) = &inst.result {
                    if let (Some(left), Some(right)) = (inst.operands.get(0), inst.operands.get(1)) {
                        let local_idx = self.get_local_index(result);
                        self.push_value(left, instructions, &&self.current_local_types);
                        self.push_value(right, instructions, &&self.current_local_types);
                        instructions.push(JasmInstruction {
                            opcode: "irem".to_string(),
                            operands: vec![],
                        });
                        if let Some(idx) = local_idx {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            IrOpcode::Neg => {
                if let Some(result) = &inst.result {
                    if let Some(operand) = inst.operands.first() {
                        let local_idx = self.get_local_index(result);
                        self.push_value(operand, instructions, &&self.current_local_types);
                        instructions.push(JasmInstruction {
                            opcode: "ineg".to_string(),
                            operands: vec![],
                        });
                        if let Some(idx) = local_idx {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            IrOpcode::Ret => {
                let returns_void = matches!(func.return_type, IrType::Void);
                let is_main = func.name == "main";
                if returns_void || is_main {
                    instructions.push(JasmInstruction {
                        opcode: "return".to_string(),
                        operands: vec![],
                    });
                } else {
                    if let Some(operand) = inst.operands.first() {
                        self.push_value(operand, instructions, &self.current_local_types);
                    }
                    instructions.push(JasmInstruction {
                        opcode: "ireturn".to_string(),
                        operands: vec![],
                    });
                }
            }
            IrOpcode::Call => {
                if let Some(ref func_name) = inst.jump_target {
                    if func_name == "println" {
                        instructions.push(JasmInstruction {
                            opcode: "getstatic".to_string(),
                            operands: vec!["java/lang/System.out".to_string(), "java/io/PrintStream".to_string()],
                        });
                        if let Some(op) = inst.operands.first() {
                            self.push_value(op, instructions, &&self.current_local_types);
                        }
                        instructions.push(JasmInstruction {
                            opcode: "invokevirtual".to_string(),
                            operands: vec!["java/io/PrintStream.println(I)V".to_string()],
                        });
                    } else if func_name == "putchar" {
                        instructions.push(JasmInstruction {
                            opcode: "getstatic".to_string(),
                            operands: vec!["java/lang/System.out".to_string(), "java/io/PrintStream".to_string()],
                        });
                        if let Some(op) = inst.operands.first() {
                            self.push_value(op, instructions, &&self.current_local_types);
                        }
                        instructions.push(JasmInstruction {
                            opcode: "invokevirtual".to_string(),
                            operands: vec!["java/io/PrintStream.print(C)V".to_string()],
                        });
                    } else if func_name == "getchar" {
                        instructions.push(JasmInstruction {
                            opcode: "invokestatic".to_string(),
                            operands: vec!["MyLangRuntime.getchar()I".to_string()],
                        });
                        if let Some(result) = &inst.result {
                            if let Some(idx) = self.get_local_index(result) {
                                instructions.push(JasmInstruction {
                                    opcode: "istore".to_string(),
                                    operands: vec![idx.to_string()],
                                });
                            }
                        }
                    } else if func_name == "puts" {
                        if let Some(op) = inst.operands.first() {
                            match op {
                                IrOperand::Constant(Constant::String(s)) => {
                                    instructions.push(JasmInstruction {
                                        opcode: "ldc".to_string(),
                                        operands: vec![format!("\"{}\"", s)],
                                    });
                                }
                                IrOperand::Variable(name, ty) => {
                                    if matches!(ty, IrType::String) {
                                        if let Some(idx) = self.get_local_index(name) {
                                            instructions.push(JasmInstruction {
                                                opcode: "aload".to_string(),
                                                operands: vec![idx.to_string()],
                                            });
                                        }
                                    } else {
                                        self.push_value(op, instructions, &self.current_local_types);
                                    }
                                }
                                _ => {
                                    self.push_value(op, instructions, &self.current_local_types);
                                }
                            }
                        }
                        instructions.push(JasmInstruction {
                            opcode: "invokestatic".to_string(),
                            operands: vec!["MyLangRuntime.puts(java/lang/String)I".to_string()],
                        });
                        if let Some(result) = &inst.result {
                            if let Some(idx) = self.get_local_index(result) {
                                instructions.push(JasmInstruction {
                                    opcode: "istore".to_string(),
                                    operands: vec![idx.to_string()],
                                });
                            }
                        } else {
                            instructions.push(JasmInstruction {
                                opcode: "pop".to_string(),
                                operands: vec![],
                            });
                        }
                    } else if func_name == "printf" {
                        for operand in &inst.operands {
                            match operand {
                                IrOperand::Constant(Constant::String(s)) => {
                                    instructions.push(JasmInstruction {
                                        opcode: "ldc".to_string(),
                                        operands: vec![format!("\"{}\"", s)],
                                    });
                                }
                                IrOperand::Variable(name, ty) => {
                                    if matches!(ty, IrType::String) {
                                        if let Some(idx) = self.get_local_index(name) {
                                            instructions.push(JasmInstruction {
                                                opcode: "aload".to_string(),
                                                operands: vec![idx.to_string()],
                                            });
                                        }
                                    } else {
                                        self.push_value(operand, instructions, &self.current_local_types);
                                    }
                                }
                                _ => {
                                    self.push_value(operand, instructions, &self.current_local_types);
                                }
                            }
                        }
                        instructions.push(JasmInstruction {
                            opcode: "invokestatic".to_string(),
                            operands: vec!["MyLangRuntime.printf(java/lang/StringI)I".to_string()],
                        });
                        if let Some(result) = &inst.result {
                            if let Some(idx) = self.get_local_index(result) {
                                instructions.push(JasmInstruction {
                                    opcode: "istore".to_string(),
                                    operands: vec![idx.to_string()],
                                });
                            }
                        } else {
                            instructions.push(JasmInstruction {
                                opcode: "pop".to_string(),
                                operands: vec![],
                            });
                        }
                    } else if func_name == "rand" {
                        instructions.push(JasmInstruction {
                            opcode: "invokestatic".to_string(),
                            operands: vec!["MyLangRuntime.rand()I".to_string()],
                        });
                        if let Some(result) = &inst.result {
                            if let Some(idx) = self.get_local_index(result) {
                                instructions.push(JasmInstruction {
                                    opcode: "istore".to_string(),
                                    operands: vec![idx.to_string()],
                                });
                            }
                        }
                    } else if func_name == "time" {
                        if let Some(op) = inst.operands.first() {
                            self.push_value(op, instructions, &&self.current_local_types);
                        }
                        instructions.push(JasmInstruction {
                            opcode: "invokestatic".to_string(),
                            operands: vec!["MyLangRuntime.time(I)I".to_string()],
                        });
                        if let Some(result) = &inst.result {
                            if let Some(idx) = self.get_local_index(result) {
                                instructions.push(JasmInstruction {
                                    opcode: "istore".to_string(),
                                    operands: vec![idx.to_string()],
                                });
                            }
                        }
                    } else if func_name == "srand" {
                        if let Some(op) = inst.operands.first() {
                            self.push_value(op, instructions, &&self.current_local_types);
                        }
                        instructions.push(JasmInstruction {
                            opcode: "invokestatic".to_string(),
                            operands: vec!["MyLangRuntime.srand(I)V".to_string()],
                        });
                    } else {
                        for operand in &inst.operands {
                            self.push_value(operand, instructions, &&self.current_local_types);
                        }
                        instructions.push(JasmInstruction {
                            opcode: "invokestatic".to_string(),
                            operands: vec![format!("MyLangRuntime.{}(...)", func_name)],
                        });
                        let is_void_call = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::Void));
                        if !is_void_call {
                            if let Some(result) = &inst.result {
                                if let Some(idx) = self.get_local_index(result) {
                                    let is_string = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::String));
                                    let store_opcode = if is_string { "astore" } else { "istore" };
                                    instructions.push(JasmInstruction {
                                        opcode: store_opcode.to_string(),
                                        operands: vec![idx.to_string()],
                                    });
                                }
                            }
                        }
                    }
                }
            }
            IrOpcode::Jump => {
                if let Some(ref target) = inst.jump_target {
                    instructions.push(JasmInstruction {
                        opcode: "goto".to_string(),
                        operands: vec![target.clone()],
                    });
                }
            }
            IrOpcode::CondBr => {
                if let Some(operand) = inst.operands.first() {
                    self.push_value(operand, instructions, &&self.current_local_types);
                    if let Some(ref true_t) = inst.true_target {
                        if let Some(ref false_t) = inst.false_target {
                            instructions.push(JasmInstruction {
                                opcode: "ifne".to_string(),
                                operands: vec![true_t.clone()],
                            });
                            instructions.push(JasmInstruction {
                                opcode: "goto".to_string(),
                                operands: vec![false_t.clone()],
                            });
                        }
                    }
                }
            }
            IrOpcode::Eq | IrOpcode::Ne | IrOpcode::Lt | IrOpcode::Le | IrOpcode::Gt | IrOpcode::Ge => {
                if let Some(result) = &inst.result {
                    if let (Some(left), Some(right)) = (inst.operands.get(0), inst.operands.get(1)) {
                        let local_idx = self.get_local_index(result);
                        self.push_value(left, instructions, &&self.current_local_types);
                        self.push_value(right, instructions, &&self.current_local_types);
                        let jop = match inst.opcode {
                            IrOpcode::Eq => "if_icmpeq",
                            IrOpcode::Ne => "if_icmpne",
                            IrOpcode::Lt => "if_icmplt",
                            IrOpcode::Le => "if_icmple",
                            IrOpcode::Gt => "if_icmpgt",
                            IrOpcode::Ge => "if_icmpge",
                            _ => "if_icmpeq",
                        };
                        let true_label = format!("{}_true", result);
                        let false_label = format!("{}_false", result);
                        instructions.push(JasmInstruction {
                            opcode: jop.to_string(),
                            operands: vec![true_label.clone()],
                        });
                        instructions.push(JasmInstruction {
                            opcode: "bipush".to_string(),
                            operands: vec!["0".to_string()],
                        });
                        instructions.push(JasmInstruction {
                            opcode: "goto".to_string(),
                            operands: vec![false_label.clone()],
                        });
                        instructions.push(JasmInstruction {
                            opcode: "label".to_string(),
                            operands: vec![true_label],
                        });
                        instructions.push(JasmInstruction {
                            opcode: "bipush".to_string(),
                            operands: vec!["1".to_string()],
                        });
                        instructions.push(JasmInstruction {
                            opcode: "label".to_string(),
                            operands: vec![false_label],
                        });
                        if let Some(idx) = local_idx {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            IrOpcode::And | IrOpcode::Or => {
                if let Some(result) = &inst.result {
                    if let (Some(left), Some(right)) = (inst.operands.get(0), inst.operands.get(1)) {
                        let local_idx = self.get_local_index(result);
                        self.push_value(left, instructions, &&self.current_local_types);
                        self.push_value(right, instructions, &&self.current_local_types);
                        let op = match inst.opcode {
                            IrOpcode::And => "iand",
                            IrOpcode::Or => "ior",
                            _ => "iand",
                        };
                        instructions.push(JasmInstruction {
                            opcode: op.to_string(),
                            operands: vec![],
                        });
                        if let Some(idx) = local_idx {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            IrOpcode::Not => {
                if let Some(result) = &inst.result {
                    if let Some(operand) = inst.operands.first() {
                        let local_idx = self.get_local_index(result);
                        self.push_value(operand, instructions, &&self.current_local_types);
                        instructions.push(JasmInstruction {
                            opcode: "bipush".to_string(),
                            operands: vec!["1".to_string()],
                        });
                        instructions.push(JasmInstruction {
                            opcode: "ixor".to_string(),
                            operands: vec![],
                        });
                        if let Some(idx) = local_idx {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn push_value(&self, operand: &IrOperand, instructions: &mut Vec<JasmInstruction>, local_types: &HashMap<String, IrType>) {
        match operand {
            IrOperand::Variable(name, ty) => {
                let is_string = matches!(ty, IrType::String);
                if let Some(idx) = self.locals.get(name).or(self.temps.get(name)) {
                    if is_string {
                        instructions.push(JasmInstruction {
                            opcode: "aload".to_string(),
                            operands: vec![idx.to_string()],
                        });
                    } else {
                        instructions.push(JasmInstruction {
                            opcode: "iload".to_string(),
                            operands: vec![idx.to_string()],
                        });
                    }
                }
            }
            IrOperand::Constant(c) => match c {
                Constant::Int(v) => {
                    if *v >= -128 && *v <= 127 {
                        instructions.push(JasmInstruction {
                            opcode: "bipush".to_string(),
                            operands: vec![v.to_string()],
                        });
                    } else if *v >= -32768 && *v <= 32767 {
                        instructions.push(JasmInstruction {
                            opcode: "sipush".to_string(),
                            operands: vec![v.to_string()],
                        });
                    } else {
                        instructions.push(JasmInstruction {
                            opcode: "ldc".to_string(),
                            operands: vec![v.to_string()],
                        });
                    }
                }
                Constant::Bool(b) => {
                    instructions.push(JasmInstruction {
                        opcode: "bipush".to_string(),
                        operands: vec![if *b { "1".to_string() } else { "0".to_string() }],
                    });
                }
                Constant::Char(ch) => {
                    instructions.push(JasmInstruction {
                        opcode: "bipush".to_string(),
                        operands: vec![(*ch as i64).to_string()],
                    });
                }
                Constant::String(s) => {
                    if !s.is_empty() && !s.starts_with('%') {
                        instructions.push(JasmInstruction {
                            opcode: "ldc".to_string(),
                            operands: vec![format!("\"{}\"", s)],
                        });
                    }
                }
            },
        }
    }

    fn build_jasm_source(&self) -> String {
        let mut source = String::new();

        source.push_str("public class MyLang {\n");
        
        if self.functions.is_empty() {
            source.push_str("    public static main([java/lang/String)V {\n");
            source.push_str("        return\n");
            source.push_str("    }\n");
        } else {
            for func in &self.functions {
                if func.name == "main" {
                    source.push_str("    public static main([java/lang/String)V {\n");
                    
                    for inst in &func.instructions {
                        if inst.opcode == "label" {
                            source.push_str(&format!("        {}:\n", inst.operands.first().unwrap_or(&String::new())));
                            continue;
                        }
                        if inst.operands.is_empty() {
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

impl Default for JasmGenerator {
    fn default() -> Self {
        Self::new()
    }
}
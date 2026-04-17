use crate::ir::*;

pub struct JasmGenerator {
    functions: Vec<JasmFunction>,
    locals: usize,
    max_stack: usize,
}

#[derive(Clone)]
struct JasmFunction {
    name: String,
    params: Vec<String>,
    instructions: Vec<JasmInstruction>,
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
            locals: 0,
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
        self.locals = 0;
        self.max_stack = 4;

        for block in &func.blocks {
            for inst in &block.instructions {
                self.translate_instruction(inst, func, &mut instructions);
            }
        }

        let params: Vec<String> = func.parameters
            .iter()
            .map(|_| "I".to_string())
            .collect();

        let func_info = JasmFunction {
            name: func.name.clone(),
            params,
            instructions,
        };

        self.functions.push(func_info);
    }

    fn translate_instruction(&mut self, inst: &IrInstruction, _func: &IrFunction, instructions: &mut Vec<JasmInstruction>) {
        match inst.opcode {
            IrOpcode::Assign => {
                if let (Some(_result), Some(operand)) = (&inst.result, inst.operands.first()) {
                    self.push_value(operand, instructions);
                    instructions.push(JasmInstruction {
                        opcode: "istore".to_string(),
                        operands: vec!["0".to_string()],
                    });
                }
            }
            IrOpcode::Add => {
                if let (Some(_result), Some(left), Some(right)) =
                    (&inst.result, inst.operands.get(0), inst.operands.get(1))
                {
                    self.push_value(left, instructions);
                    self.push_value(right, instructions);
                    instructions.push(JasmInstruction {
                        opcode: "iadd".to_string(),
                        operands: vec![],
                    });
                    instructions.push(JasmInstruction {
                        opcode: "istore".to_string(),
                        operands: vec!["0".to_string()],
                    });
                }
            }
            IrOpcode::Sub => {
                if let (Some(_result), Some(left), Some(right)) =
                    (&inst.result, inst.operands.get(0), inst.operands.get(1))
                {
                    self.push_value(left, instructions);
                    self.push_value(right, instructions);
                    instructions.push(JasmInstruction {
                        opcode: "isub".to_string(),
                        operands: vec![],
                    });
                    instructions.push(JasmInstruction {
                        opcode: "istore".to_string(),
                        operands: vec!["0".to_string()],
                    });
                }
            }
            IrOpcode::Mul => {
                if let (Some(_result), Some(left), Some(right)) =
                    (&inst.result, inst.operands.get(0), inst.operands.get(1))
                {
                    self.push_value(left, instructions);
                    self.push_value(right, instructions);
                    instructions.push(JasmInstruction {
                        opcode: "imul".to_string(),
                        operands: vec![],
                    });
                    instructions.push(JasmInstruction {
                        opcode: "istore".to_string(),
                        operands: vec!["0".to_string()],
                    });
                }
            }
            IrOpcode::Div => {
                if let (Some(_result), Some(left), Some(right)) =
                    (&inst.result, inst.operands.get(0), inst.operands.get(1))
                {
                    self.push_value(left, instructions);
                    self.push_value(right, instructions);
                    instructions.push(JasmInstruction {
                        opcode: "idiv".to_string(),
                        operands: vec![],
                    });
                    instructions.push(JasmInstruction {
                        opcode: "istore".to_string(),
                        operands: vec!["0".to_string()],
                    });
                }
            }
            IrOpcode::Neg => {
                if inst.result.is_some() {
                    if let Some(operand) = inst.operands.first() {
                        self.push_value(operand, instructions);
                        instructions.push(JasmInstruction {
                            opcode: "ineg".to_string(),
                            operands: vec![],
                        });
                        instructions.push(JasmInstruction {
                            opcode: "istore".to_string(),
                            operands: vec!["0".to_string()],
                        });
                    }
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
                            self.push_value(op, instructions);
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
                            self.push_value(op, instructions);
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
                        instructions.push(JasmInstruction {
                            opcode: "istore".to_string(),
                            operands: vec!["0".to_string()],
                        });
                    } else if func_name == "puts" || func_name == "printf" {
                        let mut pushed_string = false;
                        for operand in &inst.operands {
                            match operand {
                                IrOperand::Constant(Constant::String(s)) => {
                                    if !s.starts_with('%') && !s.is_empty() {
                                        instructions.push(JasmInstruction {
                                            opcode: "ldc".to_string(),
                                            operands: vec![format!("\"{}\"", s)],
                                        });
                                        pushed_string = true;
                                    } else if s.starts_with('%') {
                                        instructions.push(JasmInstruction {
                                            opcode: "ldc".to_string(),
                                            operands: vec![format!("\"{}\"", s)],
                                        });
                                        pushed_string = true;
                                    }
                                }
                                IrOperand::Variable(_, _) => {
                                    instructions.push(JasmInstruction {
                                        opcode: "aload".to_string(),
                                        operands: vec!["0".to_string()],
                                    });
                                }
                                _ => {
                                    self.push_value(operand, instructions);
                                }
                            }
                        }
                        if func_name == "puts" {
                            instructions.push(JasmInstruction {
                                opcode: "invokestatic".to_string(),
                                operands: vec!["MyLangRuntime.puts(Ljava/lang/String;)V".to_string()],
                            });
                        } else {
                            instructions.push(JasmInstruction {
                                opcode: "invokestatic".to_string(),
                                operands: vec!["MyLangRuntime.printf(Ljava/lang/String;I)V".to_string()],
                            });
                        }
                    } else if func_name == "rand" {
                        instructions.push(JasmInstruction {
                            opcode: "invokestatic".to_string(),
                            operands: vec!["MyLangRuntime.rand()I".to_string()],
                        });
                        instructions.push(JasmInstruction {
                            opcode: "istore".to_string(),
                            operands: vec!["0".to_string()],
                        });
                    } else if func_name == "time" || func_name == "srand" {
                        for operand in &inst.operands {
                            self.push_value(operand, instructions);
                        }
                        instructions.push(JasmInstruction {
                            opcode: "invokestatic".to_string(),
                            operands: vec![format!("MyLangRuntime.{}(I)I", func_name)],
                        });
                        if inst.result.is_some() {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec!["0".to_string()],
                            });
                        }
                    } else {
                        for operand in &inst.operands {
                            self.push_value(operand, instructions);
                        }
                        let is_void_call = func_name == "println" || func_name == "putchar" || func_name == "puts" || func_name == "printf";
                        if is_void_call {
                            instructions.push(JasmInstruction {
                                opcode: "invokestatic".to_string(),
                                operands: vec![format!("MyLangRuntime.{}(I)V", func_name)],
                            });
                        } else {
                            instructions.push(JasmInstruction {
                                opcode: "invokestatic".to_string(),
                                operands: vec![format!("MyLangRuntime.{}(I)I", func_name)],
                            });
                            if inst.result.is_some() {
                                instructions.push(JasmInstruction {
                                    opcode: "istore".to_string(),
                                    operands: vec!["0".to_string()],
                                });
                            }
                        }
                    }
                }
            }
            IrOpcode::Ret => {
                if let Some(operand) = inst.operands.first() {
                    self.push_value(operand, instructions);
                }
                instructions.push(JasmInstruction {
                    opcode: "return".to_string(),
                    operands: vec![],
                });
            }
            _ => {}
        }
    }

    fn push_value(&self, operand: &IrOperand, instructions: &mut Vec<JasmInstruction>) {
        match operand {
            IrOperand::Variable(_, _) => {
                instructions.push(JasmInstruction {
                    opcode: "iload".to_string(),
                    operands: vec!["0".to_string()],
                });
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
                            operands: vec![format!("{}", v)],
                        });
                    }
                }
                Constant::Bool(b) => {
                    instructions.push(JasmInstruction {
                        opcode: "iconst_0".to_string(),
                        operands: vec![],
                    });
                    if *b {
                        instructions.push(JasmInstruction {
                            opcode: "iconst_1".to_string(),
                            operands: vec![],
                        });
                    }
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

    fn translate_operand(&self, operand: &IrOperand) -> String {
        match operand {
            IrOperand::Variable(name, _) => name.clone(),
            IrOperand::Constant(c) => match c {
                Constant::Int(v) => v.to_string(),
                Constant::Bool(b) => b.to_string(),
                Constant::Char(ch) => (*ch as i32).to_string(),
                Constant::String(s) => format!("\"{}\"", s),
            },
        }
    }

    fn alloc_local(&mut self) -> usize {
        self.locals += 1;
        self.locals
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
                        if inst.opcode == "return" {
                            continue;
                        }
                        if inst.operands.is_empty() {
                            source.push_str(&format!("        {}\n", inst.opcode));
                        } else {
                            source.push_str(&format!("        {} {}\n", inst.opcode, inst.operands.join(" ")));
                        }
                    }
                    
                    source.push_str("        return\n");
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
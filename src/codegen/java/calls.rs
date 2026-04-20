use crate::ir::*;

use super::generator::JasmGenerator;
use super::types::JasmInstruction;

pub fn translate_call(gen: &mut JasmGenerator, inst: &IrInstruction, instructions: &mut Vec<JasmInstruction>) {
    if let Some(ref func_name) = inst.jump_target {
        match func_name.as_str() {
            "println" => {
                instructions.push(JasmInstruction {
                    opcode: "getstatic".to_string(),
                    operands: vec!["java/lang/System.out".to_string(), "java/io/PrintStream".to_string()],
                });
                if let Some(op) = inst.operands.first() {
                    gen.push_value(op, instructions);
                }
                instructions.push(JasmInstruction {
                    opcode: "invokevirtual".to_string(),
                    operands: vec!["java/io/PrintStream.println(I)V".to_string()],
                });
            }
            "putchar" => {
                instructions.push(JasmInstruction {
                    opcode: "getstatic".to_string(),
                    operands: vec!["java/lang/System.out".to_string(), "java/io/PrintStream".to_string()],
                });
                if let Some(op) = inst.operands.first() {
                    gen.push_value(op, instructions);
                }
                instructions.push(JasmInstruction {
                    opcode: "invokevirtual".to_string(),
                    operands: vec!["java/io/PrintStream.print(C)V".to_string()],
                });
                return;
            }
            "getchar" => {
                instructions.push(JasmInstruction {
                    opcode: "invokestatic".to_string(),
                    operands: vec!["MyLangRuntime.getchar()I".to_string()],
                });
                if let Some(result) = &inst.result {
                    if let Some(idx) = gen.get_local_index(result) {
                        if gen.locals.contains_key(result) || gen.temps.contains_key(result) {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        } else {
                            instructions.push(JasmInstruction {
                                opcode: "pop".to_string(),
                                operands: vec![],
                            });
                        }
                    }
                } else {
                    instructions.push(JasmInstruction {
                        opcode: "pop".to_string(),
                        operands: vec![],
                    });
                }
            }
            "puts" => {
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
                                if let Some(idx) = gen.get_local_index(name) {
                                    instructions.push(JasmInstruction {
                                        opcode: "aload".to_string(),
                                        operands: vec![idx.to_string()],
                                    });
                                }
                            } else {
                                gen.push_value(op, instructions);
                            }
                        }
                        _ => {
                            gen.push_value(op, instructions);
                        }
                    }
                }
                instructions.push(JasmInstruction {
                    opcode: "invokestatic".to_string(),
                    operands: vec!["MyLangRuntime.puts(java/lang/String)V".to_string()],
                });
                return;
            }
            "printf" => {
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
                                if let Some(idx) = gen.get_local_index(name) {
                                    instructions.push(JasmInstruction {
                                        opcode: "aload".to_string(),
                                        operands: vec![idx.to_string()],
                                    });
                                }
                            } else {
                                gen.push_value(operand, instructions);
                            }
                        }
                        _ => {
                            gen.push_value(operand, instructions);
                        }
                    }
                }
                instructions.push(JasmInstruction {
                    opcode: "invokestatic".to_string(),
                    operands: vec!["MyLangRuntime.printf(java/lang/StringI)V".to_string()],
                });
                return;
            }
            "rand" => {
                instructions.push(JasmInstruction {
                    opcode: "invokestatic".to_string(),
                    operands: vec!["MyLangRuntime.rand()I".to_string()],
                });
                if let Some(result) = &inst.result {
                    if let Some(idx) = gen.get_local_index(result) {
                        if gen.locals.contains_key(result) || gen.temps.contains_key(result) {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        } else {
                            instructions.push(JasmInstruction {
                                opcode: "pop".to_string(),
                                operands: vec![],
                            });
                        }
                    }
                } else {
                    instructions.push(JasmInstruction {
                        opcode: "pop".to_string(),
                        operands: vec![],
                    });
                }
            }
            "time" => {
                if let Some(op) = inst.operands.first() {
                    gen.push_value(op, instructions);
                }
                instructions.push(JasmInstruction {
                    opcode: "invokestatic".to_string(),
                    operands: vec!["MyLangRuntime.time(I)I".to_string()],
                });
                if let Some(result) = &inst.result {
                    if let Some(idx) = gen.get_local_index(result) {
                        if gen.locals.contains_key(result) || gen.temps.contains_key(result) {
                            instructions.push(JasmInstruction {
                                opcode: "istore".to_string(),
                                operands: vec![idx.to_string()],
                            });
                        } else {
                            instructions.push(JasmInstruction {
                                opcode: "pop".to_string(),
                                operands: vec![],
                            });
                        }
                    }
                } else {
                    instructions.push(JasmInstruction {
                        opcode: "pop".to_string(),
                        operands: vec![],
                    });
                }
            }
            "srand" => {
                if let Some(op) = inst.operands.first() {
                    gen.push_value(op, instructions);
                }
                instructions.push(JasmInstruction {
                    opcode: "invokestatic".to_string(),
                    operands: vec!["MyLangRuntime.srand(I)V".to_string()],
                });
            }
            _ => {
                for operand in &inst.operands {
                    gen.push_value(operand, instructions);
                }
                instructions.push(JasmInstruction {
                    opcode: "invokestatic".to_string(),
                    operands: vec![format!("MyLangRuntime.{}(...)", func_name)],
                });
                let is_void_call = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::Void));
                if !is_void_call {
                    if let Some(result) = &inst.result {
                        if let Some(idx) = gen.get_local_index(result) {
                            if gen.locals.contains_key(result) || gen.temps.contains_key(result) {
                                let is_string = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::String));
                                let store_opcode = if is_string { "astore" } else { "istore" };
                                instructions.push(JasmInstruction {
                                    opcode: store_opcode.to_string(),
                                    operands: vec![idx.to_string()],
                                });
                            } else {
                                instructions.push(JasmInstruction {
                                    opcode: "pop".to_string(),
                                    operands: vec![],
                                });
                            }
                        }
                    } else {
                        instructions.push(JasmInstruction {
                            opcode: "pop".to_string(),
                            operands: vec![],
                        });
                    }
                }
            }
        }
    }
}
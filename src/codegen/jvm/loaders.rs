use ristretto_classfile::attributes::Instruction;
use crate::ir::types::*;
use crate::codegen::jvm::JvmGenerator;

impl JvmGenerator {
    pub fn emit_load_operand(&self, code: &mut Vec<Instruction>, operand: &IrOperand) {
        match operand {
            IrOperand::Variable(name, ty) => {
                let slot = self.get_local_slot(name);
                if self.wrapped_vars.contains(name) {
                    // Wrapped var: load through int[1] wrapper
                    match slot {
                        0 => code.push(Instruction::Aload_0),
                        1 => code.push(Instruction::Aload_1),
                        2 => code.push(Instruction::Aload_2),
                        3 => code.push(Instruction::Aload_3),
                        _ => code.push(Instruction::Aload(slot as u8)),
                    }
                    code.push(Instruction::Iconst_0);
                    code.push(Instruction::Iaload);
                    return;
                }
                match ty {
                    IrType::String | IrType::Function(_, _) => {
                        match slot {
                            0 => code.push(Instruction::Aload_0),
                            1 => code.push(Instruction::Aload_1),
                            2 => code.push(Instruction::Aload_2),
                            3 => code.push(Instruction::Aload_3),
                            _ => code.push(Instruction::Aload(slot as u8)),
                        }
                    }
                    _ => {
                        match slot {
                            0 => code.push(Instruction::Iload_0),
                            1 => code.push(Instruction::Iload_1),
                            2 => code.push(Instruction::Iload_2),
                            3 => code.push(Instruction::Iload_3),
                            _ => code.push(Instruction::Iload(slot as u8)),
                        }
                    }
                };
            }
            IrOperand::Constant(c) => self.emit_load_constant(code, c),
            IrOperand::FuncRef(func_name) => {
                if let Some(&(class_idx, init_ref)) = self.func_ref_init_refs.get(func_name) {
                    code.push(Instruction::New(class_idx));
                    code.push(Instruction::Dup);
                    code.push(Instruction::Invokespecial(init_ref));
                } else {
                    code.push(Instruction::Iconst_0);
                }
            }
        }
    }

    pub fn emit_load_constant(&self, code: &mut Vec<Instruction>, c: &crate::ir::Constant) {
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
                    _ => code.push(Instruction::Iconst_0),
                }
            }
            Constant::Bool(true) => code.push(Instruction::Iconst_1),
            Constant::Bool(false) => code.push(Instruction::Iconst_0),
            Constant::String(s) => {
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
            Constant::Array(_) => {
                code.push(Instruction::Iconst_0);
            }
        }
    }
}

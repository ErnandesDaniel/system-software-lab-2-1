use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{IrOperand, IrType};
use ristretto_classfile::attributes::{ArrayType, Instruction};

impl JvmGenerator {
    pub fn emit_load_operand(&self, code: &mut Vec<Instruction>, operand: &IrOperand) {
        match operand {
            IrOperand::Variable(name, ty) => {
                // Global static field load
                if let Some(&field_ref) = self.global.global_field_refs.get(name) {
                    code.push(Instruction::Getstatic(field_ref));
                    return;
                }
                if self.coro.is_coroutine {
                    if let Some(&field_ref) = self.coro.coroutine_field_refs.get(name) {
                        code.push(Instruction::Aload_0);
                        code.push(Instruction::Getfield(field_ref));
                        return;
                    }
                }
                let slot = self.get_local_slot(name);
                if self.closure.wrapped_vars.contains(name) {
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
                // Use parameter's declared type if available (struct params may have wrong IR type)
                let effective_ty = self.func.param_type_map.get(name.as_str()).unwrap_or(ty);
                match effective_ty {
                    IrType::String | IrType::Function(_, _) | IrType::Array(..) => match slot {
                        0 => code.push(Instruction::Aload_0),
                        1 => code.push(Instruction::Aload_1),
                        2 => code.push(Instruction::Aload_2),
                        3 => code.push(Instruction::Aload_3),
                        _ => code.push(Instruction::Aload(slot as u8)),
                    },
                    _ => match slot {
                        0 => code.push(Instruction::Iload_0),
                        1 => code.push(Instruction::Iload_1),
                        2 => code.push(Instruction::Iload_2),
                        3 => code.push(Instruction::Iload_3),
                        _ => code.push(Instruction::Iload(slot as u8)),
                    },
                }
            }
            IrOperand::Constant(c) => self.emit_load_constant(code, c),
            IrOperand::FuncRef(func_name) => {
                if let Some(&(class_idx, init_ref)) = self.pool.func_ref_init_refs.get(func_name) {
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
            Constant::Int(n) => match *n {
                -1 => code.push(Instruction::Iconst_m1),
                0 => code.push(Instruction::Iconst_0),
                1 => code.push(Instruction::Iconst_1),
                2 => code.push(Instruction::Iconst_2),
                3 => code.push(Instruction::Iconst_3),
                4 => code.push(Instruction::Iconst_4),
                5 => code.push(Instruction::Iconst_5),
                n if (-128..=127).contains(&n) => code.push(Instruction::Bipush(n as i8)),
                n if (-32768..=32767).contains(&n) => code.push(Instruction::Sipush(n as i16)),
                n => {
                    // Large integer: try to load from pool, fallback to 0
                    if let Some(&idx) = self.pool.large_int_refs.get(&n) {
                        if idx > 0 {
                            code.push(Instruction::Ldc_w(idx));
                        } else {
                            code.push(Instruction::Iconst_0);
                        }
                    } else {
                        code.push(Instruction::Iconst_0);
                    }
                }
            },
            Constant::Bool(true) => code.push(Instruction::Iconst_1),
            Constant::Bool(false) => code.push(Instruction::Iconst_0),
            Constant::String(s) => {
                let len = s.len() + 1; // +1 for null terminator
                self.emit_load_constant(code, &Constant::Int(len as i64));
                code.push(Instruction::Newarray(ArrayType::Byte));
                for (i, byte) in s.bytes().enumerate() {
                    code.push(Instruction::Dup);
                    self.emit_load_constant(code, &Constant::Int(i as i64));
                    code.push(Instruction::Bipush(byte as i8));
                    code.push(Instruction::Bastore);
                }
                // null terminator
                code.push(Instruction::Dup);
                self.emit_load_constant(code, &Constant::Int(s.len() as i64));
                code.push(Instruction::Iconst_0);
                code.push(Instruction::Bastore);
            }
            Constant::Char(c) => {
                let val = i32::from(*c);
                if (-128..=127).contains(&val) {
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

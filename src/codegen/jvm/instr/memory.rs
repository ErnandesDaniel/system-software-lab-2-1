use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{Constant, IrInstruction, IrOperand, IrType};
use ristretto_classfile::attributes::{ArrayType, Instruction};

impl JvmGenerator {
    pub(super) fn generate_store(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if inst.operands.len() >= 4 {
            if let (Some(base), Some(field_off), Some(value), Some(index)) = (
                inst.operands.first(),
                inst.operands.get(1),
                inst.operands.get(2),
                inst.operands.get(3),
            ) {
                if self.is_struct_var(base) {
                    let byte_off = if let IrOperand::Constant(Constant::Int(b)) = field_off {
                        *b as usize
                    } else {
                        0
                    };
                    let var_name = if let IrOperand::Variable(n, _) = base {
                        n.clone()
                    } else {
                        String::new()
                    };
                    let field_slot = self.get_field_slot_for_offset(&var_name, byte_off);
                    self.emit_load_operand(code, base);
                    self.emit_load_operand(code, index);
                    code.push(Instruction::Aaload);
                    code.push(Instruction::Checkcast(self.pool.object_array_class_idx));
                    self.emit_load_constant(code, &Constant::Int(field_slot as i64));
                    self.emit_load_operand(code, value);
                    let vt = value.get_type();
                    if matches!(vt, IrType::String) {
                        code.push(Instruction::Aastore);
                    } else {
                        code.push(Instruction::Invokestatic(self.pool.integer_value_of_ref));
                        code.push(Instruction::Aastore);
                    }
                } else {
                    let vt = value.get_type();
                    let byte_off = if let IrOperand::Constant(Constant::Int(b)) = field_off {
                        *b as usize
                    } else {
                        0
                    };
                    if matches!(vt, IrType::Function(_, _) | IrType::String | IrType::Array(..)) {
                        self.emit_load_operand(code, base);
                        if byte_off > 0 {
                            let base_idx = byte_off / 4;
                            self.emit_load_operand(code, index);
                            self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                            code.push(Instruction::Iadd);
                        } else {
                            self.emit_load_operand(code, index);
                        }
                        self.emit_load_operand(code, value);
                        code.push(Instruction::Aastore);
                    } else if matches!(vt, IrType::Int | IrType::Bool) {
                        self.emit_load_operand(code, base);
                        self.emit_load_operand(code, index);
                        if byte_off > 0 {
                            let base_idx = byte_off / 4;
                            self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                            code.push(Instruction::Iadd);
                        }
                        self.emit_load_operand(code, value);
                        code.push(Instruction::Iastore);
                    } else {
                        self.emit_load_operand(code, base);
                        self.emit_load_operand(code, index);
                        code.push(Instruction::Aaload);
                        self.emit_load_operand(code, value);
                        code.push(Instruction::Iastore);
                    }
                }
                return;
            }
        }
        if inst.operands.len() >= 3 && inst.operands.len() == 3 {
            if let (Some(base), Some(offset), Some(value)) =
                (inst.operands.first(), inst.operands.get(1), inst.operands.get(2))
            {
                if let IrOperand::Constant(Constant::Int(byte_off)) = offset {
                    self.emit_load_operand(code, base);
                    if self.is_struct_var(base) {
                        self.emit_load_constant(code, &Constant::Int(byte_off / 4));
                        self.emit_load_operand(code, value);
                        self.emit_boxed_field_store(code, inst, *byte_off as usize);
                    } else {
                        let vt = value.get_type();
                        self.emit_load_constant(code, &Constant::Int(byte_off / 4));
                        self.emit_load_operand(code, value);
                        if matches!(vt, IrType::Function(_, _) | IrType::String | IrType::Array(..)) {
                            code.push(Instruction::Aastore);
                        } else {
                            code.push(Instruction::Iastore);
                        }
                    }
                }
            }
        }
    }

    pub(super) fn generate_array_load(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        // Handle single-operand global get (no index)
        if let (Some(ref result), Some(op)) = (&inst.result, inst.operands.first()) {
            if inst.operands.len() == 1 {
                if let IrOperand::Variable(name, _) = op {
                    if let Some(&field_ref) = self.global.global_field_refs.get(name) {
                        let ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                        code.push(Instruction::Getstatic(field_ref));
                        self.emit_store_result(code, result, ty);
                        return;
                    }
                }
            }
        }

        if let (Some(ref result), Some(array), Some(index)) =
            (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            if inst.operands.len() >= 4 {
                if let Some(field_off) = inst.operands.get(1) {
                    if let IrOperand::Constant(Constant::Int(byte_off)) = field_off {
                        if let Some(idx_op) = inst.operands.get(2) {
                            if self.is_struct_var(array) {
                                let var_name = if let IrOperand::Variable(n, _) = array {
                                    n.clone()
                                } else {
                                    String::new()
                                };
                                let field_slot = self.get_field_slot_for_offset(&var_name, *byte_off as usize);
                                self.emit_load_operand(code, array);
                                self.emit_load_operand(code, idx_op);
                                code.push(Instruction::Aaload);
                                code.push(Instruction::Checkcast(self.pool.object_array_class_idx));
                                self.emit_load_constant(code, &Constant::Int(field_slot as i64));
                                code.push(Instruction::Aaload);
                                self.emit_boxed_field_load(code, inst, *byte_off as usize);
                                let field_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                                self.emit_store_result(code, result, field_ty);
                            } else if self.is_struct_global_base(array) {
                                let base_idx = byte_off / 4;
                                self.emit_load_operand(code, array);
                                self.emit_load_operand(code, idx_op);
                                if base_idx > 0 {
                                    self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                                    code.push(Instruction::Iadd);
                                }
                                code.push(Instruction::Iaload);
                                self.emit_store_result(code, result, &IrType::Int);
                            } else {
                                let base_idx = byte_off / 4;
                                self.emit_load_operand(code, array);
                                self.emit_load_operand(code, idx_op);
                                code.push(Instruction::Aaload);
                                if base_idx > 0 {
                                    self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                                    code.push(Instruction::Iadd);
                                }
                                code.push(Instruction::Iaload);
                                self.emit_store_result(code, result, &IrType::Int);
                            }
                            return;
                        }
                    }
                }
            }
            if inst.operands.len() <= 3 {
                if let IrOperand::Constant(Constant::Int(byte_off)) = index {
                    let has_runtime_idx = inst.operands.len() >= 3;
                    if *byte_off > 0 || self.is_struct_var(array) || has_runtime_idx {
                        self.emit_load_operand(code, array);
                        let base_idx = byte_off / 4;
                        if self.is_struct_var(array) {
                            if has_runtime_idx {
                                self.emit_load_operand(code, inst.operands.get(2).unwrap());
                            } else {
                                self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                            }
                            code.push(Instruction::Aaload);
                            self.emit_boxed_field_load(code, inst, *byte_off as usize);
                        } else if has_runtime_idx {
                            if let Some(runtime_idx) = inst.operands.get(2) {
                                self.emit_load_operand(code, runtime_idx);
                                if base_idx > 0 {
                                    self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                                    code.push(Instruction::Iadd);
                                }
                                code.push(Instruction::Iaload);
                            }
                        } else {
                            let elem_type = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                            self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                            if matches!(elem_type, IrType::Function(_, _) | IrType::String | IrType::Array(..)) {
                                code.push(Instruction::Aaload);
                            } else {
                                code.push(Instruction::Iaload);
                            }
                        }
                        let elem_type = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                        self.emit_store_result(code, result, &elem_type);
                        return;
                    }
                }
            }
            let elem_type = inst.result_type.as_ref().unwrap_or(&IrType::Int);
            self.emit_load_operand(code, array);
            self.emit_load_operand(code, index);
            if matches!(elem_type, IrType::Function(_, _) | IrType::String | IrType::Array(..)) {
                code.push(Instruction::Aaload);
            } else {
                code.push(Instruction::Iaload);
            }
            self.emit_store_result(code, result, &elem_type);
        }
    }

    pub(super) fn generate_slice(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(base), Some(start)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            if matches!(base.get_type(), IrType::String) {
                self.emit_load_operand(code, base);
                self.emit_load_operand(code, start);
                if let Some(end) = inst.operands.get(2) {
                    self.emit_load_operand(code, end);
                } else {
                    code.push(Instruction::Iconst_m1);
                }
                if self.pool.string_slice_ref != 0 {
                    code.push(Instruction::Invokestatic(self.pool.string_slice_ref));
                }
                self.emit_store_result(code, result, &IrType::String);
            }
        }
    }

    pub(super) fn generate_str_get_byte(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(str_op), Some(idx_op)) =
            (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            self.emit_load_operand(code, str_op);
            self.emit_load_operand(code, idx_op);
            code.push(Instruction::Baload);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    pub(super) fn generate_str_set_byte(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(str_op), Some(idx_op), Some(val_op)) =
            (inst.operands.first(), inst.operands.get(1), inst.operands.get(2))
        {
            self.emit_load_operand(code, str_op);
            self.emit_load_operand(code, idx_op);
            self.emit_load_operand(code, val_op);
            code.push(Instruction::Bastore);
        }
    }

    pub(super) fn generate_alloc_array(&mut self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(IrType::Array(elem_type, size)) = inst.result_type.as_ref() {
                self.emit_load_constant(code, &Constant::Int(*size as i64));
                match elem_type.as_ref() {
                    IrType::Int | IrType::Bool => {
                        let at = if matches!(elem_type.as_ref(), IrType::Bool) {
                            ArrayType::Boolean
                        } else {
                            ArrayType::Int
                        };
                        code.push(Instruction::Newarray(at));
                    }
                    IrType::Function(_, _) | IrType::String | IrType::Array(..) => {
                        let desc = crate::codegen::jvm::types::ir_type_to_jvm_descriptor(elem_type);
                        let class_name = desc.trim_start_matches('L').trim_end_matches(';');
                        let class_idx = self
                            .pool
                            .constant_pool
                            .add_class(class_name)
                            .expect("Failed to add class for anewarray");
                        code.push(Instruction::Anewarray(class_idx));
                    }
                    _ => {
                        code.push(Instruction::Newarray(ArrayType::Int));
                    }
                }
                self.emit_store_result(code, result, &IrType::Array(elem_type.clone(), *size));
            }
        }
    }
}

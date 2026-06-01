use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{Constant, IrInstruction, IrOperand, IrType};
use ristretto_classfile::attributes::{ArrayType, Instruction};

impl JvmGenerator {
    pub(super) fn generate_store(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if inst.operands.len() >= 4 {
            if let (Some(base), Some(field_off), Some(value), Some(index)) =
                (inst.operands.first(), inst.operands.get(1), inst.operands.get(2), inst.operands.get(3))
            {
                if self.is_struct_var(base) {
                    let byte_off = if let IrOperand::Constant(Constant::Int(b)) = field_off { *b as usize } else { 0 };
                    let var_name = if let IrOperand::Variable(n, _) = base { n.clone() } else { String::new() };
                    let field_slot = self.get_field_slot_for_offset(&var_name, byte_off);
                    self.emit_load_operand(code, base);
                    self.emit_load_operand(code, index);
                    code.push(Instruction::Aaload);
                    code.push(Instruction::Checkcast(self.object_array_class_idx));
                    self.emit_load_constant(code, &Constant::Int(field_slot as i64));
                    self.emit_load_operand(code, value);
                    let vt = value.get_type();
                    if matches!(vt, IrType::String) {
                        code.push(Instruction::Aastore);
                    } else {
                        code.push(Instruction::Invokestatic(self.integer_value_of_ref));
                        code.push(Instruction::Aastore);
                    }
                } else {
                    let byte_off = if let IrOperand::Constant(Constant::Int(b)) = field_off { *b as usize } else { 0 };
                    let base_idx = byte_off / 4;
                    self.emit_load_operand(code, base);
                    self.emit_load_operand(code, index);
                    code.push(Instruction::Aaload);
                    if base_idx > 0 {
                        self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                        code.push(Instruction::Iadd);
                    }
                    self.emit_load_operand(code, value);
                    code.push(Instruction::Iastore);
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
                        self.emit_load_constant(code, &Constant::Int(byte_off / 4));
                        self.emit_load_operand(code, value);
                        code.push(Instruction::Iastore);
                    }
                }
            }
        }
    }

    pub(super) fn generate_array_load(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(array), Some(index)) =
            (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            if inst.operands.len() >= 4 {
                if let Some(field_off) = inst.operands.get(1) {
                    if let IrOperand::Constant(Constant::Int(byte_off)) = field_off {
                        if let Some(idx_op) = inst.operands.get(2) {
                            if self.is_struct_var(array) {
                                let var_name = if let IrOperand::Variable(n, _) = array { n.clone() } else { String::new() };
                                let field_slot = self.get_field_slot_for_offset(&var_name, *byte_off as usize);
                                self.emit_load_operand(code, array);
                                self.emit_load_operand(code, idx_op);
                                code.push(Instruction::Aaload);
                                code.push(Instruction::Checkcast(self.object_array_class_idx));
                                self.emit_load_constant(code, &Constant::Int(field_slot as i64));
                                code.push(Instruction::Aaload);
                                self.emit_boxed_field_load(code, inst, *byte_off as usize);
                                let field_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                                self.emit_store_result(code, result, field_ty);
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
                    if *byte_off > 0 || self.is_struct_var(array) {
                        self.emit_load_operand(code, array);
                        let idx = byte_off / 4;
                        if self.is_struct_var(array) {
                            self.emit_load_constant(code, &Constant::Int(idx as i64));
                            code.push(Instruction::Aaload);
                            self.emit_boxed_field_load(code, inst, *byte_off as usize);
                        } else {
                            self.emit_load_constant(code, &Constant::Int(idx as i64));
                            code.push(Instruction::Iaload);
                        }
                        self.emit_store_result(code, result, &IrType::Int);
                        return;
                    }
                }
            }
            self.emit_load_operand(code, array);
            self.emit_load_operand(code, index);
            code.push(Instruction::Iaload);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    pub(super) fn generate_slice(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(base), Some(start)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            if matches!(base.get_type(), IrType::String) {
                self.emit_load_operand(code, base);
                self.emit_load_operand(code, start);
                if let Some(end) = inst.operands.get(2) {
                    self.emit_load_operand(code, end);
                } else {
                    code.push(Instruction::Iconst_m1);
                }
                if self.string_slice_ref != 0 {
                    code.push(Instruction::Invokestatic(self.string_slice_ref));
                }
                self.emit_store_result(code, result, &IrType::String);
            }
        }
    }

    pub(super) fn generate_str_get_byte(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(str_op), Some(idx_op)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.emit_load_operand(code, str_op);
            self.emit_load_operand(code, idx_op);
            code.push(Instruction::Baload);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    pub(super) fn generate_str_set_byte(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(str_op), Some(idx_op), Some(val_op)) = (inst.operands.first(), inst.operands.get(1), inst.operands.get(2)) {
            self.emit_load_operand(code, str_op);
            self.emit_load_operand(code, idx_op);
            self.emit_load_operand(code, val_op);
            code.push(Instruction::Bastore);
        }
    }

    pub(super) fn generate_alloc_array(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(IrType::Array(_, size)) = inst.result_type.as_ref() {
                self.emit_load_constant(code, &Constant::Int(*size as i64));
                code.push(Instruction::Newarray(ArrayType::Int));
                self.emit_store_result(code, result, &IrType::Array(Box::new(IrType::Int), *size));
            }
        }
    }
}

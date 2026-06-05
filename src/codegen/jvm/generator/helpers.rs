use crate::ir::types::{IrInstruction, IrOperand, IrType};
use ristretto_classfile::attributes::Instruction;

use super::state::JvmGenerator;

impl JvmGenerator {
    pub fn get_local_slot(&self, name: &str) -> u16 {
        *self.func.locals.get(name).unwrap_or(&0)
    }

    pub fn emit_store_result(&self, code: &mut Vec<Instruction>, name: &str, ty: &IrType) {
        if let Some(&field_ref) = self.global.global_field_refs.get(name) {
            code.push(Instruction::Putstatic(field_ref));
            return;
        }
        if self.coro.is_coroutine {
            if let Some(&field_ref) = self.coro.coroutine_field_refs.get(name) {
                code.push(Instruction::Aload_0);
                code.push(Instruction::Swap);
                code.push(Instruction::Putfield(field_ref));
                return;
            }
        }
        let slot = self.get_local_slot(name);
        match ty {
            IrType::String | IrType::Function(_, _) | IrType::Array(..) => {
                code.push(Instruction::Astore(slot as u8));
            }
            _ => code.push(Instruction::Istore(slot as u8)),
        }
    }

    pub fn get_field_slot_for_offset(&self, var_name: &str, byte_off: usize) -> usize {
        if let Some(fields) = self.st.struct_field_types.get(var_name) {
            let mut offsets: Vec<usize> = fields.iter().map(|(o, _)| *o).collect();
            offsets.sort_unstable();
            offsets.iter().position(|o| *o == byte_off).unwrap_or(byte_off / 4)
        } else {
            byte_off / 4
        }
    }

    pub fn is_global_uses_object_array(&self, name: &str) -> bool {
        self.global.global_uses_object_array.contains(name)
    }

    pub fn get_global_jvm_descriptor(&self, name: &str, ir_type: &IrType) -> String {
        self.global_jvm_descriptor(name, ir_type)
    }

    pub fn get_global_object_array_inner_size(&self, name: &str) -> usize {
        if let Some(offsets) = self.global.global_struct_offset_sets.get(name) {
            offsets.len()
        } else if let Some(ty) = self.global.global_vars.get(name) {
            if let IrType::Array(inner, _) = ty {
                if let IrType::Array(_, int_slots) = inner.as_ref() {
                    (int_slots * 4) / 8
                } else {
                    1
                }
            } else {
                1
            }
        } else {
            1
        }
    }

    pub fn is_struct_var(&self, operand: &IrOperand) -> bool {
        if let IrOperand::Variable(name, _) = operand {
            self.st.struct_uses_object_array.contains(name) || self.global.global_uses_object_array.contains(name)
        } else {
            false
        }
    }

    pub fn is_struct_global_base(&self, operand: &IrOperand) -> bool {
        if let IrOperand::Variable(name, _) = operand {
            if let Some(ty) = self.global.global_vars.get(name) {
                return matches!(ty, IrType::Struct { .. });
            }
        }
        false
    }

    pub fn ensure_int_value_ref(&mut self) -> u16 {
        if self.pool.integer_int_value_ref == 0 {
            let int_class = self
                .pool
                .constant_pool
                .add_class("java/lang/Integer")
                .expect("Failed to add to constant pool");
            self.pool.integer_int_value_ref = self
                .pool
                .constant_pool
                .add_method_ref(int_class, "intValue", "()I")
                .expect("Failed to add to constant pool");
        }
        self.pool.integer_int_value_ref
    }

    pub fn ensure_value_of_ref(&mut self) -> u16 {
        if self.pool.integer_value_of_ref == 0 {
            let int_class = self
                .pool
                .constant_pool
                .add_class("java/lang/Integer")
                .expect("Failed to add to constant pool");
            self.pool.integer_value_of_ref = self
                .pool
                .constant_pool
                .add_method_ref(int_class, "valueOf", "(I)Ljava/lang/Integer;")
                .expect("Failed to add to constant pool");
        }
        self.pool.integer_value_of_ref
    }

    pub fn emit_boxed_field_load(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, _byte_off: usize) {
        let field_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
        match field_ty {
            IrType::String => code.push(Instruction::Checkcast(self.pool.byte_array_class_idx)),
            _ => {
                code.push(Instruction::Checkcast(self.pool.integer_class_idx));
                code.push(Instruction::Invokevirtual(self.pool.integer_int_value_ref));
            }
        }
    }

    pub fn emit_boxed_field_store(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, _byte_off: usize) {
        let is_string = inst
            .operands
            .get(2)
            .is_some_and(|o| matches!(o.get_type(), IrType::String));
        if is_string {
            code.push(Instruction::Aastore);
        } else {
            code.push(Instruction::Invokestatic(self.pool.integer_value_of_ref));
            code.push(Instruction::Aastore);
        }
    }
}

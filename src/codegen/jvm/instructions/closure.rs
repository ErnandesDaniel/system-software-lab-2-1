use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{Constant, IrInstruction, IrOperand, IrType};
use ristretto_classfile::attributes::{ArrayType, Instruction};

impl JvmGenerator {
    pub(super) fn generate_make_closure(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            let num_captures = inst.operands.len().saturating_sub(1);
            let anewarray_idx = self.pool.anewarray_int_class_idx.unwrap_or(0);

            self.emit_load_constant(code, &Constant::Int(num_captures as i64));
            code.push(Instruction::Anewarray(anewarray_idx));

            for (capture_idx, op) in inst.operands.iter().enumerate().skip(1) {
                code.push(Instruction::Dup);
                self.emit_load_constant(code, &Constant::Int((capture_idx - 1) as i64));

                if let IrOperand::Variable(name, _) = op {
                    if self.closure.wrapped_vars.contains(name) {
                        let cap_slot = self.get_local_slot(name);
                        match cap_slot {
                            0 => code.push(Instruction::Aload_0),
                            1 => code.push(Instruction::Aload_1),
                            2 => code.push(Instruction::Aload_2),
                            3 => code.push(Instruction::Aload_3),
                            _ => code.push(Instruction::Aload(cap_slot as u8)),
                        }
                    } else {
                        code.push(Instruction::Iconst_1);
                        code.push(Instruction::Newarray(ArrayType::Int));
                        code.push(Instruction::Dup);
                        code.push(Instruction::Iconst_0);
                        self.emit_load_operand(code, op);
                        code.push(Instruction::Iastore);
                    }
                } else {
                    code.push(Instruction::Iconst_1);
                    code.push(Instruction::Newarray(ArrayType::Int));
                    code.push(Instruction::Dup);
                    code.push(Instruction::Iconst_0);
                    self.emit_load_operand(code, op);
                    code.push(Instruction::Iastore);
                }

                code.push(Instruction::Aastore);
            }

            let lambda_name = if let IrOperand::FuncRef(name) = &inst.operands[0] {
                Some(name.clone())
            } else {
                None
            };

            let is_closure = lambda_name.as_ref().is_some_and(|n| self.pool.func_ref_env_field_refs.contains_key(n));

            if is_closure {
                let name = lambda_name.expect("Lambda name must exist for closure");
                let field_ref = self.pool.func_ref_env_field_refs[&name];
                let instance_slot = self.pool.func_ref_instance_slots[&name];

                code.push(Instruction::Dup);
                self.emit_store_result(code, result, &IrType::Array(Box::new(IrType::Int), 0));

                match instance_slot {
                    0 => code.push(Instruction::Aload_0),
                    1 => code.push(Instruction::Aload_1),
                    2 => code.push(Instruction::Aload_2),
                    3 => code.push(Instruction::Aload_3),
                    _ => code.push(Instruction::Aload(instance_slot as u8)),
                }

                code.push(Instruction::Swap);
                code.push(Instruction::Putfield(field_ref));
            } else {
                self.emit_store_result(code, result, &IrType::Array(Box::new(IrType::Int), 0));
            }
        }
    }

    pub(super) fn generate_call_closure(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(env_operand) = inst.operands.get(1) {
            if let IrOperand::Variable(env_name, _) = env_operand {
                let env_slot = self.get_local_slot(env_name);
                match env_slot {
                    0 => code.push(Instruction::Aload_0),
                    1 => code.push(Instruction::Aload_1),
                    2 => code.push(Instruction::Aload_2),
                    3 => code.push(Instruction::Aload_3),
                    _ => code.push(Instruction::Aload(env_slot as u8)),
                }
            }

            for arg in inst.operands.iter().skip(2) {
                self.emit_load_operand(code, arg);
            }

            let lambda_name = if let IrOperand::Variable(env_name, _) = env_operand {
                self.closure.closure_targets.get(env_name).cloned()
            } else {
                None
            };

            if let Some(ref name) = lambda_name {
                let method_idx = self.pool.method_refs.get(name).copied().unwrap_or(1);
                code.push(Instruction::Invokestatic(method_idx));
            } else {
                code.push(Instruction::Nop);
            }

            if let Some(ref result) = inst.result {
                let store_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                self.emit_store_result(code, result, store_ty);
            }
        }
    }

    pub(super) fn generate_call_indirect(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(func_op) = inst.operands.first() {
            if let IrType::Function(params, ret) = func_op.get_type() {
                let iface_name = crate::codegen::jvm::types::get_fn_interface_name(&params, &ret);
                if let Some(&method_idx) = self.pool.interface_method_refs.get(&iface_name) {
                    self.emit_load_operand(code, func_op);
                    for arg in inst.operands.iter().skip(1) {
                        self.emit_load_operand(code, arg);
                    }
                    let count = inst.operands.len() as u8;
                    code.push(Instruction::Invokeinterface(method_idx, count));
                    if let Some(ref result) = inst.result {
                        self.emit_store_result(code, result, &ret);
                    }
                } else {
                    if inst.result.is_some() {
                        code.push(Instruction::Iconst_0);
                        if let Some(ref result) = inst.result {
                            let slot = self.get_local_slot(result);
                            code.push(Instruction::Istore(slot as u8));
                        }
                    }
                }
            }
        }
    }

    pub(super) fn generate_load_captured(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(slot_op)) = (&inst.result, inst.operands.get(1)) {
            let env_slot = self.get_local_slot("__env");
            match env_slot {
                0 => code.push(Instruction::Aload_0),
                1 => code.push(Instruction::Aload_1),
                2 => code.push(Instruction::Aload_2),
                3 => code.push(Instruction::Aload_3),
                _ => code.push(Instruction::Aload(env_slot as u8)),
            }

            if let IrOperand::Constant(Constant::Int(slot)) = slot_op {
                self.emit_load_constant(code, &Constant::Int(*slot));
            }

            code.push(Instruction::Aaload);
            code.push(Instruction::Iconst_0);
            code.push(Instruction::Iaload);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    pub(super) fn generate_store_captured(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(slot_op), Some(val_op)) = (inst.operands.get(1), inst.operands.get(2)) {
            let env_slot = self.get_local_slot("__env");
            match env_slot {
                0 => code.push(Instruction::Aload_0),
                1 => code.push(Instruction::Aload_1),
                2 => code.push(Instruction::Aload_2),
                3 => code.push(Instruction::Aload_3),
                _ => code.push(Instruction::Aload(env_slot as u8)),
            }

            if let IrOperand::Constant(Constant::Int(slot)) = slot_op {
                self.emit_load_constant(code, &Constant::Int(*slot));
            }

            code.push(Instruction::Aaload);
            code.push(Instruction::Iconst_0);
            self.emit_load_operand(code, val_op);
            code.push(Instruction::Iastore);
        }
    }
}

use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{Constant, IrInstruction, IrOperand, IrType};
use ristretto_classfile::attributes::{ArrayType, Instruction};

impl JvmGenerator {
    pub(super) fn generate_make_closure(&mut self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            let num_captures = inst.operands.len().saturating_sub(1);
            let anewarray_idx = self.pool.anewarray_int_class_idx.unwrap_or(0);

            let lambda_name = if let IrOperand::FuncRef(name) = &inst.operands[0] {
                Some(name.clone())
            } else {
                None
            };

            let is_closure = lambda_name
                .as_ref()
                .is_some_and(|n| self.pool.func_ref_env_field_refs.contains_key(n));

            if is_closure {
                let name = lambda_name.as_ref().unwrap();
                let field_ref = *self.pool.func_ref_env_field_refs.get(name).unwrap();

                if let Some(&(class_idx, init_ref)) = self.pool.func_ref_init_refs.get(name) {
                    code.push(Instruction::New(class_idx));
                    code.push(Instruction::Dup);
                    code.push(Instruction::Invokespecial(init_ref));
                }

                code.push(Instruction::Dup);
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
                            let cap_slot = self.get_local_slot(name);
                            code.push(Instruction::Dup);
                            match cap_slot {
                                0 => code.push(Instruction::Astore_0),
                                1 => code.push(Instruction::Astore_1),
                                2 => code.push(Instruction::Astore_2),
                                3 => code.push(Instruction::Astore_3),
                                _ => code.push(Instruction::Astore(cap_slot as u8)),
                            }
                            self.closure.wrapped_vars.insert(name.clone());
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

                code.push(Instruction::Putfield(field_ref));

                let store_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                self.emit_store_result(code, result, store_ty);
            } else {
                self.emit_load_constant(code, &Constant::Int(num_captures as i64));
                code.push(Instruction::Anewarray(anewarray_idx));

                for (capture_idx, op) in inst.operands.iter().enumerate().skip(1) {
                    code.push(Instruction::Dup);
                    self.emit_load_constant(code, &Constant::Int((capture_idx - 1) as i64));
                    self.emit_load_operand(code, op);
                    code.push(Instruction::Aastore);
                }

                self.emit_store_result(code, result, &IrType::Array(Box::new(IrType::Int), 0));
            }
        }
    }

    pub(super) fn generate_call_closure(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(closure_operand) = inst.operands.first() {
            if let IrOperand::Variable(closure_name, _) = closure_operand {
                let closure_slot = self.get_local_slot(closure_name);
                match closure_slot {
                    0 => code.push(Instruction::Aload_0),
                    1 => code.push(Instruction::Aload_1),
                    2 => code.push(Instruction::Aload_2),
                    3 => code.push(Instruction::Aload_3),
                    _ => code.push(Instruction::Aload(closure_slot as u8)),
                }
            }

            let lambda_name = if let IrOperand::Variable(closure_name, _) = closure_operand {
                self.closure.closure_targets.get(closure_name).cloned()
            } else {
                None
            };

            if let Some(ref name) = lambda_name {
                if let Some(&field_ref) = self.pool.func_ref_env_field_refs.get(name) {
                    code.push(Instruction::Getfield(field_ref));
                }
                for arg in inst.operands.iter().skip(1) {
                    self.emit_load_operand(code, arg);
                }
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
            if let IrType::Function(params, ret) | IrType::Closure(params, ret) = func_op.get_type() {
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

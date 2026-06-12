use crate::codegen::jvm::types::ir_type_to_jvm_descriptor;
use crate::codegen::jvm::JvmGenerator;
use crate::codegen::traits;
use crate::ir::types::{Constant, IrFunction, IrOpcode, IrOperand, IrType};
use std::collections::HashMap;

impl JvmGenerator {
    pub fn reset_state(&mut self) {
        self.func.locals.clear();
        self.func.next_local_slot = 0;
        self.pool.constant_pool = ristretto_classfile::ConstantPool::default();
        self.pool.method_refs.clear();
        self.pool.string_consts.clear();
        self.closure.env_vars.clear();
        self.closure.closure_targets.clear();
        self.pool.anewarray_int_class_idx = None;
        self.closure.wrapped_vars.clear();
        self.pool.interface_method_refs.clear();
        self.pool.func_ref_init_refs.clear();
        self.pool.func_ref_instance_slots.clear();
        self.pool.func_ref_env_field_refs.clear();
        self.pool.string_slice_ref = 0;
        self.st.struct_field_types.clear();
        self.st.struct_uses_object_array.clear();
        self.pool.integer_value_of_ref = 0;
        self.pool.integer_int_value_ref = 0;
        self.pool.integer_class_idx = 0;
        self.pool.byte_array_class_idx = 0;
        self.pool.object_class_idx = 0;
        self.pool.object_array_class_idx = 0;
        self.pool.large_int_refs.clear();
        self.global.global_field_refs.clear();
        self.pool.runtime_stub_class_ref = 0;
    }

    pub fn setup_local_variables(&mut self, func: &IrFunction) {
        self.func.param_type_map.clear();
        for param in &func.parameters {
            if param.name == "__env" {
                self.closure.env_vars.insert(param.name.clone());
            }
            self.func.param_type_map.insert(param.name.clone(), param.ty.clone());
            self.func.locals.insert(param.name.clone(), self.func.next_local_slot);
            self.func.next_local_slot += 1;
        }

        for local in &func.locals {
            if !traits::is_temp(&local.name) && !self.func.locals.contains_key(&local.name) {
                self.func.locals.insert(local.name.clone(), self.func.next_local_slot);
                self.func.next_local_slot += 1;
            }
        }

        let mut temps_used: Vec<String> = Vec::new();
        let mut func_ref_instance_temps: HashMap<String, String> = HashMap::new();
        for block in &func.blocks {
            for inst in &block.instructions {
                if inst.opcode == IrOpcode::Assign {
                    if let Some(IrOperand::FuncRef(name)) = inst.operands.first() {
                        if let Some(ref result) = inst.result {
                            func_ref_instance_temps.insert(name.clone(), result.clone());
                        }
                    }
                }
                match inst.opcode {
                    IrOpcode::MakeClosure => {
                        if let Some(ref result) = inst.result {
                            if !temps_used.contains(result) {
                                temps_used.push(result.clone());
                            }
                            self.closure.env_vars.insert(result.clone());
                            if let Some(ref target) = inst.jump_target {
                                self.closure.closure_targets.insert(result.clone(), target.clone());
                                func_ref_instance_temps.insert(target.clone(), result.clone());
                            }
                        }
                    }
                    IrOpcode::Assign => {
                        if let Some(ref target) = inst.result {
                            if traits::is_temp(target) && !temps_used.contains(target) {
                                temps_used.push(target.clone());
                            }
                            if let Some(IrOperand::Variable(source, ty)) = inst.operands.first() {
                                if matches!(ty, IrType::Closure(_, _)) {
                                    if let Some(lambda) = self.closure.closure_targets.get(source) {
                                        self.closure.closure_targets.insert(target.clone(), lambda.clone());
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        if let Some(ref result) = inst.result {
                            if traits::is_temp(result) && !temps_used.contains(result) {
                                temps_used.push(result.clone());
                            }
                        }
                    }
                }
            }
        }

        for temp in temps_used {
            if !self.func.locals.contains_key(&temp) {
                self.func.locals.insert(temp, self.func.next_local_slot);
                self.func.next_local_slot += 1;
            }
        }

        for (func_name, temp_name) in &func_ref_instance_temps {
            if let Some(&slot) = self.func.locals.get(temp_name) {
                self.pool.func_ref_instance_slots.insert(func_name.clone(), slot);
            }
        }
    }

    pub fn scan_struct_field_types(&mut self, func: &IrFunction) {
        self.st.struct_field_types.clear();
        self.st.struct_uses_object_array.clear();
        for block in &func.blocks {
            for inst in &block.instructions {
                let base = inst.operands.first().and_then(|o| {
                    if let IrOperand::Variable(n, ty) = o {
                        if matches!(ty, IrType::Array(_, _)) {
                            Some(n.clone())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                });
                let Some(ref base_name) = base else { continue };
                let byte_off = inst
                    .operands
                    .get(1)
                    .and_then(|o| {
                        if let IrOperand::Constant(Constant::Int(n)) = o {
                            Some(*n as usize)
                        } else {
                            None
                        }
                    })
                    .unwrap_or(0);
                let field_ty = if inst.opcode == IrOpcode::Store {
                    inst.operands.get(2).map(|o| o.get_type()).unwrap_or(IrType::Int)
                } else {
                    inst.result_type.clone().unwrap_or(IrType::Int)
                };
                let fields = self.st.struct_field_types.entry(base_name.clone()).or_default();
                if !fields.iter().any(|(o, _)| *o == byte_off) {
                    fields.push((byte_off, field_ty.clone()));
                    if !matches!(field_ty, IrType::Int) {
                        self.st.struct_uses_object_array.insert(base_name.clone());
                    }
                }
            }
        }
        let to_remove: Vec<String> = self
            .st
            .struct_field_types
            .iter()
            .filter(|(_, fields)| fields.iter().all(|(off, _)| *off == 0))
            .map(|(name, _)| name.clone())
            .collect();
        for name in to_remove {
            self.st.struct_field_types.remove(&name);
            self.st.struct_uses_object_array.remove(&name);
        }
    }

    pub(super) fn global_jvm_descriptor(&self, name: &str, ir_type: &IrType) -> String {
        let result = if self.global.global_uses_object_array.contains(name) {
            "[Ljava/lang/Object;".to_string()
        } else if matches!(ir_type, IrType::Struct { .. }) {
            "[I".to_string()
        } else {
            ir_type_to_jvm_descriptor(ir_type)
        };
        result
    }
}

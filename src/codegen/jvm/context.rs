use crate::codegen::jvm::JvmGenerator;
use crate::codegen::jvm::types::ir_type_to_jvm_descriptor;
use crate::codegen::traits::OperandLoader;
use crate::ir::types::{Constant, IrFunction, IrOpcode, IrOperand, IrType};
use std::collections::{HashMap, HashSet};

impl JvmGenerator {
    pub fn reset_state(&mut self) {
        self.locals.clear();
        self.next_local_slot = 0;
        self.constant_pool = ristretto_classfile::ConstantPool::default();
        self.method_refs.clear();
        self.string_consts.clear();
        self.env_vars.clear();
        self.closure_targets.clear();
        self.anewarray_int_class_idx = None;
        self.wrapped_vars.clear();
        self.interface_method_refs.clear();
        self.func_ref_init_refs.clear();
        self.func_ref_instance_slots.clear();
        self.func_ref_env_field_refs.clear();
        self.is_coroutine = false;
        self.coroutine_field_refs.clear();
        self.coroutine_state_field = 0;
        self.coroutine_result_field = 0;
        self.coroutine_field_entries.clear();
        self.string_slice_ref = 0;
        self.struct_field_types.clear();
        self.struct_uses_object_array.clear();
        self.integer_value_of_ref = 0;
        self.integer_int_value_ref = 0;
        self.integer_class_idx = 0;
        self.byte_array_class_idx = 0;
        self.object_class_idx = 0;
        self.object_array_class_idx = 0;
        self.global_field_refs.clear();
        self.runtime_stub_class_ref = 0;
    }

    pub fn setup_coroutine_fields(&mut self, func: &IrFunction, class_name: &str) {
        let this_class = self.constant_pool.add_class(class_name).expect("Failed to add to constant pool");

        let state_name_idx = self.constant_pool.add_utf8("state").expect("Failed to add to constant pool");
        let state_desc_idx = self.constant_pool.add_utf8("I").expect("Failed to add to constant pool");
        self.coroutine_state_field = self
            .constant_pool
            .add_field_ref(this_class, "state", "I")
            .expect("Failed to add to constant pool");
        self.coroutine_field_entries.push((state_name_idx, state_desc_idx));

        let result_name_idx = self.constant_pool.add_utf8("result").expect("Failed to add to constant pool");
        let result_desc_idx = self.constant_pool.add_utf8("I").expect("Failed to add to constant pool");
        self.coroutine_result_field = self
            .constant_pool
            .add_field_ref(this_class, "result", "I")
            .expect("Failed to add to constant pool");
        self.coroutine_field_entries.push((result_name_idx, result_desc_idx));

        let mut field_names: Vec<String> = Vec::new();
        let mut seen_names: HashSet<String> = HashSet::new();

        for param in &func.parameters {
            if param.name == "__env" {
                continue;
            }
            if seen_names.insert(param.name.clone()) {
                field_names.push(param.name.clone());
            }
        }
        for local in &func.locals {
            if seen_names.insert(local.name.clone()) {
                field_names.push(local.name.clone());
            }
        }

        for name in &field_names {
            if self.coroutine_field_refs.contains_key(name) {
                continue;
            }
            let field_name_idx = self.constant_pool.add_utf8(&format!("var_{name}")).expect("Failed to add to constant pool");
            let field_desc_idx = self.constant_pool.add_utf8("I").expect("Failed to add to constant pool");
            let fname = format!("var_{name}");
            let fdesc = "I".to_string();
            let field_ref = self
                .constant_pool
                .add_field_ref(this_class, &fname, &fdesc)
                .expect("Failed to add to constant pool");
            self.coroutine_field_refs.insert(name.clone(), field_ref);
            self.coroutine_field_entries.push((field_name_idx, field_desc_idx));
        }

        let param_refs: Vec<Option<u16>> = func.parameters.iter().filter(|p| p.name != "__env").map(|p| {
            self.coroutine_field_refs.get(&p.name).copied()
        }).collect();
        let p1 = param_refs.first().copied().flatten();
        let p2 = param_refs.get(1).copied().flatten();
        self.coroutine_param_field_refs.push((p1, p2));
    }

    pub fn setup_local_variables(&mut self, func: &IrFunction) {
        for param in &func.parameters {
            if param.name == "__env" {
                self.env_vars.insert(param.name.clone());
            }
            self.locals.insert(param.name.clone(), self.next_local_slot);
            self.next_local_slot += 1;
        }

        for local in &func.locals {
            if !Self::is_temp(&local.name) && !self.locals.contains_key(&local.name) {
                self.locals.insert(local.name.clone(), self.next_local_slot);
                self.next_local_slot += 1;
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
                            self.env_vars.insert(result.clone());
                            if let Some(ref target) = inst.jump_target {
                                self.closure_targets.insert(result.clone(), target.clone());
                            }
                        }
                        for op in inst.operands.iter().skip(1) {
                            if let IrOperand::Variable(name, _) = op {
                                self.wrapped_vars.insert(name.clone());
                            }
                        }
                    }
                    _ => {
                        if let Some(ref result) = inst.result {
                            if Self::is_temp(result) && !temps_used.contains(result) {
                                temps_used.push(result.clone());
                            }
                        }
                    }
                }
            }
        }

        for temp in temps_used {
            if !self.locals.contains_key(&temp) {
                self.locals.insert(temp, self.next_local_slot);
                self.next_local_slot += 1;
            }
        }

        for (func_name, temp_name) in &func_ref_instance_temps {
            if let Some(&slot) = self.locals.get(temp_name) {
                self.func_ref_instance_slots.insert(func_name.clone(), slot);
            }
        }
    }

    pub fn scan_struct_field_types(&mut self, func: &IrFunction) {
        self.struct_field_types.clear();
        self.struct_uses_object_array.clear();
        for block in &func.blocks {
            for inst in &block.instructions {
                let base = inst.operands.first().and_then(|o| {
                    if let IrOperand::Variable(n, ty) = o {
                        if matches!(ty, IrType::Array(_, _)) { Some(n.clone()) } else { None }
                    } else { None }
                });
                let Some(ref base_name) = base else { continue };
                let byte_off = inst.operands.get(1).and_then(|o| {
                    if let IrOperand::Constant(Constant::Int(n)) = o { Some(*n as usize) } else { None }
                }).unwrap_or(0);
                let field_ty = if inst.opcode == IrOpcode::Store {
                    inst.operands.get(2).map(|o| o.get_type()).unwrap_or(IrType::Int)
                } else {
                    inst.result_type.clone().unwrap_or(IrType::Int)
                };
                let fields = self.struct_field_types.entry(base_name.clone()).or_default();
                if !fields.iter().any(|(o, _)| *o == byte_off) {
                    fields.push((byte_off, field_ty.clone()));
                    if !matches!(field_ty, IrType::Int) {
                        self.struct_uses_object_array.insert(base_name.clone());
                    }
                }
            }
        }
    }

    pub(super) fn global_jvm_descriptor(&self, name: &str, ir_type: &IrType) -> String {
        if self.global_uses_object_array.contains(name) {
            "[Ljava/lang/Object;".to_string()
        } else {
            ir_type_to_jvm_descriptor(ir_type)
        }
    }

}

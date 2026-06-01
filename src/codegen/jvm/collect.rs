use crate::codegen::jvm::JvmGenerator;
use crate::codegen::jvm::types::{capitalize_first, get_fn_interface_name, get_method_descriptor, ir_type_to_jvm_descriptor};
use crate::ir::types::{Constant, IrFunction, IrOpcode, IrOperand, IrType};

impl JvmGenerator {
    pub(super) fn collect_external_calls(&mut self, func: &IrFunction) {
        let runtime_stub_class = self.constant_pool.add_class("RuntimeStub").unwrap();
        self.runtime_stub_class_ref = runtime_stub_class;

        for (gname, gty) in &self.global_vars {
            if self.global_field_refs.contains_key(gname) {
                continue;
            }
            let desc = self.global_jvm_descriptor(gname, gty);
            if let Ok(field_ref) = self.constant_pool.add_field_ref(runtime_stub_class, gname, &desc) {
                self.global_field_refs.insert(gname.clone(), field_ref);
            }
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                for op in &inst.operands {
                    if let IrOperand::FuncRef(func_name) = op {
                        if !self.func_ref_init_refs.contains_key(func_name) {
                            let class_name = capitalize_first(func_name);
                            let class_idx = self.constant_pool.add_class(&class_name).unwrap();
                            let init_ref = self.constant_pool.add_method_ref(class_idx, "<init>", "()V").unwrap();
                            self.func_ref_init_refs.insert(func_name.clone(), (class_idx, init_ref));
                        }
                    }
                }
            }
        }

        for block in &func.blocks {
            for inst in &block.instructions {
                for operand in &inst.operands {
                    if let IrOperand::Constant(Constant::String(s)) = operand {
                        if !self.string_consts.contains_key(s) {
                            if let Ok(idx) = self.constant_pool.add_string(s) {
                                self.string_consts.insert(s.clone(), idx);
                            }
                        }
                    }
                }

                match inst.opcode {
                    IrOpcode::Call => {
                        if let Some(ref target) = inst.jump_target {
                            if self.method_refs.contains_key(target) {
                                continue;
                            }

                            let param_types: Vec<IrType> = inst.operands.iter().map(|o| o.get_type()).collect();
                            let return_type = inst.result_type.clone();

                            let (class_idx, method_name, descriptor) = if Self::is_external_function(target) {
                                (runtime_stub_class, target.clone(), get_method_descriptor(target))
                            } else {
                                let class_name = capitalize_first(target);
                                let user_class = self.constant_pool.add_class(&class_name).unwrap();
                                let desc = Self::build_user_method_descriptor(&param_types, return_type.as_ref());
                                (user_class, "call".to_string(), desc)
                            };

                            let method_idx = self.constant_pool.add_method_ref(class_idx, &method_name, &descriptor).unwrap();
                            self.method_refs.insert(target.clone(), method_idx);
                        }
                    }
                    IrOpcode::CallClosure => {
                        if let Some(IrOperand::Variable(env_name, _)) = inst.operands.get(1) {
                            if let Some(lambda_name) = self.closure_targets.get(env_name) {
                                if self.method_refs.contains_key(lambda_name) {
                                    continue;
                                }
                                let class_name = capitalize_first(lambda_name);
                                let user_class = self.constant_pool.add_class(&class_name).unwrap();
                                let mut param_desc = "[[I".to_string();
                                for arg in inst.operands.iter().skip(2) {
                                    param_desc.push_str(&ir_type_to_jvm_descriptor(&arg.get_type()));
                                }
                                let ret_desc = inst.result_type.as_ref().map_or_else(|| "V".to_string(), ir_type_to_jvm_descriptor);
                                let desc = format!("({param_desc}){ret_desc}");
                                let method_idx = self.constant_pool.add_method_ref(user_class, "call", &desc).unwrap();
                                self.method_refs.insert(lambda_name.clone(), method_idx);
                            }
                        }
                    }
                    IrOpcode::MakeClosure => {
                        if self.anewarray_int_class_idx.is_none() {
                            self.anewarray_int_class_idx = Some(self.constant_pool.add_class("[I").unwrap());
                        }
                        if let Some(IrOperand::FuncRef(func_name)) = inst.operands.first() {
                            if !self.func_ref_env_field_refs.contains_key(func_name) {
                                let class_name = capitalize_first(func_name);
                                let class_idx = self.constant_pool.add_class(&class_name).unwrap();
                                let field_ref = self.constant_pool.add_field_ref(class_idx, "__env", "[[I").unwrap();
                                self.func_ref_env_field_refs.insert(func_name.clone(), field_ref);
                            }
                        }
                    }
                    IrOpcode::CallIndirect => {
                        if let Some(func_op) = inst.operands.first() {
                            if let IrType::Function(params, ret) = func_op.get_type() {
                                let iface_name = get_fn_interface_name(&params, &ret);
                                if self.interface_method_refs.contains_key(&iface_name) {
                                    continue;
                                }
                                let iface_class = self.constant_pool.add_class(&iface_name).unwrap();
                                let method_desc = Self::build_user_method_descriptor(&params, Some(&ret));
                                let method_idx = self.constant_pool.add_interface_method_ref(iface_class, "apply", &method_desc).unwrap();
                                self.interface_method_refs.insert(iface_name, method_idx);
                            }
                        }
                    }
                    IrOpcode::Slice => {
                        if self.string_slice_ref == 0 {
                            let stub_class = self.constant_pool.add_class("RuntimeStub").unwrap();
                            self.string_slice_ref = self.constant_pool.add_method_ref(stub_class, "string_slice", "([BII)[B").unwrap();
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    fn is_external_function(name: &str) -> bool {
        matches!(
            name,
            "puts" | "putchar" | "getchar" | "printf"
                | "rand" | "srand" | "time" | "Sleep"
                | "malloc" | "free"
                | "map_put_jvm" | "map_get_jvm" | "map_remove_jvm" | "map_has_jvm"
                | "map_size_jvm" | "map_key_jvm" | "map_list_jvm"
                | "resume_coroutine" | "get_coroutine_state" | "set_coroutine_param" | "coro_init"
        )
    }

    fn build_user_method_descriptor(param_types: &[IrType], return_type: Option<&IrType>) -> String {
        let param_desc: String = param_types.iter().map(ir_type_to_jvm_descriptor).collect();
        let ret_desc = return_type.as_ref().map_or_else(|| "I".to_string(), |t| ir_type_to_jvm_descriptor(t));
        format!("({param_desc}){ret_desc}")
    }
}

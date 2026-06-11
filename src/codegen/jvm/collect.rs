use crate::codegen::jvm::types::{
    capitalize_first, get_fn_interface_name, get_method_descriptor, ir_type_to_jvm_descriptor,
};
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{Constant, IrFunction, IrOpcode, IrOperand, IrType};

impl JvmGenerator {
    pub(super) fn collect_external_calls(&mut self, func: &IrFunction) {
        self.collect_global_field_refs();
        self.collect_func_ref_init_refs(func);
        self.collect_instruction_refs(func);
    }

    fn collect_global_field_refs(&mut self) {
        let runtime_stub_class = self
            .pool
            .constant_pool
            .add_class("RuntimeStub")
            .expect("Failed to add RuntimeStub class");
        self.pool.runtime_stub_class_ref = runtime_stub_class;

        for (gname, gty) in &self.global.global_vars {
            if self.global.global_field_refs.contains_key(gname) {
                continue;
            }
            let desc = self.global_jvm_descriptor(gname, gty);
            if let Ok(field_ref) = self.pool.constant_pool.add_field_ref(runtime_stub_class, gname, &desc) {
                self.global.global_field_refs.insert(gname.clone(), field_ref);
            }
        }
    }

    fn collect_func_ref_init_refs(&mut self, func: &IrFunction) {
        for block in &func.blocks {
            for inst in &block.instructions {
                for op in &inst.operands {
                    if let IrOperand::FuncRef(func_name) = op {
                        if self.pool.func_ref_init_refs.contains_key(func_name) {
                            continue;
                        }
                        let class_name = capitalize_first(func_name);
                        let class_idx = self
                            .pool
                            .constant_pool
                            .add_class(&class_name)
                            .expect("Failed to add func ref class");
                        let init_ref = self
                            .pool
                            .constant_pool
                            .add_method_ref(class_idx, "<init>", "()V")
                            .expect("Failed to add func ref init method");
                        self.pool
                            .func_ref_init_refs
                            .insert(func_name.clone(), (class_idx, init_ref));
                    }
                }
            }
        }
    }

    fn collect_instruction_refs(&mut self, func: &IrFunction) {
        let runtime_stub_class = self
            .pool
            .constant_pool
            .add_class("RuntimeStub")
            .expect("Failed to add RuntimeStub class");

        for block in &func.blocks {
            for inst in &block.instructions {
                self.collect_string_constants(inst);
                self.collect_large_int_constants(inst);
                self.collect_call_ref(inst, runtime_stub_class);
                self.collect_call_closure_ref(inst);
                self.collect_make_closure_ref(inst);
                self.collect_call_indirect_ref(inst);
                self.collect_slice_ref(inst);
            }
        }
    }

    fn collect_string_constants(&mut self, inst: &crate::ir::IrInstruction) {
        for operand in &inst.operands {
            if let IrOperand::Constant(Constant::String(s)) = operand {
                if !self.pool.string_consts.contains_key(s) {
                    if let Ok(idx) = self.pool.constant_pool.add_string(s) {
                        self.pool.string_consts.insert(s.clone(), idx);
                    }
                }
            }
        }
    }

    fn collect_large_int_constants(&mut self, inst: &crate::ir::IrInstruction) {
        for operand in &inst.operands {
            if let IrOperand::Constant(Constant::Int(n)) = operand {
                let val = *n;
                if (!(-32768..=32767).contains(&val)) && !self.pool.large_int_refs.contains_key(&val) {
                    if let Ok(idx) = self.pool.constant_pool.add_integer(val as i32) {
                        // add_integer returns 1-based index; 0 is invalid
                        if idx > 0 {
                            self.pool.large_int_refs.insert(val, idx);
                        }
                    }
                }
            }
        }
    }

    fn collect_call_ref(&mut self, inst: &crate::ir::IrInstruction, runtime_stub_class: u16) {
        if inst.opcode != IrOpcode::Call {
            return;
        }
        let Some(ref target) = inst.jump_target else { return };

        let param_types: Vec<IrType> = inst.operands.iter().map(|o| o.get_type()).collect();
        let return_type = inst.result_type.clone();

        let (class_idx, method_name, descriptor) = if Self::is_external_function(target) {
            self.stub_needed = true;
            if target == "printf" {
                // printf has variable args — build descriptor from actual params
                let desc = Self::build_user_method_descriptor(&param_types, return_type.as_ref());
                (runtime_stub_class, target.clone(), desc)
            } else {
                (runtime_stub_class, target.clone(), get_method_descriptor(target))
            }
        } else {
            let class_name = capitalize_first(target);
            let user_class = self
                .pool
                .constant_pool
                .add_class(&class_name)
                .expect("Failed to add user class");
            let desc = Self::build_user_method_descriptor(&param_types, return_type.as_ref());
            (user_class, "call".to_string(), desc)
        };

        // Use (target, descriptor) as key to support overloaded functions like printf
        let key = format!("{target}|{descriptor}");
        if self.pool.method_refs.contains_key(&key) {
            return;
        }
        let method_idx = self
            .pool
            .constant_pool
            .add_method_ref(class_idx, &method_name, &descriptor)
            .expect("Failed to add method ref");
        self.pool.method_refs.insert(key, method_idx);
    }

    fn collect_call_closure_ref(&mut self, inst: &crate::ir::IrInstruction) {
        if inst.opcode != IrOpcode::CallClosure {
            return;
        }
        let Some(IrOperand::Variable(env_name, _)) = inst.operands.get(1) else {
            return;
        };
        let Some(lambda_name) = self.closure.closure_targets.get(env_name) else {
            return;
        };
        if self.pool.method_refs.contains_key(lambda_name) {
            return;
        }

        let class_name = capitalize_first(lambda_name);
        let user_class = self
            .pool
            .constant_pool
            .add_class(&class_name)
            .expect("Failed to add closure class");

        let mut param_desc = "[[I".to_string();
        for arg in inst.operands.iter().skip(2) {
            param_desc.push_str(&ir_type_to_jvm_descriptor(&arg.get_type()));
        }
        let ret_desc = inst
            .result_type
            .as_ref()
            .map_or_else(|| "V".to_string(), ir_type_to_jvm_descriptor);
        let desc = format!("({param_desc}){ret_desc}");

        let method_idx = self
            .pool
            .constant_pool
            .add_method_ref(user_class, "call", &desc)
            .expect("Failed to add call closure method ref");
        self.pool.method_refs.insert(lambda_name.clone(), method_idx);
    }

    fn collect_make_closure_ref(&mut self, inst: &crate::ir::IrInstruction) {
        if inst.opcode != IrOpcode::MakeClosure {
            return;
        }
        if self.pool.anewarray_int_class_idx.is_none() {
            self.pool.anewarray_int_class_idx =
                Some(self.pool.constant_pool.add_class("[I").expect("Failed to add [I class"));
        }
        let Some(IrOperand::FuncRef(func_name)) = inst.operands.first() else {
            return;
        };
        if self.pool.func_ref_env_field_refs.contains_key(func_name) {
            return;
        }
        let class_name = capitalize_first(func_name);
        let class_idx = self
            .pool
            .constant_pool
            .add_class(&class_name)
            .expect("Failed to add func class for closure env");
        let field_ref = self
            .pool
            .constant_pool
            .add_field_ref(class_idx, "__env", "[[I")
            .expect("Failed to add env field ref");
        self.pool.func_ref_env_field_refs.insert(func_name.clone(), field_ref);
    }

    fn collect_call_indirect_ref(&mut self, inst: &crate::ir::IrInstruction) {
        if inst.opcode != IrOpcode::CallIndirect {
            return;
        }
        let Some(func_op) = inst.operands.first() else { return };
        let IrType::Function(params, ret) = func_op.get_type() else {
            return;
        };
        let iface_name = get_fn_interface_name(&params, &ret);
        if self.pool.interface_method_refs.contains_key(&iface_name) {
            return;
        }
        let iface_class = self
            .pool
            .constant_pool
            .add_class(&iface_name)
            .expect("Failed to add interface class");
        let method_desc = Self::build_user_method_descriptor(&params, Some(&ret));
        let method_idx = self
            .pool
            .constant_pool
            .add_interface_method_ref(iface_class, "apply", &method_desc)
            .expect("Failed to add interface method ref");
        self.pool.interface_method_refs.insert(iface_name, method_idx);
    }

    fn collect_slice_ref(&mut self, inst: &crate::ir::IrInstruction) {
        if inst.opcode != IrOpcode::Slice {
            // Also collect for pointer Add (string + int) used in JVM codegen
            if inst.opcode != IrOpcode::Add {
                return;
            }
            let has_ptr = inst.operands.first().is_some_and(|o| o.get_type().is_pointer());
            if !has_ptr {
                return;
            }
        }
        if self.pool.string_slice_ref != 0 {
            return;
        }
        self.stub_needed = true;
        let stub_class = self
            .pool
            .constant_pool
            .add_class("RuntimeStub")
            .expect("Failed to add RuntimeStub class");
        self.pool.string_slice_ref = self
            .pool
            .constant_pool
            .add_method_ref(stub_class, "string_slice", "([BII)[B")
            .expect("Failed to add string_slice method ref");
    }

    fn is_external_function(name: &str) -> bool {
        matches!(
            name,
            "puts"
                | "putchar"
                | "getchar"
                | "printf"
                | "rand"
                | "srand"
                | "time"
                | "Sleep"
                | "malloc"
                | "free"
                | "map_put_jvm"
                | "map_get_jvm"
                | "map_remove_jvm"
                | "map_has_jvm"
                | "map_size_jvm"
                | "map_key_jvm"
                | "map_list_jvm"
                | "fopen"
                | "fgetc"
                | "fclose"
                | "atoi"
                | "fflush"
        )
    }

    fn build_user_method_descriptor(param_types: &[IrType], return_type: Option<&IrType>) -> String {
        let param_desc: String = param_types.iter().map(ir_type_to_jvm_descriptor).collect();
        let ret_desc = return_type
            .as_ref()
            .map_or_else(|| "I".to_string(), |t| ir_type_to_jvm_descriptor(t));
        format!("({param_desc}){ret_desc}")
    }
}

use crate::codegen::jvm::types::{get_fn_interface_name, ir_type_to_jvm_descriptor};
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{IrFunction, IrType};
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Method, MethodAccessFlags};

impl JvmGenerator {
    pub(super) fn build_class_file(
        &mut self,
        class_name: &str,
        func: &IrFunction,
        code: Vec<Instruction>,
    ) -> Vec<u8> {
        let func_name = &func.name;
        let this_class = self.constant_pool.add_class(class_name).unwrap();
        let super_class = self.constant_pool.add_class("java/lang/Object").unwrap();

        let code_attr_name_idx = self.constant_pool.add_utf8("Code").unwrap();
        let max_locals = if self.is_coroutine { 1 } else { self.next_local_slot };
        let max_stack = self.estimate_max_stack(&code);

        let code_size: u32 = code.iter().map(|i| self.instr_size(i) as u32).sum();
        assert!(code_size <= 65535,
            "Function {func_name}: Generated code size ({code_size}) exceeds JVM limit of 65535 bytes. max_stack={max_stack}, max_locals={max_locals}"
        );

        if self.is_coroutine {
            return self.build_coroutine_class(class_name, func, code, this_class, super_class, code_attr_name_idx, max_stack);
        }

        let mut methods = Vec::new();

        let is_func_ref_target = self.func_ref_targets.contains(&func.name);
        let has_env_param = func.parameters.first().is_some_and(|p| p.name == "__env");

        let param_types: String = func
            .parameters
            .iter()
            .map(|p| {
                if p.name == "__env" {
                    "[[I".to_string()
                } else {
                    ir_type_to_jvm_descriptor(&p.ty)
                }
            })
            .collect();
        let return_type = ir_type_to_jvm_descriptor(&func.return_type);
        let call_desc = format!("({param_types}){return_type}");

        if func.name == "main" {
            let call_name_idx = self.constant_pool.add_utf8("call").unwrap();
            let call_desc_idx = self.constant_pool.add_utf8(&call_desc).unwrap();

            let call_code_attr = Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack,
                max_locals,
                code: code.clone(),
                exception_table: vec![],
                attributes: vec![],
            };

            methods.push(Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: call_name_idx,
                descriptor_index: call_desc_idx,
                attributes: vec![call_code_attr],
            });

            let main_name_idx = self.constant_pool.add_utf8("main").unwrap();
            let main_desc_idx = self.constant_pool.add_utf8("([Ljava/lang/String;)V").unwrap();

            let call_ref_idx = self.constant_pool.add_method_ref(this_class, "call", &call_desc).unwrap();
            let system_class = self.constant_pool.add_class("java/lang/System").unwrap();
            let exit_ref_idx = self.constant_pool.add_method_ref(system_class, "exit", "(I)V").unwrap();

            let main_code = vec![
                Instruction::Invokestatic(call_ref_idx),
                Instruction::Invokestatic(exit_ref_idx),
                Instruction::Return,
            ];

            methods.push(Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: main_name_idx,
                descriptor_index: main_desc_idx,
                attributes: vec![Attribute::Code {
                    name_index: code_attr_name_idx,
                    max_stack: 2,
                    max_locals: 1,
                    code: main_code,
                    exception_table: vec![],
                    attributes: vec![],
                }],
            });
        } else {
            let call_name_idx = self.constant_pool.add_utf8("call").unwrap();
            let call_desc_idx = self.constant_pool.add_utf8(&call_desc).unwrap();

            methods.push(Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: call_name_idx,
                descriptor_index: call_desc_idx,
                attributes: vec![Attribute::Code {
                    name_index: code_attr_name_idx,
                    max_stack,
                    max_locals,
                    code: code.clone(),
                    exception_table: vec![],
                    attributes: vec![],
                }],
            });

            if is_func_ref_target {
                let init_name_idx = self.constant_pool.add_utf8("<init>").unwrap();
                let init_desc_idx = self.constant_pool.add_utf8("()V").unwrap();
                let obj_class = self.constant_pool.add_class("java/lang/Object").unwrap();
                let obj_init_ref = self.constant_pool.add_method_ref(obj_class, "<init>", "()V").unwrap();
                let init_code = vec![
                    Instruction::Aload_0,
                    Instruction::Invokespecial(obj_init_ref),
                    Instruction::Return,
                ];
                methods.push(Method {
                    access_flags: MethodAccessFlags::PUBLIC,
                    name_index: init_name_idx,
                    descriptor_index: init_desc_idx,
                    attributes: vec![Attribute::Code {
                        name_index: code_attr_name_idx,
                        max_stack: 1,
                        max_locals: 1,
                        code: init_code,
                        exception_table: vec![],
                        attributes: vec![],
                    }],
                });
            }
            if is_func_ref_target {
                let apply_name_idx = self.constant_pool.add_utf8("apply").unwrap();
                let user_param_types: Vec<IrType> = func
                    .parameters
                    .iter()
                    .filter(|p| p.name != "__env")
                    .map(|p| p.ty.clone())
                    .collect();
                let instance_call_desc = format!(
                    "({}){}",
                    user_param_types.iter().map(ir_type_to_jvm_descriptor).collect::<String>(),
                    ir_type_to_jvm_descriptor(&func.return_type)
                );
                let instance_call_desc_idx = self.constant_pool.add_utf8(&instance_call_desc).unwrap();
                let static_call_ref = self.constant_pool.add_method_ref(this_class, "call", &call_desc).unwrap();

                let mut instance_call_code = Vec::new();

                if has_env_param {
                    instance_call_code.push(Instruction::Aload_0);
                    let env_field_ref = self.constant_pool.add_field_ref(this_class, "__env", "[[I").unwrap();
                    instance_call_code.push(Instruction::Getfield(env_field_ref));
                }

                let mut slot = 1;
                for param in &func.parameters {
                    if param.name == "__env" { continue; }
                    let use_aload = param.ty == IrType::String || matches!(param.ty, IrType::Function(_, _));
                    match slot {
                        1 => { if use_aload { instance_call_code.push(Instruction::Aload_1); } else { instance_call_code.push(Instruction::Iload_1); } }
                        2 => { if use_aload { instance_call_code.push(Instruction::Aload_2); } else { instance_call_code.push(Instruction::Iload_2); } }
                        3 => { if use_aload { instance_call_code.push(Instruction::Aload_3); } else { instance_call_code.push(Instruction::Iload_3); } }
                        _ => { if use_aload { instance_call_code.push(Instruction::Aload(slot as u8)); } else { instance_call_code.push(Instruction::Iload(slot as u8)); } }
                    }
                    slot += 1;
                }
                instance_call_code.push(Instruction::Invokestatic(static_call_ref));
                match &func.return_type {
                    IrType::Void => instance_call_code.push(Instruction::Return),
                    IrType::String => instance_call_code.push(Instruction::Areturn),
                    _ => instance_call_code.push(Instruction::Ireturn),
                }

                methods.push(Method {
                    access_flags: MethodAccessFlags::PUBLIC,
                    name_index: apply_name_idx,
                    descriptor_index: instance_call_desc_idx,
                    attributes: vec![Attribute::Code {
                        name_index: code_attr_name_idx,
                        max_stack: max_stack.max(4),
                        max_locals: slot.max(1),
                        code: instance_call_code,
                        exception_table: vec![],
                        attributes: vec![],
                    }],
                });
            }
        }

        let interfaces: Vec<u16> = if is_func_ref_target {
            let user_params: Vec<IrType> = func.parameters.iter().filter(|p| p.name != "__env").map(|p| p.ty.clone()).collect();
            let iface_name = get_fn_interface_name(&user_params, &func.return_type);
            vec![self.constant_pool.add_class(&iface_name).unwrap()]
        } else {
            vec![]
        };

        let env_field_name_idx;
        let env_field_desc_idx;
        let has_env = has_env_param;
        if has_env {
            env_field_name_idx = self.constant_pool.add_utf8("__env").unwrap();
            env_field_desc_idx = self.constant_pool.add_utf8("[[I").unwrap();
        } else {
            env_field_name_idx = 0;
            env_field_desc_idx = 0;
        }

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_5,
            constant_pool: self.constant_pool.clone(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class,
            super_class,
            interfaces,
            fields: if has_env {
                vec![ristretto_classfile::Field {
                    access_flags: ristretto_classfile::FieldAccessFlags::PUBLIC,
                    name_index: env_field_name_idx,
                    descriptor_index: env_field_desc_idx,
                    field_type: ristretto_classfile::FieldType::parse("[[I").unwrap(),
                    attributes: vec![],
                }]
            } else { vec![] },
            methods,
            attributes: vec![],
            code_source_url: None,
        };

        let mut buffer = Vec::new();
        match class_file.to_bytes(&mut buffer) {
            Ok(()) => buffer,
            Err(e) => {
                panic!("Failed to serialize class file: {e:?}. max_stack={max_stack}, max_locals={max_locals}, code_len={}", code.len());
            }
        }
    }
}

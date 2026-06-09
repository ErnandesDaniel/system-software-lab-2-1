#![allow(clippy::expect_used, clippy::panic, clippy::too_many_arguments)]

use crate::codegen::jvm::types::{get_fn_interface_name, ir_type_to_jvm_descriptor};
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{IrFunction, IrType};
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Method, MethodAccessFlags};

const MAIN_MAX_STACK: u16 = 2;
const INSTANCE_METHOD_MAX_STACK: u16 = 4;

impl JvmGenerator {
    pub(super) fn build_class_file(&mut self, class_name: &str, func: &IrFunction, code: Vec<Instruction>) -> Vec<u8> {
        let func_name = &func.name;
        let this_class = self
            .pool
            .constant_pool
            .add_class(class_name)
            .expect("Failed to add class to constant pool");
        let super_class = self
            .pool
            .constant_pool
            .add_class("java/lang/Object")
            .expect("Failed to add Object class to constant pool");

        let code_attr_name_idx = self
            .pool
            .constant_pool
            .add_utf8("Code")
            .expect("Failed to add UTF8 'Code'");
        let max_locals = self.func.next_local_slot;
        let max_stack = JvmGenerator::estimate_max_stack(&code);

        let code_size: u32 = code.iter().map(|i| JvmGenerator::instr_size(i) as u32).sum();
        assert!(code_size <= 65535,
            "Function {func_name}: Generated code size ({code_size}) exceeds JVM limit of 65535 bytes. max_stack={max_stack}, max_locals={max_locals}"
        );

        let mut methods = Vec::new();

        let is_func_ref_target = self.closure.func_ref_targets.contains(&func.name);
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
            self.build_main_method(
                &mut methods,
                &code,
                &call_desc,
                &call_desc,
                code_attr_name_idx,
                max_stack,
                max_locals,
                this_class,
            );
        } else {
            self.build_user_method(
                &mut methods,
                &code,
                &call_desc,
                code_attr_name_idx,
                max_stack,
                max_locals,
            );
            if is_func_ref_target {
                self.build_func_ref_methods(
                    &mut methods,
                    func,
                    &call_desc,
                    code_attr_name_idx,
                    max_stack,
                    max_locals,
                    has_env_param,
                    this_class,
                );
            }
        }

        let interfaces = self.build_interfaces(is_func_ref_target, func);

        let (env_field_name_idx, env_field_desc_idx) = self.build_env_fields(has_env_param);

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_5,
            constant_pool: self.pool.constant_pool.clone(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class,
            super_class,
            interfaces,
            fields: if has_env_param {
                vec![ristretto_classfile::Field {
                    access_flags: ristretto_classfile::FieldAccessFlags::PUBLIC,
                    name_index: env_field_name_idx,
                    descriptor_index: env_field_desc_idx,
                    field_type: ristretto_classfile::FieldType::parse("[[I").expect("Failed to parse field type"),
                    attributes: vec![],
                }]
            } else {
                vec![]
            },
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

    fn build_main_method(
        &mut self,
        methods: &mut Vec<Method>,
        code: &[Instruction],
        call_desc: &str,
        _original_desc: &str,
        code_attr_name_idx: u16,
        max_stack: u16,
        max_locals: u16,
        this_class: u16,
    ) {
        let call_name_idx = self
            .pool
            .constant_pool
            .add_utf8("call")
            .expect("Failed to add UTF8 'call'");
        let call_desc_idx = self
            .pool
            .constant_pool
            .add_utf8(call_desc)
            .expect("Failed to add call descriptor");

        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: call_name_idx,
            descriptor_index: call_desc_idx,
            attributes: vec![Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack,
                max_locals,
                code: code.to_vec(),
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        let main_name_idx = self
            .pool
            .constant_pool
            .add_utf8("main")
            .expect("Failed to add UTF8 'main'");
        let main_desc_idx = self
            .pool
            .constant_pool
            .add_utf8("([Ljava/lang/String;)V")
            .expect("Failed to add main descriptor");

        let call_ref_idx = self
            .pool
            .constant_pool
            .add_method_ref(this_class, "call", call_desc)
            .expect("Failed to add method ref for call");
        let system_class = self
            .pool
            .constant_pool
            .add_class("java/lang/System")
            .expect("Failed to add System class");
        let exit_ref_idx = self
            .pool
            .constant_pool
            .add_method_ref(system_class, "exit", "(I)V")
            .expect("Failed to add exit method ref");

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
                max_stack: MAIN_MAX_STACK,
                max_locals: 1,
                code: main_code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_user_method(
        &mut self,
        methods: &mut Vec<Method>,
        code: &[Instruction],
        call_desc: &str,
        code_attr_name_idx: u16,
        max_stack: u16,
        max_locals: u16,
    ) {
        let call_name_idx = self
            .pool
            .constant_pool
            .add_utf8("call")
            .expect("Failed to add UTF8 'call'");
        let call_desc_idx = self
            .pool
            .constant_pool
            .add_utf8(call_desc)
            .expect("Failed to add call descriptor");

        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: call_name_idx,
            descriptor_index: call_desc_idx,
            attributes: vec![Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack,
                max_locals,
                code: code.to_vec(),
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_func_ref_methods(
        &mut self,
        methods: &mut Vec<Method>,
        func: &IrFunction,
        call_desc: &str,
        code_attr_name_idx: u16,
        max_stack: u16,
        _max_locals: u16,
        has_env_param: bool,
        this_class: u16,
    ) {
        let init_name_idx = self
            .pool
            .constant_pool
            .add_utf8("<init>")
            .expect("Failed to add UTF8 '<init>'");
        let init_desc_idx = self
            .pool
            .constant_pool
            .add_utf8("()V")
            .expect("Failed to add init descriptor");
        let obj_class = self
            .pool
            .constant_pool
            .add_class("java/lang/Object")
            .expect("Failed to add Object class");
        let obj_init_ref = self
            .pool
            .constant_pool
            .add_method_ref(obj_class, "<init>", "()V")
            .expect("Failed to add Object init ref");
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

        let apply_name_idx = self
            .pool
            .constant_pool
            .add_utf8("apply")
            .expect("Failed to add UTF8 'apply'");
        let user_param_types: Vec<IrType> = func
            .parameters
            .iter()
            .filter(|p| p.name != "__env")
            .map(|p| p.ty.clone())
            .collect();
        let instance_call_desc = format!(
            "({}){}",
            user_param_types
                .iter()
                .map(ir_type_to_jvm_descriptor)
                .collect::<String>(),
            ir_type_to_jvm_descriptor(&func.return_type)
        );
        let instance_call_desc_idx = self
            .pool
            .constant_pool
            .add_utf8(&instance_call_desc)
            .expect("Failed to add instance call descriptor");
        let static_call_ref = self
            .pool
            .constant_pool
            .add_method_ref(this_class, "call", call_desc)
            .expect("Failed to add static call ref for func ref");

        let mut instance_call_code = Vec::new();

        if has_env_param {
            instance_call_code.push(Instruction::Aload_0);
            let env_field_ref = self
                .pool
                .constant_pool
                .add_field_ref(this_class, "__env", "[[I")
                .expect("Failed to add env field ref");
            instance_call_code.push(Instruction::Getfield(env_field_ref));
        }

        let mut slot = 1;
        for param in &func.parameters {
            if param.name == "__env" {
                continue;
            }
            let use_aload = param.ty == IrType::String || matches!(param.ty, IrType::Function(_, _));
            match slot {
                1 => {
                    if use_aload {
                        instance_call_code.push(Instruction::Aload_1);
                    } else {
                        instance_call_code.push(Instruction::Iload_1);
                    }
                }
                2 => {
                    if use_aload {
                        instance_call_code.push(Instruction::Aload_2);
                    } else {
                        instance_call_code.push(Instruction::Iload_2);
                    }
                }
                3 => {
                    if use_aload {
                        instance_call_code.push(Instruction::Aload_3);
                    } else {
                        instance_call_code.push(Instruction::Iload_3);
                    }
                }
                _ => {
                    if use_aload {
                        instance_call_code.push(Instruction::Aload(slot as u8));
                    } else {
                        instance_call_code.push(Instruction::Iload(slot as u8));
                    }
                }
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
                max_stack: max_stack.max(INSTANCE_METHOD_MAX_STACK),
                max_locals: slot.max(1),
                code: instance_call_code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_interfaces(&mut self, is_func_ref_target: bool, func: &IrFunction) -> Vec<u16> {
        if !is_func_ref_target {
            return vec![];
        }
        let user_params: Vec<IrType> = func
            .parameters
            .iter()
            .filter(|p| p.name != "__env")
            .map(|p| p.ty.clone())
            .collect();
        let iface_name = get_fn_interface_name(&user_params, &func.return_type);
        vec![self
            .pool
            .constant_pool
            .add_class(&iface_name)
            .expect("Failed to add interface class")]
    }

    fn build_env_fields(&mut self, has_env: bool) -> (u16, u16) {
        if !has_env {
            return (0, 0);
        }
        let name_idx = self
            .pool
            .constant_pool
            .add_utf8("__env")
            .expect("Failed to add UTF8 '__env'");
        let desc_idx = self
            .pool
            .constant_pool
            .add_utf8("[[I")
            .expect("Failed to add env descriptor");
        (name_idx, desc_idx)
    }

    pub fn generate_fn_interface(&mut self, params: &[IrType], ret: &IrType) -> Vec<u8> {
        let iface_name = get_fn_interface_name(params, ret);
        let this_class = self
            .pool
            .constant_pool
            .add_class(&iface_name)
            .expect("Failed to add to constant pool");
        let super_class = self
            .pool
            .constant_pool
            .add_class("java/lang/Object")
            .expect("Failed to add to constant pool");

        let method_desc = format!(
            "({}){}",
            params
                .iter()
                .map(crate::codegen::jvm::types::ir_type_to_jvm_descriptor)
                .collect::<String>(),
            crate::codegen::jvm::types::ir_type_to_jvm_descriptor(ret)
        );
        let method_name_idx = self
            .pool
            .constant_pool
            .add_utf8("apply")
            .expect("Failed to add to constant pool");
        let method_desc_idx = self
            .pool
            .constant_pool
            .add_utf8(&method_desc)
            .expect("Failed to add to constant pool");

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_5,
            constant_pool: self.pool.constant_pool.clone(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::INTERFACE | ClassAccessFlags::ABSTRACT,
            this_class,
            super_class,
            interfaces: vec![],
            fields: vec![],
            methods: vec![Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::ABSTRACT,
                name_index: method_name_idx,
                descriptor_index: method_desc_idx,
                attributes: vec![],
            }],
            attributes: vec![],
            code_source_url: None,
        };

        let mut buffer = Vec::new();
        match class_file.to_bytes(&mut buffer) {
            Ok(()) => buffer,
            Err(e) => panic!("Failed to generate interface class: {e:?}"),
        }
    }
}

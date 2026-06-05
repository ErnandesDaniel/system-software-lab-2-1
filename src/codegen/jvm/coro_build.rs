use crate::codegen::jvm::types::get_fn_interface_name;
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{IrFunction, IrType};
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Method, MethodAccessFlags};

impl JvmGenerator {
    pub(super) fn build_coroutine_class(
        &mut self,
        _class_name: &str,
        _func: &IrFunction,
        code: Vec<Instruction>,
        this_class: u16,
        super_class: u16,
        code_attr_name_idx: u16,
        max_stack: u16,
    ) -> Vec<u8> {
        let code_len = code.len();
        let mut methods = Vec::new();

        let init_name_idx = self
            .pool
            .constant_pool
            .add_utf8("<init>")
            .expect("Failed to add to constant pool");
        let init_desc_idx = self
            .pool
            .constant_pool
            .add_utf8("()V")
            .expect("Failed to add to constant pool");
        let obj_init_ref = self
            .pool
            .constant_pool
            .add_method_ref(super_class, "<init>", "()V")
            .expect("Failed to add to constant pool");
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

        let resume_name_idx = self
            .pool
            .constant_pool
            .add_utf8("resume")
            .expect("Failed to add to constant pool");
        let resume_desc_idx = self
            .pool
            .constant_pool
            .add_utf8("()I")
            .expect("Failed to add to constant pool");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC,
            name_index: resume_name_idx,
            descriptor_index: resume_desc_idx,
            attributes: vec![Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack,
                max_locals: 1,
                code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        let get_state_name_idx = self
            .pool
            .constant_pool
            .add_utf8("getState")
            .expect("Failed to add to constant pool");
        let get_state_desc_idx = self
            .pool
            .constant_pool
            .add_utf8("()I")
            .expect("Failed to add to constant pool");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC,
            name_index: get_state_name_idx,
            descriptor_index: get_state_desc_idx,
            attributes: vec![Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack: 1,
                max_locals: 1,
                code: vec![
                    Instruction::Aload_0,
                    Instruction::Getfield(self.coro.coroutine_state_field),
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        let get_result_name_idx = self
            .pool
            .constant_pool
            .add_utf8("getResult")
            .expect("Failed to add to constant pool");
        let get_result_desc_idx = self
            .pool
            .constant_pool
            .add_utf8("()I")
            .expect("Failed to add to constant pool");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC,
            name_index: get_result_name_idx,
            descriptor_index: get_result_desc_idx,
            attributes: vec![Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack: 1,
                max_locals: 1,
                code: vec![
                    Instruction::Aload_0,
                    Instruction::Getfield(self.coro.coroutine_result_field),
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        let fields: Vec<ristretto_classfile::Field> = self
            .coro
            .coroutine_field_entries
            .iter()
            .map(|&(name_idx, desc_idx, ref desc_str)| {
                ristretto_classfile::Field {
                    access_flags: ristretto_classfile::FieldAccessFlags::PUBLIC,
                    name_index: name_idx,
                    descriptor_index: desc_idx,
                    field_type: ristretto_classfile::FieldType::parse(desc_str).unwrap_or_else(|_| panic!("Bad field desc: {desc_str}")),
                    attributes: vec![],
                }
            })
            .collect();

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_5,
            constant_pool: self.pool.constant_pool.clone(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class,
            super_class,
            interfaces: vec![],
            fields,
            methods,
            attributes: vec![],
            code_source_url: None,
        };

        let mut buffer = Vec::new();
        match class_file.to_bytes(&mut buffer) {
            Ok(()) => buffer,
            Err(e) => panic!("Failed to serialize coroutine class file: {e:?}. max_stack={max_stack}, max_locals=1, code_len={code_len}"),
        }
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

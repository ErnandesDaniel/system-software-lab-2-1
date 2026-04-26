use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Method, MethodAccessFlags};
use crate::ir::types::*;
use crate::codegen::jvm::JvmGenerator;
use crate::codegen::jvm::types::ir_type_to_jvm_descriptor;

impl JvmGenerator {
    pub fn build_class_file(&mut self, class_name: &str, func: &IrFunction, code: Vec<ristretto_classfile::attributes::Instruction>) -> Vec<u8> {
        let this_class = self.constant_pool.add_class(class_name).unwrap();
        let super_class = self.constant_pool.add_class("java/lang/Object").unwrap();

        let code_attr_name_idx = self.constant_pool.add_utf8("Code").unwrap();
        let max_locals = self.next_local_slot;
        let max_stack = self.estimate_max_stack(&code);

        let mut methods = Vec::new();

        let param_types: String = func.parameters.iter()
            .map(|p| ir_type_to_jvm_descriptor(&p.ty))
            .collect();
        let return_type = ir_type_to_jvm_descriptor(&func.return_type);
        let call_desc = format!("({}){}", param_types, return_type);

        if func.name == "main" {
            let call_name_idx = self.constant_pool.add_utf8("call").unwrap();
            let call_desc_idx = self.constant_pool.add_utf8(&call_desc).unwrap();
            
            let call_code_attr = Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack,
                max_locals,
                code,
                exception_table: vec![],
                attributes: vec![],
            };

            let call_method = Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: call_name_idx,
                descriptor_index: call_desc_idx,
                attributes: vec![call_code_attr],
            };
            methods.push(call_method);

            let main_name_idx = self.constant_pool.add_utf8("main").unwrap();
            let main_desc = "([Ljava/lang/String;)V".to_string();
            let main_desc_idx = self.constant_pool.add_utf8(&main_desc).unwrap();

            let call_ref_idx = self.constant_pool
                .add_method_ref(this_class, "call", &call_desc)
                .unwrap();

            let main_code = vec![
                Instruction::Invokestatic(call_ref_idx),
                Instruction::Return,
            ];

            let main_code_attr = Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack: 1,
                max_locals: 1,
                code: main_code,
                exception_table: vec![],
                attributes: vec![],
            };

            let main_method = Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: main_name_idx,
                descriptor_index: main_desc_idx,
                attributes: vec![main_code_attr],
            };
            methods.push(main_method);
        } else {
            let call_name_idx = self.constant_pool.add_utf8("call").unwrap();
            let call_desc_idx = self.constant_pool.add_utf8(&call_desc).unwrap();

            let call_code_attr = Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack,
                max_locals,
                code,
                exception_table: vec![],
                attributes: vec![],
            };

            let call_method = Method {
                access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
                name_index: call_name_idx,
                descriptor_index: call_desc_idx,
                attributes: vec![call_code_attr],
            };
            methods.push(call_method);
        }

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_6,
            constant_pool: self.constant_pool.clone(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class,
            super_class,
            interfaces: vec![],
            fields: vec![],
            methods,
            attributes: vec![],
            code_source_url: None,
        };

        let mut buffer = Vec::new();
        class_file.to_bytes(&mut buffer).unwrap();
        buffer
    }

    fn estimate_max_stack(&self, code: &[Instruction]) -> u16 {
        let pushes = code.iter().filter(|i| self.instr_pushes(i)).count() as u16;
        (pushes / 2 + 2).max(4)
    }

    fn instr_pushes(&self, instr: &Instruction) -> bool {
        matches!(instr,
            Instruction::Iconst_m1 | Instruction::Iconst_0 |
            Instruction::Iconst_1 | Instruction::Iconst_2 |
            Instruction::Iconst_3 | Instruction::Iconst_4 |
            Instruction::Iconst_5 | Instruction::Bipush(_) |
            Instruction::Sipush(_) | Instruction::Ldc(_) |
            Instruction::Ldc_w(_) | Instruction::Iload(_) |
            Instruction::Iload_0 | Instruction::Iload_1 |
            Instruction::Iload_2 | Instruction::Iload_3 |
            Instruction::Iaload | Instruction::Invokestatic(_)
        )
    }
}

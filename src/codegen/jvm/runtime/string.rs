#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::too_many_arguments, clippy::vec_init_then_push)]

use crate::codegen::jvm::JvmGenerator;
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{Method, MethodAccessFlags};

impl JvmGenerator {
    pub(super) fn build_string_slice(&mut self, methods: &mut Vec<Method>, code_attr: u16) {
        let ss_name = self.pool.constant_pool.add_utf8("string_slice").expect("utf8");
        let ss_desc = self.pool.constant_pool.add_utf8("([BII)[B").expect("utf8");
        let system_class = self.pool.constant_pool.add_class("java/lang/System").expect("class");
        let arraycopy_ref = self
            .pool
            .constant_pool
            .add_method_ref(system_class, "arraycopy", "(Ljava/lang/Object;ILjava/lang/Object;II)V")
            .expect("mref");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: ss_name,
            descriptor_index: ss_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5,
                max_locals: 7,
                code: vec![
                    Instruction::Iload_2,
                    Instruction::Iload_1,
                    Instruction::Isub,
                    Instruction::Dup,
                    Instruction::Istore(4),
                    Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Byte),
                    Instruction::Astore(3),
                    Instruction::Aload_0,
                    Instruction::Iload_1,
                    Instruction::Aload(3),
                    Instruction::Iconst_0,
                    Instruction::Iload(4),
                    Instruction::Invokestatic(arraycopy_ref),
                    Instruction::Aload(3),
                    Instruction::Areturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    pub(super) fn build_printf1(
        &mut self,
        methods: &mut Vec<Method>,
        code_attr: u16,
        string_class: u16,
        string_byte_init: u16,
        str_init_3arg: u16,
        system_out_ref: u16,
        print_str_ref: u16,
        int_to_str: u16,
        str_replace: u16,
    ) {
        let printf_name = self.pool.constant_pool.add_utf8("printf").expect("utf8");
        let printf1_desc = self.pool.constant_pool.add_utf8("([BI)I").expect("utf8");
        let mut c1 = Vec::new();
        c1.push(Instruction::Iconst_2);
        c1.push(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Byte));
        c1.push(Instruction::Dup);
        c1.push(Instruction::Iconst_0);
        c1.push(Instruction::Bipush(37));
        c1.push(Instruction::Bastore);
        c1.push(Instruction::Dup);
        c1.push(Instruction::Iconst_1);
        c1.push(Instruction::Bipush(100));
        c1.push(Instruction::Bastore);
        c1.push(Instruction::Astore(2));
        c1.push(Instruction::New(string_class));
        c1.push(Instruction::Dup);
        c1.push(Instruction::Aload_0);
        c1.push(Instruction::Iconst_0);
        c1.push(Instruction::Aload_0);
        c1.push(Instruction::Invokestatic(self.pool.nullscan_ref));
        c1.push(Instruction::Invokespecial(str_init_3arg));
        c1.push(Instruction::Astore(3));
        c1.push(Instruction::New(string_class));
        c1.push(Instruction::Dup);
        c1.push(Instruction::Aload(2));
        c1.push(Instruction::Invokespecial(string_byte_init));
        c1.push(Instruction::Astore(2));
        c1.push(Instruction::Iload_1);
        c1.push(Instruction::Invokestatic(int_to_str));
        c1.push(Instruction::Astore(4));
        c1.push(Instruction::Aload(3));
        c1.push(Instruction::Aload(2));
        c1.push(Instruction::Aload(4));
        c1.push(Instruction::Invokevirtual(str_replace));
        c1.push(Instruction::Getstatic(system_out_ref));
        c1.push(Instruction::Swap);
        c1.push(Instruction::Invokevirtual(print_str_ref));
        c1.push(Instruction::Iconst_0);
        c1.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: printf_name,
            descriptor_index: printf1_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5,
                max_locals: 5,
                code: c1,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    pub(super) fn build_printf2(
        &mut self,
        methods: &mut Vec<Method>,
        code_attr: u16,
        string_class: u16,
        string_byte_init: u16,
        str_init_3arg: u16,
        system_out_ref: u16,
        print_str_ref: u16,
        int_to_str: u16,
        str_replace: u16,
    ) {
        let printf_name = self.pool.constant_pool.add_utf8("printf").expect("utf8");
        let printf2_desc = self.pool.constant_pool.add_utf8("([BII)I").expect("utf8");
        let mut c2 = Vec::new();
        c2.push(Instruction::Iconst_2);
        c2.push(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Byte));
        c2.push(Instruction::Dup);
        c2.push(Instruction::Iconst_0);
        c2.push(Instruction::Bipush(37));
        c2.push(Instruction::Bastore);
        c2.push(Instruction::Dup);
        c2.push(Instruction::Iconst_1);
        c2.push(Instruction::Bipush(100));
        c2.push(Instruction::Bastore);
        c2.push(Instruction::Astore(5));
        c2.push(Instruction::New(string_class));
        c2.push(Instruction::Dup);
        c2.push(Instruction::Aload_0);
        c2.push(Instruction::Iconst_0);
        c2.push(Instruction::Aload_0);
        c2.push(Instruction::Invokestatic(self.pool.nullscan_ref));
        c2.push(Instruction::Invokespecial(str_init_3arg));
        c2.push(Instruction::Astore(3));
        c2.push(Instruction::New(string_class));
        c2.push(Instruction::Dup);
        c2.push(Instruction::Aload(5));
        c2.push(Instruction::Invokespecial(string_byte_init));
        c2.push(Instruction::Astore(5));
        c2.push(Instruction::Aload(3));
        c2.push(Instruction::Aload(5));
        c2.push(Instruction::Iload_1);
        c2.push(Instruction::Invokestatic(int_to_str));
        c2.push(Instruction::Invokevirtual(str_replace));
        c2.push(Instruction::Astore(3));
        c2.push(Instruction::Aload(3));
        c2.push(Instruction::Aload(5));
        c2.push(Instruction::Iload_2);
        c2.push(Instruction::Invokestatic(int_to_str));
        c2.push(Instruction::Invokevirtual(str_replace));
        c2.push(Instruction::Getstatic(system_out_ref));
        c2.push(Instruction::Swap);
        c2.push(Instruction::Invokevirtual(print_str_ref));
        c2.push(Instruction::Iconst_0);
        c2.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: printf_name,
            descriptor_index: printf2_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5,
                max_locals: 6,
                code: c2,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    pub(super) fn build_printf_methods(
        &mut self,
        methods: &mut Vec<Method>,
        code_attr: u16,
        string_class: u16,
        string_byte_init: u16,
        str_init_3arg: u16,
        system_out_ref: u16,
        printstream_class: u16,
        int_class: u16,
    ) {
        let print_str_ref = self
            .pool
            .constant_pool
            .add_method_ref(printstream_class, "print", "(Ljava/lang/String;)V")
            .expect("mref");
        let int_to_str = self
            .pool
            .constant_pool
            .add_method_ref(int_class, "toString", "(I)Ljava/lang/String;")
            .expect("mref");
        let str_replace = self
            .pool
            .constant_pool
            .add_method_ref(
                string_class,
                "replaceFirst",
                "(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
            )
            .expect("mref");
        self.build_printf1(
            methods,
            code_attr,
            string_class,
            string_byte_init,
            str_init_3arg,
            system_out_ref,
            print_str_ref,
            int_to_str,
            str_replace,
        );
        self.build_printf2(
            methods,
            code_attr,
            string_class,
            string_byte_init,
            str_init_3arg,
            system_out_ref,
            print_str_ref,
            int_to_str,
            str_replace,
        );
    }

    pub(super) fn build_putchar(
        &mut self,
        methods: &mut Vec<Method>,
        code_attr: u16,
        system_out_ref: u16,
        print_char_ref: u16,
    ) {
        let n = self.pool.constant_pool.add_utf8("putchar").unwrap();
        let d = self.pool.constant_pool.add_utf8("(I)I").unwrap();
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: n,
            descriptor_index: d,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 2,
                max_locals: 1,
                code: vec![
                    Instruction::Getstatic(system_out_ref),
                    Instruction::Iload_0,
                    Instruction::I2c,
                    Instruction::Invokevirtual(print_char_ref),
                    Instruction::Iload_0,
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    pub(super) fn build_puts(
        &mut self,
        methods: &mut Vec<Method>,
        code_attr: u16,
        system_out_ref: u16,
        string_class: u16,
        str_init_3arg: u16,
        println_string_ref: u16,
    ) {
        let n = self.pool.constant_pool.add_utf8("puts").unwrap();
        let d = self.pool.constant_pool.add_utf8("([B)I").unwrap();
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: n,
            descriptor_index: d,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 6,
                max_locals: 2,
                code: vec![
                    Instruction::Getstatic(system_out_ref),
                    Instruction::New(string_class),
                    Instruction::Dup,
                    Instruction::Aload_0,
                    Instruction::Iconst_0,
                    Instruction::Aload_0,
                    Instruction::Invokestatic(self.pool.nullscan_ref),
                    Instruction::Invokespecial(str_init_3arg),
                    Instruction::Invokevirtual(println_string_ref),
                    Instruction::Iconst_0,
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    pub(super) fn build_malloc(&mut self, methods: &mut Vec<Method>, code_attr: u16) {
        let n = self.pool.constant_pool.add_utf8("malloc").unwrap();
        let d = self.pool.constant_pool.add_utf8("(I)[B").unwrap();
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: n,
            descriptor_index: d,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 1,
                max_locals: 1,
                code: vec![
                    Instruction::Iload_0,
                    Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Byte),
                    Instruction::Areturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    pub(super) fn build_free(&mut self, methods: &mut Vec<Method>, code_attr: u16) {
        let n = self.pool.constant_pool.add_utf8("free").unwrap();
        let d = self.pool.constant_pool.add_utf8("([B)V").unwrap();
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: n,
            descriptor_index: d,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 0,
                max_locals: 1,
                code: vec![Instruction::Return],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    pub(super) fn build_atoi(
        &mut self,
        methods: &mut Vec<Method>,
        code_attr: u16,
        string_class: u16,
        str_init_3arg: u16,
        int_parse: u16,
    ) {
        let atoi_name = self.pool.constant_pool.add_utf8("atoi").expect("utf8");
        let atoi_desc = self.pool.constant_pool.add_utf8("([B)I").expect("utf8");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: atoi_name,
            descriptor_index: atoi_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5,
                max_locals: 3,
                code: vec![
                    Instruction::Iconst_0,
                    Instruction::Istore(1),
                    Instruction::Goto(4),
                    Instruction::Iinc(1, 1),
                    Instruction::Iload(1),
                    Instruction::Aload_0,
                    Instruction::Arraylength,
                    Instruction::If_icmpge(21),
                    Instruction::Aload_0,
                    Instruction::Iload(1),
                    Instruction::Baload,
                    Instruction::Istore(2),
                    Instruction::Iload(2),
                    Instruction::Ifeq(21),
                    Instruction::Iload(2),
                    Instruction::Bipush(10),
                    Instruction::If_icmpeq(21),
                    Instruction::Iload(2),
                    Instruction::Bipush(13),
                    Instruction::If_icmpeq(21),
                    Instruction::Goto(3),
                    Instruction::New(string_class),
                    Instruction::Dup,
                    Instruction::Aload_0,
                    Instruction::Iconst_0,
                    Instruction::Iload(1),
                    Instruction::Invokespecial(str_init_3arg),
                    Instruction::Invokestatic(int_parse),
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }
}

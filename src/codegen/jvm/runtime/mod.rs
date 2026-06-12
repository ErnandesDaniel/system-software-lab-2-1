#![allow(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::too_many_arguments)]

mod string;

use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{IrFunction, IrType};
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Field, FieldAccessFlags, FieldType, Method, MethodAccessFlags};

impl JvmGenerator {
    pub(super) fn generate_runtime_stub(&mut self, _functions: &[IrFunction]) -> Vec<u8> {
        let this_class = self.pool.constant_pool.add_class("RuntimeStub").unwrap();
        let super_class = self.pool.constant_pool.add_class("java/lang/Object").unwrap();
        let code_attr = self.pool.constant_pool.add_utf8("Code").unwrap();
        let mut methods = Vec::new();

        let system_class = self.pool.constant_pool.add_class("java/lang/System").unwrap();
        let system_out_ref = self
            .pool
            .constant_pool
            .add_field_ref(system_class, "out", "Ljava/io/PrintStream;")
            .unwrap();
        let printstream_class = self.pool.constant_pool.add_class("java/io/PrintStream").unwrap();
        let print_char_ref = self
            .pool
            .constant_pool
            .add_method_ref(printstream_class, "print", "(C)V")
            .unwrap();
        let println_string_ref = self
            .pool
            .constant_pool
            .add_method_ref(printstream_class, "println", "(Ljava/lang/String;)V")
            .unwrap();

        self.build_putchar(&mut methods, code_attr, system_out_ref, print_char_ref);

        let string_class = self.pool.constant_pool.add_class("java/lang/String").unwrap();
        let string_byte_init = self
            .pool
            .constant_pool
            .add_method_ref(string_class, "<init>", "([B)V")
            .unwrap();
        let file_fds_name = self.pool.constant_pool.add_utf8("fileStreams").unwrap();
        let file_fds_desc = self.pool.constant_pool.add_utf8("[Ljava/io/InputStream;").unwrap();
        let file_next_name = self.pool.constant_pool.add_utf8("fileNext").unwrap();
        let file_next_desc = self.pool.constant_pool.add_utf8("I").unwrap();
        let file_fds_ref = self
            .pool
            .constant_pool
            .add_field_ref(this_class, "fileStreams", "[Ljava/io/InputStream;")
            .unwrap();
        let file_next_ref = self
            .pool
            .constant_pool
            .add_field_ref(this_class, "fileNext", "I")
            .unwrap();
        let fis_class = self.pool.constant_pool.add_class("java/io/FileInputStream").unwrap();
        let fis_init = self
            .pool
            .constant_pool
            .add_method_ref(fis_class, "<init>", "(Ljava/lang/String;)V")
            .unwrap();
        let is_class = self.pool.constant_pool.add_class("java/io/InputStream").unwrap();
        let is_read = self.pool.constant_pool.add_method_ref(is_class, "read", "()I").unwrap();
        let is_close = self
            .pool
            .constant_pool
            .add_method_ref(is_class, "close", "()V")
            .unwrap();
        let int_class = self.pool.constant_pool.add_class("java/lang/Integer").unwrap();
        let int_parse = self
            .pool
            .constant_pool
            .add_method_ref(int_class, "parseInt", "(Ljava/lang/String;)I")
            .unwrap();
        let str_init_3arg = self
            .pool
            .constant_pool
            .add_method_ref(string_class, "<init>", "([BII)V")
            .unwrap();

        self.build_nullscan(&mut methods, code_attr, this_class);
        self.build_fopen(
            &mut methods,
            code_attr,
            string_class,
            str_init_3arg,
            fis_class,
            fis_init,
            file_fds_ref,
            file_next_ref,
            is_class,
        );
        self.build_puts(
            &mut methods,
            code_attr,
            system_out_ref,
            string_class,
            str_init_3arg,
            println_string_ref,
        );
        self.build_fgetc(&mut methods, code_attr, file_fds_ref, is_class, is_read);
        self.build_fclose(&mut methods, code_attr, file_fds_ref, is_class, is_close);
        self.build_malloc(&mut methods, code_attr);
        self.build_free(&mut methods, code_attr);
        self.build_string_slice(&mut methods, code_attr);
        self.build_printf_methods(
            &mut methods,
            code_attr,
            string_class,
            string_byte_init,
            str_init_3arg,
            system_out_ref,
            printstream_class,
            int_class,
        );
        self.build_atoi(&mut methods, code_attr, string_class, str_init_3arg, int_parse);

        let mut clinit_code = vec![
            Instruction::Bipush(16),
            Instruction::Anewarray(is_class),
            Instruction::Putstatic(file_fds_ref),
        ];
        for (gname, gty) in &self.global.global_vars {
            let runtime_stub_class = self.pool.runtime_stub_class_ref;
            let desc = self.global_jvm_descriptor(gname, gty);
            let fr = self.pool.constant_pool.add_field_ref(runtime_stub_class, gname, &desc).unwrap();
            self.global.global_field_refs.insert(gname.clone(), fr);
                if self.global.global_uses_object_array.contains(gname) {
                    let size = self.get_global_object_array_inner_size(gname) as i8;
                    if self.pool.object_class_idx == 0 {
                        self.pool.object_class_idx = self.pool.constant_pool.add_class("java/lang/Object").unwrap();
                    }
                    clinit_code.push(Instruction::Bipush(size));
                    clinit_code.push(Instruction::Anewarray(self.pool.object_class_idx));
                    clinit_code.push(Instruction::Putstatic(fr));
                } else if let IrType::Array(elem, n) = gty {
                    if *n > 0 {
                        let sz = *n as i16;
                        clinit_code.push(if sz <= 127 {
                            Instruction::Bipush(sz as i8)
                        } else {
                            Instruction::Sipush(sz)
                        });
                        match elem.as_ref() {
                            IrType::Int | IrType::Bool => {
                                let at = if matches!(elem.as_ref(), IrType::Bool) {
                                    ristretto_classfile::attributes::ArrayType::Boolean
                                } else {
                                    ristretto_classfile::attributes::ArrayType::Int
                                };
                                clinit_code.push(Instruction::Newarray(at));
                            }
                            IrType::Function(_, _) | IrType::Closure(_, _) | IrType::String | IrType::Array(..) => {
                                let desc = crate::codegen::jvm::types::ir_type_to_jvm_descriptor(elem);
                                let class_name = desc.trim_start_matches('L').trim_end_matches(';');
                                if let Ok(class_idx) = self.pool.constant_pool.add_class(class_name) {
                                    clinit_code.push(Instruction::Anewarray(class_idx));
                                }
                            }
                            _ => {
                                clinit_code.push(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Int));
                            }
                        }
                        clinit_code.push(Instruction::Putstatic(fr));
                    }
                } else if self.global.global_struct_offset_sets.contains_key(gname) {
                    let offsets = self.global.global_struct_offset_sets.get(gname).unwrap();
                    let size = (offsets.iter().max().unwrap_or(&0) / 4 + 1) as i16;
                    clinit_code.push(if size <= 127 {
                        Instruction::Bipush(size as i8)
                    } else {
                        Instruction::Sipush(size)
                    });
                    clinit_code.push(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Int));
                    clinit_code.push(Instruction::Putstatic(fr));
                }
        }
        clinit_code.push(Instruction::Return);

        // Re-add field refs so generatE_runtime_stub uses fresh pool indices
        self.global.global_field_refs.clear();
        self.collect_global_field_refs();
        methods.push(Method {
            access_flags: MethodAccessFlags::STATIC,
            name_index: self.pool.constant_pool.add_utf8("<clinit>").unwrap(),
            descriptor_index: self.pool.constant_pool.add_utf8("()V").unwrap(),
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 3,
                max_locals: 0,
                code: clinit_code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        let mut fields = vec![];
        fields.push(Field {
            access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC,
            name_index: file_fds_name,
            descriptor_index: file_fds_desc,
            field_type: FieldType::parse("[Ljava/io/InputStream;").unwrap(),
            attributes: vec![],
        });
        fields.push(Field {
            access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC,
            name_index: file_next_name,
            descriptor_index: file_next_desc,
            field_type: FieldType::parse("I").unwrap(),
            attributes: vec![],
        });
        for (gname, gty) in &self.global.global_vars {
            let desc = self.global_jvm_descriptor(gname, gty);
            let desc_idx = self.pool.constant_pool.add_utf8(&desc).unwrap();
            fields.push(Field {
                access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC,
                name_index: self.pool.constant_pool.add_utf8(gname).unwrap(),
                descriptor_index: desc_idx,
                field_type: FieldType::parse(&desc).unwrap(),
                attributes: vec![],
            });
        }

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
        let mut buf = Vec::new();
        match class_file.to_bytes(&mut buf) {
            Ok(()) => buf,
            Err(e) => panic!("RuntimeStub serialization: {e:?}"),
        }
    }

    fn build_nullscan(&mut self, methods: &mut Vec<Method>, code_attr: u16, this_class: u16) {
        let n = self.pool.constant_pool.add_utf8("nullscan").unwrap();
        let d = self.pool.constant_pool.add_utf8("([B)I").unwrap();
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: n,
            descriptor_index: d,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 3,
                max_locals: 3,
                code: vec![
                    Instruction::Iconst_0,
                    Instruction::Istore(1),
                    Instruction::Goto(4),
                    Instruction::Iinc(1, 1),
                    Instruction::Iload(1),
                    Instruction::Aload_0,
                    Instruction::Arraylength,
                    Instruction::If_icmpge(13),
                    Instruction::Aload_0,
                    Instruction::Iload(1),
                    Instruction::Baload,
                    Instruction::Ifeq(13),
                    Instruction::Goto(3),
                    Instruction::Iload(1),
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
        self.pool.nullscan_ref = self
            .pool
            .constant_pool
            .add_method_ref(this_class, "nullscan", "([B)I")
            .unwrap();
    }

    fn build_fopen(
        &mut self,
        methods: &mut Vec<Method>,
        code_attr: u16,
        string_class: u16,
        str_init_3arg: u16,
        fis_class: u16,
        fis_init: u16,
        file_fds_ref: u16,
        file_next_ref: u16,
        _is_class: u16,
    ) {
        let n = self.pool.constant_pool.add_utf8("fopen").unwrap();
        let d = self.pool.constant_pool.add_utf8("([B[B)I").unwrap();
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: n,
            descriptor_index: d,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 5,
                max_locals: 4,
                code: vec![
                    Instruction::Aload_0,
                    Instruction::Invokestatic(self.pool.nullscan_ref),
                    Instruction::Istore(3),
                    Instruction::New(string_class),
                    Instruction::Dup,
                    Instruction::Aload_0,
                    Instruction::Iconst_0,
                    Instruction::Iload(3),
                    Instruction::Invokespecial(str_init_3arg),
                    Instruction::Astore(2),
                    Instruction::New(fis_class),
                    Instruction::Dup,
                    Instruction::Aload_2,
                    Instruction::Invokespecial(fis_init),
                    Instruction::Astore_2,
                    Instruction::Getstatic(file_fds_ref),
                    Instruction::Getstatic(file_next_ref),
                    Instruction::Aload_2,
                    Instruction::Aastore,
                    Instruction::Getstatic(file_next_ref),
                    Instruction::Dup,
                    Instruction::Iconst_1,
                    Instruction::Iadd,
                    Instruction::Putstatic(file_next_ref),
                    Instruction::Iconst_1,
                    Instruction::Iadd,
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_fgetc(
        &mut self,
        methods: &mut Vec<Method>,
        code_attr: u16,
        file_fds_ref: u16,
        is_class: u16,
        is_read: u16,
    ) {
        let n = self.pool.constant_pool.add_utf8("fgetc").unwrap();
        let d = self.pool.constant_pool.add_utf8("(I)I").unwrap();
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: n,
            descriptor_index: d,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 2,
                max_locals: 2,
                code: vec![
                    Instruction::Iload_0,
                    Instruction::Iconst_1,
                    Instruction::Isub,
                    Instruction::Istore(1),
                    Instruction::Getstatic(file_fds_ref),
                    Instruction::Iload(1),
                    Instruction::Aaload,
                    Instruction::Checkcast(is_class),
                    Instruction::Invokevirtual(is_read),
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_fclose(
        &mut self,
        methods: &mut Vec<Method>,
        code_attr: u16,
        file_fds_ref: u16,
        is_class: u16,
        is_close: u16,
    ) {
        let n = self.pool.constant_pool.add_utf8("fclose").unwrap();
        let d = self.pool.constant_pool.add_utf8("(I)I").unwrap();
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: n,
            descriptor_index: d,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 3,
                max_locals: 2,
                code: vec![
                    Instruction::Iload_0,
                    Instruction::Iconst_1,
                    Instruction::Isub,
                    Instruction::Istore(1),
                    Instruction::Getstatic(file_fds_ref),
                    Instruction::Iload(1),
                    Instruction::Aaload,
                    Instruction::Checkcast(is_class),
                    Instruction::Invokevirtual(is_close),
                    Instruction::Getstatic(file_fds_ref),
                    Instruction::Iload(1),
                    Instruction::Aconst_null,
                    Instruction::Aastore,
                    Instruction::Iconst_0,
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }
}

use crate::codegen::jvm::types::capitalize_first;
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::IrFunction;
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Field, FieldAccessFlags, FieldType, Method, MethodAccessFlags};

const COROUTINE_MAX_STACK: u16 = 4;
const COROUTINE_MAX_LOCALS: u16 = 3;

impl JvmGenerator {
    pub(super) fn generate_runtime_stub(&mut self, functions: &[IrFunction]) -> Vec<u8> {
        let this_class = self.pool.constant_pool.add_class("RuntimeStub")
            .expect("Failed to add RuntimeStub class");
        let super_class = self.pool.constant_pool.add_class("java/lang/Object")
            .expect("Failed to add Object class");
        let code_attr = self.pool.constant_pool.add_utf8("Code")
            .expect("Failed to add 'Code' UTF8");
        let coro_field_name = self.pool.constant_pool.add_utf8("coroutines")
            .expect("Failed to add 'coroutines' UTF8");
        let coro_field_desc = self.pool.constant_pool.add_utf8("[Ljava/lang/Object;")
            .expect("Failed to add coro field descriptor");
        let coro_field_ref = self.pool.constant_pool.add_field_ref(this_class, "coroutines", "[Ljava/lang/Object;")
            .expect("Failed to add coroutines field ref");

        let coro_info: Vec<(String, u16)> = functions.iter()
            .filter(|f| f.is_coroutine)
            .map(|f| {
                let name = if f.name == "main" { "Main".to_string() } else { capitalize_first(&f.name) };
                let class_idx = self.pool.constant_pool.add_class(&name)
                    .expect("Failed to add coroutine class");
                let _ = self.pool.constant_pool.add_method_ref(class_idx, "<init>", "()V")
                    .expect("Failed to add coroutine init ref");
                let _ = self.pool.constant_pool.add_method_ref(class_idx, "resume", "()I")
                    .expect("Failed to add coroutine resume ref");
                (name, class_idx)
            }).collect();
        let count = coro_info.len();
        let mut methods = Vec::new();

        if count > 0 {
            self.build_coro_init(&mut methods, &coro_info, count, code_attr, coro_field_ref);
            self.build_coro_resume(&mut methods, &coro_info, code_attr, coro_field_ref);
            self.build_coro_get_state(&mut methods, &coro_info, code_attr, coro_field_ref);
            self.build_coro_set_param(&mut methods, &coro_info, code_attr, coro_field_ref);
        }

        let putchar_name = self.pool.constant_pool.add_utf8("putchar")
            .expect("Failed to add 'putchar' UTF8");
        let putchar_desc = self.pool.constant_pool.add_utf8("(I)I")
            .expect("Failed to add putchar descriptor");
        let system_class = self.pool.constant_pool.add_class("java/lang/System")
            .expect("Failed to add System class");
        let system_out_ref = self.pool.constant_pool.add_field_ref(system_class, "out", "Ljava/io/PrintStream;")
            .expect("Failed to add System.out field ref");
        let printstream_class = self.pool.constant_pool.add_class("java/io/PrintStream")
            .expect("Failed to add PrintStream class");
        let print_char_ref = self.pool.constant_pool.add_method_ref(printstream_class, "print", "(C)V")
            .expect("Failed to add print method ref");
        let println_string_ref = self.pool.constant_pool.add_method_ref(printstream_class, "println", "(Ljava/lang/String;)V")
            .expect("Failed to add println method ref");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: putchar_name,
            descriptor_index: putchar_desc,
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

        let string_class = self.pool.constant_pool.add_class("java/lang/String")
            .expect("Failed to add String class");
        let string_byte_init = self.pool.constant_pool.add_method_ref(string_class, "<init>", "([B)V")
            .expect("Failed to add String(byte[]) init");

        let puts_name = self.pool.constant_pool.add_utf8("puts")
            .expect("Failed to add 'puts' UTF8");
        let puts_desc = self.pool.constant_pool.add_utf8("([B)I")
            .expect("Failed to add puts descriptor");
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: puts_name,
            descriptor_index: puts_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: 4,
                max_locals: 1,
                code: vec![
                    Instruction::Getstatic(system_out_ref),
                    Instruction::New(string_class),
                    Instruction::Dup,
                    Instruction::Aload_0,
                    Instruction::Invokespecial(string_byte_init),
                    Instruction::Invokevirtual(println_string_ref),
                    Instruction::Iconst_0,
                    Instruction::Ireturn,
                ],
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_5,
            constant_pool: self.pool.constant_pool.clone(),
            access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class,
            super_class,
            interfaces: vec![],
            fields: if count > 0 {
                vec![Field {
                    access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC,
                    name_index: coro_field_name,
                    descriptor_index: coro_field_desc,
                    field_type: FieldType::parse("[Ljava/lang/Object;")
                        .expect("Failed to parse coro field type"),
                    attributes: vec![],
                }]
            } else { vec![] },
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

    fn build_coro_init(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        count: usize,
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let init_name = self.pool.constant_pool.add_utf8("coro_init")
            .expect("Failed to add 'coro_init' UTF8");
        let init_desc = self.pool.constant_pool.add_utf8("()V")
            .expect("Failed to add coro_init descriptor");
        let mut c = Vec::new();
        push_iconst(&mut c, count);
        c.push(Instruction::Anewarray(
            self.pool.constant_pool.add_class("java/lang/Object")
                .expect("Failed to add Object class")
        ));
        c.push(Instruction::Putstatic(coro_field_ref));
        for (i, (_, ci)) in coro_info.iter().enumerate() {
            let ir = self.pool.constant_pool.add_method_ref(*ci, "<init>", "()V")
                .expect("Failed to add init ref");
            c.push(Instruction::Getstatic(coro_field_ref));
            push_iconst(&mut c, i);
            c.push(Instruction::New(*ci));
            c.push(Instruction::Dup);
            c.push(Instruction::Invokespecial(ir));
            c.push(Instruction::Aastore);
        }
        c.push(Instruction::Return);
        let max_init_stack = COROUTINE_MAX_STACK.max(2 + count as u16);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: init_name,
            descriptor_index: init_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: max_init_stack,
                max_locals: 0,
                code: c,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_coro_resume(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let resume_name = self.pool.constant_pool.add_utf8("resume_coroutine")
            .expect("Failed to add 'resume_coroutine' UTF8");
        let resume_desc = self.pool.constant_pool.add_utf8("(I)I")
            .expect("Failed to add resume descriptor");
        let mut code = Vec::new();
        for (i, (_, ci)) in coro_info.iter().enumerate() {
            code.push(Instruction::Iload_0);
            push_iconst(&mut code, i);
            let skip_at = code.len();
            code.push(Instruction::If_icmpne(0));
            code.push(Instruction::Getstatic(coro_field_ref));
            push_iconst(&mut code, i);
            code.push(Instruction::Aaload);
            code.push(Instruction::Checkcast(*ci));
            let rm = self.pool.constant_pool.add_method_ref(*ci, "resume", "()I")
                .expect("Failed to add resume method ref");
            code.push(Instruction::Invokevirtual(rm));
            code.push(Instruction::Ireturn);
            code[skip_at] = Instruction::If_icmpne(code.len() as u16);
        }
        code.push(Instruction::Iconst_1);
        code.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: resume_name,
            descriptor_index: resume_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: COROUTINE_MAX_STACK,
                max_locals: 1,
                code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_coro_get_state(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let state_name = self.pool.constant_pool.add_utf8("get_coroutine_state")
            .expect("Failed to add 'get_coroutine_state' UTF8");
        let state_desc = self.pool.constant_pool.add_utf8("(I)I")
            .expect("Failed to add get_state descriptor");
        let mut sc = Vec::new();
        for (i, (_, ci)) in coro_info.iter().enumerate() {
            sc.push(Instruction::Iload_0);
            push_iconst(&mut sc, i);
            let skip_at = sc.len();
            sc.push(Instruction::If_icmpne(0));
            sc.push(Instruction::Getstatic(coro_field_ref));
            push_iconst(&mut sc, i);
            sc.push(Instruction::Aaload);
            sc.push(Instruction::Checkcast(*ci));
            let sm = self.pool.constant_pool.add_method_ref(*ci, "getState", "()I")
                .expect("Failed to add getState method ref");
            sc.push(Instruction::Invokevirtual(sm));
            sc.push(Instruction::Ireturn);
            sc[skip_at] = Instruction::If_icmpne(sc.len() as u16);
        }
        sc.push(Instruction::Iconst_m1);
        sc.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: state_name,
            descriptor_index: state_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: COROUTINE_MAX_STACK,
                max_locals: 1,
                code: sc,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }

    fn build_coro_set_param(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let set_name = self.pool.constant_pool.add_utf8("set_coroutine_param")
            .expect("Failed to add 'set_coroutine_param' UTF8");
        let set_desc = self.pool.constant_pool.add_utf8("(III)V")
            .expect("Failed to add set_param descriptor");
        let mut set_code = Vec::new();
        for (i, (_, ci)) in coro_info.iter().enumerate() {
            set_code.push(Instruction::Iload_0);
            push_iconst(&mut set_code, i);
            let skip_at = set_code.len();
            set_code.push(Instruction::If_icmpne(0));
            set_code.push(Instruction::Getstatic(coro_field_ref));
            push_iconst(&mut set_code, i);
            set_code.push(Instruction::Aaload);
            set_code.push(Instruction::Checkcast(*ci));
            let (p1_ref, p2_ref) = self.coro.coroutine_param_field_refs[i];
            if let Some(fr) = p1_ref {
                set_code.push(Instruction::Dup);
                set_code.push(Instruction::Iload_1);
                set_code.push(Instruction::Putfield(fr));
            }
            if let Some(fr) = p2_ref {
                set_code.push(Instruction::Dup);
                set_code.push(Instruction::Iload_2);
                set_code.push(Instruction::Putfield(fr));
            }
            set_code.push(Instruction::Pop);
            set_code.push(Instruction::Return);
            set_code[skip_at] = Instruction::If_icmpne(set_code.len() as u16);
        }
        set_code.push(Instruction::Return);
        methods.push(Method {
            access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC,
            name_index: set_name,
            descriptor_index: set_desc,
            attributes: vec![Attribute::Code {
                name_index: code_attr,
                max_stack: COROUTINE_MAX_STACK,
                max_locals: COROUTINE_MAX_LOCALS,
                code: set_code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });
    }
}

fn push_iconst(c: &mut Vec<Instruction>, n: usize) {
    match n {
        0 => c.push(Instruction::Iconst_0),
        1 => c.push(Instruction::Iconst_1),
        2 => c.push(Instruction::Iconst_2),
        3 => c.push(Instruction::Iconst_3),
        4 => c.push(Instruction::Iconst_4),
        5 => c.push(Instruction::Iconst_5),
        _ => c.push(Instruction::Bipush(n as i8)),
    }
}

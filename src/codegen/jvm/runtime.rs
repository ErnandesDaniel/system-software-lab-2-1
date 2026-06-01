use crate::codegen::jvm::types::capitalize_first;
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::IrFunction;
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Field, FieldAccessFlags, FieldType, Method, MethodAccessFlags};

impl JvmGenerator {
    pub(super) fn generate_runtime_stub(&mut self, functions: &[IrFunction]) -> Vec<u8> {
        let class_name = "RuntimeStub";
        let this_class = self.constant_pool.add_class(class_name).unwrap();
        let super_class = self.constant_pool.add_class("java/lang/Object").unwrap();
        let code_attr = self.constant_pool.add_utf8("Code").unwrap();
        let coro_field_name = self.constant_pool.add_utf8("coroutines").unwrap();
        let coro_field_desc = self.constant_pool.add_utf8("[Ljava/lang/Object;").unwrap();
        let obj_class = self.constant_pool.add_class("java/lang/Object").unwrap();
        let coro_field_ref = self.constant_pool.add_field_ref(this_class, "coroutines", "[Ljava/lang/Object;").unwrap();

        let coro_info: Vec<(String, u16)> = functions.iter().filter(|f| f.is_coroutine).map(|f| {
            let name = if f.name == "main" { "Main".to_string() } else { capitalize_first(&f.name) };
            let class_idx = self.constant_pool.add_class(&name).unwrap();
            let _ = self.constant_pool.add_method_ref(class_idx, "<init>", "()V").unwrap();
            let _ = self.constant_pool.add_method_ref(class_idx, "resume", "()I").unwrap();
            (name, class_idx)
        }).collect();
        let count = coro_info.len();
        let mut methods = Vec::new();

        if count > 0 {
            let init_name = self.constant_pool.add_utf8("coro_init").unwrap();
            let init_desc = self.constant_pool.add_utf8("()V").unwrap();
            let mut c = Vec::new();
            push_iconst(&mut c, count);
            c.push(Instruction::Anewarray(obj_class));
            c.push(Instruction::Putstatic(coro_field_ref));
            for (i, (_, ci)) in coro_info.iter().enumerate() {
                let ir = self.constant_pool.add_method_ref(*ci, "<init>", "()V").unwrap();
                c.push(Instruction::Getstatic(coro_field_ref));
                push_iconst(&mut c, i);
                c.push(Instruction::New(*ci)); c.push(Instruction::Dup); c.push(Instruction::Invokespecial(ir));
                c.push(Instruction::Aastore);
            }
            c.push(Instruction::Return);
            methods.push(Method { access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC, name_index: init_name, descriptor_index: init_desc, attributes: vec![Attribute::Code { name_index: code_attr, max_stack: 4.max(2 + count as u16), max_locals: 0, code: c, exception_table: vec![], attributes: vec![] }] });

            let resume_name = self.constant_pool.add_utf8("resume_coroutine").unwrap();
            let resume_desc = self.constant_pool.add_utf8("(I)I").unwrap();
            let mut code = Vec::new();
            for (i, (_, ci)) in coro_info.iter().enumerate() {
                code.push(Instruction::Iload_0);
                push_iconst(&mut code, i);
                let skip_at = code.len(); code.push(Instruction::If_icmpne(0));
                code.push(Instruction::Getstatic(coro_field_ref));
                push_iconst(&mut code, i);
                code.push(Instruction::Aaload);
                code.push(Instruction::Checkcast(*ci));
                let rm = self.constant_pool.add_method_ref(*ci, "resume", "()I").unwrap();
                code.push(Instruction::Invokevirtual(rm));
                code.push(Instruction::Ireturn);
                code[skip_at] = Instruction::If_icmpne((code.len() - skip_at - 1) as u16);
            }
            code.push(Instruction::Iconst_1); code.push(Instruction::Ireturn);
            methods.push(Method { access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC, name_index: resume_name, descriptor_index: resume_desc, attributes: vec![Attribute::Code { name_index: code_attr, max_stack: 4, max_locals: 1, code, exception_table: vec![], attributes: vec![] }] });

            let state_name = self.constant_pool.add_utf8("get_coroutine_state").unwrap();
            let state_desc = self.constant_pool.add_utf8("(I)I").unwrap();
            let mut sc = Vec::new();
            for (i, (_, ci)) in coro_info.iter().enumerate() {
                sc.push(Instruction::Iload_0);
                push_iconst(&mut sc, i);
                let skip_at = sc.len(); sc.push(Instruction::If_icmpne(0));
                sc.push(Instruction::Getstatic(coro_field_ref));
                push_iconst(&mut sc, i);
                sc.push(Instruction::Aaload);
                sc.push(Instruction::Checkcast(*ci));
                let sm = self.constant_pool.add_method_ref(*ci, "getState", "()I").unwrap();
                sc.push(Instruction::Invokevirtual(sm));
                sc.push(Instruction::Ireturn);
                sc[skip_at] = Instruction::If_icmpne((sc.len() - skip_at - 1) as u16);
            }
            sc.push(Instruction::Iconst_m1); sc.push(Instruction::Ireturn);
            methods.push(Method { access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC, name_index: state_name, descriptor_index: state_desc, attributes: vec![Attribute::Code { name_index: code_attr, max_stack: 4, max_locals: 1, code: sc, exception_table: vec![], attributes: vec![] }] });

            let set_name = self.constant_pool.add_utf8("set_coroutine_param").unwrap();
            let set_desc = self.constant_pool.add_utf8("(III)V").unwrap();
            let mut set_code = Vec::new();
            for (i, (_, ci)) in coro_info.iter().enumerate() {
                set_code.push(Instruction::Iload_0);
                push_iconst(&mut set_code, i);
                let skip_at = set_code.len(); set_code.push(Instruction::If_icmpne(0));
                set_code.push(Instruction::Getstatic(coro_field_ref));
                push_iconst(&mut set_code, i);
                set_code.push(Instruction::Aaload);
                set_code.push(Instruction::Checkcast(*ci));
                let (p1_ref, p2_ref) = self.coroutine_param_field_refs[i];
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
                set_code[skip_at] = Instruction::If_icmpne((set_code.len() - skip_at - 1) as u16);
            }
            set_code.push(Instruction::Return);
            methods.push(Method { access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC, name_index: set_name, descriptor_index: set_desc, attributes: vec![Attribute::Code { name_index: code_attr, max_stack: 4, max_locals: 3, code: set_code, exception_table: vec![], attributes: vec![] }] });
        }

        fn push_iconst(c: &mut Vec<Instruction>, n: usize) {
            match n { 0 => c.push(Instruction::Iconst_0), 1 => c.push(Instruction::Iconst_1), 2 => c.push(Instruction::Iconst_2), 3 => c.push(Instruction::Iconst_3), 4 => c.push(Instruction::Iconst_4), 5 => c.push(Instruction::Iconst_5), _ => c.push(Instruction::Bipush(n as i8)) }
        }

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_5, constant_pool: self.constant_pool.clone(), access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class, super_class, interfaces: vec![],
            fields: if count > 0 { vec![Field { access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC, name_index: coro_field_name, descriptor_index: coro_field_desc, field_type: FieldType::parse("[Ljava/lang/Object;").unwrap(), attributes: vec![] }] } else { vec![] },
            methods, attributes: vec![], code_source_url: None,
        };
        let mut buf = Vec::new();
        match class_file.to_bytes(&mut buf) { Ok(()) => buf, Err(e) => panic!("RuntimeStub: {e:?}"), }
    }
}

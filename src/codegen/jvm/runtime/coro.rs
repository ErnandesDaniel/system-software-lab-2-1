use crate::codegen::jvm::JvmGenerator;
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::Method;

impl JvmGenerator {
    pub(super) fn build_coro_init(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        count: usize,
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let init_name = self.pool.constant_pool.add_utf8("coro_init")
            .expect("init utf8");
        let init_desc = self.pool.constant_pool.add_utf8("()V")
            .expect("init desc");
        let mut c = Vec::new();
        push_iconst(&mut c, count);
        c.push(Instruction::Anewarray(
            self.pool.constant_pool.add_class("java/lang/Object")
                .expect("obj class")
        ));
        c.push(Instruction::Putstatic(coro_field_ref));
        for (i, (_, ci)) in coro_info.iter().enumerate() {
            let ir = self.pool.constant_pool.add_method_ref(*ci, "<init>", "()V")
                .expect("init ref");
            c.push(Instruction::Getstatic(coro_field_ref));
            push_iconst(&mut c, i);
            c.push(Instruction::New(*ci));
            c.push(Instruction::Dup);
            c.push(Instruction::Invokespecial(ir));
            c.push(Instruction::Aastore);
        }
        c.push(Instruction::Return);
        let max_init_stack = (COROUTINE_MAX_STACK).max(2 + count as u16);
        methods.push(Method {
            access_flags: ristretto_classfile::MethodAccessFlags::PUBLIC
                | ristretto_classfile::MethodAccessFlags::STATIC,
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

    pub(super) fn build_coro_resume(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let resume_name = self.pool.constant_pool.add_utf8("resume_coroutine")
            .expect("resume utf8");
        let resume_desc = self.pool.constant_pool.add_utf8("(I)I")
            .expect("resume desc");
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
                .expect("resume ref");
            code.push(Instruction::Invokevirtual(rm));
            code.push(Instruction::Ireturn);
            code[skip_at] = Instruction::If_icmpne(code.len() as u16);
        }
        code.push(Instruction::Iconst_1);
        code.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: ristretto_classfile::MethodAccessFlags::PUBLIC
                | ristretto_classfile::MethodAccessFlags::STATIC,
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

    pub(super) fn build_coro_get_state(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let state_name = self.pool.constant_pool.add_utf8("get_coroutine_state")
            .expect("state utf8");
        let state_desc = self.pool.constant_pool.add_utf8("(I)I")
            .expect("state desc");
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
                .expect("getState ref");
            sc.push(Instruction::Invokevirtual(sm));
            sc.push(Instruction::Ireturn);
            sc[skip_at] = Instruction::If_icmpne(sc.len() as u16);
        }
        sc.push(Instruction::Iconst_m1);
        sc.push(Instruction::Ireturn);
        methods.push(Method {
            access_flags: ristretto_classfile::MethodAccessFlags::PUBLIC
                | ristretto_classfile::MethodAccessFlags::STATIC,
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

    pub(super) fn build_coro_set_param(
        &mut self,
        methods: &mut Vec<Method>,
        coro_info: &[(String, u16)],
        code_attr: u16,
        coro_field_ref: u16,
    ) {
        let set_name = self.pool.constant_pool.add_utf8("set_coroutine_param")
            .expect("set utf8");
        let set_desc = self.pool.constant_pool.add_utf8("(III)V")
            .expect("set desc");
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
            access_flags: ristretto_classfile::MethodAccessFlags::PUBLIC
                | ristretto_classfile::MethodAccessFlags::STATIC,
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

const COROUTINE_MAX_STACK: u16 = 4;
const COROUTINE_MAX_LOCALS: u16 = 3;

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

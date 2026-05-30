use crate::codegen::jvm::types::{capitalize_first, get_fn_interface_name, ir_type_to_jvm_descriptor};
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{IrFunction, IrType};
use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Method, MethodAccessFlags};

impl JvmGenerator {
    pub fn build_class_file(
        &mut self,
        class_name: &str,
        func: &IrFunction,
        code: Vec<ristretto_classfile::attributes::Instruction>,
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

            let call_ref_idx = self
                .constant_pool
                .add_method_ref(this_class, "call", &call_desc)
                .unwrap();

            let system_class = self.constant_pool.add_class("java/lang/System").unwrap();
            let exit_ref_idx = self.constant_pool.add_method_ref(system_class, "exit", "(I)V").unwrap();

            let main_code = vec![
                Instruction::Invokestatic(call_ref_idx),
                Instruction::Invokestatic(exit_ref_idx),
                Instruction::Return,
            ];

            let main_code_attr = Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack: 2,
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
                code: code.clone(),
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

            if is_func_ref_target {
                // Default constructor: public <init>()V
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
                // Instance apply method that delegates to static call (for functional interface dispatch)
                let apply_name_idx = self.constant_pool.add_utf8("apply").unwrap();
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
                let instance_call_desc_idx = self.constant_pool.add_utf8(&instance_call_desc).unwrap();
                let static_call_ref = self
                    .constant_pool
                    .add_method_ref(this_class, "call", &call_desc)
                    .unwrap();

                let mut instance_call_code = Vec::new();

                // For closures, load this.__env first
                if has_env_param {
                    instance_call_code.push(Instruction::Aload_0);
                    let env_field_ref = self.constant_pool.add_field_ref(this_class, "__env", "[[I").unwrap();
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
            let user_params: Vec<IrType> = func
                .parameters
                .iter()
                .filter(|p| p.name != "__env")
                .map(|p| p.ty.clone())
                .collect();
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
            version: ristretto_classfile::JAVA_6,
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
                panic!(
                    "Failed to serialize class file: {:?}. max_stack={}, max_locals={}, code_len={}",
                    e,
                    max_stack,
                    max_locals,
                    code.len()
                );
            }
        }
    }

    fn build_coroutine_class(
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

        // --- Default constructor: <init>()V ---
        let init_name_idx = self.constant_pool.add_utf8("<init>").unwrap();
        let init_desc_idx = self.constant_pool.add_utf8("()V").unwrap();
        let obj_init_ref = self
            .constant_pool
            .add_method_ref(super_class, "<init>", "()V")
            .unwrap();
        let init_code = vec![
            Instruction::Aload_0,
            Instruction::Invokespecial(obj_init_ref),
            Instruction::Return,
        ];
        methods.push(ristretto_classfile::Method {
            access_flags: ristretto_classfile::MethodAccessFlags::PUBLIC,
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

        // --- resume()I method ---
        let resume_name_idx = self.constant_pool.add_utf8("resume").unwrap();
        let resume_desc_idx = self.constant_pool.add_utf8("()I").unwrap();
        let resume_code_attr = Attribute::Code {
            name_index: code_attr_name_idx,
            max_stack,
            max_locals: 1,
            code,
            exception_table: vec![],
            attributes: vec![],
        };
        methods.push(ristretto_classfile::Method {
            access_flags: ristretto_classfile::MethodAccessFlags::PUBLIC,
            name_index: resume_name_idx,
            descriptor_index: resume_desc_idx,
            attributes: vec![resume_code_attr],
        });

        // --- getState()I method ---
        let get_state_name_idx = self.constant_pool.add_utf8("getState").unwrap();
        let get_state_desc_idx = self.constant_pool.add_utf8("()I").unwrap();
        let get_state_code = vec![
            Instruction::Aload_0,
            Instruction::Getfield(self.coroutine_state_field),
            Instruction::Ireturn,
        ];
        methods.push(ristretto_classfile::Method {
            access_flags: ristretto_classfile::MethodAccessFlags::PUBLIC,
            name_index: get_state_name_idx,
            descriptor_index: get_state_desc_idx,
            attributes: vec![Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack: 1,
                max_locals: 1,
                code: get_state_code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // --- getResult()I method ---
        let get_result_name_idx = self.constant_pool.add_utf8("getResult").unwrap();
        let get_result_desc_idx = self.constant_pool.add_utf8("()I").unwrap();
        let get_result_code = vec![
            Instruction::Aload_0,
            Instruction::Getfield(self.coroutine_result_field),
            Instruction::Ireturn,
        ];
        methods.push(ristretto_classfile::Method {
            access_flags: ristretto_classfile::MethodAccessFlags::PUBLIC,
            name_index: get_result_name_idx,
            descriptor_index: get_result_desc_idx,
            attributes: vec![Attribute::Code {
                name_index: code_attr_name_idx,
                max_stack: 1,
                max_locals: 1,
                code: get_result_code,
                exception_table: vec![],
                attributes: vec![],
            }],
        });

        // Build fields list
        let fields: Vec<ristretto_classfile::Field> = self
            .coroutine_field_entries
            .iter()
            .map(|&(name_idx, desc_idx)| ristretto_classfile::Field {
                access_flags: ristretto_classfile::FieldAccessFlags::PUBLIC,
                name_index: name_idx,
                descriptor_index: desc_idx,
                field_type: ristretto_classfile::FieldType::parse("I").unwrap(),
                attributes: vec![],
            })
            .collect();

        let class_file = ristretto_classfile::ClassFile {
            version: ristretto_classfile::JAVA_6,
            constant_pool: self.constant_pool.clone(),
            access_flags: ristretto_classfile::ClassAccessFlags::PUBLIC
                | ristretto_classfile::ClassAccessFlags::SUPER,
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
            Err(e) => {
                panic!(
                    "Failed to serialize coroutine class file: {e:?}. max_stack={max_stack}, max_locals=1, code_len={code_len}"
                );
            }
        }
    }

    pub fn generate_fn_interface(&mut self, params: &[IrType], ret: &IrType) -> Vec<u8> {
        let iface_name = get_fn_interface_name(params, ret);
        let this_class = self.constant_pool.add_class(&iface_name).unwrap();
        let super_class = self.constant_pool.add_class("java/lang/Object").unwrap();

        let method_desc = format!(
            "({}){}",
            params.iter().map(ir_type_to_jvm_descriptor).collect::<String>(),
            ir_type_to_jvm_descriptor(ret)
        );
        let method_name_idx = self.constant_pool.add_utf8("apply").unwrap();
        let method_desc_idx = self.constant_pool.add_utf8(&method_desc).unwrap();

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_6,
            constant_pool: self.constant_pool.clone(),
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

    fn estimate_max_stack(&self, code: &[Instruction]) -> u16 {
        let mut max_depth = 0u16;
        let mut depth = 0i32;
        for instr in code {
            let stack_delta = self.instr_stack_delta(instr);
            depth += stack_delta;
            if depth > i32::from(max_depth) {
                max_depth = depth as u16;
            }
        }
        max_depth.max(4)
    }

    fn instr_stack_delta(&self, instr: &Instruction) -> i32 {
        match instr {
            Instruction::Nop | Instruction::Iinc(_, _) => 0,
            Instruction::Aconst_null => 1,
            Instruction::Iconst_m1
            | Instruction::Iconst_0
            | Instruction::Iconst_1
            | Instruction::Iconst_2
            | Instruction::Iconst_3
            | Instruction::Iconst_4
            | Instruction::Iconst_5 => 1,
            Instruction::Lconst_0 | Instruction::Lconst_1 => 2,
            Instruction::Fconst_0 | Instruction::Fconst_1 | Instruction::Fconst_2 => 1,
            Instruction::Dconst_0 | Instruction::Dconst_1 => 2,
            Instruction::Bipush(_) => 1,
            Instruction::Sipush(_) => 1,
            Instruction::Ldc(_) | Instruction::Ldc_w(_) | Instruction::Ldc2_w(_) => 1,
            Instruction::Iload(_)
            | Instruction::Iload_0
            | Instruction::Iload_1
            | Instruction::Iload_2
            | Instruction::Iload_3 => 1,
            Instruction::Lload(_)
            | Instruction::Lload_0
            | Instruction::Lload_1
            | Instruction::Lload_2
            | Instruction::Lload_3 => 2,
            Instruction::Fload(_)
            | Instruction::Fload_0
            | Instruction::Fload_1
            | Instruction::Fload_2
            | Instruction::Fload_3 => 1,
            Instruction::Dload(_)
            | Instruction::Dload_0
            | Instruction::Dload_1
            | Instruction::Dload_2
            | Instruction::Dload_3 => 2,
            Instruction::Aload(_)
            | Instruction::Aload_0
            | Instruction::Aload_1
            | Instruction::Aload_2
            | Instruction::Aload_3 => 1,
            Instruction::Istore(_)
            | Instruction::Istore_0
            | Instruction::Istore_1
            | Instruction::Istore_2
            | Instruction::Istore_3 => -1,
            Instruction::Lstore(_)
            | Instruction::Lstore_0
            | Instruction::Lstore_1
            | Instruction::Lstore_2
            | Instruction::Lstore_3 => -2,
            Instruction::Fstore(_)
            | Instruction::Fstore_0
            | Instruction::Fstore_1
            | Instruction::Fstore_2
            | Instruction::Fstore_3 => -1,
            Instruction::Dstore(_)
            | Instruction::Dstore_0
            | Instruction::Dstore_1
            | Instruction::Dstore_2
            | Instruction::Dstore_3 => -2,
            Instruction::Astore(_)
            | Instruction::Astore_0
            | Instruction::Astore_1
            | Instruction::Astore_2
            | Instruction::Astore_3 => -1,
            Instruction::Pop => -1,
            Instruction::Pop2 => -2,
            Instruction::Dup | Instruction::Dup_x1 | Instruction::Dup_x2 => 1,
            Instruction::Dup2 | Instruction::Dup2_x1 | Instruction::Dup2_x2 => 2,
            Instruction::Swap => 0,
            Instruction::Iadd
            | Instruction::Ladd
            | Instruction::Fadd
            | Instruction::Dadd
            | Instruction::Isub
            | Instruction::Lsub
            | Instruction::Fsub
            | Instruction::Dsub
            | Instruction::Imul
            | Instruction::Lmul
            | Instruction::Fmul
            | Instruction::Dmul
            | Instruction::Idiv
            | Instruction::Ldiv
            | Instruction::Fdiv
            | Instruction::Ddiv
            | Instruction::Irem
            | Instruction::Lrem
            | Instruction::Frem
            | Instruction::Drem
            | Instruction::Ishl
            | Instruction::Lshl
            | Instruction::Ishr
            | Instruction::Lshr
            | Instruction::Iushr
            | Instruction::Lushr
            | Instruction::Iand
            | Instruction::Land
            | Instruction::Ior
            | Instruction::Lor
            | Instruction::Ixor
            | Instruction::Lxor => -1, // pop 2, push 1
            Instruction::Ineg | Instruction::Lneg | Instruction::Fneg | Instruction::Dneg => 0, // pop 1, push 1
            Instruction::I2l
            | Instruction::I2f
            | Instruction::I2d
            | Instruction::L2i
            | Instruction::L2f
            | Instruction::L2d
            | Instruction::F2i
            | Instruction::F2l
            | Instruction::F2d
            | Instruction::D2i
            | Instruction::D2l
            | Instruction::D2f
            | Instruction::I2b
            | Instruction::I2c
            | Instruction::I2s => 0,
            Instruction::Lcmp | Instruction::Fcmpl | Instruction::Fcmpg | Instruction::Dcmpl | Instruction::Dcmpg => -1,
            Instruction::Ifeq(_)
            | Instruction::Ifne(_)
            | Instruction::Iflt(_)
            | Instruction::Ifge(_)
            | Instruction::Ifgt(_)
            | Instruction::Ifle(_) => -1,
            Instruction::If_icmpeq(_)
            | Instruction::If_icmpne(_)
            | Instruction::If_icmplt(_)
            | Instruction::If_icmpge(_)
            | Instruction::If_icmpgt(_)
            | Instruction::If_icmple(_) => -2,
            Instruction::If_acmpeq(_) | Instruction::If_acmpne(_) => -2,
            Instruction::Goto(_) | Instruction::Jsr(_) => 0,
            Instruction::Ret(_) => 0,
            Instruction::Tableswitch { .. } | Instruction::Lookupswitch { .. } => -1,
            Instruction::Ireturn
            | Instruction::Lreturn
            | Instruction::Freturn
            | Instruction::Dreturn
            | Instruction::Areturn
            | Instruction::Return => {
                // Return clears the stack — but for max depth calculation, just use a large negative
                -999
            }
            Instruction::Getstatic(_) => 1,
            Instruction::Putstatic(_) => -1,
            Instruction::Getfield(_) => 0, // pops ref, pushes value
            Instruction::Putfield(_) => -2,
            Instruction::Invokevirtual(_) | Instruction::Invokespecial(_) => -1,
            Instruction::Invokestatic(_) => {
                // net = return_value - args. Estimate as -1 (1 ret, 2 args average)
                -1
            }
            Instruction::Invokeinterface(_, _) => -1,
            Instruction::Invokedynamic(_) => -1,
            Instruction::New(_) => 1,
            Instruction::Newarray(_) => 0,  // pops count, pushes ref
            Instruction::Anewarray(_) => 0, // pops count, pushes ref
            Instruction::Arraylength => 0,  // pops ref, pushes int
            Instruction::Athrow => -1,
            Instruction::Checkcast(_) | Instruction::Instanceof(_) => 0,
            Instruction::Monitorenter | Instruction::Monitorexit => -1,
            Instruction::Wide => 0,
            Instruction::Ifnull(_) | Instruction::Ifnonnull(_) => -1,
            Instruction::Goto_w(_) | Instruction::Jsr_w(_) => 0,
            Instruction::Iaload
            | Instruction::Laload
            | Instruction::Faload
            | Instruction::Daload
            | Instruction::Aaload
            | Instruction::Baload
            | Instruction::Caload
            | Instruction::Saload => -1,
            Instruction::Iastore
            | Instruction::Lastore
            | Instruction::Fastore
            | Instruction::Dastore
            | Instruction::Aastore
            | Instruction::Bastore
            | Instruction::Castore
            | Instruction::Sastore => -3,
            _ => 0,
        }
    }

    pub fn instr_size(&self, instr: &Instruction) -> usize {
        match instr {
            Instruction::Nop => 1,
            Instruction::Aconst_null => 1,
            Instruction::Iconst_m1
            | Instruction::Iconst_0
            | Instruction::Iconst_1
            | Instruction::Iconst_2
            | Instruction::Iconst_3
            | Instruction::Iconst_4
            | Instruction::Iconst_5 => 1,
            Instruction::Lconst_0 | Instruction::Lconst_1 => 1,
            Instruction::Fconst_0 | Instruction::Fconst_1 | Instruction::Fconst_2 => 1,
            Instruction::Dconst_0 | Instruction::Dconst_1 => 1,
            Instruction::Bipush(_) => 2,
            Instruction::Sipush(_) => 3,
            Instruction::Ldc(_) => 2,
            Instruction::Ldc_w(_) | Instruction::Ldc2_w(_) => 3,
            Instruction::Iload_0 | Instruction::Iload_1 | Instruction::Iload_2 | Instruction::Iload_3 => 1,
            Instruction::Lload_0 | Instruction::Lload_1 | Instruction::Lload_2 | Instruction::Lload_3 => 1,
            Instruction::Fload_0 | Instruction::Fload_1 | Instruction::Fload_2 | Instruction::Fload_3 => 1,
            Instruction::Dload_0 | Instruction::Dload_1 | Instruction::Dload_2 | Instruction::Dload_3 => 1,
            Instruction::Aload_0 | Instruction::Aload_1 | Instruction::Aload_2 | Instruction::Aload_3 => 1,
            Instruction::Iload(_)
            | Instruction::Lload(_)
            | Instruction::Fload(_)
            | Instruction::Dload(_)
            | Instruction::Aload(_) => 2,
            Instruction::Istore_0 | Instruction::Istore_1 | Instruction::Istore_2 | Instruction::Istore_3 => 1,
            Instruction::Lstore_0 | Instruction::Lstore_1 | Instruction::Lstore_2 | Instruction::Lstore_3 => 1,
            Instruction::Fstore_0 | Instruction::Fstore_1 | Instruction::Fstore_2 | Instruction::Fstore_3 => 1,
            Instruction::Dstore_0 | Instruction::Dstore_1 | Instruction::Dstore_2 | Instruction::Dstore_3 => 1,
            Instruction::Astore_0 | Instruction::Astore_1 | Instruction::Astore_2 | Instruction::Astore_3 => 1,
            Instruction::Istore(_)
            | Instruction::Lstore(_)
            | Instruction::Fstore(_)
            | Instruction::Dstore(_)
            | Instruction::Astore(_) => 2,
            Instruction::Pop | Instruction::Pop2 => 1,
            Instruction::Dup
            | Instruction::Dup_x1
            | Instruction::Dup_x2
            | Instruction::Dup2
            | Instruction::Dup2_x1
            | Instruction::Dup2_x2 => 1,
            Instruction::Swap => 1,
            Instruction::Iadd | Instruction::Ladd | Instruction::Fadd | Instruction::Dadd => 1,
            Instruction::Isub | Instruction::Lsub | Instruction::Fsub | Instruction::Dsub => 1,
            Instruction::Imul | Instruction::Lmul | Instruction::Fmul | Instruction::Dmul => 1,
            Instruction::Idiv | Instruction::Ldiv | Instruction::Fdiv | Instruction::Ddiv => 1,
            Instruction::Irem | Instruction::Lrem | Instruction::Frem | Instruction::Drem => 1,
            Instruction::Ineg | Instruction::Lneg | Instruction::Fneg | Instruction::Dneg => 1,
            Instruction::Ishl | Instruction::Lshl => 1,
            Instruction::Ishr | Instruction::Lshr => 1,
            Instruction::Iushr | Instruction::Lushr => 1,
            Instruction::Iand | Instruction::Land => 1,
            Instruction::Ior | Instruction::Lor => 1,
            Instruction::Ixor | Instruction::Lxor => 1,
            Instruction::Iinc(_, _) => 3,
            Instruction::I2l
            | Instruction::I2f
            | Instruction::I2d
            | Instruction::L2i
            | Instruction::L2f
            | Instruction::L2d
            | Instruction::F2i
            | Instruction::F2l
            | Instruction::F2d
            | Instruction::D2i
            | Instruction::D2l
            | Instruction::D2f => 1,
            Instruction::I2b | Instruction::I2c | Instruction::I2s => 1,
            Instruction::Lcmp => 1,
            Instruction::Fcmpl | Instruction::Fcmpg | Instruction::Dcmpl | Instruction::Dcmpg => 1,
            Instruction::Ifeq(_)
            | Instruction::Ifne(_)
            | Instruction::Iflt(_)
            | Instruction::Ifge(_)
            | Instruction::Ifgt(_)
            | Instruction::Ifle(_) => 3,
            Instruction::If_icmpeq(_)
            | Instruction::If_icmpne(_)
            | Instruction::If_icmplt(_)
            | Instruction::If_icmpge(_)
            | Instruction::If_icmpgt(_)
            | Instruction::If_icmple(_) => 3,
            Instruction::If_acmpeq(_) | Instruction::If_acmpne(_) => 3,
            Instruction::Goto(_) => 3,
            Instruction::Jsr(_) => 3,
            Instruction::Ret(_) => 2,
            Instruction::Tableswitch { .. } => 1,
            Instruction::Lookupswitch { .. } => 1,
            Instruction::Ireturn
            | Instruction::Lreturn
            | Instruction::Freturn
            | Instruction::Dreturn
            | Instruction::Areturn
            | Instruction::Return => 1,
            Instruction::Getstatic(_) => 3,
            Instruction::Putstatic(_) => 3,
            Instruction::Getfield(_) => 3,
            Instruction::Putfield(_) => 3,
            Instruction::Invokevirtual(_) | Instruction::Invokespecial(_) | Instruction::Invokestatic(_) => 3,
            Instruction::Invokeinterface(_, _) => 5,
            Instruction::Invokedynamic(_) => 5,
            Instruction::New(_) => 3,
            Instruction::Newarray(_) => 2,
            Instruction::Anewarray(_) => 3,
            Instruction::Arraylength => 1,
            Instruction::Athrow => 1,
            Instruction::Checkcast(_) => 3,
            Instruction::Instanceof(_) => 3,
            Instruction::Monitorenter | Instruction::Monitorexit => 1,
            Instruction::Wide => 1,
            Instruction::Ifnull(_) | Instruction::Ifnonnull(_) => 3,
            Instruction::Goto_w(_) | Instruction::Jsr_w(_) => 5,
            _ => 1,
        }
    }

    pub fn generate_runtime_stub(&mut self, functions: &[IrFunction]) -> Vec<u8> {
        use ristretto_classfile::{Field, FieldAccessFlags, FieldType};
        let class_name = "RuntimeStub";
        let this_class = self.constant_pool.add_class(class_name).unwrap();
        let super_class = self.constant_pool.add_class("java/lang/Object").unwrap();
        let code_attr = self.constant_pool.add_utf8("Code").unwrap();
        let coro_field_name = self.constant_pool.add_utf8("coroutines").unwrap();
        let coro_field_desc = self.constant_pool.add_utf8("[Ljava/lang/Object;").unwrap();
        let obj_class = self.constant_pool.add_class("java/lang/Object").unwrap();
        let coro_field_ref = self.constant_pool.add_field_ref(this_class, "coroutines", "[Ljava/lang/Object;").unwrap();

        // Pre-register all coroutine class indices
        let coro_info: Vec<(String, u16)> = functions.iter().filter(|f| f.is_coroutine).map(|f| {
            let name = if f.name == "main" { "Main".to_string() } else { capitalize_first(&f.name) };
            let class_idx = self.constant_pool.add_class(&name).unwrap();
            let init_ref = self.constant_pool.add_method_ref(class_idx, "<init>", "()V").unwrap();
            let resume_ref = self.constant_pool.add_method_ref(class_idx, "resume", "()I").unwrap();
            (name, class_idx)
        }).collect();
        let count = coro_info.len();
        let mut methods = Vec::new();

        if count > 0 {
            // coro_init()V
            let init_name = self.constant_pool.add_utf8("coro_init").unwrap();
            let init_desc = self.constant_pool.add_utf8("()V").unwrap();
            let mut c = Vec::new();
            push_iconst(&mut c, count);
            c.push(Instruction::Anewarray(obj_class));
            c.push(Instruction::Putstatic(coro_field_ref));
            for (i, (cn, ci)) in coro_info.iter().enumerate() {
                let ir = self.constant_pool.add_method_ref(*ci, "<init>", "()V").unwrap();
                c.push(Instruction::Getstatic(coro_field_ref));
                push_iconst(&mut c, i);
                c.push(Instruction::New(*ci)); c.push(Instruction::Dup); c.push(Instruction::Invokespecial(ir));
                c.push(Instruction::Aastore);
            }
            c.push(Instruction::Return);
            methods.push(Method { access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC, name_index: init_name, descriptor_index: init_desc, attributes: vec![Attribute::Code { name_index: code_attr, max_stack: 4.max(2 + count as u16), max_locals: 0, code: c, exception_table: vec![], attributes: vec![] }] });

            // resume_coroutine(I)I
            let resume_name = self.constant_pool.add_utf8("resume_coroutine").unwrap();
            let resume_desc = self.constant_pool.add_utf8("(I)I").unwrap();
            let mut code = Vec::new();
            for (i, (cn, ci)) in coro_info.iter().enumerate() {
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
                let off = (code.len() - skip_at - 1) as u16;
                code[skip_at] = Instruction::If_icmpne(off);
            }
            code.push(Instruction::Iconst_1); code.push(Instruction::Ireturn);
            methods.push(Method { access_flags: MethodAccessFlags::PUBLIC | MethodAccessFlags::STATIC, name_index: resume_name, descriptor_index: resume_desc, attributes: vec![Attribute::Code { name_index: code_attr, max_stack: 4, max_locals: 1, code, exception_table: vec![], attributes: vec![] }] });
        }

        fn push_iconst(c: &mut Vec<Instruction>, n: usize) {
            match n { 0 => c.push(Instruction::Iconst_0), 1 => c.push(Instruction::Iconst_1), 2 => c.push(Instruction::Iconst_2), 3 => c.push(Instruction::Iconst_3), 4 => c.push(Instruction::Iconst_4), 5 => c.push(Instruction::Iconst_5), _ => c.push(Instruction::Bipush(n as i8)) }
        }

        let class_file = ClassFile {
            version: ristretto_classfile::JAVA_6, constant_pool: self.constant_pool.clone(), access_flags: ClassAccessFlags::PUBLIC | ClassAccessFlags::SUPER,
            this_class, super_class, interfaces: vec![],
            fields: if count > 0 { vec![Field { access_flags: FieldAccessFlags::PUBLIC | FieldAccessFlags::STATIC, name_index: coro_field_name, descriptor_index: coro_field_desc, field_type: FieldType::parse("[Ljava/lang/Object;").unwrap(), attributes: vec![] }] } else { vec![] },
            methods, attributes: vec![], code_source_url: None,
        };
        let mut buf = Vec::new();
        match class_file.to_bytes(&mut buf) { Ok(()) => buf, Err(e) => panic!("RuntimeStub: {e:?}"), }
    }
}

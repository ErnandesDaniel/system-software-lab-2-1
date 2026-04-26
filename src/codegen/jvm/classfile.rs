use ristretto_classfile::attributes::{Attribute, Instruction};
use ristretto_classfile::{ClassAccessFlags, ClassFile, Method, MethodAccessFlags};
use crate::ir::types::*;
use crate::codegen::jvm::JvmGenerator;
use crate::codegen::jvm::types::ir_type_to_jvm_descriptor;

impl JvmGenerator {
    pub fn build_class_file(&mut self, class_name: &str, func: &IrFunction, code: Vec<ristretto_classfile::attributes::Instruction>) -> Vec<u8> {
        // Debug info at start
        let func_name = &func.name;
        let code_len = code.len();
        
        let this_class = self.constant_pool.add_class(class_name).unwrap();
        let super_class = self.constant_pool.add_class("java/lang/Object").unwrap();

        let code_attr_name_idx = self.constant_pool.add_utf8("Code").unwrap();
        let max_locals = self.next_local_slot;
        let max_stack = self.estimate_max_stack(&code);
        
        // Validate code size - JVM limits method size to 65535 bytes
        let code_size: u32 = code.iter().map(|i| self.instr_size(i) as u32).sum();
        if code_size > 65535 {
            panic!("Function {}: Generated code size ({}) exceeds JVM limit of 65535 bytes. max_stack={}, max_locals={}", 
                   func_name, code_size, max_stack, max_locals);
        }

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

        let code_len = code.len();
        let constant_pool_len = self.constant_pool.len();
        let mut buffer = Vec::new();
        match class_file.to_bytes(&mut buffer) {
            Ok(_) => buffer,
            Err(e) => {
                panic!("Failed to serialize class file: {:?}. code_len={}, max_stack={}, max_locals={}, constant_pool_len={}", 
                       e, code_len, max_stack, max_locals, constant_pool_len);
            }
        }
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
    
    fn instr_size(&self, instr: &Instruction) -> usize {
        match instr {
            Instruction::Nop => 1,
            Instruction::Aconst_null => 1,
            Instruction::Iconst_m1 | Instruction::Iconst_0 | Instruction::Iconst_1 | 
            Instruction::Iconst_2 | Instruction::Iconst_3 | Instruction::Iconst_4 | 
            Instruction::Iconst_5 => 1,
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
            Instruction::Iload(_) | Instruction::Lload(_) | Instruction::Fload(_) | 
            Instruction::Dload(_) | Instruction::Aload(_) => 2,
            Instruction::Istore_0 | Instruction::Istore_1 | Instruction::Istore_2 | Instruction::Istore_3 => 1,
            Instruction::Lstore_0 | Instruction::Lstore_1 | Instruction::Lstore_2 | Instruction::Lstore_3 => 1,
            Instruction::Fstore_0 | Instruction::Fstore_1 | Instruction::Fstore_2 | Instruction::Fstore_3 => 1,
            Instruction::Dstore_0 | Instruction::Dstore_1 | Instruction::Dstore_2 | Instruction::Dstore_3 => 1,
            Instruction::Astore_0 | Instruction::Astore_1 | Instruction::Astore_2 | Instruction::Astore_3 => 1,
            Instruction::Istore(_) | Instruction::Lstore(_) | Instruction::Fstore(_) | 
            Instruction::Dstore(_) | Instruction::Astore(_) => 2,
            Instruction::Pop | Instruction::Pop2 => 1,
            Instruction::Dup | Instruction::Dup_x1 | Instruction::Dup_x2 | 
            Instruction::Dup2 | Instruction::Dup2_x1 | Instruction::Dup2_x2 => 1,
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
            Instruction::I2l | Instruction::I2f | Instruction::I2d | Instruction::L2i | 
            Instruction::L2f | Instruction::L2d | Instruction::F2i | Instruction::F2l | 
            Instruction::F2d | Instruction::D2i | Instruction::D2l | Instruction::D2f => 1,
            Instruction::I2b | Instruction::I2c | Instruction::I2s => 1,
            Instruction::Lcmp => 1,
            Instruction::Fcmpl | Instruction::Fcmpg | Instruction::Dcmpl | Instruction::Dcmpg => 1,
            Instruction::Ifeq(_) | Instruction::Ifne(_) | Instruction::Iflt(_) | 
            Instruction::Ifge(_) | Instruction::Ifgt(_) | Instruction::Ifle(_) => 3,
            Instruction::If_icmpeq(_) | Instruction::If_icmpne(_) | Instruction::If_icmplt(_) | 
            Instruction::If_icmpge(_) | Instruction::If_icmpgt(_) | Instruction::If_icmple(_) => 3,
            Instruction::If_acmpeq(_) | Instruction::If_acmpne(_) => 3,
            Instruction::Goto(_) => 3,
            Instruction::Jsr(_) => 3,
            Instruction::Ret(_) => 2,
            Instruction::Tableswitch { .. } => 1,
            Instruction::Lookupswitch { .. } => 1,
            Instruction::Ireturn | Instruction::Lreturn | Instruction::Freturn | 
            Instruction::Dreturn | Instruction::Areturn | Instruction::Return => 1,
            Instruction::Getstatic(_) => 3,
            Instruction::Putstatic(_) => 3,
            Instruction::Getfield(_) => 3,
            Instruction::Putfield(_) => 3,
            Instruction::Invokevirtual(_) | Instruction::Invokespecial(_) | 
            Instruction::Invokestatic(_) => 3,
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
}

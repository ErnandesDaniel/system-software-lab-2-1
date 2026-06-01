use crate::codegen::jvm::JvmGenerator;
use ristretto_classfile::attributes::Instruction;

impl JvmGenerator {
    pub(super) fn estimate_max_stack(&self, code: &[Instruction]) -> u16 {
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
            | Instruction::Iconst_0 | Instruction::Iconst_1 | Instruction::Iconst_2
            | Instruction::Iconst_3 | Instruction::Iconst_4 | Instruction::Iconst_5 => 1,
            Instruction::Lconst_0 | Instruction::Lconst_1 => 2,
            Instruction::Fconst_0 | Instruction::Fconst_1 | Instruction::Fconst_2 => 1,
            Instruction::Dconst_0 | Instruction::Dconst_1 => 2,
            Instruction::Bipush(_) => 1,
            Instruction::Sipush(_) => 1,
            Instruction::Ldc(_) | Instruction::Ldc_w(_) | Instruction::Ldc2_w(_) => 1,
            Instruction::Iload(_) | Instruction::Iload_0 | Instruction::Iload_1 | Instruction::Iload_2 | Instruction::Iload_3 => 1,
            Instruction::Lload(_) | Instruction::Lload_0 | Instruction::Lload_1 | Instruction::Lload_2 | Instruction::Lload_3 => 2,
            Instruction::Fload(_) | Instruction::Fload_0 | Instruction::Fload_1 | Instruction::Fload_2 | Instruction::Fload_3 => 1,
            Instruction::Dload(_) | Instruction::Dload_0 | Instruction::Dload_1 | Instruction::Dload_2 | Instruction::Dload_3 => 2,
            Instruction::Aload(_) | Instruction::Aload_0 | Instruction::Aload_1 | Instruction::Aload_2 | Instruction::Aload_3 => 1,
            Instruction::Istore(_) | Instruction::Istore_0 | Instruction::Istore_1 | Instruction::Istore_2 | Instruction::Istore_3 => -1,
            Instruction::Lstore(_) | Instruction::Lstore_0 | Instruction::Lstore_1 | Instruction::Lstore_2 | Instruction::Lstore_3 => -2,
            Instruction::Fstore(_) | Instruction::Fstore_0 | Instruction::Fstore_1 | Instruction::Fstore_2 | Instruction::Fstore_3 => -1,
            Instruction::Dstore(_) | Instruction::Dstore_0 | Instruction::Dstore_1 | Instruction::Dstore_2 | Instruction::Dstore_3 => -2,
            Instruction::Astore(_) | Instruction::Astore_0 | Instruction::Astore_1 | Instruction::Astore_2 | Instruction::Astore_3 => -1,
            Instruction::Pop => -1,
            Instruction::Pop2 => -2,
            Instruction::Dup | Instruction::Dup_x1 | Instruction::Dup_x2 => 1,
            Instruction::Dup2 | Instruction::Dup2_x1 | Instruction::Dup2_x2 => 2,
            Instruction::Swap => 0,
            Instruction::Iadd | Instruction::Ladd | Instruction::Fadd | Instruction::Dadd
            | Instruction::Isub | Instruction::Lsub | Instruction::Fsub | Instruction::Dsub
            | Instruction::Imul | Instruction::Lmul | Instruction::Fmul | Instruction::Dmul
            | Instruction::Idiv | Instruction::Ldiv | Instruction::Fdiv | Instruction::Ddiv
            | Instruction::Irem | Instruction::Lrem | Instruction::Frem | Instruction::Drem
            | Instruction::Ishl | Instruction::Lshl | Instruction::Ishr | Instruction::Lshr
            | Instruction::Iushr | Instruction::Lushr
            | Instruction::Iand | Instruction::Land | Instruction::Ior | Instruction::Lor
            | Instruction::Ixor | Instruction::Lxor => -1,
            Instruction::Ineg | Instruction::Lneg | Instruction::Fneg | Instruction::Dneg => 0,
            Instruction::I2l | Instruction::I2f | Instruction::I2d
            | Instruction::L2i | Instruction::L2f | Instruction::L2d
            | Instruction::F2i | Instruction::F2l | Instruction::F2d
            | Instruction::D2i | Instruction::D2l | Instruction::D2f
            | Instruction::I2b | Instruction::I2c | Instruction::I2s => 0,
            Instruction::Lcmp | Instruction::Fcmpl | Instruction::Fcmpg | Instruction::Dcmpl | Instruction::Dcmpg => -1,
            Instruction::Ifeq(_) | Instruction::Ifne(_) | Instruction::Iflt(_) | Instruction::Ifge(_) | Instruction::Ifgt(_) | Instruction::Ifle(_) => -1,
            Instruction::If_icmpeq(_) | Instruction::If_icmpne(_) | Instruction::If_icmplt(_) | Instruction::If_icmpge(_) | Instruction::If_icmpgt(_) | Instruction::If_icmple(_) => -2,
            Instruction::If_acmpeq(_) | Instruction::If_acmpne(_) => -2,
            Instruction::Goto(_) | Instruction::Jsr(_) => 0,
            Instruction::Ret(_) => 0,
            Instruction::Tableswitch { .. } | Instruction::Lookupswitch { .. } => -1,
            Instruction::Ireturn | Instruction::Lreturn | Instruction::Freturn | Instruction::Dreturn | Instruction::Areturn | Instruction::Return => -999,
            Instruction::Getstatic(_) => 1,
            Instruction::Putstatic(_) => -1,
            Instruction::Getfield(_) => 0,
            Instruction::Putfield(_) => -2,
            Instruction::Invokevirtual(_) | Instruction::Invokespecial(_) => -1,
            Instruction::Invokestatic(_) => -1,
            Instruction::Invokeinterface(_, _) => -1,
            Instruction::Invokedynamic(_) => -1,
            Instruction::New(_) => 1,
            Instruction::Newarray(_) => 0,
            Instruction::Anewarray(_) => 0,
            Instruction::Arraylength => 0,
            Instruction::Athrow => -1,
            Instruction::Checkcast(_) | Instruction::Instanceof(_) => 0,
            Instruction::Monitorenter | Instruction::Monitorexit => -1,
            Instruction::Wide => 0,
            Instruction::Ifnull(_) | Instruction::Ifnonnull(_) => -1,
            Instruction::Goto_w(_) | Instruction::Jsr_w(_) => 0,
            Instruction::Iaload | Instruction::Laload | Instruction::Faload | Instruction::Daload
            | Instruction::Aaload | Instruction::Baload | Instruction::Caload | Instruction::Saload => -1,
            Instruction::Iastore | Instruction::Lastore | Instruction::Fastore | Instruction::Dastore
            | Instruction::Aastore | Instruction::Bastore | Instruction::Castore | Instruction::Sastore => -3,
            _ => 0,
        }
    }

    pub(super) fn instr_size(&self, instr: &Instruction) -> usize {
        match instr {
            Instruction::Nop => 1,
            Instruction::Aconst_null => 1,
            Instruction::Iconst_m1 | Instruction::Iconst_0 | Instruction::Iconst_1 | Instruction::Iconst_2 | Instruction::Iconst_3 | Instruction::Iconst_4 | Instruction::Iconst_5 => 1,
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
            Instruction::Iload(_) | Instruction::Lload(_) | Instruction::Fload(_) | Instruction::Dload(_) | Instruction::Aload(_) => 2,
            Instruction::Istore_0 | Instruction::Istore_1 | Instruction::Istore_2 | Instruction::Istore_3 => 1,
            Instruction::Lstore_0 | Instruction::Lstore_1 | Instruction::Lstore_2 | Instruction::Lstore_3 => 1,
            Instruction::Fstore_0 | Instruction::Fstore_1 | Instruction::Fstore_2 | Instruction::Fstore_3 => 1,
            Instruction::Dstore_0 | Instruction::Dstore_1 | Instruction::Dstore_2 | Instruction::Dstore_3 => 1,
            Instruction::Astore_0 | Instruction::Astore_1 | Instruction::Astore_2 | Instruction::Astore_3 => 1,
            Instruction::Istore(_) | Instruction::Lstore(_) | Instruction::Fstore(_) | Instruction::Dstore(_) | Instruction::Astore(_) => 2,
            Instruction::Pop | Instruction::Pop2 => 1,
            Instruction::Dup | Instruction::Dup_x1 | Instruction::Dup_x2 | Instruction::Dup2 | Instruction::Dup2_x1 | Instruction::Dup2_x2 => 1,
            Instruction::Swap => 1,
            Instruction::Iadd | Instruction::Ladd | Instruction::Fadd | Instruction::Dadd
            | Instruction::Isub | Instruction::Lsub | Instruction::Fsub | Instruction::Dsub
            | Instruction::Imul | Instruction::Lmul | Instruction::Fmul | Instruction::Dmul
            | Instruction::Idiv | Instruction::Ldiv | Instruction::Fdiv | Instruction::Ddiv
            | Instruction::Irem | Instruction::Lrem | Instruction::Frem | Instruction::Drem
            | Instruction::Ineg | Instruction::Lneg | Instruction::Fneg | Instruction::Dneg
            | Instruction::Ishl | Instruction::Lshl | Instruction::Ishr | Instruction::Lshr
            | Instruction::Iushr | Instruction::Lushr
            | Instruction::Iand | Instruction::Land | Instruction::Ior | Instruction::Lor
            | Instruction::Ixor | Instruction::Lxor => 1,
            Instruction::Iinc(_, _) => 3,
            Instruction::I2l | Instruction::I2f | Instruction::I2d | Instruction::L2i | Instruction::L2f | Instruction::L2d
            | Instruction::F2i | Instruction::F2l | Instruction::F2d | Instruction::D2i | Instruction::D2l | Instruction::D2f
            | Instruction::I2b | Instruction::I2c | Instruction::I2s => 1,
            Instruction::Lcmp => 1,
            Instruction::Fcmpl | Instruction::Fcmpg | Instruction::Dcmpl | Instruction::Dcmpg => 1,
            Instruction::Ifeq(_) | Instruction::Ifne(_) | Instruction::Iflt(_) | Instruction::Ifge(_) | Instruction::Ifgt(_) | Instruction::Ifle(_) => 3,
            Instruction::If_icmpeq(_) | Instruction::If_icmpne(_) | Instruction::If_icmplt(_) | Instruction::If_icmpge(_) | Instruction::If_icmpgt(_) | Instruction::If_icmple(_) => 3,
            Instruction::If_acmpeq(_) | Instruction::If_acmpne(_) => 3,
            Instruction::Goto(_) => 3,
            Instruction::Jsr(_) => 3,
            Instruction::Ret(_) => 2,
            Instruction::Tableswitch { .. } => 1,
            Instruction::Lookupswitch { .. } => 1,
            Instruction::Ireturn | Instruction::Lreturn | Instruction::Freturn | Instruction::Dreturn | Instruction::Areturn | Instruction::Return => 1,
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
}

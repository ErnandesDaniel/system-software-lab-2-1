use crate::codegen::jvm::JvmGenerator;
use ristretto_classfile::attributes::Instruction;

impl JvmGenerator {
    pub(super) fn estimate_max_stack(code: &[Instruction]) -> u16 {
        let mut max_depth = 0u16;
        let mut depth = 0i32;
        for instr in code {
            let stack_delta = Self::instr_stack_delta(instr);
            depth += stack_delta;
            if depth > i32::from(max_depth) {
                max_depth = depth as u16;
            }
        }
        max_depth.max(4)
    }

    fn instr_stack_delta(instr: &Instruction) -> i32 {
        match instr {
            Instruction::Lconst_0
            | Instruction::Lconst_1
            | Instruction::Dconst_0
            | Instruction::Dconst_1
            | Instruction::Lload(_)
            | Instruction::Lload_0
            | Instruction::Lload_1
            | Instruction::Lload_2
            | Instruction::Lload_3
            | Instruction::Dload(_)
            | Instruction::Dload_0
            | Instruction::Dload_1
            | Instruction::Dload_2
            | Instruction::Dload_3
            | Instruction::Dup2
            | Instruction::Dup2_x1
            | Instruction::Dup2_x2 => 2,
            Instruction::Aconst_null
            | Instruction::Iconst_m1
            | Instruction::Iconst_0
            | Instruction::Iconst_1
            | Instruction::Iconst_2
            | Instruction::Iconst_3
            | Instruction::Iconst_4
            | Instruction::Iconst_5
            | Instruction::Fconst_0
            | Instruction::Fconst_1
            | Instruction::Fconst_2
            | Instruction::Bipush(_)
            | Instruction::Sipush(_)
            | Instruction::Ldc(_)
            | Instruction::Ldc_w(_)
            | Instruction::Ldc2_w(_)
            | Instruction::Iload(_)
            | Instruction::Iload_0
            | Instruction::Iload_1
            | Instruction::Iload_2
            | Instruction::Iload_3
            | Instruction::Fload(_)
            | Instruction::Fload_0
            | Instruction::Fload_1
            | Instruction::Fload_2
            | Instruction::Fload_3
            | Instruction::Aload(_)
            | Instruction::Aload_0
            | Instruction::Aload_1
            | Instruction::Aload_2
            | Instruction::Aload_3
            | Instruction::Dup
            | Instruction::Dup_x1
            | Instruction::Dup_x2
            | Instruction::Getstatic(_)
            | Instruction::New(_)
            | Instruction::Invokestatic(_)
            | Instruction::Invokeinterface(_, _)
            | Instruction::Invokedynamic(_) => 1,
            Instruction::Iastore
            | Instruction::Lastore
            | Instruction::Fastore
            | Instruction::Dastore
            | Instruction::Aastore
            | Instruction::Bastore
            | Instruction::Castore
            | Instruction::Sastore => -3,
            Instruction::Lstore(_)
            | Instruction::Lstore_0
            | Instruction::Lstore_1
            | Instruction::Lstore_2
            | Instruction::Lstore_3
            | Instruction::Dstore(_)
            | Instruction::Dstore_0
            | Instruction::Dstore_1
            | Instruction::Dstore_2
            | Instruction::Dstore_3
            | Instruction::Pop2
            | Instruction::If_icmpeq(_)
            | Instruction::If_icmpne(_)
            | Instruction::If_icmplt(_)
            | Instruction::If_icmpge(_)
            | Instruction::If_icmpgt(_)
            | Instruction::If_icmple(_)
            | Instruction::If_acmpeq(_)
            | Instruction::If_acmpne(_)
            | Instruction::Putfield(_) => -2,
            Instruction::Ireturn
            | Instruction::Lreturn
            | Instruction::Freturn
            | Instruction::Dreturn
            | Instruction::Areturn
            | Instruction::Return => -999,
            Instruction::Istore(_)
            | Instruction::Istore_0
            | Instruction::Istore_1
            | Instruction::Istore_2
            | Instruction::Istore_3
            | Instruction::Fstore(_)
            | Instruction::Fstore_0
            | Instruction::Fstore_1
            | Instruction::Fstore_2
            | Instruction::Fstore_3
            | Instruction::Astore(_)
            | Instruction::Astore_0
            | Instruction::Astore_1
            | Instruction::Astore_2
            | Instruction::Astore_3
            | Instruction::Pop
            | Instruction::Iadd
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
            | Instruction::Lxor
            | Instruction::Lcmp
            | Instruction::Fcmpl
            | Instruction::Fcmpg
            | Instruction::Dcmpl
            | Instruction::Dcmpg
            | Instruction::Ifeq(_)
            | Instruction::Ifne(_)
            | Instruction::Iflt(_)
            | Instruction::Ifge(_)
            | Instruction::Ifgt(_)
            | Instruction::Ifle(_)
            | Instruction::Tableswitch { .. }
            | Instruction::Lookupswitch { .. }
            | Instruction::Invokespecial(_)
            | Instruction::Invokevirtual(_)
            | Instruction::Athrow
            | Instruction::Monitorenter
            | Instruction::Monitorexit
            | Instruction::Ifnull(_)
            | Instruction::Ifnonnull(_)
            | Instruction::Putstatic(_)
            | Instruction::Iaload
            | Instruction::Laload
            | Instruction::Faload
            | Instruction::Daload
            | Instruction::Aaload
            | Instruction::Baload
            | Instruction::Caload
            | Instruction::Saload => -1,
            _ => 0,
        }
    }

    pub(super) fn instr_size(instr: &Instruction) -> usize {
        match instr {
            Instruction::Sipush(_)
            | Instruction::Iinc(_, _)
            | Instruction::Ifeq(_)
            | Instruction::Ifne(_)
            | Instruction::Iflt(_)
            | Instruction::Ifge(_)
            | Instruction::Ifgt(_)
            | Instruction::Ifle(_)
            | Instruction::If_icmpeq(_)
            | Instruction::If_icmpne(_)
            | Instruction::If_icmplt(_)
            | Instruction::If_icmpge(_)
            | Instruction::If_icmpgt(_)
            | Instruction::If_icmple(_)
            | Instruction::If_acmpeq(_)
            | Instruction::If_acmpne(_)
            | Instruction::Goto(_)
            | Instruction::Jsr(_)
            | Instruction::Getstatic(_)
            | Instruction::Putstatic(_)
            | Instruction::Getfield(_)
            | Instruction::Putfield(_)
            | Instruction::Invokevirtual(_)
            | Instruction::Invokespecial(_)
            | Instruction::Invokestatic(_)
            | Instruction::New(_)
            | Instruction::Anewarray(_)
            | Instruction::Checkcast(_)
            | Instruction::Instanceof(_)
            | Instruction::Ifnull(_)
            | Instruction::Ifnonnull(_)
            | Instruction::Ldc_w(_)
            | Instruction::Ldc2_w(_) => 3,
            Instruction::Bipush(_)
            | Instruction::Ldc(_)
            | Instruction::Ret(_)
            | Instruction::Newarray(_)
            | Instruction::Iload(_)
            | Instruction::Lload(_)
            | Instruction::Fload(_)
            | Instruction::Dload(_)
            | Instruction::Aload(_)
            | Instruction::Istore(_)
            | Instruction::Lstore(_)
            | Instruction::Fstore(_)
            | Instruction::Dstore(_)
            | Instruction::Astore(_) => 2,
            Instruction::Invokeinterface(_, _)
            | Instruction::Invokedynamic(_)
            | Instruction::Goto_w(_)
            | Instruction::Jsr_w(_) => 5,
            _ => 1,
        }
    }
}

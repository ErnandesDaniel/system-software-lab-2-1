//! JVM bytecode utilities
//! 
//! This module provides helper functions for working with JVM bytecode.
//! The main code generation logic is in the parent module (mod.rs).

use ristretto_classfile::attributes::Instruction;

/// Calculate the size of a JVM instruction in bytes
pub fn instruction_size(instr: &Instruction) -> usize {
    use ristretto_classfile::attributes::Instruction as Ri;
    
    match instr {
        Ri::Nop => 1,
        Ri::Aconst_null => 1,
        Ri::Iconst_m1 | Ri::Iconst_0 | Ri::Iconst_1 | Ri::Iconst_2 |
        Ri::Iconst_3 | Ri::Iconst_4 | Ri::Iconst_5 => 1,
        Ri::Lconst_0 | Ri::Lconst_1 => 1,
        Ri::Fconst_0 | Ri::Fconst_1 | Ri::Fconst_2 => 1,
        Ri::Dconst_0 | Ri::Dconst_1 => 1,
        Ri::Bipush(_) => 2,
        Ri::Sipush(_) => 3,
        Ri::Ldc(_) => 2,
        Ri::Ldc_w(_) | Ri::Ldc2_w(_) => 3,
        Ri::Iload_0 | Ri::Iload_1 | Ri::Iload_2 | Ri::Iload_3 => 1,
        Ri::Lload_0 | Ri::Lload_1 | Ri::Lload_2 | Ri::Lload_3 => 1,
        Ri::Fload_0 | Ri::Fload_1 | Ri::Fload_2 | Ri::Fload_3 => 1,
        Ri::Dload_0 | Ri::Dload_1 | Ri::Dload_2 | Ri::Dload_3 => 1,
        Ri::Aload_0 | Ri::Aload_1 | Ri::Aload_2 | Ri::Aload_3 => 1,
        Ri::Iload(_) | Ri::Lload(_) | Ri::Fload(_) | Ri::Dload(_) | Ri::Aload(_) => 2,
        Ri::Istore_0 | Ri::Istore_1 | Ri::Istore_2 | Ri::Istore_3 => 1,
        Ri::Lstore_0 | Ri::Lstore_1 | Ri::Lstore_2 | Ri::Lstore_3 => 1,
        Ri::Fstore_0 | Ri::Fstore_1 | Ri::Fstore_2 | Ri::Fstore_3 => 1,
        Ri::Dstore_0 | Ri::Dstore_1 | Ri::Dstore_2 | Ri::Dstore_3 => 1,
        Ri::Astore_0 | Ri::Astore_1 | Ri::Astore_2 | Ri::Astore_3 => 1,
        Ri::Istore(_) | Ri::Lstore(_) | Ri::Fstore(_) | Ri::Dstore(_) | Ri::Astore(_) => 2,
        Ri::Pop | Ri::Pop2 => 1,
        Ri::Dup | Ri::Dup_x1 | Ri::Dup_x2 | Ri::Dup2 | Ri::Dup2_x1 | Ri::Dup2_x2 => 1,
        Ri::Swap => 1,
        Ri::Iadd | Ri::Ladd | Ri::Fadd | Ri::Dadd => 1,
        Ri::Isub | Ri::Lsub | Ri::Fsub | Ri::Dsub => 1,
        Ri::Imul | Ri::Lmul | Ri::Fmul | Ri::Dmul => 1,
        Ri::Idiv | Ri::Ldiv | Ri::Fdiv | Ri::Ddiv => 1,
        Ri::Irem | Ri::Lrem | Ri::Frem | Ri::Drem => 1,
        Ri::Ineg | Ri::Lneg | Ri::Fneg | Ri::Dneg => 1,
        Ri::Ishl | Ri::Lshl => 1,
        Ri::Ishr | Ri::Lshr => 1,
        Ri::Iushr | Ri::Lushr => 1,
        Ri::Iand | Ri::Land => 1,
        Ri::Ior | Ri::Lor => 1,
        Ri::Ixor | Ri::Lxor => 1,
        Ri::Iinc(_, _) => 3,
        Ri::I2l | Ri::I2f | Ri::I2d | Ri::L2i | Ri::L2f | Ri::L2d | Ri::F2i | Ri::F2l | Ri::F2d | Ri::D2i | Ri::D2l | Ri::D2f => 1,
        Ri::I2b | Ri::I2c | Ri::I2s => 1,
        Ri::Lcmp => 1,
        Ri::Fcmpl | Ri::Fcmpg | Ri::Dcmpl | Ri::Dcmpg => 1,
        Ri::Ifeq(_) | Ri::Ifne(_) | Ri::Iflt(_) | Ri::Ifge(_) | Ri::Ifgt(_) | Ri::Ifle(_) => 3,
        Ri::If_icmpeq(_) | Ri::If_icmpne(_) | Ri::If_icmplt(_) | Ri::If_icmpge(_) | Ri::If_icmpgt(_) | Ri::If_icmple(_) => 3,
        Ri::If_acmpeq(_) | Ri::If_acmpne(_) => 3,
        Ri::Goto(_) => 3,
        Ri::Jsr(_) => 3,
        Ri::Ret(_) => 2,
        Ri::Tableswitch { .. } => 1, // Variable size, simplified
        Ri::Lookupswitch { .. } => 1, // Variable size, simplified
        Ri::Ireturn | Ri::Lreturn | Ri::Freturn | Ri::Dreturn | Ri::Areturn | Ri::Return => 1,
        Ri::Getstatic(_) => 3,
        Ri::Putstatic(_) => 3,
        Ri::Getfield(_) => 3,
        Ri::Putfield(_) => 3,
        Ri::Invokevirtual(_) | Ri::Invokespecial(_) | Ri::Invokestatic(_) => 3,
        Ri::Invokeinterface(_, _) => 5,
        Ri::Invokedynamic(_) => 5,
        Ri::New(_) => 3,
        Ri::Newarray(_) => 2,
        Ri::Anewarray(_) => 3,
        Ri::Arraylength => 1,
        Ri::Athrow => 1,
        Ri::Checkcast(_) => 3,
        Ri::Instanceof(_) => 3,
        Ri::Monitorenter | Ri::Monitorexit => 1,
        Ri::Wide => 1,
        Ri::Ifnull(_) | Ri::Ifnonnull(_) => 3,
        Ri::Goto_w(_) | Ri::Jsr_w(_) => 5,
        _ => 1,
    }
}

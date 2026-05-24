use ristretto_classfile::attributes::Instruction;
use ristretto_classfile::attributes::ArrayType;
use crate::ir::types::*;
use crate::codegen::jvm::JvmGenerator;
use crate::codegen::jvm::types::BinaryOp;

impl JvmGenerator {
    pub fn generate_instruction(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, global_offset: u16) {
        match inst.opcode {
            IrOpcode::Assign => self.generate_assign(code, inst),
            IrOpcode::Add => self.generate_binary_op(code, inst, BinaryOp::Add),
            IrOpcode::Sub => self.generate_binary_op(code, inst, BinaryOp::Sub),
            IrOpcode::Mul => self.generate_binary_op(code, inst, BinaryOp::Mul),
            IrOpcode::Div => self.generate_binary_op(code, inst, BinaryOp::Div),
            IrOpcode::Mod => self.generate_binary_op(code, inst, BinaryOp::Mod),
            IrOpcode::Neg => self.generate_neg(code, inst),
            IrOpcode::Pos => self.generate_pos(code, inst),
            IrOpcode::And => self.generate_logical_and(code, inst, global_offset),
            IrOpcode::Or => self.generate_logical_or(code, inst, global_offset),
            IrOpcode::Not => self.generate_logical_not(code, inst, global_offset),
            IrOpcode::BitAnd => self.generate_binary_op(code, inst, BinaryOp::BitAnd),
            IrOpcode::BitOr => self.generate_binary_op(code, inst, BinaryOp::BitOr),
            IrOpcode::BitNot => self.generate_bit_not(code, inst),
            IrOpcode::Eq => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Eq, global_offset),
            IrOpcode::Ne => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Ne, global_offset),
            IrOpcode::Lt => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Lt, global_offset),
            IrOpcode::Le => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Le, global_offset),
            IrOpcode::Gt => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Gt, global_offset),
            IrOpcode::Ge => self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Ge, global_offset),
            IrOpcode::Call => self.generate_call(code, inst),
            IrOpcode::Ret => self.generate_return(code, inst),
            IrOpcode::Jump => self.generate_jump(code, inst),
            IrOpcode::CondBr => self.generate_conditional_branch(code, inst),
            IrOpcode::Load => self.generate_array_load(code, inst),
            IrOpcode::Slice => {}
            IrOpcode::Alloca => {}
            IrOpcode::Store => {}
            IrOpcode::Cast => {}
            IrOpcode::CoroYield => {}
            IrOpcode::CallIndirect => {}
            IrOpcode::MakeClosure => self.generate_make_closure(code, inst),
            IrOpcode::CallClosure => self.generate_call_closure(code, inst),
            IrOpcode::LoadCaptured => self.generate_load_captured(code, inst),
            IrOpcode::StoreCaptured => self.generate_store_captured(code, inst),
        }
    }

    fn generate_assign(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(ref operand)) = (&inst.result, inst.operands.first()) {
            if self.wrapped_vars.contains(result) {
                // Store through wrapper: aload wrapper; iconst_0; <value>; iastore
                let slot = self.get_local_slot(result);
                match slot {
                    0 => code.push(Instruction::Aload_0),
                    1 => code.push(Instruction::Aload_1),
                    2 => code.push(Instruction::Aload_2),
                    3 => code.push(Instruction::Aload_3),
                    _ => code.push(Instruction::Aload(slot as u8)),
                }
                code.push(Instruction::Iconst_0);
                self.emit_load_operand(code, operand);
                code.push(Instruction::Iastore);
            } else {
                self.emit_load_operand(code, operand);
                let slot = self.get_local_slot(result);
                match operand.get_type() {
                    IrType::String => code.push(Instruction::Astore(slot as u8)),
                    _ => code.push(Instruction::Istore(slot as u8)),
                }
            }
        }
    }

    fn generate_binary_op(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, op: BinaryOp) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, left);
            self.emit_load_operand(code, right);

            let instr = match op {
                BinaryOp::Add => Instruction::Iadd,
                BinaryOp::Sub => Instruction::Isub,
                BinaryOp::Mul => Instruction::Imul,
                BinaryOp::Div => Instruction::Idiv,
                BinaryOp::Mod => Instruction::Irem,
                BinaryOp::BitAnd => Instruction::Iand,
                BinaryOp::BitOr => Instruction::Ior,
            };
            code.push(instr);

            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_neg(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Ineg);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_pos(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_bit_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Iconst_m1);
            code.push(Instruction::Ixor);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_call(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref target) = inst.jump_target {
            for operand in &inst.operands {
                self.emit_load_operand(code, operand);
            }

            let method_idx = self.method_refs.get(target).copied().unwrap_or(1);
            code.push(Instruction::Invokestatic(method_idx));

            if let Some(ref result) = inst.result {
                let slot = self.get_local_slot(result);
                let is_string = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::String));
                if is_string {
                    code.push(Instruction::Astore(slot as u8));
                } else {
                    code.push(Instruction::Istore(slot as u8));
                }
            }
        }
    }

    fn generate_return(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            self.emit_load_operand(code, operand);
            if operand.get_type() == IrType::String {
                code.push(Instruction::Areturn);
            } else {
                code.push(Instruction::Ireturn);
            }
        } else {
            code.push(Instruction::Return);
        }
    }

    fn generate_jump(&self, _code: &mut Vec<Instruction>, _inst: &IrInstruction) {
        // Jump is handled entirely in generate_instruction_with_placeholders
        // to properly manage branch targets and avoid duplicate instructions
    }

    fn generate_conditional_branch(&self, _code: &mut Vec<Instruction>, _inst: &IrInstruction) {
        // CondBr is handled entirely in generate_instruction_with_placeholders
        // to properly manage branch targets and avoid duplicate instructions
    }

    fn generate_array_load(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(array), Some(index)) = (&inst.result, inst.operands.get(0), inst.operands.get(1)) {
            self.emit_load_operand(code, array);
            self.emit_load_operand(code, index);
            code.push(Instruction::Iaload);
            let slot = self.get_local_slot(result);
            code.push(Instruction::Istore(slot as u8));
        }
    }

    fn generate_make_closure(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        // operands[0] = FuncRef(lambda_name)
        // operands[1..] = captured variables
        // result = env temp (holds int[][] reference)
        if let Some(ref result) = inst.result {
            let num_captures = inst.operands.len().saturating_sub(1);
            let anewarray_idx = self.anewarray_int_class_idx.unwrap_or(0);

            // Create env array: int[num_captures][]
            self.emit_load_constant(code, &Constant::Int(num_captures as i64));
            code.push(Instruction::Anewarray(anewarray_idx));

            // For each capture, store wrapper ref (or create new one) into env array
            for (capture_idx, op) in inst.operands.iter().enumerate().skip(1) {
                code.push(Instruction::Dup);                   // dup env ref for storing
                self.emit_load_constant(code, &Constant::Int((capture_idx - 1) as i64)); // slot index

                // If captured variable is a wrapped var, reuse its existing wrapper
                // Otherwise, create a new int[1] wrapper with the current value
                if let IrOperand::Variable(name, _) = op {
                    if self.wrapped_vars.contains(name) {
                        let cap_slot = self.get_local_slot(name);
                        match cap_slot {
                            0 => code.push(Instruction::Aload_0),
                            1 => code.push(Instruction::Aload_1),
                            2 => code.push(Instruction::Aload_2),
                            3 => code.push(Instruction::Aload_3),
                            _ => code.push(Instruction::Aload(cap_slot as u8)),
                        }
                    } else {
                        code.push(Instruction::Iconst_1);
                        code.push(Instruction::Newarray(ArrayType::Int));
                        code.push(Instruction::Dup);
                        code.push(Instruction::Iconst_0);
                        self.emit_load_operand(code, op);
                        code.push(Instruction::Iastore);
                    }
                } else {
                    code.push(Instruction::Iconst_1);
                    code.push(Instruction::Newarray(ArrayType::Int));
                    code.push(Instruction::Dup);
                    code.push(Instruction::Iconst_0);
                    self.emit_load_operand(code, op);
                    code.push(Instruction::Iastore);
                }

                code.push(Instruction::Aastore);               // env[i] = wrapper
            }

            let result_slot = self.get_local_slot(result);
            code.push(Instruction::Astore(result_slot as u8));
        }
    }

    fn generate_call_closure(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        // operands[0] = func ptr (Variable, not loaded — used only for IR tracking)
        // operands[1] = env ptr (Variable)
        // operands[2..] = call args
        if let Some(env_operand) = inst.operands.get(1) {
            // Load env reference (aload since it's a reference type)
            if let IrOperand::Variable(env_name, _) = env_operand {
                let env_slot = self.get_local_slot(env_name);
                match env_slot {
                    0 => code.push(Instruction::Aload_0),
                    1 => code.push(Instruction::Aload_1),
                    2 => code.push(Instruction::Aload_2),
                    3 => code.push(Instruction::Aload_3),
                    _ => code.push(Instruction::Aload(env_slot as u8)),
                }
            }

            // Load regular args (operands[2..])
            for arg in inst.operands.iter().skip(2) {
                self.emit_load_operand(code, arg);
            }

            // Look up method ref: use closure_targets to find lambda name from env ptr
            let lambda_name = if let IrOperand::Variable(env_name, _) = env_operand {
                self.closure_targets.get(env_name).cloned()
            } else {
                None
            };

            if let Some(ref name) = lambda_name {
                let method_idx = self.method_refs.get(name).copied().unwrap_or(1);
                code.push(Instruction::Invokestatic(method_idx));
            } else {
                code.push(Instruction::Nop);
            }

            // Store result if any
            if let Some(ref result) = inst.result {
                let slot = self.get_local_slot(result);
                let is_string = inst.result_type.as_ref().map_or(false, |t| matches!(t, IrType::String));
                if is_string {
                    code.push(Instruction::Astore(slot as u8));
                } else {
                    code.push(Instruction::Istore(slot as u8));
                }
            }
        }
    }

    fn generate_load_captured(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        // operands[0] = __env variable
        // operands[1] = slot index (Constant::Int)
        // result = loaded value
        if let (Some(ref result), Some(slot_op)) = (&inst.result, inst.operands.get(1)) {
            let env_slot = self.get_local_slot("__env");
            match env_slot {
                0 => code.push(Instruction::Aload_0),
                1 => code.push(Instruction::Aload_1),
                2 => code.push(Instruction::Aload_2),
                3 => code.push(Instruction::Aload_3),
                _ => code.push(Instruction::Aload(env_slot as u8)),
            }

            if let IrOperand::Constant(Constant::Int(slot)) = slot_op {
                self.emit_load_constant(code, &Constant::Int(*slot));
            }

            code.push(Instruction::Aaload);    // [[I → [I  (wrapper array)
            code.push(Instruction::Iconst_0);
            code.push(Instruction::Iaload);     // [I → int (captured value)

            let result_slot = self.get_local_slot(result);
            code.push(Instruction::Istore(result_slot as u8));
        }
    }

    fn generate_store_captured(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        // operands[0] = __env variable
        // operands[1] = slot index (Constant::Int)
        // operands[2] = value to store
        if let (Some(slot_op), Some(val_op)) = (inst.operands.get(1), inst.operands.get(2)) {
            let env_slot = self.get_local_slot("__env");
            match env_slot {
                0 => code.push(Instruction::Aload_0),
                1 => code.push(Instruction::Aload_1),
                2 => code.push(Instruction::Aload_2),
                3 => code.push(Instruction::Aload_3),
                _ => code.push(Instruction::Aload(env_slot as u8)),
            }

            if let IrOperand::Constant(Constant::Int(slot)) = slot_op {
                self.emit_load_constant(code, &Constant::Int(*slot));
            }

            code.push(Instruction::Aaload);    // [[I → [I  (wrapper array)
            code.push(Instruction::Iconst_0);
            self.emit_load_operand(code, val_op); // value
            code.push(Instruction::Iastore);     // wrapper[0] = value
        }
    }
}

use crate::codegen::jvm::types::BinaryOp;
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{Constant, IrInstruction, IrOpcode, IrOperand, IrType};
use ristretto_classfile::attributes::ArrayType;
use ristretto_classfile::attributes::Instruction;

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
            IrOpcode::BitXor => self.generate_binary_op(code, inst, BinaryOp::BitXor),
            IrOpcode::BitNot => self.generate_bit_not(code, inst),
            IrOpcode::Eq => {
                self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Eq, global_offset);
            }
            IrOpcode::Ne => {
                self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Ne, global_offset);
            }
            IrOpcode::Lt => {
                self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Lt, global_offset);
            }
            IrOpcode::Le => {
                self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Le, global_offset);
            }
            IrOpcode::Gt => {
                self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Gt, global_offset);
            }
            IrOpcode::Ge => {
                self.generate_comparison(code, inst, crate::codegen::jvm::types::ComparisonOp::Ge, global_offset);
            }
            IrOpcode::Call => self.generate_call(code, inst),
            IrOpcode::Ret => self.generate_return(code, inst),
            IrOpcode::Jump => self.generate_jump(code, inst),
            IrOpcode::CondBr => self.generate_conditional_branch(code, inst),
            IrOpcode::Load => self.generate_array_load(code, inst),
            IrOpcode::Slice => self.generate_slice(code, inst),
            IrOpcode::Alloca => {}
            IrOpcode::Store => self.generate_store(code, inst),
            IrOpcode::Cast => {}
            IrOpcode::CoroYield => self.generate_coro_yield(code, inst),
            IrOpcode::CallIndirect => self.generate_call_indirect(code, inst),
            IrOpcode::MakeClosure => self.generate_make_closure(code, inst),
            IrOpcode::CallClosure => self.generate_call_closure(code, inst),
            IrOpcode::LoadCaptured => self.generate_load_captured(code, inst),
            IrOpcode::StoreCaptured => self.generate_store_captured(code, inst),
            IrOpcode::StrGetByte => self.generate_str_get_byte(code, inst),
            IrOpcode::StrSetByte => self.generate_str_set_byte(code, inst),
            IrOpcode::AllocArray => self.generate_alloc_array(code, inst),
        }
    }

    fn generate_assign(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
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
                self.emit_store_result(code, result, &operand.get_type());
            }
        }
    }

    fn generate_binary_op(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, op: BinaryOp) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
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
                BinaryOp::BitXor => Instruction::Ixor,
            };
            code.push(instr);

            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    fn generate_neg(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Ineg);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    fn generate_pos(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    fn generate_bit_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            code.push(Instruction::Iconst_m1);
            code.push(Instruction::Ixor);
            self.emit_store_result(code, result, &IrType::Int);
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
                let store_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                self.emit_store_result(code, result, store_ty);
            }
        }
    }

    fn generate_return(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if self.is_coroutine {
            if let Some(operand) = inst.operands.first() {
                code.push(Instruction::Aload_0);
                self.emit_load_operand(code, operand);
                code.push(Instruction::Putfield(self.coroutine_result_field));
            }
            code.push(Instruction::Aload_0);
            code.push(Instruction::Iconst_m1);
            code.push(Instruction::Putfield(self.coroutine_state_field));
            code.push(Instruction::Iconst_1);
            code.push(Instruction::Ireturn);
        } else if let Some(operand) = inst.operands.first() {
            self.emit_load_operand(code, operand);
            match operand.get_type() {
                IrType::String | IrType::Function(_, _) => code.push(Instruction::Areturn),
                _ => code.push(Instruction::Ireturn),
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

    fn generate_store(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if inst.operands.len() >= 4 {
            // 4-operand store: [base, field_offset, value, index] → array-of-struct field write
            if let (Some(base), Some(field_off), Some(value), Some(index)) =
                (inst.operands.first(), inst.operands.get(1), inst.operands.get(2), inst.operands.get(3))
            {
                if self.is_struct_var(base) {
                    // Object[] path: entries[i].field = value
                    let byte_off = if let IrOperand::Constant(Constant::Int(b)) = field_off { *b as usize } else { 0 };
                    let var_name = if let IrOperand::Variable(n, _) = base { n.clone() } else { String::new() };
                    let field_slot = self.get_field_slot_for_offset(&var_name, byte_off);
                    // Load entries array
                    self.emit_load_operand(code, base);  // Object[]
                    // Load index i
                    self.emit_load_operand(code, index); // int
                    // aaload → entries[i] → Object
                    code.push(Instruction::Aaload);
                    // checkcast Object[]
                    code.push(Instruction::Checkcast(self.object_array_class_idx));
                    // Load field slot
                    self.emit_load_constant(code, &Constant::Int(field_slot as i64));
                    // Load value
                    self.emit_load_operand(code, value);
                    // Box if int, store reference
                    let vt = value.get_type();
                    if matches!(vt, IrType::String) {
                        code.push(Instruction::Aastore);
                    } else {
                        code.push(Instruction::Invokestatic(self.integer_value_of_ref));
                        code.push(Instruction::Aastore);
                    }
                } else {
                    // int[][] path: entries[i][field_slot] = value
                    let byte_off = if let IrOperand::Constant(Constant::Int(b)) = field_off { *b as usize } else { 0 };
                    let base_idx = byte_off / 4;
                    self.emit_load_operand(code, base); // int[][]
                    self.emit_load_operand(code, index); // i
                    code.push(Instruction::Aaload);      // entries[i] → int[]
                    if base_idx > 0 {
                        self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                        code.push(Instruction::Iadd);
                    }
                    // entries[i] has correct index on stack, load value
                    self.emit_load_operand(code, value);
                    code.push(Instruction::Iastore);
                }
                return;
            }
        }
        if inst.operands.len() >= 3 {
            // 3-operand store: [base, offset, value] → struct field write
            if inst.operands.len() == 3 {
                if let (Some(base), Some(offset), Some(value)) =
                    (inst.operands.first(), inst.operands.get(1), inst.operands.get(2))
                {
                    if let IrOperand::Constant(Constant::Int(byte_off)) = offset {
                        self.emit_load_operand(code, base);
                        if self.is_struct_var(base) {
                            self.emit_load_constant(code, &Constant::Int(byte_off / 4));
                            self.emit_load_operand(code, value);
                            self.emit_boxed_field_store(code, inst, *byte_off as usize);
                            return;
                        } else {
                            self.emit_load_constant(code, &Constant::Int(byte_off / 4));
                            self.emit_load_operand(code, value);
                            code.push(Instruction::Iastore);
                            return;
                        }
                    }
                }
            }
        }
    }

    fn generate_array_load(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(array), Some(index)) =
            (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            // 4-operand Load: [base, field_offset, element_index, elem_stride]
            // entries[i].key → operands: [entries, 0(key), i, 16]
            if inst.operands.len() >= 4 {
                if let Some(field_off) = inst.operands.get(1) {
                    if let IrOperand::Constant(Constant::Int(byte_off)) = field_off {
                        if let Some(idx_op) = inst.operands.get(2) {
                            if self.is_struct_var(array) {
                                // Object[] path: entries[i][field_slot]
                                let var_name = if let IrOperand::Variable(n, _) = array { n.clone() } else { String::new() };
                                let field_slot = self.get_field_slot_for_offset(&var_name, *byte_off as usize);
                                // Load entries array
                                self.emit_load_operand(code, array);  // Object[]
                                // Load index i
                                self.emit_load_operand(code, idx_op); // int
                                // aaload → entries[i] → Object
                                code.push(Instruction::Aaload);
                    // checkcast Object[]
                    code.push(Instruction::Checkcast(self.object_array_class_idx));
                    // Load field slot
                    self.emit_load_constant(code, &Constant::Int(field_slot as i64));
                    // aaload → entries[i][field] → Object
                    code.push(Instruction::Aaload);
                    // Unbox if needed
                    self.emit_boxed_field_load(code, inst, *byte_off as usize);
                                // Store result with correct type
                                let field_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                                self.emit_store_result(code, result, field_ty);
                            } else {
                                // int[][] path: entries[i][field_slot]
                                let base_idx = byte_off / 4;
                                self.emit_load_operand(code, array);  // int[][]
                                self.emit_load_operand(code, idx_op); // i
                                code.push(Instruction::Aaload);       // entries[i] → int[]
                                if base_idx > 0 {
                                    self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                                    code.push(Instruction::Iadd);
                                }
                                code.push(Instruction::Iaload);
                                self.emit_store_result(code, result, &IrType::Int);
                            }
                            return;
                        }
                    }
                }
            }
            // 2/3-operand: struct field or array access
            if inst.operands.len() <= 3 {
                if let IrOperand::Constant(Constant::Int(byte_off)) = index {
                    if *byte_off > 0 || self.is_struct_var(array) {
                        self.emit_load_operand(code, array);
                        let idx = byte_off / 4;
                        if self.is_struct_var(array) {
                            self.emit_load_constant(code, &Constant::Int(idx as i64));
                            code.push(Instruction::Aaload);
                            self.emit_boxed_field_load(code, inst, *byte_off as usize);
                        } else {
                            self.emit_load_constant(code, &Constant::Int(idx as i64));
                            code.push(Instruction::Iaload);
                        }
                        self.emit_store_result(code, result, &IrType::Int);
                        return;
                    }
                }
            }
            self.emit_load_operand(code, array);
            self.emit_load_operand(code, index);
            code.push(Instruction::Iaload);
            self.emit_store_result(code, result, &IrType::Int);
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
                code.push(Instruction::Dup); // dup env ref for storing
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

                code.push(Instruction::Aastore); // env[i] = wrapper
            }

            // After filling env array, stack = [env_array]

            // Check if we need to set __env on the lambda instance
            let lambda_name = if let IrOperand::FuncRef(name) = &inst.operands[0] {
                Some(name.clone())
            } else {
                None
            };

            let is_closure = lambda_name
                .as_ref()
                .is_some_and(|n| self.func_ref_env_field_refs.contains_key(n));

            if is_closure {
                let name = lambda_name.unwrap();
                let field_ref = self.func_ref_env_field_refs[&name];
                let instance_slot = self.func_ref_instance_slots[&name];

                // stack: env_array
                code.push(Instruction::Dup);
                // stack: env_array, env_array
                self.emit_store_result(code, result, &IrType::Array(Box::new(IrType::Int), 0));
                // stack: env_array (dup copy still on stack)

                // Load lambda instance
                match instance_slot {
                    0 => code.push(Instruction::Aload_0),
                    1 => code.push(Instruction::Aload_1),
                    2 => code.push(Instruction::Aload_2),
                    3 => code.push(Instruction::Aload_3),
                    _ => code.push(Instruction::Aload(instance_slot as u8)),
                }
                // stack: env_array, instance_ref

                code.push(Instruction::Swap);
                // stack: instance_ref, env_array

                code.push(Instruction::Putfield(field_ref));
                // stack: empty, instance.__env = env
            } else {
                self.emit_store_result(code, result, &IrType::Array(Box::new(IrType::Int), 0));
            }
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
                let store_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                self.emit_store_result(code, result, store_ty);
            }
        }
    }

    fn generate_call_indirect(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        // operands[0] = function variable (reference to functional interface instance)
        // operands[1..] = arguments
        if let Some(func_op) = inst.operands.first() {
            if let IrType::Function(params, ret) = func_op.get_type() {
                let iface_name = crate::codegen::jvm::types::get_fn_interface_name(&params, &ret);
                if let Some(&method_idx) = self.interface_method_refs.get(&iface_name) {
                    // Load the function reference (aload — it's stored as reference)
                    self.emit_load_operand(code, func_op);
                    // Load arguments
                    for arg in inst.operands.iter().skip(1) {
                        self.emit_load_operand(code, arg);
                    }
                    // invokeinterface: count = args + 1 (for `this`)
                    let count = (inst.operands.len()) as u8; // func_ref + args
                    code.push(Instruction::Invokeinterface(method_idx, count));
                    // Store result if any
                    if let Some(ref result) = inst.result {
                        let store_ty = if matches!(&*ret, IrType::String) { &*ret } else { &IrType::Int };
                        self.emit_store_result(code, result, store_ty);
                    }
                } else {
                    // Fallback: push dummy result
                    if inst.result.is_some() {
                        code.push(Instruction::Iconst_0);
                        if let Some(ref result) = inst.result {
                            let slot = self.get_local_slot(result);
                            code.push(Instruction::Istore(slot as u8));
                        }
                    }
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

            code.push(Instruction::Aaload); // [[I → [I  (wrapper array)
            code.push(Instruction::Iconst_0);
            code.push(Instruction::Iaload); // [I → int (captured value)

            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    fn generate_alloc_array(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(IrType::Array(_, size)) = inst.result_type.as_ref() {
                // Create new int[size] array
                self.emit_load_constant(code, &Constant::Int(*size as i64));
                code.push(Instruction::Newarray(ArrayType::Int));
                self.emit_store_result(code, result, &IrType::Array(Box::new(IrType::Int), *size));
            }
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

            code.push(Instruction::Aaload); // [[I → [I  (wrapper array)
            code.push(Instruction::Iconst_0);
            self.emit_load_operand(code, val_op); // value
            code.push(Instruction::Iastore); // wrapper[0] = value
        }
    }

    fn generate_str_get_byte(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(str_op), Some(idx_op)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            self.emit_load_operand(code, str_op);
            self.emit_load_operand(code, idx_op);
            code.push(Instruction::Baload);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    fn generate_str_set_byte(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(str_op), Some(idx_op), Some(val_op)) = (inst.operands.first(), inst.operands.get(1), inst.operands.get(2))
        {
            self.emit_load_operand(code, str_op);
            self.emit_load_operand(code, idx_op);
            self.emit_load_operand(code, val_op);
            code.push(Instruction::Bastore);
        }
    }

    fn generate_slice(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(base), Some(start)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            if matches!(base.get_type(), IrType::String) {
                self.emit_load_operand(code, base);
                self.emit_load_operand(code, start);
                if let Some(end) = inst.operands.get(2) {
                    self.emit_load_operand(code, end);
                } else {
                    code.push(Instruction::Iconst_m1);
                }
                if self.string_slice_ref != 0 {
                    code.push(Instruction::Invokestatic(self.string_slice_ref));
                }
                self.emit_store_result(code, result, &IrType::String);
            }
        }
    }

    fn generate_coro_yield(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            if let IrOperand::Constant(Constant::Int(state)) = operand {
                code.push(Instruction::Aload_0);
                code.push(Instruction::Iconst_m1);
                code.push(Instruction::Putfield(self.coroutine_state_field));
                code.push(Instruction::Aload_0);
                self.emit_load_constant(code, &Constant::Int(*state));
                code.push(Instruction::Putfield(self.coroutine_state_field));
                code.push(Instruction::Iconst_0);
                code.push(Instruction::Ireturn);
            }
        }
    }
}

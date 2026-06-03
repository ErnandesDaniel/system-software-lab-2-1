
use crate::codegen::jvm::types::{BinaryOp, ComparisonOp};
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{Constant, IrInstruction, IrOpcode, IrOperand, IrType};
use ristretto_classfile::attributes::{ArrayType, Instruction};

impl JvmGenerator {
    pub fn generate_instruction(&mut self, code: &mut Vec<Instruction>, inst: &IrInstruction, global_offset: u16) {
        match inst.opcode {
            IrOpcode::Assign => self.generate_assign(code, inst),
            IrOpcode::Add => self.generate_binary_op(code, inst, BinaryOp::Add),
            IrOpcode::Sub => self.generate_binary_op(code, inst, BinaryOp::Sub),
            IrOpcode::Mul => self.generate_binary_op(code, inst, BinaryOp::Mul),
            IrOpcode::Div => self.generate_binary_op(code, inst, BinaryOp::Div),
            IrOpcode::Mod => self.generate_binary_op(code, inst, BinaryOp::Mod),
            IrOpcode::Neg => self.generate_neg(code, inst),
            IrOpcode::And => self.generate_logical_and(code, inst, global_offset),
            IrOpcode::Or => self.generate_logical_or(code, inst, global_offset),
            IrOpcode::Not => self.generate_logical_not(code, inst, global_offset),
            IrOpcode::BitAnd => self.generate_binary_op(code, inst, BinaryOp::BitAnd),
            IrOpcode::BitOr => self.generate_binary_op(code, inst, BinaryOp::BitOr),
            IrOpcode::BitXor => self.generate_binary_op(code, inst, BinaryOp::BitXor),
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
            IrOpcode::Slice => self.generate_slice(code, inst),
            IrOpcode::Store => self.generate_store(code, inst),
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
            if self.closure.wrapped_vars.contains(result) {
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
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            // string + int → pointer arithmetic: slice from offset to end
            if op == BinaryOp::Add && left.get_type().is_pointer() {
                self.emit_load_operand(code, left);
                self.emit_load_operand(code, right);
                self.emit_load_operand(code, left);
                code.push(Instruction::Arraylength);
                if self.pool.string_slice_ref != 0 {
                    code.push(Instruction::Invokestatic(self.pool.string_slice_ref));
                }
                self.emit_store_result(code, result, &IrType::String);
                return;
            }

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
}



impl JvmGenerator {
    pub(super) fn generate_make_closure(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            let num_captures = inst.operands.len().saturating_sub(1);
            let anewarray_idx = self.pool.anewarray_int_class_idx.unwrap_or(0);

            self.emit_load_constant(code, &Constant::Int(num_captures as i64));
            code.push(Instruction::Anewarray(anewarray_idx));

            for (capture_idx, op) in inst.operands.iter().enumerate().skip(1) {
                code.push(Instruction::Dup);
                self.emit_load_constant(code, &Constant::Int((capture_idx - 1) as i64));

                if let IrOperand::Variable(name, _) = op {
                    if self.closure.wrapped_vars.contains(name) {
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

                code.push(Instruction::Aastore);
            }

            let lambda_name = if let IrOperand::FuncRef(name) = &inst.operands[0] {
                Some(name.clone())
            } else {
                None
            };

            let is_closure = lambda_name.as_ref().is_some_and(|n| self.pool.func_ref_env_field_refs.contains_key(n));

            if is_closure {
                let name = lambda_name.expect("Lambda name must exist for closure");
                let field_ref = self.pool.func_ref_env_field_refs[&name];
                let instance_slot = self.pool.func_ref_instance_slots[&name];

                code.push(Instruction::Dup);
                self.emit_store_result(code, result, &IrType::Array(Box::new(IrType::Int), 0));

                match instance_slot {
                    0 => code.push(Instruction::Aload_0),
                    1 => code.push(Instruction::Aload_1),
                    2 => code.push(Instruction::Aload_2),
                    3 => code.push(Instruction::Aload_3),
                    _ => code.push(Instruction::Aload(instance_slot as u8)),
                }

                code.push(Instruction::Swap);
                code.push(Instruction::Putfield(field_ref));
            } else {
                self.emit_store_result(code, result, &IrType::Array(Box::new(IrType::Int), 0));
            }
        }
    }

    pub(super) fn generate_call_closure(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(env_operand) = inst.operands.get(1) {
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

            for arg in inst.operands.iter().skip(2) {
                self.emit_load_operand(code, arg);
            }

            let lambda_name = if let IrOperand::Variable(env_name, _) = env_operand {
                self.closure.closure_targets.get(env_name).cloned()
            } else {
                None
            };

            if let Some(ref name) = lambda_name {
                let method_idx = self.pool.method_refs.get(name).copied().unwrap_or(1);
                code.push(Instruction::Invokestatic(method_idx));
            } else {
                code.push(Instruction::Nop);
            }

            if let Some(ref result) = inst.result {
                let store_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                self.emit_store_result(code, result, store_ty);
            }
        }
    }

    pub(super) fn generate_call_indirect(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(func_op) = inst.operands.first() {
            if let IrType::Function(params, ret) = func_op.get_type() {
                let iface_name = crate::codegen::jvm::types::get_fn_interface_name(&params, &ret);
                if let Some(&method_idx) = self.pool.interface_method_refs.get(&iface_name) {
                    self.emit_load_operand(code, func_op);
                    for arg in inst.operands.iter().skip(1) {
                        self.emit_load_operand(code, arg);
                    }
                    let count = inst.operands.len() as u8;
                    code.push(Instruction::Invokeinterface(method_idx, count));
                    if let Some(ref result) = inst.result {
                        self.emit_store_result(code, result, &ret);
                    }
                } else {
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

    pub(super) fn generate_load_captured(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
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

            code.push(Instruction::Aaload);
            code.push(Instruction::Iconst_0);
            code.push(Instruction::Iaload);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    pub(super) fn generate_store_captured(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
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

            code.push(Instruction::Aaload);
            code.push(Instruction::Iconst_0);
            self.emit_load_operand(code, val_op);
            code.push(Instruction::Iastore);
        }
    }
}



impl JvmGenerator {
    pub(super) fn generate_call(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref target) = inst.jump_target {
            for operand in &inst.operands {
                self.emit_load_operand(code, operand);
            }

            let param_types: String = inst.operands.iter().map(|o| crate::codegen::jvm::types::ir_type_to_jvm_descriptor(&o.get_type())).collect();
            let ret_desc = inst.result_type.as_ref().map_or("I".to_string(), crate::codegen::jvm::types::ir_type_to_jvm_descriptor);
            let key = format!("{target}|({param_types}){ret_desc}");
            let method_idx = self.pool.method_refs.get(&key).copied()
                .or_else(|| self.pool.method_refs.get(target).copied())
                .unwrap_or(1);
            code.push(Instruction::Invokestatic(method_idx));

            if let Some(ref result) = inst.result {
                let store_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                self.emit_store_result(code, result, store_ty);
            }
        }
    }

    pub(super) fn generate_return(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if self.coro.is_coroutine {
            if let Some(operand) = inst.operands.first() {
                self.emit_load_operand(code, operand);
                code.push(Instruction::Aload_0);
                code.push(Instruction::Swap);
                code.push(Instruction::Putfield(self.coro.coroutine_result_field));
            }
            code.push(Instruction::Aload_0);
            code.push(Instruction::Iconst_m1);
            code.push(Instruction::Putfield(self.coro.coroutine_state_field));
            code.push(Instruction::Iconst_1);
            code.push(Instruction::Ireturn);
        } else if let Some(operand) = inst.operands.first() {
            self.emit_load_operand(code, operand);
            match operand.get_type() {
                IrType::String | IrType::Function(_, _) | IrType::Array(_, _) => code.push(Instruction::Areturn),
                _ => code.push(Instruction::Ireturn),
            }
        } else {
            code.push(Instruction::Return);
        }
    }

    pub(super) fn generate_jump(&self, _code: &mut Vec<Instruction>, _inst: &IrInstruction) {}

    pub(super) fn generate_conditional_branch(&self, _code: &mut Vec<Instruction>, _inst: &IrInstruction) {}

    pub(super) fn generate_coro_yield(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(operand) = inst.operands.first() {
            if let IrOperand::Constant(Constant::Int(state)) = operand {
                code.push(Instruction::Aload_0);
                code.push(Instruction::Iconst_m1);
                code.push(Instruction::Putfield(self.coro.coroutine_state_field));
                code.push(Instruction::Aload_0);
                self.emit_load_constant(code, &Constant::Int(*state));
                code.push(Instruction::Putfield(self.coro.coroutine_state_field));
                code.push(Instruction::Iconst_0);
                code.push(Instruction::Ireturn);
            }
        }
    }
}



impl JvmGenerator {
    pub(super) fn generate_store(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if inst.operands.len() >= 4 {
            if let (Some(base), Some(field_off), Some(value), Some(index)) =
                (inst.operands.first(), inst.operands.get(1), inst.operands.get(2), inst.operands.get(3))
            {
                if self.is_struct_var(base) {
                    let byte_off = if let IrOperand::Constant(Constant::Int(b)) = field_off { *b as usize } else { 0 };
                    let var_name = if let IrOperand::Variable(n, _) = base { n.clone() } else { String::new() };
                    let field_slot = self.get_field_slot_for_offset(&var_name, byte_off);
                    self.emit_load_operand(code, base);
                    self.emit_load_operand(code, index);
                    code.push(Instruction::Aaload);
                    code.push(Instruction::Checkcast(self.pool.object_array_class_idx));
                    self.emit_load_constant(code, &Constant::Int(field_slot as i64));
                    self.emit_load_operand(code, value);
                    let vt = value.get_type();
                    if matches!(vt, IrType::String) {
                        code.push(Instruction::Aastore);
                    } else {
                        code.push(Instruction::Invokestatic(self.pool.integer_value_of_ref));
                        code.push(Instruction::Aastore);
                    }
                } else {
                    let vt = value.get_type();
                    let byte_off = if let IrOperand::Constant(Constant::Int(b)) = field_off { *b as usize } else { 0 };
                    if matches!(vt, IrType::Function(_, _) | IrType::String | IrType::Array(..)) {
                        self.emit_load_operand(code, base);
                        if byte_off > 0 {
                            let base_idx = byte_off / 4;
                            self.emit_load_operand(code, index);
                            self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                            code.push(Instruction::Iadd);
                        } else {
                            self.emit_load_operand(code, index);
                        }
                        self.emit_load_operand(code, value);
                        code.push(Instruction::Aastore);
                    } else if matches!(vt, IrType::Int | IrType::Bool) {
                        self.emit_load_operand(code, base);
                        self.emit_load_operand(code, index);
                        if byte_off > 0 {
                            let base_idx = byte_off / 4;
                            self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                            code.push(Instruction::Iadd);
                        }
                        self.emit_load_operand(code, value);
                        code.push(Instruction::Iastore);
                    } else {
                        self.emit_load_operand(code, base);
                        self.emit_load_operand(code, index);
                        code.push(Instruction::Aaload);
                        self.emit_load_operand(code, value);
                        code.push(Instruction::Iastore);
                    }
                }
                return;
            }
        }
        if inst.operands.len() >= 3 && inst.operands.len() == 3 {
            if let (Some(base), Some(offset), Some(value)) =
                (inst.operands.first(), inst.operands.get(1), inst.operands.get(2))
            {
                if let IrOperand::Constant(Constant::Int(byte_off)) = offset {
                    self.emit_load_operand(code, base);
                    if self.is_struct_var(base) {
                        self.emit_load_constant(code, &Constant::Int(byte_off / 4));
                        self.emit_load_operand(code, value);
                        self.emit_boxed_field_store(code, inst, *byte_off as usize);
                    } else {
                        let vt = value.get_type();
                        self.emit_load_constant(code, &Constant::Int(byte_off / 4));
                        self.emit_load_operand(code, value);
                        if matches!(vt, IrType::Function(_, _) | IrType::String | IrType::Array(..)) {
                            code.push(Instruction::Aastore);
                        } else {
                            code.push(Instruction::Iastore);
                        }
                    }
                }
            }
        }
    }

    pub(super) fn generate_array_load(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(op)) = (&inst.result, inst.operands.first()) {
            if inst.operands.len() == 1 {
                if let IrOperand::Variable(name, _) = op {
                    if let Some(&field_ref) = self.global.global_field_refs.get(name) {
                        let ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                        code.push(Instruction::Getstatic(field_ref));
                        self.emit_store_result(code, result, ty);
                        return;
                    }
                }
            }
        }

        if let (Some(ref result), Some(array), Some(index)) =
            (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            if inst.operands.len() >= 4 {
                if let Some(field_off) = inst.operands.get(1) {
                    if let IrOperand::Constant(Constant::Int(byte_off)) = field_off {
                        if let Some(idx_op) = inst.operands.get(2) {
                            if self.is_struct_var(array) {
                                let var_name = if let IrOperand::Variable(n, _) = array { n.clone() } else { String::new() };
                                let field_slot = self.get_field_slot_for_offset(&var_name, *byte_off as usize);
                                self.emit_load_operand(code, array);
                                self.emit_load_operand(code, idx_op);
                                code.push(Instruction::Aaload);
                                code.push(Instruction::Checkcast(self.pool.object_array_class_idx));
                                self.emit_load_constant(code, &Constant::Int(field_slot as i64));
                                code.push(Instruction::Aaload);
                                self.emit_boxed_field_load(code, inst, *byte_off as usize);
                                let field_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                                self.emit_store_result(code, result, field_ty);
                            } else {
                                let base_idx = byte_off / 4;
                                self.emit_load_operand(code, array);
                                self.emit_load_operand(code, idx_op);
                                code.push(Instruction::Aaload);
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
            if inst.operands.len() <= 3 {
                if let IrOperand::Constant(Constant::Int(byte_off)) = index {
                    let has_runtime_idx = inst.operands.len() >= 3;
                    if *byte_off > 0 || self.is_struct_var(array) || has_runtime_idx {
                        self.emit_load_operand(code, array);
                        let base_idx = byte_off / 4;
                        if self.is_struct_var(array) {
                            if has_runtime_idx {
                                self.emit_load_operand(code, inst.operands.get(2).unwrap());
                            } else {
                                self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                            }
                            code.push(Instruction::Aaload);
                            self.emit_boxed_field_load(code, inst, *byte_off as usize);
                        } else if has_runtime_idx {
                            if let Some(runtime_idx) = inst.operands.get(2) {
                                self.emit_load_operand(code, runtime_idx);
                                if base_idx > 0 {
                                    self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                                    code.push(Instruction::Iadd);
                                }
                                code.push(Instruction::Iaload);
                            }
                        } else {
                            let elem_type = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                            self.emit_load_constant(code, &Constant::Int(base_idx as i64));
                            if matches!(elem_type, IrType::Function(_, _) | IrType::String | IrType::Array(..)) {
                                code.push(Instruction::Aaload);
                            } else {
                                code.push(Instruction::Iaload);
                            }
                        }
                        let elem_type = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                        self.emit_store_result(code, result, &elem_type);
                        return;
                    }
                }
            }
            let elem_type = inst.result_type.as_ref().unwrap_or(&IrType::Int);
            self.emit_load_operand(code, array);
            self.emit_load_operand(code, index);
            if matches!(elem_type, IrType::Function(_, _) | IrType::String | IrType::Array(..)) {
                code.push(Instruction::Aaload);
            } else {
                code.push(Instruction::Iaload);
            }
            self.emit_store_result(code, result, &elem_type);
        }
    }

    pub(super) fn generate_slice(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(base), Some(start)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            if matches!(base.get_type(), IrType::String) {
                self.emit_load_operand(code, base);
                self.emit_load_operand(code, start);
                if let Some(end) = inst.operands.get(2) {
                    self.emit_load_operand(code, end);
                } else {
                    code.push(Instruction::Iconst_m1);
                }
                if self.pool.string_slice_ref != 0 {
                    code.push(Instruction::Invokestatic(self.pool.string_slice_ref));
                }
                self.emit_store_result(code, result, &IrType::String);
            }
        }
    }

    pub(super) fn generate_str_get_byte(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(ref result), Some(str_op), Some(idx_op)) = (&inst.result, inst.operands.first(), inst.operands.get(1)) {
            self.emit_load_operand(code, str_op);
            self.emit_load_operand(code, idx_op);
            code.push(Instruction::Baload);
            self.emit_store_result(code, result, &IrType::Int);
        }
    }

    pub(super) fn generate_str_set_byte(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let (Some(str_op), Some(idx_op), Some(val_op)) = (inst.operands.first(), inst.operands.get(1), inst.operands.get(2)) {
            self.emit_load_operand(code, str_op);
            self.emit_load_operand(code, idx_op);
            self.emit_load_operand(code, val_op);
            code.push(Instruction::Bastore);
        }
    }

    pub(super) fn generate_alloc_array(&mut self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref result) = inst.result {
            if let Some(IrType::Array(elem_type, size)) = inst.result_type.as_ref() {
                self.emit_load_constant(code, &Constant::Int(*size as i64));
                match elem_type.as_ref() {
                    IrType::Int | IrType::Bool => {
                        let at = if matches!(elem_type.as_ref(), IrType::Bool) { ArrayType::Boolean } else { ArrayType::Int };
                        code.push(Instruction::Newarray(at));
                    }
                    IrType::Function(_, _) | IrType::String | IrType::Array(..) => {
                        let desc = crate::codegen::jvm::types::ir_type_to_jvm_descriptor(elem_type);
                        let class_name = desc.trim_start_matches('L').trim_end_matches(';');
                        let class_idx = self.pool.constant_pool.add_class(class_name)
                            .expect("Failed to add class for anewarray");
                        code.push(Instruction::Anewarray(class_idx));
                    }
                    _ => {
                        code.push(Instruction::Newarray(ArrayType::Int));
                    }
                }
                self.emit_store_result(code, result, &IrType::Array(elem_type.clone(), *size));
            }
        }
    }
}



impl JvmGenerator {
    fn emit_if_icmpeq(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmpeq(target_idx));
    }

    fn emit_if_icmpne(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmpne(target_idx));
    }

    fn emit_if_icmplt(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmplt(target_idx));
    }

    fn emit_if_icmple(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmple(target_idx));
    }

    fn emit_if_icmpgt(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmpgt(target_idx));
    }

    fn emit_if_icmpge(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::If_icmpge(target_idx));
    }

    fn emit_goto(&self, code: &mut Vec<Instruction>, target_idx: u16) {
        code.push(Instruction::Goto(target_idx));
    }

    pub fn generate_logical_and(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, global_offset: u16) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            self.emit_load_operand(code, left);
            let first_ifeq = code.len();
            code.push(Instruction::Ifeq(0)); // placeholder

            self.emit_load_operand(code, right);
            let second_ifeq = code.len();
            code.push(Instruction::Ifeq(0)); // placeholder

            code.push(Instruction::Iconst_1);
            code.push(Instruction::Goto(0)); // placeholder

            let iconst_0_idx = u16::try_from(code.len()).expect("Logical: code length exceeds u16") + global_offset;
            code.push(Instruction::Iconst_0);
            let istore_idx = u16::try_from(code.len()).expect("Code length exceeds u16 limit") + global_offset;
            self.emit_store_result(code, result, &IrType::Int);

            code[first_ifeq] = Instruction::Ifeq(iconst_0_idx);
            code[second_ifeq] = Instruction::Ifeq(iconst_0_idx);
            code[second_ifeq + 2] = Instruction::Goto(istore_idx);
        }
    }

    pub fn generate_logical_or(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, global_offset: u16) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            self.emit_load_operand(code, left);
            let first_ifne = code.len();
            code.push(Instruction::Ifne(0)); // placeholder

            self.emit_load_operand(code, right);
            let second_ifne = code.len();
            code.push(Instruction::Ifne(0)); // placeholder

            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(0)); // placeholder

            let iconst_1_idx = u16::try_from(code.len()).expect("Code length exceeds u16 limit") + global_offset;
            code.push(Instruction::Iconst_1);
            let istore_idx = u16::try_from(code.len()).expect("Code length exceeds u16 limit") + global_offset;
            self.emit_store_result(code, result, &IrType::Int);

            code[first_ifne] = Instruction::Ifne(iconst_1_idx);
            code[second_ifne] = Instruction::Ifne(iconst_1_idx);
            code[second_ifne + 2] = Instruction::Goto(istore_idx);
        }
    }

    pub fn generate_logical_not(&self, code: &mut Vec<Instruction>, inst: &IrInstruction, global_offset: u16) {
        if let (Some(ref result), Some(operand)) = (&inst.result, inst.operands.first()) {
            self.emit_load_operand(code, operand);
            let start_idx = code.len();
            code.push(Instruction::Ifeq(0)); // placeholder
            code.push(Instruction::Iconst_0);
            code.push(Instruction::Goto(0)); // placeholder

            let iconst_1_idx = u16::try_from(code.len()).expect("Code length exceeds u16 limit") + global_offset;
            code.push(Instruction::Iconst_1);
            let istore_idx = u16::try_from(code.len()).expect("Code length exceeds u16 limit") + global_offset;
            self.emit_store_result(code, result, &IrType::Int);

            code[start_idx] = Instruction::Ifeq(iconst_1_idx);
            code[start_idx + 2] = Instruction::Goto(istore_idx);
        }
    }

    pub fn generate_comparison(
        &self,
        code: &mut Vec<Instruction>,
        inst: &IrInstruction,
        op: ComparisonOp,
        global_offset: u16,
    ) {
        if let (Some(ref result), Some(left), Some(right)) = (&inst.result, inst.operands.first(), inst.operands.get(1))
        {
            let left_is_ref = matches!(left.get_type(), IrType::String | IrType::Function(_, _) | IrType::Array(..));
            let right_is_ref = matches!(right.get_type(), IrType::String | IrType::Function(_, _) | IrType::Array(..));
            let both_int = !left_is_ref && !right_is_ref;

            if both_int {
                self.emit_load_operand(code, left);
                self.emit_load_operand(code, right);

                let start_idx = code.len();
                let iconst_1_idx = u16::try_from(start_idx + 3).expect("Code offset exceeds u16") + global_offset;
                let istore_idx = u16::try_from(start_idx + 4).expect("Code offset exceeds u16") + global_offset;

                match op {
                    ComparisonOp::Eq => self.emit_if_icmpeq(code, iconst_1_idx),
                    ComparisonOp::Ne => self.emit_if_icmpne(code, iconst_1_idx),
                    ComparisonOp::Lt => self.emit_if_icmplt(code, iconst_1_idx),
                    ComparisonOp::Le => self.emit_if_icmple(code, iconst_1_idx),
                    ComparisonOp::Gt => self.emit_if_icmpgt(code, iconst_1_idx),
                    ComparisonOp::Ge => self.emit_if_icmpge(code, iconst_1_idx),
                }

                code.push(Instruction::Iconst_0);
                self.emit_goto(code, istore_idx);

                code.push(Instruction::Iconst_1);
            } else if left_is_ref && right_is_ref {
                self.emit_load_operand(code, left);
                self.emit_load_operand(code, right);

                let start_idx = code.len();
                let iconst_1_idx = u16::try_from(start_idx + 3).expect("Code offset exceeds u16") + global_offset;
                let istore_idx = u16::try_from(start_idx + 4).expect("Code offset exceeds u16") + global_offset;

                match op {
                    ComparisonOp::Eq => code.push(Instruction::If_acmpeq(iconst_1_idx)),
                    ComparisonOp::Ne => code.push(Instruction::If_acmpne(iconst_1_idx)),
                    _ => {}
                }

                code.push(Instruction::Iconst_0);
                self.emit_goto(code, istore_idx);

                code.push(Instruction::Iconst_1);
            } else {
                // reference vs int (null check: ref == 0)
                let ref_operand = if left_is_ref { left } else { right };
                self.emit_load_operand(code, ref_operand);

                let start_idx = code.len();
                let iconst_1_idx = u16::try_from(start_idx + 3).expect("Code offset exceeds u16") + global_offset;
                let istore_idx = u16::try_from(start_idx + 4).expect("Code offset exceeds u16") + global_offset;

                match op {
                    ComparisonOp::Eq => code.push(Instruction::Ifnull(iconst_1_idx)),
                    ComparisonOp::Ne => code.push(Instruction::Ifnonnull(iconst_1_idx)),
                    _ => {}
                }

                code.push(Instruction::Iconst_0);
                self.emit_goto(code, istore_idx);

                code.push(Instruction::Iconst_1);
            }

            self.emit_store_result(code, result, &IrType::Int);
        }
    }
}

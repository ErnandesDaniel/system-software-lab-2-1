use crate::codegen::jvm::types;
use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{Constant, IrInstruction, IrOperand, IrType};
use ristretto_classfile::attributes::Instruction;

impl JvmGenerator {
    pub(super) fn generate_call(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref target) = inst.jump_target {
            let (key, param_jvm_types) = if types::is_external_function(target) {
                let desc = types::get_method_descriptor(target);
                let params = types::parse_method_descriptor_params(&desc);
                (format!("{target}|{desc}"), params)
            } else {
                let param_types: Vec<IrType> = inst.operands.iter().map(|o| o.get_type()).collect();
                let param_desc: String = param_types.iter().map(|t| types::ir_type_to_jvm_descriptor(t)).collect();
                let ret_desc = inst
                    .result_type
                    .as_ref()
                    .map_or("V".to_string(), |t| types::ir_type_to_jvm_descriptor(t));
                (format!("{target}|({param_desc}){ret_desc}"), Vec::new())
            };

            for (i, operand) in inst.operands.iter().enumerate() {
                if types::is_external_function(target) && i < param_jvm_types.len() {
                    let jvm_desc = &param_jvm_types[i];
                    // Load operand using the JVM descriptor type (may differ from IR type)
                    self.emit_load_operand_as(code, operand, jvm_desc);
                } else {
                    self.emit_load_operand(code, operand);
                }
            }

            let method_idx = self
                .pool
                .method_refs
                .get(&key)
                .copied()
                .or_else(|| self.pool.method_refs.get(target).copied())
                .unwrap_or(1);
            code.push(Instruction::Invokestatic(method_idx));

            if let Some(ref result) = inst.result {
                if types::is_external_function(target) {
                    let desc = types::get_method_descriptor(target);
                    let ret = types::parse_method_descriptor_return(&desc);
                    let store_ty = if ret == "[B" || ret.starts_with("[") || ret.starts_with("L") {
                        IrType::String
                    } else {
                        IrType::Int
                    };
                    self.emit_store_result(code, result, &store_ty);
                } else {
                    let store_ty = inst.result_type.as_ref().unwrap_or(&IrType::Int);
                    self.emit_store_result(code, result, store_ty);
                }
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

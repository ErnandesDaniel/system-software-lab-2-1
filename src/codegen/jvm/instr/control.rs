use crate::codegen::jvm::JvmGenerator;
use crate::ir::types::{Constant, IrInstruction, IrOperand, IrType};
use ristretto_classfile::attributes::Instruction;

impl JvmGenerator {
    pub(super) fn generate_call(&self, code: &mut Vec<Instruction>, inst: &IrInstruction) {
        if let Some(ref target) = inst.jump_target {
            for operand in &inst.operands {
                self.emit_load_operand(code, operand);
            }

            let param_types: String = inst.operands.iter().map(|o| crate::codegen::jvm::types::ir_type_to_jvm_descriptor(&o.get_type())).collect();
            let ret_desc = inst.result_type.as_ref().map_or("V".to_string(), crate::codegen::jvm::types::ir_type_to_jvm_descriptor);
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

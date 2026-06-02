use crate::codegen::jvm::{JvmGenerator, JvmInst, JumpPlaceholder};
use crate::codegen::traits;
use crate::ir::types::{IrBlock, IrFunction, IrInstruction, IrOpcode, IrOperand, IrType};
use ristretto_classfile::attributes::Instruction;
use std::collections::{HashMap, HashSet};

impl JvmGenerator {
    pub fn generate_bytecode(&mut self, func: &IrFunction) -> Vec<Instruction> {
        let mut instructions: Vec<JvmInst> = Vec::new();
        let mut block_to_inst_idx: HashMap<String, usize> = HashMap::new();

        if self.coro.is_coroutine {
            let coro_handler_set: std::collections::HashSet<&str> = func
                .coroutine_blocks
                .iter()
                .map(|s| s.as_str())
                .collect();
            let entry_block_id = func
                .blocks
                .iter()
                .find(|b| !coro_handler_set.contains(b.id.as_str()))
                .map(|b| b.id.clone())
                .unwrap_or_else(|| func.blocks[0].id.clone());
            instructions.push(JvmInst::Real(Instruction::Aload_0));
            instructions.push(JvmInst::Real(Instruction::Getfield(self.coro.coroutine_state_field)));
            instructions.push(JvmInst::Placeholder(JumpPlaceholder::Ifeq {
                block_id: entry_block_id,
            }));
            for state_idx in 1..=func.yield_count {
                if let Some(block_id) = func.coroutine_blocks.get(state_idx) {
                    instructions.push(JvmInst::Real(Instruction::Aload_0));
                    instructions.push(JvmInst::Real(Instruction::Getfield(self.coro.coroutine_state_field)));
                    instructions.push(JvmInst::Real(Instruction::Bipush(state_idx as i8)));
                    instructions.push(JvmInst::Placeholder(JumpPlaceholder::IfIcmpeq {
                        block_id: block_id.clone(),
                    }));
                }
            }
        }

        if !self.coro.is_coroutine {
            let string_slots = collect_string_slots(self, func);
            let env_slot_nums: HashSet<u16> = self
                .closure.env_vars
                .iter()
                .filter_map(|name| self.func.locals.get(name))
                .copied()
                .collect();
            let wrapped_slot_nums: HashSet<u16> = self
                .closure.wrapped_vars
                .iter()
                .filter_map(|name| self.func.locals.get(name))
                .copied()
                .collect();
            let fn_slot_nums: HashSet<u16> = func
                .locals
                .iter()
                .filter(|l| matches!(l.ty, IrType::Function(_, _)))
                .filter_map(|l| self.func.locals.get(&l.name))
                .copied()
                .collect();
            let array_ref_slot_nums: HashSet<u16> = func
                .locals
                .iter()
                .filter(|l| matches!(&l.ty, IrType::Array(et, _) if !matches!(**et, IrType::Int)))
                .filter_map(|l| self.func.locals.get(&l.name))
                .copied()
                .collect();
            let struct_slot_nums: HashSet<u16> = func
                .locals
                .iter()
                .filter(|l| matches!(&l.ty, IrType::Array(et, _) if **et == IrType::Int))
                .filter_map(|l| self.func.locals.get(&l.name))
                .copied()
                .collect();
            let mut temp_ref_slots: HashSet<u16> = HashSet::new();
            for block in &func.blocks {
                for inst in &block.instructions {
                    if let Some(ref result) = inst.result {
                        if let Some(ref ty) = inst.result_type {
                            if is_ref_type(ty) {
                                if let Some(&slot) = self.func.locals.get(result) {
                                    temp_ref_slots.insert(slot);
                                }
                            }
                        }
                    }
                    for op in &inst.operands {
                        if let IrOperand::Variable(name, ty) = op {
                            if is_ref_type(ty) && traits::is_temp(name) {
                                if let Some(&slot) = self.func.locals.get(name) {
                                    temp_ref_slots.insert(slot);
                                }
                            }
                        }
                    }
                }
            }
            let num_params = func.parameters.len() as u16;
            for slot in num_params..self.func.next_local_slot {
                if self.func.locals.values().any(|&s| s == slot) {
                    if string_slots.contains(&slot) || env_slot_nums.contains(&slot) || fn_slot_nums.contains(&slot) || array_ref_slot_nums.contains(&slot) || temp_ref_slots.contains(&slot) {
                        instructions.push(JvmInst::Real(Instruction::Aconst_null));
                        instructions.push(JvmInst::Real(Instruction::Astore(slot as u8)));
                    } else if wrapped_slot_nums.contains(&slot) {
                        instructions.push(JvmInst::Real(Instruction::Iconst_1));
                        instructions.push(JvmInst::Real(Instruction::Newarray(
                            ristretto_classfile::attributes::ArrayType::Int,
                        )));
                        instructions.push(JvmInst::Real(Instruction::Astore(slot as u8)));
                    } else if struct_slot_nums.contains(&slot) {
                        let name = func.locals.iter().find(|l| self.func.locals.get(&l.name) == Some(&slot)).map(|l| l.name.clone());
                        if name.as_ref().is_some_and(|n| self.st.struct_uses_object_array.contains(n)) {
                            let arr_size = func.locals.iter()
                                .find(|l| self.func.locals.get(&l.name) == Some(&slot))
                                .map_or(1, |l| match &l.ty {
                                    IrType::Array(_, size) => *size as u8,
                                    _ => 1,
                                });
                            instructions.push(JvmInst::Real(Instruction::Bipush(arr_size as i8)));
                            instructions.push(JvmInst::Real(Instruction::Anewarray(self.pool.object_class_idx)));
                            instructions.push(JvmInst::Real(Instruction::Astore(slot as u8)));
                        } else {
                            let arr_size = func.locals.iter()
                                .find(|l| self.func.locals.get(&l.name) == Some(&slot))
                                .map_or(1, |l| match &l.ty {
                                    IrType::Array(_, size) => *size as u8,
                                    _ => 1,
                                });
                            instructions.push(JvmInst::Real(Instruction::Bipush(arr_size as i8)));
                            instructions.push(JvmInst::Real(Instruction::Newarray(
                                ristretto_classfile::attributes::ArrayType::Int,
                            )));
                            instructions.push(JvmInst::Real(Instruction::Astore(slot as u8)));
                        }
                    } else {
                        instructions.push(JvmInst::Real(Instruction::Iconst_0));
                        instructions.push(JvmInst::Real(Instruction::Istore(slot as u8)));
                    }
                }
            }
        }

        let ordered_blocks = Self::reorder_blocks_for_jvm(&func.blocks);

        let mut inst_idx = instructions.len();
        for block in &ordered_blocks {
            block_to_inst_idx.insert(block.id.clone(), inst_idx);

            for ir_inst in &block.instructions {
                let jvm_insts = self.generate_instruction_with_placeholders(ir_inst, inst_idx as u16);
                inst_idx += jvm_insts.len();
                instructions.extend(jvm_insts);
            }
        }

        let block_inst_indices: HashMap<String, u16> = block_to_inst_idx
            .iter()
            .map(|(id, &idx)| (id.clone(), idx as u16))
            .collect();

        let result: Vec<Instruction> = instructions
            .into_iter()
            .map(|jvm_inst| match jvm_inst {
                JvmInst::Real(instr) => instr,
                JvmInst::Placeholder(p) => {
                    let target_block = match &p {
                        JumpPlaceholder::Goto { block_id }
                        | JumpPlaceholder::Ifne { block_id }
                        | JumpPlaceholder::Ifeq { block_id }
                        | JumpPlaceholder::IfIcmpeq { block_id } => block_id,
                    };

                    let target_idx = block_inst_indices.get(target_block).copied().unwrap_or(0);
                    match &p {
                        JumpPlaceholder::Goto { .. } => Instruction::Goto(target_idx),
                        JumpPlaceholder::Ifne { .. } => Instruction::Ifne(target_idx),
                        JumpPlaceholder::Ifeq { .. } => Instruction::Ifeq(target_idx),
                        JumpPlaceholder::IfIcmpeq { .. } => Instruction::If_icmpeq(target_idx),
                    }
                }
            })
            .collect();

        let total = result.len() as u16;
        let has_out_of_bounds = block_inst_indices.values().any(|&idx| idx >= total);
        if has_out_of_bounds {
            let mut extended = result;
            extended.push(Instruction::Nop);
            extended
        } else {
            result
        }
    }

    fn generate_instruction_with_placeholders(&mut self, inst: &IrInstruction, global_offset: u16) -> Vec<JvmInst> {
        let mut code: Vec<Instruction> = Vec::new();

        self.generate_instruction(&mut code, inst, global_offset);

        match inst.opcode {
            IrOpcode::Jump => {
                if let Some(ref target) = inst.jump_target {
                    vec![JvmInst::Placeholder(JumpPlaceholder::Goto {
                        block_id: target.clone(),
                    })]
                } else {
                    vec![JvmInst::Real(Instruction::Nop)]
                }
            }
            IrOpcode::CondBr => {
                if let (Some(ref true_target), Some(ref false_target)) = (&inst.true_target, &inst.false_target) {
                    if let Some(operand) = inst.operands.first() {
                        self.emit_load_operand(&mut code, operand);
                    }
                    code.into_iter()
                        .map(JvmInst::Real)
                        .chain(vec![
                            JvmInst::Placeholder(JumpPlaceholder::Ifeq {
                                block_id: false_target.clone(),
                            }),
                            JvmInst::Placeholder(JumpPlaceholder::Goto {
                                block_id: true_target.clone(),
                            }),
                        ])
                        .collect()
                } else if let Some(ref target) = inst.jump_target {
                    if let Some(operand) = inst.operands.first() {
                        self.emit_load_operand(&mut code, operand);
                    }
                    code.into_iter()
                        .map(JvmInst::Real)
                        .chain(vec![JvmInst::Placeholder(JumpPlaceholder::Ifne {
                            block_id: target.clone(),
                        })])
                        .collect()
                } else {
                    code.into_iter().map(JvmInst::Real).collect()
                }
            }
            _ => code.into_iter().map(JvmInst::Real).collect(),
        }
    }

    fn reorder_blocks_for_jvm<'a>(blocks: &'a [IrBlock]) -> Vec<&'a IrBlock> {
        if blocks.is_empty() {
            return Vec::new();
        }

        let block_map: HashMap<String, &IrBlock> = blocks.iter().map(|b| (b.id.clone(), b)).collect();
        let mut referenced = HashSet::new();
        for block in blocks {
            for succ in &block.successors {
                referenced.insert(succ.clone());
            }
        }

        let entry_idx = blocks.iter().position(|b| !referenced.contains(&b.id)).unwrap_or(0);
        let mut seen = HashSet::new();
        let mut result = Vec::new();

        let mut dfs_stack = vec![&blocks[entry_idx]];
        while let Some(b) = dfs_stack.pop() {
            if seen.insert(b.id.clone()) {
                result.push(b);
                for succ_id in b.successors.iter().rev() {
                    if !seen.contains(succ_id) {
                        if let Some(succ) = block_map.get(succ_id) {
                            dfs_stack.push(succ);
                        }
                    }
                }
            }
        }

        for b in blocks {
            if !seen.contains(&b.id) {
                result.push(b);
            }
        }

        result
    }
}

fn is_ref_type(ty: &IrType) -> bool {
    match ty {
        IrType::String | IrType::Function(_, _) => true,
        IrType::Array(et, _) => !matches!(et.as_ref(), IrType::Int),
        _ => false,
    }
}

fn collect_string_slots(jg: &JvmGenerator, func: &IrFunction) -> Vec<u16> {
    let mut slots = Vec::new();
    for param in &func.parameters {
        if param.ty == IrType::String {
            if let Some(&slot) = jg.func.locals.get(&param.name) {
                slots.push(slot);
            }
        }
    }
    for local in &func.locals {
        if local.ty == IrType::String {
            if let Some(&slot) = jg.func.locals.get(&local.name) {
                if !slots.contains(&slot) {
                    slots.push(slot);
                }
            }
        }
    }
    for block in &func.blocks {
        for inst in &block.instructions {
            for op in &inst.operands {
                if let IrOperand::Variable(name, ty) = op {
                    if *ty == IrType::String {
                        if let Some(&slot) = jg.func.locals.get(name) {
                            if !slots.contains(&slot) {
                                slots.push(slot);
                            }
                        }
                    }
                }
            }
            if let Some(ref result) = inst.result {
                if Some(IrType::String) == inst.result_type
                    || inst.operands.first().is_some_and(|op| op.get_type() == IrType::String)
                {
                    if let Some(&slot) = jg.func.locals.get(result) {
                        if !slots.contains(&slot) {
                            slots.push(slot);
                        }
                    }
                }
            }
        }
    }
    slots.sort_unstable();
    slots.dedup();
    slots
}

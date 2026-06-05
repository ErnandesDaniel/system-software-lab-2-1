use crate::codegen::jvm::{JumpPlaceholder, JvmGenerator, JvmInst};
use crate::codegen::traits;
use crate::ir::types::{IrBlock, IrFunction, IrInstruction, IrOpcode, IrOperand, IrType};
use ristretto_classfile::attributes::Instruction;
use std::collections::{HashMap, HashSet};

impl JvmGenerator {
    pub fn generate_bytecode(&mut self, func: &IrFunction) -> Vec<Instruction> {
        let mut insts: Vec<JvmInst> = Vec::new();
        let mut starts: HashMap<String, usize> = HashMap::new();

        if self.coro.is_coroutine {
            self.emit_coro_entry(func, &mut insts);
        } else {
            self.emit_init_prologue(func, &mut insts);
        }

        let ordered = Self::reorder(&func.blocks);
        let mut idx = insts.len();
        for b in &ordered {
            starts.insert(b.id.clone(), idx);
            for ir in &b.instructions {
                let jv = self.inst_to_jvm(ir, idx as u16);
                idx += jv.len();
                insts.extend(jv);
            }
        }

        let targets: HashMap<String, u16> = starts.iter().map(|(k, &v)| (k.clone(), v as u16)).collect();
        let mut flat: Vec<Instruction> = insts.into_iter().map(|j| self.resolve_jump(j, &targets)).collect();
        self.ensure_trailing_return(func, &mut flat);

        if targets.values().any(|&i| i >= flat.len() as u16) {
            flat.push(Instruction::Nop);
        }
        flat
    }

    fn emit_coro_entry(&self, func: &IrFunction, insts: &mut Vec<JvmInst>) {
        let entry = func.blocks[0].id.clone();
        let _cset: HashSet<&str> = func.coroutine_blocks.iter().skip(1).map(|s| s.as_str()).collect();
        insts.push(JvmInst::Real(Instruction::Aload_0));
        insts.push(JvmInst::Real(Instruction::Getfield(self.coro.coroutine_state_field)));
        insts.push(JvmInst::Placeholder(JumpPlaceholder::Ifeq { block_id: entry }));
        for s in 1..=func.yield_count {
            if let Some(bid) = func.coroutine_blocks.get(s) {
                insts.push(JvmInst::Real(Instruction::Aload_0));
                insts.push(JvmInst::Real(Instruction::Getfield(self.coro.coroutine_state_field)));
                insts.push(JvmInst::Real(Instruction::Bipush(s as i8)));
                insts.push(JvmInst::Placeholder(JumpPlaceholder::IfIcmpeq {
                    block_id: bid.clone(),
                }));
            }
        }
    }

    fn emit_init_prologue(&self, func: &IrFunction, insts: &mut Vec<JvmInst>) {
        let np = func.parameters.len() as u16;
        let strs = Self::collect_str_slots(self, func);
        let str_set: HashSet<u16> = strs.iter().copied().collect();
        let envs: HashSet<u16> = self
            .closure
            .env_vars
            .iter()
            .filter_map(|n| self.func.locals.get(n))
            .copied()
            .collect();
        let wraps: HashSet<u16> = self
            .closure
            .wrapped_vars
            .iter()
            .filter_map(|n| self.func.locals.get(n))
            .copied()
            .collect();
        let fn_s: HashSet<u16> = func
            .locals
            .iter()
            .filter(|l| matches!(l.ty, IrType::Function(_, _)))
            .filter_map(|l| self.func.locals.get(&l.name))
            .copied()
            .collect();
        let arr_ref: HashSet<u16> = func
            .locals
            .iter()
            .filter(|l| matches!(&l.ty, IrType::Array(et, _) if !matches!(**et, IrType::Int)))
            .filter_map(|l| self.func.locals.get(&l.name))
            .copied()
            .collect();
        let arr_int: HashSet<u16> = func
            .locals
            .iter()
            .filter(|l| matches!(&l.ty, IrType::Array(et, _) if **et == IrType::Int))
            .filter_map(|l| self.func.locals.get(&l.name))
            .copied()
            .collect();
        let mut tref: HashSet<u16> = HashSet::new();
        for b in &func.blocks {
            for i in &b.instructions {
                if let Some(ref r) = i.result {
                    if let Some(ref t) = i.result_type {
                        if Self::is_jvm_ref(t) {
                            if let Some(&s) = self.func.locals.get(r) {
                                tref.insert(s);
                            }
                        }
                    }
                }
                for o in &i.operands {
                    if let IrOperand::Variable(n, t) = o {
                        if Self::is_jvm_ref(t) && traits::is_temp(n) {
                            if let Some(&s) = self.func.locals.get(n) {
                                tref.insert(s);
                            }
                        }
                    }
                }
            }
        }
        for slot in np..self.func.next_local_slot {
            if !self.func.locals.values().any(|&s| s == slot) {
                continue;
            }
            let init = Self::pick_init(
                self, slot, &str_set, &envs, &wraps, &fn_s, &arr_ref, &arr_int, &tref, func,
            );
            insts.extend(init);
        }
    }

    fn pick_init(
        _jg: &JvmGenerator,
        slot: u16,
        strs: &HashSet<u16>,
        envs: &HashSet<u16>,
        wraps: &HashSet<u16>,
        fn_s: &HashSet<u16>,
        arr_ref: &HashSet<u16>,
        arr_int: &HashSet<u16>,
        tref: &HashSet<u16>,
        func: &IrFunction,
    ) -> Vec<JvmInst> {
        if strs.contains(&slot)
            || envs.contains(&slot)
            || fn_s.contains(&slot)
            || arr_ref.contains(&slot)
            || tref.contains(&slot)
        {
            vec![
                JvmInst::Real(Instruction::Aconst_null),
                JvmInst::Real(Instruction::Astore(slot as u8)),
            ]
        } else if wraps.contains(&slot) {
            vec![
                JvmInst::Real(Instruction::Iconst_1),
                JvmInst::Real(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Int)),
                JvmInst::Real(Instruction::Astore(slot as u8)),
            ]
        } else if arr_int.contains(&slot) {
            let sz = func
                .locals
                .iter()
                .find(|l| _jg.func.locals.get(&l.name) == Some(&slot))
                .map_or(1, |l| match &l.ty {
                    IrType::Array(_, s) => *s as u8,
                    _ => 1,
                });
            let nm = func
                .locals
                .iter()
                .find(|l| _jg.func.locals.get(&l.name) == Some(&slot))
                .map(|l| l.name.clone());
            if nm.as_ref().is_some_and(|n| _jg.st.struct_uses_object_array.contains(n)) {
                vec![
                    JvmInst::Real(Instruction::Bipush(sz as i8)),
                    JvmInst::Real(Instruction::Anewarray(_jg.pool.object_class_idx)),
                    JvmInst::Real(Instruction::Astore(slot as u8)),
                ]
            } else {
                vec![
                    JvmInst::Real(Instruction::Bipush(sz as i8)),
                    JvmInst::Real(Instruction::Newarray(ristretto_classfile::attributes::ArrayType::Int)),
                    JvmInst::Real(Instruction::Astore(slot as u8)),
                ]
            }
        } else {
            vec![
                JvmInst::Real(Instruction::Iconst_0),
                JvmInst::Real(Instruction::Istore(slot as u8)),
            ]
        }
    }

    fn collect_str_slots(jg: &JvmGenerator, func: &IrFunction) -> Vec<u16> {
        let mut v = Vec::new();
        for p in &func.parameters {
            if p.ty == IrType::String {
                if let Some(&s) = jg.func.locals.get(&p.name) {
                    v.push(s);
                }
            }
        }
        for l in &func.locals {
            if l.ty == IrType::String {
                if let Some(&s) = jg.func.locals.get(&l.name) {
                    if !v.contains(&s) {
                        v.push(s);
                    }
                }
            }
        }
        for b in &func.blocks {
            for i in &b.instructions {
                for o in &i.operands {
                    if let IrOperand::Variable(n, t) = o {
                        if *t == IrType::String {
                            if let Some(&s) = jg.func.locals.get(n) {
                                if !v.contains(&s) {
                                    v.push(s);
                                }
                            }
                        }
                    }
                }
                if let Some(ref r) = i.result {
                    if Some(IrType::String) == i.result_type
                        || i.operands.first().is_some_and(|o| o.get_type() == IrType::String)
                    {
                        if let Some(&s) = jg.func.locals.get(r) {
                            if !v.contains(&s) {
                                v.push(s);
                            }
                        }
                    }
                }
            }
        }
        v.sort_unstable();
        v.dedup();
        v
    }

    fn is_jvm_ref(ty: &IrType) -> bool {
        matches!(ty, IrType::String | IrType::Function(_, _))
            || matches!(ty, IrType::Array(et, _) if !matches!(et.as_ref(), IrType::Int))
    }

    fn inst_to_jvm(&mut self, inst: &IrInstruction, offset: u16) -> Vec<JvmInst> {
        let mut code: Vec<Instruction> = Vec::new();
        self.generate_instruction(&mut code, inst, offset);
        match inst.opcode {
            IrOpcode::Jump => vec![JvmInst::Placeholder(JumpPlaceholder::Goto {
                block_id: inst.jump_target.clone().unwrap_or_default(),
            })],
            IrOpcode::CondBr => {
                if let (Some(tt), Some(ft)) = (&inst.true_target, &inst.false_target) {
                    if let Some(op) = inst.operands.first() {
                        self.emit_load_operand(&mut code, op);
                    }
                    code.into_iter()
                        .map(JvmInst::Real)
                        .chain(vec![
                            JvmInst::Placeholder(JumpPlaceholder::Ifeq { block_id: ft.clone() }),
                            JvmInst::Placeholder(JumpPlaceholder::Goto { block_id: tt.clone() }),
                        ])
                        .collect()
                } else if let Some(t) = &inst.jump_target {
                    if let Some(op) = inst.operands.first() {
                        self.emit_load_operand(&mut code, op);
                    }
                    code.into_iter()
                        .map(JvmInst::Real)
                        .chain(vec![JvmInst::Placeholder(JumpPlaceholder::Ifne {
                            block_id: t.clone(),
                        })])
                        .collect()
                } else {
                    code.into_iter().map(JvmInst::Real).collect()
                }
            }
            _ => code.into_iter().map(JvmInst::Real).collect(),
        }
    }

    fn resolve_jump(&self, jvm: JvmInst, targets: &HashMap<String, u16>) -> Instruction {
        match jvm {
            JvmInst::Real(i) => i,
            JvmInst::Placeholder(p) => {
                let bid = match &p {
                    JumpPlaceholder::Goto { block_id }
                    | JumpPlaceholder::Ifne { block_id }
                    | JumpPlaceholder::Ifeq { block_id }
                    | JumpPlaceholder::IfIcmpeq { block_id } => block_id,
                };
                let idx = targets.get(bid).copied().unwrap_or(0);
                match p {
                    JumpPlaceholder::Goto { .. } => Instruction::Goto(idx),
                    JumpPlaceholder::Ifne { .. } => Instruction::Ifne(idx),
                    JumpPlaceholder::Ifeq { .. } => Instruction::Ifeq(idx),
                    JumpPlaceholder::IfIcmpeq { .. } => Instruction::If_icmpeq(idx),
                }
            }
        }
    }

    fn ensure_trailing_return(&self, func: &IrFunction, code: &mut Vec<Instruction>) {
        if code
            .last()
            .is_some_and(|i| matches!(i, Instruction::Return | Instruction::Ireturn | Instruction::Areturn))
        {
            return;
        }
        match &func.return_type {
            IrType::Void => code.push(Instruction::Return),
            IrType::String | IrType::Function(_, _) | IrType::Array(..) => {
                code.push(Instruction::Aconst_null);
                code.push(Instruction::Areturn);
            }
            _ => {
                code.push(Instruction::Iconst_0);
                code.push(Instruction::Ireturn);
            }
        }
    }

    fn reorder<'a>(blocks: &'a [IrBlock]) -> Vec<&'a IrBlock> {
        if blocks.is_empty() {
            return vec![];
        }
        let map: HashMap<String, &IrBlock> = blocks.iter().map(|b| (b.id.clone(), b)).collect();
        let refd: HashSet<String> = blocks.iter().flat_map(|b| b.successors.iter().cloned()).collect();
        let entry = blocks.iter().position(|b| !refd.contains(&b.id)).unwrap_or(0);
        let mut seen = HashSet::new();
        let mut out = Vec::new();
        let mut stack = vec![&blocks[entry]];
        while let Some(b) = stack.pop() {
            if seen.insert(b.id.clone()) {
                out.push(b);
                for s in b.successors.iter().rev() {
                    if !seen.contains(s) {
                        if let Some(sb) = map.get(s) {
                            stack.push(sb);
                        }
                    }
                }
            }
        }
        for b in blocks {
            if !seen.contains(&b.id) {
                out.push(b);
            }
        }
        out
    }
}

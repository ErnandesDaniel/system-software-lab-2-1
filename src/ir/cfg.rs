use crate::ir::types::{IrFunction, IrProgram};
use std::collections::{HashMap, HashSet};

pub struct ControlFlowGraph {
    pub entry_block: String,
    pub predecessors: HashMap<String, Vec<String>>,
    pub successors: HashMap<String, Vec<String>>,
    pub dominators: HashMap<String, HashSet<String>>,
    pub loops: Vec<LoopInfo>,
}

pub struct LoopInfo {
    pub header: String,
    pub body: HashSet<String>,
    pub exiting_edges: Vec<(String, String)>,
}

impl ControlFlowGraph {
    pub fn build(func: &IrFunction) -> Self {
        let successors = Self::compute_successors(func);
        let predecessors = Self::compute_predecessors(&successors);
        let entry_block = func.blocks.first().map(|b| b.id.clone()).unwrap_or_default();
        let dominators = Self::compute_dominators(&entry_block, &successors, &predecessors);
        let loops = Self::find_loops(&entry_block, &successors, &predecessors, &dominators);

        Self {
            entry_block,
            predecessors,
            successors,
            dominators,
            loops,
        }
    }

    pub fn build_program(program: &IrProgram) -> HashMap<String, Self> {
        program
            .functions
            .iter()
            .map(|f| (f.name.clone(), Self::build(f)))
            .collect()
    }

    fn compute_successors(func: &IrFunction) -> HashMap<String, Vec<String>> {
        let mut succs = HashMap::new();
        for block in &func.blocks {
            let deps: Vec<String> = block
                .instructions
                .iter()
                .filter_map(|inst| {
                    if let Some(ref target) = inst.jump_target {
                        if !matches!(
                            inst.opcode,
                            crate::ir::IrOpcode::Call | crate::ir::IrOpcode::MakeClosure
                        ) {
                            return Some(target.clone());
                        }
                    }
                    if let Some(ref t) = inst.true_target {
                        return Some(t.clone());
                    }
                    if let Some(ref f) = inst.false_target {
                        return Some(f.clone());
                    }
                    None
                })
                .collect();
            succs.insert(block.id.clone(), deps);
        }
        succs
    }

    fn compute_predecessors(
        successors: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, Vec<String>> {
        let mut preds: HashMap<String, Vec<String>> = HashMap::new();
        for (from, targets) in successors {
            for to in targets {
                preds.entry(to.clone()).or_default().push(from.clone());
            }
        }
        preds
    }

    fn compute_dominators(
        entry: &str,
        successors: &HashMap<String, Vec<String>>,
        predecessors: &HashMap<String, Vec<String>>,
    ) -> HashMap<String, HashSet<String>> {
        let all_blocks: HashSet<String> = successors.keys().cloned().collect();
        let mut doms: HashMap<String, HashSet<String>> = HashMap::new();

        for block in &all_blocks {
            if block == entry {
                let mut s = HashSet::new();
                s.insert(entry.to_string());
                doms.insert(entry.to_string(), s);
            } else {
                doms.insert(block.clone(), all_blocks.clone());
            }
        }

        loop {
            let mut changed = false;
            for block in all_blocks.iter() {
                if block == entry {
                    continue;
                }
                let pred_blocks = predecessors.get(block).cloned().unwrap_or_default();
                if pred_blocks.is_empty() {
                    continue;
                }
                let mut new_doms: HashSet<String> = pred_blocks
                    .iter()
                    .map(|p| doms.get(p).cloned().unwrap_or_default())
                    .reduce(|a, b| a.intersection(&b).cloned().collect())
                    .unwrap_or_default();
                new_doms.insert(block.clone());

                if new_doms != doms[block] {
                    doms.insert(block.clone(), new_doms);
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }

        doms
    }

    fn find_loops(
        _entry: &str,
        successors: &HashMap<String, Vec<String>>,
        predecessors: &HashMap<String, Vec<String>>,
        dominators: &HashMap<String, HashSet<String>>,
    ) -> Vec<LoopInfo> {
        let mut loops = Vec::new();

        for (header, preds) in predecessors {
            for back_edge_src in preds {
                if back_edge_src == header {
                    continue;
                }
                let header_doms = dominators.get(header).cloned().unwrap_or_default();
                if !header_doms.contains(back_edge_src) {
                    continue;
                }

                let mut body = HashSet::new();
                let mut stack = vec![back_edge_src.clone()];
                let mut exiting_edges = Vec::new();

                while let Some(node) = stack.pop() {
                    if body.insert(node.clone()) {
                        if let Some(preds_of_node) = predecessors.get(&node) {
                            for p in preds_of_node {
                                if p == header {
                                    continue;
                                }
                                if !body.contains(p) {
                                    stack.push(p.clone());
                                }
                            }
                        }
                    }
                }

                for node in &body {
                    if let Some(succs) = successors.get(node) {
                        for s in succs {
                            if !body.contains(s) {
                                exiting_edges.push((node.clone(), s.clone()));
                            }
                        }
                    }
                }

                loops.push(LoopInfo {
                    header: header.clone(),
                    body,
                    exiting_edges,
                });
            }
        }

        loops
    }
}

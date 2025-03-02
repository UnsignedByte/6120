use std::{
    collections::{HashMap, HashSet},
    vec,
};

use bril_rs::{EffectOps, Function, Instruction, Type, ValueOps};
use itertools::Itertools;
use utils::{
    AnalysisPass, BBFunction, CFG, DominatorTree, FunctionPass, InstrExt, Pass, run_passes,
    setup_logger_from_env,
};

#[derive(Debug, Default)]
struct NameStack {
    levels: Vec<usize>,
    names: HashMap<String, Vec<String>>,
}

impl NameStack {
    pub fn push_level(&mut self) {
        self.levels.push(0);
    }

    pub fn pop_level(&mut self) {
        let level = self.levels.pop();

        if let Some(level) = level {
            for (_, v) in self.names.iter_mut() {
                if level >= v.len() {
                    v.clear();
                } else {
                    v.truncate(v.len() - level);
                }
            }
        }
    }

    pub fn get(&self, name: &str) -> String {
        self.names
            .get(name)
            .and_then(|v| v.last())
            .cloned()
            .unwrap_or_else(|| name.to_owned())
    }

    pub fn push(&mut self, name: &str, new: String) -> String {
        let entry = self.names.entry(name.to_owned()).or_default();

        if let Some(level) = self.levels.last_mut() {
            *level += 1;
        }

        entry.push(new.clone());

        new
    }

    /// Get the shadow name for a phi node
    fn shadow_name(name: &str, bidx: usize) -> String {
        format!("{}.{}.shadow", name, bidx)
    }

    /// Get a unique name for an instruction
    fn unique_name(name: &str, bidx: usize, iidx: usize) -> String {
        format!("{}.{}.{}", name, bidx, iidx)
    }
}

struct PhiNodes {
    writes: Vec<HashSet<(Type, String)>>,
    nodes: Vec<HashSet<(Type, String)>>,
}

impl PhiNodes {
    pub fn new(doms: &DominatorTree) -> Self {
        let writes: Vec<HashSet<_>> = doms
            .iter()
            .map(|bb| {
                bb.iter()
                    .filter_map(|i| i.dest().and_then(|d| i.get_type().map(|t| (t, d))))
                    .collect()
            })
            .collect();

        let mut nodes = vec![HashSet::new(); doms.len()];
        for bb in doms.iter() {
            // bb's domination frontier needs phi nodes
            for df in doms.dominance_frontier(bb.idx) {
                nodes[*df].extend(writes[bb.idx].iter().cloned());
            }
        }

        Self { nodes, writes }
    }

    pub fn get(&self, block: usize) -> &HashSet<(Type, String)> {
        &self.nodes[block]
    }
}

struct ToSSA;

impl ToSSA {
    fn rename(doms: &mut DominatorTree, bidx: usize, stack: &mut NameStack, phi_nodes: &PhiNodes) {
        stack.push_level();
        log::debug!("Renaming block {}", doms.get(bidx).label_or_default());
        log::debug!("Stack: {:?}", stack);

        let block = doms.get_mut(bidx);
        // Insert the get instructions
        let gets = phi_nodes
            .get(bidx)
            .iter()
            .map(|(ty, dst)| {
                let shadow = NameStack::shadow_name(dst, bidx);
                stack.push(dst, shadow.clone());
                Instruction::Value {
                    dest: shadow,
                    op: ValueOps::Get,
                    args: vec![],
                    funcs: vec![],
                    labels: vec![],
                    op_type: ty.clone(),
                    pos: None,
                }
            })
            .collect_vec();

        for (i, instr) in block.iter_mut().enumerate() {
            log::debug!("Renaming {}", instr);
            match instr {
                Instruction::Constant { dest, .. } => {
                    let new = NameStack::unique_name(dest, bidx, i);
                    stack.push(dest, new.clone());
                    *dest = new;
                }
                Instruction::Value { args, dest, .. } => {
                    for arg in args {
                        *arg = stack.get(arg);
                    }

                    let new = NameStack::unique_name(dest, bidx, i);
                    stack.push(dest, new.clone());
                    *dest = new;
                }
                Instruction::Effect { args, .. } => {
                    for arg in args {
                        *arg = stack.get(arg);
                    }
                }
            }

            log::debug!("Renamed to {}", instr);
        }

        // Insert the get instructions
        for get in gets {
            block.insert(0, get);
        }

        let phis = doms
            .cfg
            .succs(bidx)
            .into_iter()
            .flat_map(|v| phi_nodes.get(v).iter().map(move |(ty, dst)| (v, ty, dst)))
            .map(|(succ, _, dst)| Instruction::Effect {
                args: vec![NameStack::shadow_name(dst, succ), stack.get(dst)],
                funcs: vec![],
                labels: vec![],
                op: EffectOps::Set,
                pos: None,
            })
            .collect_vec();

        doms.get_mut(bidx).extend(phis);

        // Rename all immediately dominated blocks
        for child in 0..doms.len() {
            if doms.immediate_doms(child) == Some(bidx) {
                ToSSA::rename(doms, child, stack, phi_nodes);
            }
        }

        stack.pop_level();
    }
}

impl Pass for ToSSA {
    fn function(&mut self, func: Function) -> Function {
        let mut name_stack = NameStack::default();

        let mut doms = DominatorTree::from(func);

        let phi_nodes = PhiNodes::new(&doms);

        ToSSA::rename(&mut doms, 0, &mut name_stack, &phi_nodes);

        doms.into()
    }
}

fn main() {
    setup_logger_from_env();
    run_passes(&mut [Box::new(ToSSA)]);
}

use std::{
    collections::{HashMap, HashSet},
    vec,
};

use bril_rs::{Argument, EffectOps, Function, Instruction, Type, ValueOps};
use itertools::Itertools;
use utils::{DominatorTree, InstrExt, Pass, pass_pipeline, setup_logger_from_env};

#[derive(Debug, Default)]
struct NameStack {
    levels: HashMap<String, Vec<usize>>,
    names: HashMap<String, Vec<String>>,
}

impl NameStack {
    pub fn new(args: &[Argument]) -> Self {
        let mut names = HashMap::new();
        for arg in args {
            names.insert(arg.name.clone(), vec![arg.name.clone()]);
        }

        Self {
            levels: HashMap::new(),
            names,
        }
    }

    pub fn push_level(&mut self) {
        for name in self.names.keys() {
            self.levels.entry(name.clone()).or_default().push(0);
        }
    }

    pub fn pop_level(&mut self) {
        for (key, v) in self.names.iter_mut() {
            let level = self.levels.get(key).and_then(|v| v.last().copied());

            if let Some(level) = level {
                if level >= v.len() {
                    v.clear();
                } else {
                    v.truncate(v.len() - level);
                }
            }
        }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        self.names.get(name).and_then(|v| v.last()).cloned()
    }

    pub fn push(&mut self, name: &str, new: String) -> String {
        let entry = self.names.entry(name.to_owned()).or_default();

        let level = self.levels.entry(name.to_owned()).or_default();
        if let Some(last) = level.last_mut() {
            *last += 1;
        } else {
            level.push(1);
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

        Self { nodes }
    }

    pub fn get(&self, block: usize) -> &HashSet<(Type, String)> {
        &self.nodes[block]
    }
}

struct ToSSA;

impl ToSSA {
    fn rename(
        doms: &mut DominatorTree,
        bidx: usize,
        stack: &mut NameStack,
        phi_nodes: &PhiNodes,
        undefined: &mut HashMap<String, Type>,
    ) {
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
                        *arg = stack.get(arg).unwrap();
                    }

                    let new = NameStack::unique_name(dest, bidx, i);
                    stack.push(dest, new.clone());
                    *dest = new;
                }
                Instruction::Effect { args, .. } => {
                    for arg in args {
                        *arg = stack.get(arg).unwrap();
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
            .map(|(succ, ty, dst)| {
                let old_name = match stack.get(dst) {
                    Some(name) => name,
                    None => {
                        // Add it to the undefined set
                        undefined.insert(dst.clone(), ty.clone());
                        dst.clone()
                    }
                };

                Instruction::Effect {
                    args: vec![NameStack::shadow_name(dst, succ), old_name],
                    funcs: vec![],
                    labels: vec![],
                    op: EffectOps::Set,
                    pos: None,
                }
            })
            .collect_vec();

        doms.get_mut(bidx).extend(phis);

        // Rename all immediately dominated blocks
        for child in 0..doms.len() {
            if doms.immediate_doms(child) == Some(bidx) {
                ToSSA::rename(doms, child, stack, phi_nodes, undefined);
            }
        }

        stack.pop_level();
    }
}

impl Pass for ToSSA {
    fn function(&mut self, func: Function) -> Function {
        log::debug!("Converting function {} to SSA", func.name);
        let mut name_stack = NameStack::new(&func.args);

        let mut doms = DominatorTree::from(func);

        let phi_nodes = PhiNodes::new(&doms);

        let mut undefined = HashMap::new();

        ToSSA::rename(&mut doms, 0, &mut name_stack, &phi_nodes, &mut undefined);

        // Add x: type = undef for all undefined variables
        for (undef, ty) in undefined {
            let instr = Instruction::Value {
                args: vec![],
                dest: undef,
                funcs: vec![],
                labels: vec![],
                op: ValueOps::Undef,
                pos: None,
                op_type: ty,
            };

            doms.get_mut(0).insert(0, instr);
        }

        doms.into()
    }
}

fn main() {
    setup_logger_from_env();
    pass_pipeline!(ToSSA);
}

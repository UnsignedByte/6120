use bril_rs::{Argument, EffectOps, Function, Instruction, Type, ValueOps};
use itertools::Itertools;
use linked_hash_map::LinkedHashMap;
use linked_hash_set::LinkedHashSet;
use std::{collections::HashMap, vec};
use utils::{
    BBFunction, BasicBlock, DominatorTree, InstrExt, Pass, RemoveUnlabeledBlocks, pass_pipeline,
    setup_logger_from_env,
};

#[derive(Debug, Default, Clone)]
struct NameStack {
    names: HashMap<String, Vec<String>>,
}

impl NameStack {
    pub fn new(args: &[Argument]) -> Self {
        let mut names = HashMap::new();
        for arg in args {
            names.insert(arg.name.clone(), vec![arg.name.clone()]);
        }

        Self { names }
    }

    pub fn get(&self, name: &str) -> Option<String> {
        self.names.get(name).and_then(|v| v.last()).cloned()
    }

    pub fn push(&mut self, name: &str, new: String) -> String {
        let entry = self.names.entry(name.to_owned()).or_default();

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
    nodes: Vec<LinkedHashMap<String, Type>>,
}

impl PhiNodes {
    pub fn new(doms: &DominatorTree) -> Self {
        let writes: Vec<LinkedHashMap<_, _>> = doms
            .iter()
            .map(|bb| {
                bb.iter()
                    .filter_map(|i| i.dest().and_then(|d| i.get_type().map(|t| (d, t))))
                    .collect()
            })
            .collect();

        // Map from variable to the blocks that write to it
        let mut defs = LinkedHashMap::new();
        for (block, writes) in writes.iter().enumerate() {
            for (dest, ty) in writes {
                defs.entry((dest.clone(), ty.clone()))
                    .or_insert_with(LinkedHashSet::new)
                    .insert(block);
            }
        }

        let mut nodes = vec![LinkedHashMap::new(); doms.len()];

        for ((dest, ty), mut defs) in defs {
            while let Some(d) = defs.pop_front() {
                for block in doms.dominance_frontier(d) {
                    if nodes[*block].insert(dest.clone(), ty.clone()).is_none() {
                        // Newly inserted, add it to the defs
                        defs.insert(*block);
                    }
                }
            }
        }

        Self { nodes }
    }

    pub fn get(&self, block: usize) -> impl Iterator<Item = (&String, &Type)> {
        self.nodes[block].iter()
    }
}

struct ToSSA;

impl ToSSA {
    fn rename(
        doms: &mut DominatorTree,
        bidx: usize,
        stack: &mut NameStack,
        phi_nodes: &PhiNodes,
        undefined: &mut LinkedHashMap<String, Type>,
    ) {
        let old_stack = stack.clone();
        log::info!("Renaming block {}", doms.get(bidx).label_or_default());
        log::debug!("Stack: {:?}", stack);

        let block = doms.get_mut(bidx);
        // Insert the get instructions
        let gets = phi_nodes
            .get(bidx)
            .map(|(dst, ty)| {
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
            .cfg()
            .succs(bidx)
            .into_iter()
            .flat_map(|v| phi_nodes.get(v).map(move |(dst, ty)| (v, dst, ty)))
            .map(|(succ, dst, ty)| {
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

        // Undo the stack changes
        *stack = old_stack;
    }
}

impl Pass for ToSSA {
    fn function(&mut self, func: Function) -> Function {
        let func = BBFunction::from(func);
        // If the entry node has a label, create a dummy entry label.
        // This is to deal with the case that the entry node has a `get`.
        let func = if func.get(0).label.is_some() {
            func.with_blocks(|blocks| {
                std::iter::once(BasicBlock::new(0, None, vec![]))
                    .chain(blocks)
                    .collect()
            })
        } else {
            func
        };

        log::info!("Converting function {} to SSA", func.name);
        let mut name_stack = NameStack::new(&func.args);

        let mut doms = DominatorTree::from(func);

        let phi_nodes = PhiNodes::new(&doms);

        let mut undefined = LinkedHashMap::new();

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
    pass_pipeline!(RemoveUnlabeledBlocks, ToSSA);
}

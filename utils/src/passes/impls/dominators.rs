use crate::{BasicBlock, CFG, DataflowLabel, DataflowPass};
use itertools::Itertools;
use linked_hash_set::LinkedHashSet;

/// Helper pass to calculate the dominators for a given CFG
pub struct DominatorPass;

impl DataflowPass<LinkedHashSet<usize>> for DominatorPass {
    fn entry(&self, _: &crate::BBFunction) -> LinkedHashSet<usize> {
        std::iter::once(0).collect()
    }

    fn init(&self, func: &crate::BBFunction) -> LinkedHashSet<usize> {
        (0..func.len()).collect()
    }

    fn meet(&self, in_vals: &[LinkedHashSet<usize>]) -> LinkedHashSet<usize> {
        match in_vals {
            [] => LinkedHashSet::new(),
            [first] => first.clone(),
            [first, rest @ ..] => rest.iter().fold(first.clone(), |acc, val| &acc & val),
        }
    }

    fn transfer(&self, block: &BasicBlock, in_val: &LinkedHashSet<usize>) -> LinkedHashSet<usize> {
        // The dominators of a block are always the block itself

        let mut doms = in_val.clone();
        doms.insert(block.idx);
        doms
    }
}

/// Dominator set node used to display graphs
/// For dominator sets
#[derive(Clone)]
pub struct DominatorSetNode {
    doms: LinkedHashSet<usize>,
}

impl DataflowLabel for DominatorSetNode {
    fn in_label(&self, _: &CFG) -> Option<String> {
        None
    }

    fn out_label(&self, cfg: &CFG) -> Option<String> {
        // Create a set of node names
        let names: Vec<String> = self
            .doms
            .iter()
            .sorted_unstable()
            .map(|idx| cfg.get(*idx).label_or_default().to_string())
            .collect();

        // Return a string representation of the set
        Some(names.join("\\n"))
    }
}

impl From<LinkedHashSet<usize>> for DominatorSetNode {
    fn from(doms: LinkedHashSet<usize>) -> Self {
        Self { doms }
    }
}

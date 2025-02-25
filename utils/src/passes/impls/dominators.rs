use itertools::Itertools;

use crate::{DataflowLabel, DataflowPass, CFG};
use std::collections::HashSet;

/// Helper pass to calculate the dominators for a given CFG
pub struct DominatorPass;

impl DataflowPass<HashSet<usize>> for DominatorPass {
    fn init(&self, func: &crate::BBFunction, bidx: usize) -> HashSet<usize> {
        match bidx {
            0 => {
                // Entry node dominates only itself
                [0].into()
            }
            _ => {
                // All other nodes can be initialized to the full set of blocks
                (0..func.len()).collect()
            }
        }
    }

    fn meet(&self, in_vals: &[HashSet<usize>]) -> HashSet<usize> {
        match in_vals {
            [] => HashSet::new(),
            [first] => first.clone(),
            [first, rest @ ..] => rest.iter().fold(first.clone(), |acc, val| &acc & val),
        }
    }

    fn transfer(
        &self,
        _: &crate::BBFunction,
        bidx: usize,
        in_val: &HashSet<usize>,
    ) -> HashSet<usize> {
        // The dominators of a block are always the block itself

        let mut doms = in_val.clone();
        doms.insert(bidx);
        doms
    }
}

/// Dominator set node used to display graphs
/// For dominator sets
#[derive(Clone)]
pub struct DominatorSetNode {
    doms: HashSet<usize>,
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

impl From<HashSet<usize>> for DominatorSetNode {
    fn from(doms: HashSet<usize>) -> Self {
        Self { doms }
    }
}

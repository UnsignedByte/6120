use crate::DataflowPass;
use std::collections::HashSet;

/// Helper pass to calculate the dominators for a given CFG
pub(crate) struct DominatorPass;

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

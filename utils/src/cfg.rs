use crate::BBFunction;
use bril_rs::{EffectOps, Instruction};

/// Control Flow Graph representation.
pub struct CFG {
    pub func: BBFunction,
    preds: Vec<Vec<usize>>,
    succs: Vec<Vec<usize>>,
    reversed: bool,
}

impl CFG {
    pub fn new(func: BBFunction) -> Self {
        let blocks = &func.blocks;
        let n = blocks.len();

        let succs: Vec<Vec<usize>> = blocks
            .iter()
            .map(|block| {
                // Branch/Return/Jump Instruction handling
                block.instrs.iter().last().and_then(|instr| match instr {
                    Instruction::Effect {
                        op: EffectOps::Jump | EffectOps::Branch,
                        labels,
                        ..
                    } => Some(
                        // Jump and Branch instructions have successors as the target
                        labels
                            .iter()
                            .map(|label| {
                                func.get_block_idx(label).unwrap_or_else(|| {
                                    panic!("Label {} not found", label);
                                })
                            })
                            .collect(),
                    ),
                    Instruction::Effect {
                        op: EffectOps::Return,
                        ..
                    } => Some(vec![]),
                    _ => None, // Defer handling to later
                })
            })
            .enumerate()
            .map(|(i, succs)| {
                if let Some(s) = succs {
                    // If the block has a branch/return/jump instruction, return the labels
                    s
                } else if i + 1 < n {
                    // If the block is not the final block, add the next block as a successor
                    vec![i + 1]
                } else {
                    // Final block has no successors
                    vec![]
                }
            })
            .collect();

        let mut preds = vec![vec![]; n];
        for (i, succs) in succs.iter().enumerate() {
            for &j in succs {
                preds[j].push(i);
            }
        }

        Self {
            func,
            preds,
            succs,
            reversed: false,
        }
    }

    pub fn reverse(self) -> Self {
        Self {
            func: self.func,
            preds: self.succs,
            succs: self.preds,
            reversed: !self.reversed,
        }
    }

    pub fn len(&self) -> usize {
        self.func.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.func.is_empty()
    }

    /// Check whether a block idx is an entry block (no predecessors)
    pub fn is_entry(&self, idx: usize) -> bool {
        self.preds[idx].is_empty()
    }

    pub fn preds(&self, idx: usize) -> &[usize] {
        &self.preds[idx]
    }

    pub fn succs(&self, idx: usize) -> &[usize] {
        &self.succs[idx]
    }
}

use std::{collections::LinkedList, fmt::Display};

use crate::{BBFunction, BasicBlock, CFG};

/// Results of a dataflow analysis
pub struct Dataflow<Val> {
    cfg: CFG,
    pub in_vals: Vec<Val>,
    pub out_vals: Vec<Val>,
}

impl<Val: Display> Display for Dataflow<Val> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "@{} {{{{", self.cfg.func.name)?;
        for (i, (in_val, out_val)) in self.in_vals.iter().zip(&self.out_vals).enumerate() {
            writeln!(f, ".{}:", i)?;
            writeln!(f, "  In:  {}", in_val)?;
            writeln!(f, "  Out: {}", out_val)?;
        }
        writeln!(f, "}}}}")?;
        Ok(())
    }
}

/// Trait for dataflow analysis passes
pub trait DataflowPass<Val>
where
    Val: Eq + Clone + Default,
{
    /// Initial values for entry blocks
    fn init(&self, func: &BBFunction) -> Val;

    /// Transfer function
    fn transfer(&self, block: &BasicBlock, in_val: &Val) -> Val;

    /// Merge function
    fn merge(&self, in_vals: &[Val]) -> Val;

    fn func(&mut self, cfg: CFG) -> Dataflow<Val> {
        let n = cfg.len();

        let mut in_vals = vec![Val::default(); n];
        let mut out_vals = vec![Val::default(); n];

        let mut worklist: LinkedList<_> = (0..n).collect();
        while let Some(i) = worklist.pop_front() {
            if cfg.is_entry(i) {
                in_vals[i] = self.init(&cfg.func);
            } else {
                let inputs = cfg
                    .preds(i)
                    .iter()
                    .map(|&j| out_vals[j].clone())
                    .collect::<Vec<_>>();

                in_vals[i] = self.merge(&inputs);
            }

            let block = &cfg.func.blocks[i];
            let new_vals = self.transfer(block, &in_vals[i]);

            if new_vals != out_vals[i] {
                out_vals[i] = new_vals;
                for &j in cfg.succs(i) {
                    worklist.push_back(j);
                }
            }
        }

        Dataflow {
            cfg,
            in_vals,
            out_vals,
        }
    }
}

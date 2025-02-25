use crate::{BBFunction, BasicBlock, GraphLike, CFG};
use graphviz_rust::{
    dot_generator::{attr, id, node},
    dot_structures::{Attribute, Id, Node, NodeId, Stmt},
};
use itertools::Itertools;
use std::fmt::Debug;
use std::{collections::LinkedList, fmt::Display};

/// Results of a dataflow analysis
pub struct Dataflow<Val> {
    pub cfg: CFG,
    pub in_vals: Vec<Val>,
    pub out_vals: Vec<Val>,
    pub exit_val: Val,
}

impl<F> Dataflow<F> {
    pub fn from<T>(value: Dataflow<T>) -> Self
    where
        F: From<T>,
    {
        Dataflow {
            cfg: value.cfg,
            in_vals: value.in_vals.into_iter().map(F::from).collect(),
            out_vals: value.out_vals.into_iter().map(F::from).collect(),
            exit_val: F::from(value.exit_val),
        }
    }
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

/// Trait for dataflow analysis labels
/// Allows conerting dataflow values to string labels
pub trait DataflowLabel
where
    Self: Sized,
{
    fn in_label(&self, df: &Dataflow<Self>) -> Option<String>;
    fn out_label(&self, df: &Dataflow<Self>) -> Option<String>;
}

/// Represents a node in the dataflow graph
pub(crate) struct DataflowNode<'a, Val> {
    bb: &'a BasicBlock,
    df: &'a Dataflow<Val>,
    i: usize,
}

impl<'a, Val> DataflowNode<'a, Val>
where
    Val: DataflowLabel,
{
    pub fn new(bb: &'a BasicBlock, df: &'a Dataflow<Val>, i: usize) -> Self {
        DataflowNode { bb, df, i }
    }

    fn label(&self) -> String {
        let DataflowNode {
            bb,
            df: dataflow,
            i,
        } = self;
        format!(
            r#""{{{}|{}}}""#,
            bb.label_or_default(),
            vec![
                dataflow.in_vals[*i].in_label(dataflow),
                dataflow.out_vals[*i].out_label(dataflow)
            ]
            .into_iter()
            .flatten()
            .join("|")
        )
    }
}

impl<Val> GraphLike<DataflowNode<'_, Val>> for Dataflow<Val>
where
    Val: DataflowLabel,
{
    fn node_attrs<'d>(&self, node: DataflowNode<'_, Val>) -> Vec<Attribute> {
        vec![attr!("label", &node.label()), attr!("shape", "Mrecord")]
    }

    fn graph_attrs(&self) -> Vec<Stmt> {
        self.cfg.graph_attrs()
    }

    fn graph_nodes(&self, gid: &[usize]) -> Vec<Stmt> {
        // Create the exit node
        let exit_node = &format!("{}_exit", self.graph_id(gid));
        self.cfg
            .func
            .blocks
            .iter()
            .enumerate()
            .map(|(i, block)| self.node(gid, DataflowNode::new(block, self, i), i))
            .chain(std::iter::once(
                node!(exit_node; attr!("label", "exit"), attr!("color", "purple"), attr!("rank", "sink")).into()
            ))
            .collect()
    }

    fn graph_edges(&self, gid: &[usize]) -> Vec<Stmt> {
        self.cfg.graph_edges(gid)
    }
}

/// Trait for dataflow analysis passes
pub trait DataflowPass<Val>
where
    Val: Eq + Clone + Default + Debug,
{
    /// Initial values for entry blocks
    fn init(&self, func: &BBFunction, bidx: usize) -> Val;

    /// Meet function
    fn meet(&self, in_vals: &[Val]) -> Val;

    /// Transfer function
    fn transfer(&self, func: &BBFunction, bidx: usize, in_val: &Val) -> Val;

    /// Transfer function for the exit block
    fn finish(&self, _func: &BBFunction, exit_val: Val) -> Val {
        exit_val
    }

    fn cfg(&mut self, cfg: CFG) -> Dataflow<Val> {
        let n = cfg.len();

        let mut in_vals = vec![Val::default(); n];
        let mut out_vals = vec![];
        for i in 0..n {
            out_vals.push(self.init(&cfg.func, i));
        }

        let mut worklist: LinkedList<_> = (0..n).collect();
        while let Some(i) = worklist.pop_front() {
            let inputs = cfg
                .preds(i)
                .iter()
                .map(|&j| out_vals[j].clone())
                .collect_vec();

            log::trace!("Collected inputs for block {}: {:?}", i, inputs);

            in_vals[i] = self.meet(&inputs);

            log::trace!("Merged inputs for block {}: {:?}", i, in_vals[i]);

            let new_vals = self.transfer(&cfg.func, i, &in_vals[i]);

            log::trace!("New values for block {}: {:?}", i, new_vals);

            if new_vals != out_vals[i] {
                out_vals[i] = new_vals;
                for j in cfg.succs(i) {
                    worklist.push_back(j);
                }
            }
        }

        // The exit value can be computed by meeting all the out values of exit block(s)
        let exit_val = cfg
            .exits()
            .into_iter()
            .map(|i| out_vals[i].clone())
            .collect_vec();
        let exit_val = self.meet(&exit_val);
        let exit_val = self.finish(&cfg.func, exit_val);

        Dataflow {
            cfg,
            in_vals,
            out_vals,
            exit_val,
        }
    }
}

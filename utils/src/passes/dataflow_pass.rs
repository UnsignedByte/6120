use crate::{BBFunction, BasicBlock, CFG, CallGraph, GraphLike, draw};
use graphviz_rust::{
    dot_generator::{attr, id, node_id},
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

impl<Val: PartialEq> PartialEq for Dataflow<Val> {
    fn eq(&self, other: &Self) -> bool {
        self.in_vals == other.in_vals
            && self.out_vals == other.out_vals
            && self.exit_val == other.exit_val
    }
}

impl<Val: Eq> Eq for Dataflow<Val> {}

impl<Val: Display> Display for Dataflow<Val> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "@{} {{{{", self.cfg.name())?;
        for (i, (in_val, out_val)) in self.in_vals.iter().zip(&self.out_vals).enumerate() {
            writeln!(f, ".{}:", i)?;
            writeln!(f, "  In:  {}", in_val)?;
            writeln!(f, "  Out: {}", out_val)?;
        }
        writeln!(f, "}}}}")?;
        Ok(())
    }
}

impl<Val: Debug> Debug for Dataflow<Val> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "@{} {{{{", self.cfg.name())?;
        for (i, (in_val, out_val)) in self.in_vals.iter().zip(&self.out_vals).enumerate() {
            writeln!(f, ".{}:", i)?;
            writeln!(f, "  In:  {:?}", in_val)?;
            writeln!(f, "  Out: {:?}", out_val)?;
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
    fn in_label(&self, cfg: &CFG) -> Option<String>;
    fn out_label(&self, cfg: &CFG) -> Option<String>;
}

/// Represents a node in the dataflow graph
pub(crate) struct DataflowNode<'a, Val> {
    bb: Option<&'a BasicBlock>,
    df: &'a Dataflow<Val>,
    i: usize,
}

impl<'a, Val> DataflowNode<'a, Val>
where
    Val: DataflowLabel,
{
    pub fn new(bb: &'a BasicBlock, df: &'a Dataflow<Val>, i: usize) -> Self {
        DataflowNode {
            bb: Some(bb),
            df,
            i,
        }
    }

    pub fn exit(df: &'a Dataflow<Val>) -> Self {
        DataflowNode {
            bb: None,
            df,
            i: df.cfg.len(),
        }
    }

    fn is_exit(&self) -> bool {
        self.bb.is_none()
    }

    fn is_entry(&self) -> bool {
        self.bb.map(|bb| bb.is_entry()).unwrap_or(false)
    }

    fn label(&self) -> String {
        let DataflowNode {
            bb,
            df: dataflow,
            i,
        } = self;
        if let Some(bb) = bb {
            let node_labels = if dataflow.cfg.reversed() {
                vec![
                    dataflow.out_vals[*i].out_label(&dataflow.cfg),
                    dataflow.in_vals[*i].in_label(&dataflow.cfg),
                ]
            } else {
                vec![
                    dataflow.in_vals[*i].in_label(&dataflow.cfg),
                    dataflow.out_vals[*i].out_label(&dataflow.cfg),
                ]
            };

            format!(
                r#""{{{}|{}}}""#,
                bb.label_or_default(),
                node_labels.into_iter().flatten().join("|")
            )
        } else if let Some(label) = dataflow.exit_val.out_label(&dataflow.cfg) {
            return format!(r#""{{exit|{}}}""#, label);
        } else {
            return "exit".to_owned();
        }
    }
}

impl<Val> GraphLike<DataflowNode<'_, Val>> for Dataflow<Val>
where
    Val: DataflowLabel,
{
    fn node_attrs<'d>(&self, node: DataflowNode<'_, Val>) -> Vec<Attribute> {
        let color = if node.is_entry() {
            "blue"
        } else if node.is_exit() {
            "purple"
        } else {
            "black"
        };

        vec![
            attr!("label", &node.label()),
            attr!("shape", "Mrecord"),
            attr!("color", color),
        ]
    }

    fn graph_attrs(&self) -> Vec<Stmt> {
        self.cfg.graph_attrs()
    }

    fn graph_nodes(&self, gid: &[usize]) -> Vec<Stmt> {
        // Create the exit node
        let exit_node = &format!("{}_exit", self.graph_id(gid));
        self.cfg
            .iter()
            .enumerate()
            .map(|(i, block)| self.node(gid, DataflowNode::new(block, self, i), i))
            .chain(std::iter::once(
                Node {
                    id: node_id!(exit_node),
                    attributes: self.node_attrs(DataflowNode::exit(self)),
                }
                .into(),
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
    Val: Eq + Clone + Debug,
{
    /// Whether this dataflow pass is reversed
    fn reversed(&self) -> bool {
        false
    }

    /// Initial values generated from arguments
    fn entry(&self, func: &BBFunction) -> Val {
        self.init(func)
    }

    /// Initial values for entry blocks
    fn init(&self, func: &BBFunction) -> Val;

    /// Meet function
    fn meet(&self, in_vals: &[Val]) -> Val;

    /// Transfer function
    fn transfer(&self, block: &BasicBlock, in_val: &Val) -> Val;

    /// Transfer function for the exit block
    fn finish(&self, _func: &BBFunction, exit_val: Val) -> Val {
        exit_val
    }

    fn cfg(&mut self, cfg: CFG) -> Dataflow<Val> {
        let cfg = if cfg.reversed() != self.reversed() {
            cfg.reverse()
        } else {
            cfg
        };

        let n = cfg.len();

        let mut in_vals = vec![self.init(cfg.func()); n];
        let mut out_vals = vec![self.init(cfg.func()); n];

        let mut worklist: LinkedList<_> = (0..n).collect();
        while let Some(i) = worklist.pop_front() {
            in_vals[i] = if cfg.is_entry(i) {
                self.entry(cfg.func())
            } else {
                let inputs = cfg
                    .preds(i)
                    .iter()
                    .map(|&j| out_vals[j].clone())
                    .collect_vec();
                log::trace!("Collected inputs for block {}: {:?}", i, inputs);
                self.meet(&inputs)
            };

            log::trace!("Merged inputs for block {}: {:?}", i, in_vals[i]);

            let new_vals = self.transfer(cfg.func().get(i), &in_vals[i]);

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
        let exit_val = self.finish(cfg.func(), exit_val);

        Dataflow {
            cfg,
            in_vals,
            out_vals,
            exit_val,
        }
    }
}

pub fn draw_dataflow<Pass, Val, GraphNode>(
    call_graph: CallGraph,
    directional: bool,
    strict: bool,
) -> String
where
    Val: Eq + Clone + Debug,
    Pass: DataflowPass<Val> + Default,
    GraphNode: From<Val> + DataflowLabel,
    Dataflow<GraphNode>:,
{
    let results: Vec<_> = call_graph
        .prog()
        .functions
        .iter()
        .map(|f| Pass::default().cfg(CFG::from(f.clone())))
        .map(<Dataflow<GraphNode>>::from)
        .collect();

    draw((call_graph, results), directional, strict)
}

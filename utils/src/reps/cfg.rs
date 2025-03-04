use crate::{BBFunction, BasicBlock, GraphLike};
use bril_rs::{EffectOps, Function, Instruction};
use graphviz_rust::{
    dot_generator::{attr, edge, id, node, node_id},
    dot_structures::{Attribute, Edge, EdgeTy, Id, Node, NodeId, Stmt, Vertex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlowEdge {
    Exit,
    Branch(usize, usize),
    Jump(usize),
}

impl FlowEdge {
    pub fn vec(&self) -> Vec<usize> {
        match self {
            FlowEdge::Exit => vec![],
            FlowEdge::Branch(t, f) => vec![*t, *f],
            FlowEdge::Jump(j) => vec![*j],
        }
    }
}
/// Control Flow Graph representation.
#[derive(Debug, Clone)]
pub struct CFG {
    func: BBFunction,
    preds: Vec<Vec<usize>>,
    succs: Vec<FlowEdge>,
    reversed: bool,
}

impl CFG {
    pub fn new(func: BBFunction) -> Self {
        let n: usize = func.len();

        let succs: Vec<_> = func
            .iter()
            .map(|block| {
                // Branch/Return/Jump Instruction handling
                block.control_flow().map(|instr| {
                    log::trace!("Block {} has control flow instruction {}", block.idx, instr);
                    match instr {
                        Instruction::Effect {
                            op: op @ (EffectOps::Jump | EffectOps::Branch),
                            labels,
                            ..
                        } => {
                            let mut labels = labels.iter().map(|l| {
                                func.get_block_idx(l).unwrap_or_else(|| {
                                    panic!("Label {} not found", l);
                                })
                            });

                            match op {
                                EffectOps::Jump => FlowEdge::Jump(labels.next().unwrap()),
                                EffectOps::Branch => {
                                    FlowEdge::Branch(labels.next().unwrap(), labels.next().unwrap())
                                }
                                _ => unreachable!(),
                            }
                        }
                        Instruction::Effect {
                            op: EffectOps::Return,
                            ..
                        } => FlowEdge::Exit,
                        _ => unreachable!("Expected control flow instruction"),
                    }
                })
            })
            .enumerate()
            .map(|(i, succs)| {
                if let Some(s) = succs {
                    // If the block has a branch/return/jump instruction, return the labels
                    s
                } else if i + 1 < n {
                    // If the block is not the final block, add the next block as a successor
                    FlowEdge::Jump(i + 1)
                } else {
                    // Final block has no successors
                    FlowEdge::Exit
                }
            })
            .collect();

        let mut preds = vec![vec![]; n];
        for (i, succs) in succs.iter().enumerate() {
            match succs {
                FlowEdge::Exit => {}
                FlowEdge::Branch(t, f) => {
                    preds[*t].push(i);
                    preds[*f].push(i);
                }
                FlowEdge::Jump(j) => {
                    preds[*j].push(i);
                }
            }
        }

        Self {
            func,
            preds,
            succs,
            reversed: false,
        }
    }

    pub fn name(&self) -> &str {
        &self.func.name
    }

    pub fn func(&self) -> &BBFunction {
        &self.func
    }

    pub fn reverse(self) -> Self {
        Self {
            func: self.func,
            preds: self.preds,
            succs: self.succs,
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

    pub fn iter(&self) -> impl Iterator<Item = &BasicBlock> {
        self.func.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut BasicBlock> {
        self.func.iter_mut()
    }

    /// Check whether a block idx is an entry block (no predecessors)
    pub fn is_entry(&self, idx: usize) -> bool {
        match self.reversed {
            true => matches!(self.succs[idx], FlowEdge::Exit),
            false => self.preds[idx].is_empty(),
        }
    }

    pub fn preds(&self, idx: usize) -> Vec<usize> {
        match self.reversed {
            true => self.succs[idx].vec(),
            false => self.preds[idx].clone(),
        }
    }

    pub fn succs(&self, idx: usize) -> Vec<usize> {
        match self.reversed {
            true => self.preds[idx].clone(),
            false => self.succs[idx].vec(),
        }
    }

    pub fn exits(&self) -> Vec<usize> {
        (0..self.len())
            .filter(|&i| self.succs[i] == FlowEdge::Exit)
            .collect()
    }

    pub fn get(&self, idx: usize) -> &BasicBlock {
        self.func.get(idx)
    }

    pub fn get_mut(&mut self, idx: usize) -> &mut BasicBlock {
        self.func.get_mut(idx)
    }
}

impl From<BBFunction> for CFG {
    fn from(func: BBFunction) -> Self {
        Self::new(func)
    }
}

impl From<Function> for CFG {
    fn from(func: Function) -> Self {
        BBFunction::new(func).into()
    }
}

impl From<CFG> for BBFunction {
    fn from(cfg: CFG) -> Self {
        cfg.func
    }
}

impl From<CFG> for Function {
    fn from(cfg: CFG) -> Self {
        cfg.func.into()
    }
}

impl GraphLike<&BasicBlock> for CFG {
    fn node_attrs(&self, node: &BasicBlock) -> Vec<Attribute> {
        node.node_attrs()
    }

    fn graph_attrs(&self) -> Vec<Stmt> {
        vec![
            attr!("label", &format!(r#""{}""#, self.func.name)).into(),
            attr!("color", "darkgray").into(),
            attr!("style", "rounded").into(),
            attr!("bgcolor", "lightgray").into(),
            attr!("margin", 10).into(),
        ]
    }

    fn graph_nodes(&self, gid: &[usize]) -> Vec<Stmt> {
        // Create the exit node
        let exit_node = &format!("{}_exit", self.graph_id(gid));
        self.func
            .iter()
            .enumerate()
            .map(|(i, bb)| self.node(gid, bb, i))
            .chain(std::iter::once(
                node!(exit_node; attr!("label", "exit"), attr!("color", "purple"), attr!("rank", "sink")).into()
            ))
            .collect()
    }

    fn graph_edges(&self, gid: &[usize]) -> Vec<Stmt> {
        let exit_node = &format!("{}_exit", self.graph_id(gid));
        self.succs
            .iter()
            .enumerate()
            .flat_map(|(i, succs)| match succs {
                FlowEdge::Exit => vec![
                    edge!(
                        self.node_id(gid, i) => node_id!(exit_node);
                        attr!("color", "black")
                    )
                    .into(),
                ],
                FlowEdge::Branch(t, f) => vec![
                    edge!(self.node_id(gid, i) => self.node_id(gid, *t); attr!("color", "green"))
                        .into(),
                    edge!(self.node_id(gid, i) => self.node_id(gid, *f); attr!("color", "red"))
                        .into(),
                ],
                FlowEdge::Jump(j) => {
                    vec![edge!(self.node_id(gid, i) => self.node_id(gid, *j)).into()]
                }
            })
            .collect()
    }
}

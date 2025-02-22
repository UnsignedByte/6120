use crate::{BBFunction, BasicBlock, GraphLike};
use bril_rs::{EffectOps, Function, Instruction};
use graphviz_rust::{
    dot_generator::{attr, edge, id, node, node_id},
    dot_structures::{Attribute, Edge, EdgeTy, Id, Node, NodeId, Stmt, Vertex},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlowEdge {
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
pub struct CFG {
    pub func: BBFunction,
    pub(crate) preds: Vec<Vec<usize>>,
    pub(crate) succs: Vec<FlowEdge>,
    reversed: bool,
}

impl CFG {
    pub fn new(func: BBFunction) -> Self {
        let blocks = &func.blocks;
        let n = blocks.len();

        let succs: Vec<_> = blocks
            .iter()
            .map(|block| {
                // Branch/Return/Jump Instruction handling
                block.instrs.iter().last().and_then(|instr| match instr {
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
                            EffectOps::Jump => Some(FlowEdge::Jump(labels.next().unwrap())),
                            EffectOps::Branch => Some(FlowEdge::Branch(
                                labels.next().unwrap(),
                                labels.next().unwrap(),
                            )),
                            _ => unreachable!(),
                        }
                    }
                    Instruction::Effect {
                        op: EffectOps::Return,
                        ..
                    } => Some(FlowEdge::Exit),
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
        &self.func.blocks[idx]
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
        ]
    }

    fn graph_nodes(&self, gid: &[usize]) -> Vec<Stmt> {
        // Create the exit node
        let exit_node = &format!("{}_exit", self.graph_id(gid));
        self.func
            .blocks
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
                FlowEdge::Exit => vec![edge!(
                    self.node_id(gid, i) => node_id!(exit_node);
                    attr!("color", "black")
                )
                .into()],
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

use crate::{BBFunction, BasicBlock, GraphLike};
use bril_rs::{EffectOps, Instruction};
use graphviz_rust::{
    dot_generator::{attr, edge, id},
    dot_structures::{Attribute, Edge, EdgeTy, Id, Stmt, Vertex},
};

enum FlowEdge {
    Exit,
    Branch(usize, usize),
    Jump(usize),
}

impl FlowEdge {
    fn vec(&self) -> Vec<usize> {
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
    preds: Vec<Vec<usize>>,
    succs: Vec<FlowEdge>,
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

    pub fn get(&self, idx: usize) -> &BasicBlock {
        &self.func.blocks[idx]
    }
}

impl GraphLike for CFG {
    type N = BasicBlock;

    fn node_attrs(&self, node: &BasicBlock) -> Vec<Attribute> {
        vec![
            attr!("label", &format!(r#""{}""#, node.label_or_default())),
            attr!("shape", "oval"),
        ]
    }

    fn graph_stmts(&self, gid: &[usize]) -> Vec<Stmt> {
        let mut stmts = vec![
            attr!("label", &format!(r#""{}""#, self.func.name)).into(),
            attr!("color", "blue").into(),
        ];

        // Add nodes
        stmts.extend(
            self.func
                .blocks
                .iter()
                .enumerate()
                .map(|(i, bb)| self.node(gid, bb, i).into()),
        );

        // Add edges
        stmts.extend(
            self.succs
                .iter()
                .enumerate()
                .flat_map(|(i, succs)| match succs {
                    FlowEdge::Exit => vec![],
                    FlowEdge::Branch(t, f) => vec![
                    edge!(self.node_id(gid, i) => self.node_id(gid, *t); attr!("color", "green"))
                        .into(),
                    edge!(self.node_id(gid, i) => self.node_id(gid, *f); attr!("color", "red"))
                        .into(),
                ],
                    FlowEdge::Jump(j) => {
                        vec![edge!(self.node_id(gid, i) => self.node_id(gid, *j)).into()]
                    }
                }),
        );

        stmts
    }
}

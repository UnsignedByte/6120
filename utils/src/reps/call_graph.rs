use std::collections::HashMap;

use bril_rs::{Code, EffectOps, Function, Instruction, Program, ValueOps};
use graphviz_rust::{
    dot_generator::{attr, edge, id, node_id},
    dot_structures::{Attribute, Edge, EdgeTy, Id, NodeId, Stmt, Vertex},
};
use itertools::Itertools;

use crate::GraphLike;

use super::CFG;

/// Call graph of the program
pub struct CallGraph {
    prog: Program,
    idx_map: HashMap<String, usize>,
    preds: Vec<Vec<usize>>,
    succs: Vec<Vec<usize>>,
}

impl CallGraph {
    pub fn new(prog: Program) -> Self {
        let funcs = &prog.functions;
        let n = funcs.len();

        let idx_map: HashMap<String, usize> = funcs
            .iter()
            .enumerate()
            .map(|(i, func)| (func.name.clone(), i))
            .collect();

        let succs: Vec<Vec<usize>> = funcs
            .iter()
            .map(|func| {
                func.instrs
                    .iter()
                    .filter_map(|instr| match instr {
                        Code::Instruction(
                            Instruction::Effect {
                                op: EffectOps::Call,
                                funcs,
                                ..
                            }
                            | Instruction::Value {
                                op: ValueOps::Call,
                                funcs,
                                ..
                            },
                        ) => Some(funcs.iter().map(|f| idx_map[f])),
                        _ => None,
                    })
                    .flatten()
                    .unique()
                    .collect()
            })
            .collect();

        let mut preds = vec![vec![]; n];
        for (i, succs) in succs.iter().enumerate() {
            for j in succs {
                preds[*j].push(i);
            }
        }

        Self {
            prog,
            idx_map,
            preds,
            succs,
        }
    }

    pub fn len(&self) -> usize {
        self.prog.functions.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.prog.functions.is_empty()
    }

    pub fn preds(&self, name: &str) -> &[usize] {
        &self.preds[self.idx_map[name]]
    }

    pub fn succs(&self, name: &str) -> &[usize] {
        &self.succs[self.idx_map[name]]
    }

    pub fn get(&self, name: &str) -> &Function {
        &self.prog.functions[self.idx_map[name]]
    }
}

impl GraphLike<CFG> for CallGraph {
    fn node(&self, gid: &[usize], cfg: CFG, id: usize) -> Stmt {
        let new_gid = gid.iter().chain([id].iter()).copied().collect::<Vec<_>>();

        cfg.graph(&new_gid).into()
    }

    fn graph_stmts(&self, gid: &[usize]) -> Vec<Stmt> {
        let mut stmts = vec![attr!("peripheries", "0").into()];

        // Add nodes
        stmts.extend(
            self.prog
                .functions
                .iter()
                .enumerate()
                .map(|(i, bb)| self.node(gid, bb.clone().into(), i)),
        );

        // Add edges
        stmts.extend(self.succs.iter().enumerate().flat_map(|(i, succs)| {
            succs.iter().map(move |&j| {
                // Because of the limitations of graphviz cluster subgraphs, we need to generate the edges between the exit and entry nodes
                let src_cluster = self.node_id(gid, i).0;
                let dst_cluster = self.node_id(gid, j).0;

                let src_entry = format!("{}_exit", src_cluster);
                let dst_entry = format!("{}_0", dst_cluster);

                edge!(
                    node_id!(src_entry) => node_id!(dst_entry);
                    attr!("color", "purple")
                )
                .into()
            })
        }));

        stmts
    }
}

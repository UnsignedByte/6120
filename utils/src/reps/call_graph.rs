use crate::{BasicBlock, Dataflow, DataflowNode, GraphLike};
use bril_rs::{Code, EffectOps, Function, Instruction, Program, ValueOps};
use graphviz_rust::{
    dot_generator::{attr, edge, id, node_id},
    dot_structures::{Attribute, Edge, EdgeTy, Id, NodeId, Stmt, Subgraph, Vertex},
};
use itertools::Itertools;
use std::collections::HashMap;

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

    pub fn prog(&self) -> &Program {
        &self.prog
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

impl<'bb, SG> GraphLike<SG> for CallGraph
where
    SG: From<Function> + GraphLike<&'bb BasicBlock>,
{
    fn node(&self, gid: &[usize], node: SG, id: usize) -> Stmt {
        let new_gid = gid.iter().chain([id].iter()).copied().collect::<Vec<_>>();

        Subgraph {
            id: id!(&format!(
                "{}_wrapper",
                <Self as GraphLike<SG>>::graph_id(self, &new_gid)
            )),
            stmts: vec![
                attr!("peripheries", 0).into(),
                attr!("margin", 15).into(),
                node.graph(&new_gid).into(),
            ],
        }
        .into()
    }

    fn graph_attrs(&self) -> Vec<Stmt> {
        vec![attr!("peripheries", 0).into(), attr!("margin", 10).into()]
    }

    fn graph_nodes(&self, gid: &[usize]) -> Vec<Stmt> {
        self.prog
            .functions
            .iter()
            .enumerate()
            .map(|(i, bb)| self.node(gid, SG::from(bb.clone()), i))
            .collect()
    }

    fn graph_edges(&self, gid: &[usize]) -> Vec<Stmt> {
        self.succs
            .iter()
            .enumerate()
            .flat_map(|(i, succs)| {
                succs.iter().map(move |&j| {
                    // Because of the limitations of graphviz cluster subgraphs, we need to generate the edges between the exit and entry nodes
                    let src_cluster = <Self as GraphLike<SG>>::node_id(self, gid, i).0;
                    let dst_cluster = <Self as GraphLike<SG>>::node_id(self, gid, j).0;

                    let src_exit = format!("{}_0", src_cluster);
                    let dst_entry = format!("{}_0", dst_cluster);

                    edge!(
                        node_id!(src_exit) => node_id!(dst_entry);
                        attr!("color", "purple"),
                        attr!("lhead", dst_cluster),
                        attr!("ltail", src_cluster)
                    )
                    .into()
                })
            })
            .collect()
    }
}

impl GraphLike<Function> for CallGraph {
    fn node_attrs(&self, node: Function) -> Vec<Attribute> {
        vec![
            attr!("label", &format!(r#""{}""#, node.name)),
            attr!("shape", "oval"),
            attr!("color", "darkgray"),
            attr!("style", "filled"),
            attr!("fillcolor", "lightgray"),
        ]
    }

    fn graph_attrs(&self) -> Vec<Stmt> {
        vec![attr!("peripheries", "0").into()]
    }

    fn graph_nodes(&self, gid: &[usize]) -> Vec<Stmt> {
        self.prog
            .functions
            .iter()
            .enumerate()
            .map(|(i, bb)| self.node(gid, bb.clone(), i))
            .collect()
    }

    fn graph_edges(&self, gid: &[usize]) -> Vec<Stmt> {
        self.succs
            .iter()
            .enumerate()
            .flat_map(|(i, succs)| {
                succs.iter().map(move |&j| {
                    let src = <Self as GraphLike<Function>>::node_id(self, gid, i);
                    let dst = <Self as GraphLike<Function>>::node_id(self, gid, j);

                    edge!(
                        src => dst;
                        attr!("color", "purple")
                    )
                    .into()
                })
            })
            .collect()
    }
}

impl<'bb, Val: 'bb> GraphLike<&Dataflow<Val>> for (CallGraph, Vec<Dataflow<Val>>)
where
    Dataflow<Val>: GraphLike<DataflowNode<'bb, Val>>,
{
    fn node(&self, gid: &[usize], node: &Dataflow<Val>, id: usize) -> Stmt {
        let new_gid = gid.iter().chain([id].iter()).copied().collect::<Vec<_>>();

        Subgraph {
            id: id!(&format!(
                "{}_wrapper",
                <Self as GraphLike<&Dataflow<Val>>>::graph_id(self, &new_gid)
            )),
            stmts: vec![
                attr!("peripheries", 0).into(),
                attr!("margin", 15).into(),
                node.graph(&new_gid).into(),
            ],
        }
        .into()
    }

    fn graph_attrs(&self) -> Vec<Stmt> {
        vec![attr!("peripheries", "0").into()]
    }

    fn graph_nodes(&self, gid: &[usize]) -> Vec<Stmt> {
        self.1
            .iter()
            .enumerate()
            .map(|(i, val)| self.node(gid, val, i))
            .collect()
    }

    fn graph_edges(&self, gid: &[usize]) -> Vec<Stmt> {
        self.0
            .succs
            .iter()
            .enumerate()
            .flat_map(|(i, succs)| {
                succs.iter().map(move |&j| {
                    // Because of the limitations of graphviz cluster subgraphs, we need to generate the edges between the exit and entry nodes
                    let src_cluster = <Self as GraphLike<&Dataflow<Val>>>::node_id(self, gid, i).0;
                    let dst_cluster = <Self as GraphLike<&Dataflow<Val>>>::node_id(self, gid, j).0;

                    let src_exit = format!("{}_0", src_cluster);
                    let dst_entry = format!("{}_0", dst_cluster);

                    edge!(
                        node_id!(src_exit) => node_id!(dst_entry);
                        attr!("color", "purple"),
                        attr!("lhead", dst_cluster),
                        attr!("ltail", src_cluster)
                    )
                    .into()
                })
            })
            .collect()
    }
}

use std::collections::HashSet;

use bril_rs::Function;
use graphviz_rust::{
    dot_generator::{attr, edge, id, node, node_id},
    dot_structures::{Attribute, Edge, EdgeTy, Id, Node, NodeId, Stmt, Vertex},
};

use crate::{Dataflow, DataflowPass, DominatorPass, GraphLike, CFG};

use super::BasicBlock;

pub struct DominatorTree {
    /// The control flow graph.
    pub cfg: CFG,
    /// The strictdominator set for each basic block.
    strict_doms: Vec<HashSet<usize>>,
    /// Immediate dominator for each basic block.
    immediate_doms: Vec<Option<usize>>,
}

impl DominatorTree {
    pub fn new(cfg: CFG) -> Self {
        let Dataflow {
            out_vals: mut doms,
            cfg,
            exit_val: exit_doms,
            ..
        } = DominatorPass.cfg(cfg);

        // Add the exit doms to the set
        doms.push(exit_doms);

        let mut strict_doms = doms;
        log::trace!("Dominators: {:?}", strict_doms);

        // Make dominator sets strict
        for (i, sdom) in strict_doms.iter_mut().enumerate() {
            sdom.remove(&i);
        }

        log::trace!("Strict Dominators: {:?}", strict_doms);

        // A dominator is immediate if it is in the dominator set of the block but not in the dominator set of any other dominator.

        let mut immediate_doms = vec![];
        for doms in &strict_doms {
            // These are dominators that dominate other dominators, so cannot be immediate dominators.
            let non_immediate_doms: HashSet<_> = doms
                .iter()
                .flat_map(|&d| &strict_doms[d])
                .copied()
                .collect();

            let candidates: Vec<_> = doms.difference(&non_immediate_doms).collect();

            // There should be at most one immediate dominator.
            assert!(candidates.len() <= 1, "Multiple immediate dominators found");

            immediate_doms.push(candidates.into_iter().next().copied());
        }

        log::trace!("Immediate Dominaors: {:?}", immediate_doms);

        Self {
            cfg,
            strict_doms,
            immediate_doms,
        }
    }

    pub fn strict_doms(&self, idx: usize) -> &HashSet<usize> {
        &self.strict_doms[idx]
    }

    pub fn immediate_dom(&self, idx: usize) -> Option<usize> {
        self.immediate_doms[idx]
    }

    pub fn strictly_dominates(&self, a: usize, b: usize) -> bool {
        self.strict_doms[b].contains(&a)
    }

    pub fn dominates(&self, a: usize, b: usize) -> bool {
        a == b || self.strictly_dominates(a, b)
    }
}

impl From<CFG> for DominatorTree {
    fn from(cfg: CFG) -> Self {
        Self::new(cfg)
    }
}

impl From<Function> for DominatorTree {
    fn from(func: Function) -> Self {
        CFG::from(func).into()
    }
}

impl GraphLike<&BasicBlock> for DominatorTree {
    fn node_attrs(&self, bb: &BasicBlock) -> Vec<Attribute> {
        bb.node_attrs()
    }

    fn graph_stmts(&self, gid: &[usize]) -> Vec<Stmt> {
        let mut stmts = vec![
            attr!("label", &format!(r#""{}""#, self.cfg.name())).into(),
            attr!("color", "darkgray").into(),
            attr!("style", "rounded").into(),
            attr!("bgcolor", "lightgray").into(),
        ];

        // Add nodes
        stmts.extend(
            self.cfg
                .func
                .blocks
                .iter()
                .enumerate()
                .map(|(i, bb)| self.node(gid, bb, i)),
        );

        // Create the exit node
        let exit_node = &format!("{}_exit", self.graph_id(gid));
        stmts.push(node!(exit_node; attr!("label", "exit"), attr!("color", "purple"), attr!("rank", "sink")).into());

        // Add edges
        stmts.extend(
            self.immediate_doms
                .iter()
                .enumerate()
                .filter_map(|(node, &dominator)| {
                    dominator.map(|dom| {
                        let src = self.node_id(gid, dom);
                        if node == self.cfg.len() {
                            edge!(src => node_id!(exit_node))
                        } else {
                            let dst = self.node_id(gid, node);
                            edge!(src => dst)
                        }
                    })
                })
                .map(Stmt::Edge),
        );

        stmts
    }
}

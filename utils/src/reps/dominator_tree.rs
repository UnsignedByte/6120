use std::collections::HashSet;

use bril_rs::Function;
use graphviz_rust::{
    dot_generator::{attr, edge, id, node, node_id},
    dot_structures::{Attribute, Edge, EdgeTy, Id, Node, NodeId, Stmt, Vertex},
};

use crate::{Dataflow, DataflowPass, DominatorPass, GraphLike, CFG};

use super::{FlowEdge, BasicBlock};

pub struct DominatorTree {
    /// The control flow graph.
    pub cfg: CFG,
    /// The strict dominator set for each basic block.
    strict_doms: Vec<HashSet<usize>>,
    /// The set of blocks strictly dominated by each block.
    strict_dom_bys: Vec<HashSet<usize>>,
    /// Immediate dominator for each basic block.
    immediate_doms: Vec<Option<usize>>,
    /// Dominance frontier for each basic block.
    dominance_frontiers: Vec<HashSet<usize>>,
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

        let immediate_doms = strict_doms.iter().map(|sdoms| {
            // These are dominators that dominate other dominators, so cannot be immediate dominators.
            let non_immediate_doms: HashSet<_> = sdoms
                .iter()
                .flat_map(|&d| &strict_doms[d])
                .copied()
                .collect();

            let candidates: Vec<_> = sdoms.difference(&non_immediate_doms).collect();

            // There should be at most one immediate dominator.
            assert!(candidates.len() <= 1, "Multiple immediate dominators found");

            candidates.into_iter().next().copied()
        }).collect();

        log::trace!("Immediate Dominaors: {:?}", immediate_doms);

        // A block's dominance frontier is a set of values it does not dominate
        // but it dominates a predecessor of the value.
        // First, find the set of nodes dominated by each block.
        let mut strict_dom_bys = vec![HashSet::new(); cfg.len() + 1];
        for (i, doms) in strict_doms.iter().enumerate() {
            for &dom in doms {
                // dom dominates i
                strict_dom_bys[dom].insert(i);
            }
        }

        // Now, find the dominance frontier for each block.
        let dominance_frontiers = strict_dom_bys
            .iter()
            .enumerate()
            .map(|(i, sdom_by)| {
                // Get the set of all successors of dominated blocks
                let candidates = sdom_by.iter()
                    .flat_map(|&dom| cfg.succs(dom))
                    .chain(cfg.succs(i)) // i also dominates itself
                    .collect::<HashSet<_>>();

                // The dominance frontier is the set of all successors that are not strictly dominated by i
                candidates.difference(sdom_by).copied().collect()
            })
            .collect();

        Self {
            cfg,
            strict_doms,
            immediate_doms,
            dominance_frontiers,
            strict_dom_bys,
        }
    }

    pub fn strict_doms(&self, idx: usize) -> &HashSet<usize> {
        &self.strict_doms[idx]
    }

    pub fn immediate_dom(&self, idx: usize) -> Option<usize> {
        self.immediate_doms[idx]
    }

    pub fn dominance_frontier(&self, idx: usize) -> &HashSet<usize> {
        &self.dominance_frontiers[idx]
    }

    pub fn strictly_dominates(&self, a: usize, b: usize) -> bool {
        self.strict_doms[b].contains(&a)
    }

    pub fn strictly_dominated_by(&self, a: usize, b: usize) -> bool {
        self.strict_dom_bys[a].contains(&b)
    }

    pub fn dominates(&self, a: usize, b: usize) -> bool {
        a == b || self.strictly_dominates(a, b)
    }

    pub fn dominated_by(&self, a: usize, b: usize) -> bool {
        a == b || self.strictly_dominated_by(a, b)
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

        // Add Dominator tree edges
        stmts.extend(
            self.immediate_doms
                .iter()
                .enumerate()
                .filter_map(|(node, &dominator)| {
                    dominator.map(|dom| {
                        let src = self.node_id(gid, dom);
                        
                        if node == self.cfg.len() {
                            (src, node_id!(exit_node))
                        } else {
                            (src,  self.node_id(gid, node))
                        }
                    })
                })
                .map(|(src, dst)| {
                    edge!(src => dst; attr!("color", "black")).into()
                })
        );

        // Add CFG edges in dashed gray
        stmts.extend(
            self.cfg
                .succs
                .iter()
                .enumerate()
                .flat_map(|(src, succs)| {
                    let src = self.node_id(gid, src);
                    
                    match succs {
                    FlowEdge::Exit => 
                    vec![(src, node_id!(exit_node), "gray")],
                    FlowEdge::Branch(a, b) => vec![
                        (src.clone(), self.node_id(gid, *a), "forestgreen"),
                        (src, self.node_id(gid, *b), "firebrick"),
                    ],
                    FlowEdge::Jump(dst) => vec![(src, self.node_id(gid, *dst), "gray")],
                }
            }).map(|(src, dst, color)| {
                edge!(src => dst; attr!("color", color), attr!("style", "dashed"), attr!("constraint", "false"), attr!("penwidth", 0.75), attr!("arrowsize", 0.75)).into()
            })
        );

        stmts
    }
}

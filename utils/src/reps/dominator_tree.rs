use std::collections::HashSet;

use bril_rs::Function;
use graphviz_rust::{
    dot_generator::{attr, edge, id, node_id},
    dot_structures::{Attribute, Edge, EdgeTy, Id, NodeId, Stmt, Vertex},
};

use crate::{Dataflow, DataflowPass, DominatorPass, GraphLike, CFG};

use super::BasicBlock;

#[derive(Debug, Clone)]
pub struct DominatorTree {
    /// The control flow graph.
    pub cfg: CFG,
    /// The strict dominator set for each basic block.
    strict_doms: Vec<HashSet<usize>>,
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

        let n = cfg.len();

        // Add the exit doms to the set
        doms.push(exit_doms);

        // A block's dominance frontier is a set of values it does not dominate
        // but it dominates a predecessor of the value.
        // First, find the set of nodes dominated by each block.
        let mut dom_bys = vec![HashSet::new(); n + 1];
        for (i, doms) in doms.iter().enumerate() {
            for &dom in doms {
                // dom dominates i
                dom_bys[dom].insert(i);
            }
        }

        // Now, find the dominance frontier for each block.
        let dominance_frontiers = dom_bys
            .iter()
            .map(|dom_by| {
                // Get the set of all successors of dominated blocks
                let candidates = dom_by
                    .iter()
                    .flat_map(|&dom| if dom < n { cfg.succs(dom) } else { vec![] })
                    .collect::<HashSet<_>>();

                // The dominance frontier is the set of all successors that are not strictly dominated by i
                candidates.difference(dom_by).copied().collect()
            })
            .collect();

        let mut strict_doms = doms;
        log::trace!("Dominators: {:?}", strict_doms);

        // Make dominator sets strict
        for (i, sdom) in strict_doms.iter_mut().enumerate() {
            sdom.remove(&i);
        }

        log::trace!("Strict Dominators: {:?}", strict_doms);

        // A dominator is immediate if it is in the dominator set of the block but not in the dominator set of any other dominator.

        let immediate_doms = strict_doms
            .iter()
            .map(|sdoms| {
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
            })
            .collect();

        log::trace!("Immediate Dominators: {:?}", immediate_doms);

        log::trace!("Dominance Frontiers: {:?}", dominance_frontiers);

        Self {
            cfg,
            strict_doms,
            immediate_doms,
            dominance_frontiers,
        }
    }

    pub fn strict_doms(&self, idx: usize) -> &HashSet<usize> {
        &self.strict_doms[idx]
    }

    pub fn dominators(&self, idx: usize) -> HashSet<usize> {
        self.strict_doms[idx]
            .iter()
            .copied()
            .chain(std::iter::once(idx))
            .collect()
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
        self.strictly_dominates(b, a)
    }

    pub fn dominates(&self, a: usize, b: usize) -> bool {
        a == b || self.strictly_dominates(a, b)
    }

    pub fn dominated_by(&self, a: usize, b: usize) -> bool {
        self.dominates(b, a)
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

    fn graph_attrs(&self) -> Vec<Stmt> {
        vec![
            attr!("label", &format!(r#""{}""#, self.cfg.name())).into(),
            attr!("color", "darkgray").into(),
            attr!("style", "rounded").into(),
            attr!("bgcolor", "lightgray").into(),
        ]
    }

    fn graph_nodes(&self, gid: &[usize]) -> Vec<Stmt> {
        self.cfg.graph_nodes(gid)
    }

    fn graph_edges(&self, gid: &[usize]) -> Vec<Stmt> {
        let exit_node = &format!("{}_exit", self.graph_id(gid));
        self.immediate_doms // Add Dominator tree edges
            .iter()
            .enumerate()
            .filter_map(|(node, &dominator)| {
                dominator.map(|dom| {
                    let src = self.node_id(gid, dom);

                    if node == self.cfg.len() {
                        (src, node_id!(exit_node))
                    } else {
                        (src, self.node_id(gid, node))
                    }
                })
            })
            .map(|(src, dst)| edge!(src => dst; attr!("color", "black")).into())
            .chain(self.cfg.graph_edges(gid).into_iter().map(|e| {
                match e {
                    Stmt::Edge(mut e) => {
                        // Replace the color of the edge
                        if let Some(Attribute(_, Id::Plain(v))) =
                            e.attributes.iter_mut().find(|attr| {
                                if let Attribute(Id::Plain(k), _) = attr {
                                    k == "color"
                                } else {
                                    false
                                }
                            })
                        {
                            *v = match v.as_str() {
                                "black" => "gray",
                                "red" => "firebrick",
                                "green" => "forestgreen",
                                _ => panic!("Unexpected color: {}", v),
                            }
                            .to_owned()
                        } else {
                            // Set color to gray
                            e.attributes.push(attr!("color", "gray"));
                        }

                        // add new attributes
                        e.attributes.extend(vec![
                            attr!("style", "dashed"),
                            attr!("constraint", "false"),
                            attr!("penwidth", 0.75),
                            attr!("arrowsize", 0.75),
                        ]);

                        e.into()
                    }
                    _ => unreachable!(),
                }
            }))
            .collect()
    }
}

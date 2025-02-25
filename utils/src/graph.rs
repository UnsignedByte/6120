use crate::{CFG, CallGraph, Dataflow, DataflowLabel, DataflowPass};
use graphviz_rust::{
    dot_generator::{attr, id},
    dot_structures::{Attribute, Graph, Id, Node, NodeId, Stmt, Subgraph},
    printer::{DotPrinter, PrinterContext},
};
use std::fmt::Debug;

pub trait GraphLike<N> {
    fn node_id(&self, gid: &[usize], id: usize) -> NodeId {
        NodeId(
            Id::Plain(format!(
                "cluster_{}",
                gid.iter()
                    .chain(std::iter::once(&id))
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("_"),
            )),
            None,
        )
    }

    fn node_attrs(&self, _node: N) -> Vec<Attribute> {
        vec![]
    }

    fn node(&self, gid: &[usize], node: N, id: usize) -> Stmt {
        Node {
            id: self.node_id(gid, id),
            attributes: self.node_attrs(node),
        }
        .into()
    }

    fn graph_id(&self, gid: &[usize]) -> Id {
        Id::Plain(format!(
            "cluster_{}",
            gid.iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("_")
        ))
    }

    fn graph_attrs(&self) -> Vec<Stmt> {
        vec![]
    }

    fn graph_nodes(&self, _gid: &[usize]) -> Vec<Stmt> {
        vec![]
    }

    fn graph_edges(&self, _gid: &[usize]) -> Vec<Stmt> {
        vec![]
    }

    fn graph(&self, gid: &[usize]) -> Subgraph {
        Subgraph {
            id: self.graph_id(gid),
            stmts: self
                .graph_attrs()
                .into_iter()
                .chain(self.graph_nodes(gid))
                .chain(self.graph_edges(gid))
                .collect(),
        }
    }
}

pub fn draw<N>(graph: impl GraphLike<N>, directional: bool, strict: bool) -> String {
    let sg = graph.graph(&[]);
    let stmts = vec![attr!("compound", "true").into(), sg.into()];

    let g = match directional {
        true => Graph::DiGraph {
            id: id!("G"),
            strict,
            stmts,
        },
        false => Graph::Graph {
            id: Id::Plain("G".to_owned()),
            strict,
            stmts,
        },
    };

    g.print(&mut PrinterContext::default())
}

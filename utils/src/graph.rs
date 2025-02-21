use graphviz_rust::{
    dot_generator::{attr, id},
    dot_structures::{Attribute, Graph, Id, Node, NodeId, Stmt, Subgraph},
    printer::{DotPrinter, PrinterContext},
};

pub trait GraphLike {
    type N;

    fn node_id(&self, gid: &[usize], id: usize) -> NodeId {
        NodeId(
            Id::Plain(format!(
                "cluster_{}_{}",
                gid.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("_"),
                id
            )),
            None,
        )
    }

    fn node_attrs(&self, _node: &Self::N) -> Vec<Attribute> {
        vec![]
    }

    fn node(&self, gid: &[usize], node: &Self::N, id: usize) -> Stmt {
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

    fn graph_stmts(&self, _gid: &[usize]) -> Vec<Stmt> {
        vec![]
    }

    fn graph(&self, gid: &[usize]) -> Subgraph {
        Subgraph {
            id: self.graph_id(gid),
            stmts: self.graph_stmts(gid),
        }
    }
}

pub fn draw(graph: &[Box<impl GraphLike>], directional: bool, strict: bool) -> String {
    let mut stmts = vec![attr!("compound", "true").into()];

    stmts.extend(graph.iter().enumerate().map(|(i, g)| g.graph(&[i]).into()));

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

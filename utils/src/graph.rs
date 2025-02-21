use graphviz_rust::{
    dot_generator::id,
    dot_structures::{Attribute, Graph, Id, Node, NodeId, Stmt, Subgraph},
    printer::{DotPrinter, PrinterContext},
};

pub trait GraphLike {
    type N;

    fn node_id(&self, gid: &[usize], id: usize) -> NodeId {
        NodeId(
            Id::Plain(format!(
                "subgraph_{}_node_{}",
                gid.iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
                    .join("_"),
                id
            )),
            None,
        )
    }

    fn node_attrs(&self, node: &Self::N) -> Vec<Attribute>;

    fn node(&self, gid: &[usize], node: &Self::N, id: usize) -> Node {
        Node {
            id: self.node_id(gid, id),
            attributes: self.node_attrs(node),
        }
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

    fn graph_stmts(&self, gid: &[usize]) -> Vec<Stmt>;

    fn graph(&self, gid: &[usize]) -> Subgraph {
        Subgraph {
            id: self.graph_id(gid),
            stmts: self.graph_stmts(gid),
        }
    }
}

pub fn draw(graph: &[Box<impl GraphLike>], directional: bool, strict: bool) -> String {
    let stmts = graph
        .iter()
        .enumerate()
        .map(|(i, g)| g.graph(&[i]).into())
        .collect::<Vec<_>>();

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

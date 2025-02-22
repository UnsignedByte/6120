use graphviz_rust::{
    dot_generator::{attr, id},
    dot_structures::{Attribute, Graph, Id, Node, NodeId, Stmt, Subgraph},
    printer::{DotPrinter, PrinterContext},
};
use utils::{BasicBlock, DominatorTree, GraphLike};

struct DominationPath {
    tree: DominatorTree,
    selected: usize,
}

impl GraphLike<&BasicBlock> for DominationPath {
    fn node_attrs(&self, node: &BasicBlock) -> Vec<Attribute> {
        vec![if node.idx == self.selected {
            // if this node is the selected node, color it red
            attr!("color", "red")
        } else if self.tree.strictly_dominates(node.idx, self.selected) {
            // if this node dominates the selected node, color it green
            attr!("color", "green")
        } else {
            attr!("color", "black")
        }]
    }
}

fn main() {}

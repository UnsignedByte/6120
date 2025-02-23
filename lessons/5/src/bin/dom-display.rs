use bril_rs::Function;
use graphviz_rust::{
    dot_generator::{attr, id, node},
    dot_structures::{Attribute, Id, Node, NodeId, Stmt},
};
use utils::{
    draw, run_analysis, setup_logger_from_env, AnalysisPass, BasicBlock, CallGraph, DominatorTree,
    GraphLike,
};

#[derive(Debug, Clone)]
struct DomDisplay {
    tree: DominatorTree,
    selected: usize,
}

impl GraphLike<&BasicBlock> for DomDisplay {
    fn node_attrs(&self, node: &BasicBlock) -> Vec<Attribute> {
        let mut attrs = self.tree.cfg.node_attrs(node);

        log::debug!("Selected: {}, Node: {}", self.selected, node.idx);

        let mut colors = vec![];

        if self
            .tree
            .dominance_frontier(self.selected)
            .contains(&node.idx)
        {
            // Node is in the dominance frontier of the selected node
            colors.push("cadetblue2");
        }

        if self.tree.strictly_dominates(node.idx, self.selected) {
            // Node is strictly dominated by the selected node
            colors.push("darkseagreen4");
        }

        if node.idx == self.selected {
            // Node is the selected node
            colors.push("coral2");
        }

        if self.tree.strictly_dominated_by(node.idx, self.selected) {
            // Node strictly dominates the selected node
            colors.push("darkseagreen1");
        }

        let style = if colors.len() > 1 { "wedged" } else { "filled" };
        let color = format!(r#""{}""#, colors.join(":"));

        // Change the colors based on the dominance relationship
        attrs.extend(vec![
            attr!("style", style),
            attr!("fillcolor", color),
            attr!("color", "gray"),
        ]);

        attrs
    }

    fn graph_attrs(&self) -> Vec<Stmt> {
        self.tree.cfg.graph_attrs()
    }

    fn graph_nodes(&self, gid: &[usize]) -> Vec<Stmt> {
        let exit_node = &format!("{}_exit", self.graph_id(gid));
        self.tree.cfg
                .func
                .blocks
                .iter()
                .enumerate()
                .map(|(i, bb)| self.node(gid, bb, i))
                .chain(std::iter::once(node!(exit_node; attr!("label", "exit"), attr!("color", "purple"), attr!("rank", "sink")).into())).collect()
    }

    fn graph_edges(&self, gid: &[usize]) -> Vec<Stmt> {
        self.tree.cfg.graph_edges(gid)
    }
}

impl From<Function> for DomDisplay {
    fn from(func: Function) -> Self {
        let tree = DominatorTree::from(func);

        // Find the block in the function named "selected"
        let selected = tree
            .cfg
            .func
            .blocks
            .iter()
            .find(|bb| bb.label == Some("selected".to_owned()))
            .map(|bb| bb.idx)
            .expect("No block named 'selected'");

        log::debug!("{} selected block: {}", tree.cfg.name(), selected);

        Self { tree, selected }
    }
}

/// Pass to find the selected block and display its dominance information
struct DomDisplayPass;

impl AnalysisPass for DomDisplayPass {
    fn program(&mut self, prog: &bril_rs::Program) -> Result<(), String> {
        let call_graph = CallGraph::new(prog.clone());

        let dot = draw::<DomDisplay>(call_graph, true, false);

        println!("{}", dot);

        Ok(())
    }
}

fn main() {
    setup_logger_from_env();
    run_analysis(DomDisplayPass);
}

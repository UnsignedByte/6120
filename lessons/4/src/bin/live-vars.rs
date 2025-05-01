use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use utils::{
    AnalysisPass, BBFunction, BasicBlock, CFG, CallGraph, DataflowLabel, DataflowPass, InstrExt,
    draw_dataflow, run_analysis, setup_logger_from_env,
};

#[derive(Default)]
struct LiveVars;

impl DataflowPass<HashSet<String>> for LiveVars {
    fn reversed(&self) -> bool {
        true
    }

    fn init(&self, _: &BBFunction) -> HashSet<String> {
        HashSet::default()
    }

    fn meet(&self, in_vals: &[HashSet<String>]) -> HashSet<String> {
        // The meet in live vars is set union
        in_vals.iter().flatten().cloned().collect()
    }

    fn transfer(&self, block: &BasicBlock, in_val: &HashSet<String>) -> HashSet<String> {
        let mut out_vals = in_val.clone();

        for insn in block.iter().rev() {
            log::trace!("Processing instruction: {}", insn);

            // Remove the destination from the set
            if let Some(dest) = insn.dest() {
                out_vals.remove(&dest);
            }

            // Add the arguments to the set
            if let Some(args) = insn.args() {
                for arg in args {
                    out_vals.insert(arg);
                }
            }
        }

        out_vals
    }
}

/// Dominator set node used to display graphs
/// For dominator sets
#[derive(Clone)]
pub struct GraphNode {
    vars: HashSet<String>,
}

impl DataflowLabel for GraphNode {
    fn in_label(&self, _: &CFG) -> Option<String> {
        // Create a set of variables
        let mut vars = self.vars.iter().cloned().collect_vec();

        vars.sort_unstable();

        // Return a string representation of the set
        Some(format!("In: {}\\l", vars.join(", ")))
    }

    fn out_label(&self, _: &CFG) -> Option<String> {
        // Create a set of variables
        let mut vars = self.vars.iter().cloned().collect_vec();

        vars.sort_unstable();

        // Return a string representation of the set
        Some(format!("Out: {}\\l", vars.join(", ")))
    }
}

impl From<HashSet<String>> for GraphNode {
    fn from(vars: HashSet<String>) -> Self {
        Self { vars }
    }
}

struct Drawer;

impl AnalysisPass for Drawer {
    fn program(&mut self, prog: &bril_rs::Program) -> Result<(), String> {
        let call_graph = CallGraph::new(prog.clone());

        let dot = draw_dataflow::<LiveVars, HashSet<String>, GraphNode>(call_graph, true, false);

        println!("{}", dot);

        Ok(())
    }
}

fn main() {
    setup_logger_from_env();
    run_analysis(Drawer);
}

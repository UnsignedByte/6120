use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use utils::{
    AnalysisPass, BBFunction, BasicBlock, CFG, CallGraph, DataflowLabel, DataflowPass, InstrExt,
    draw_dataflow, run_analysis, setup_logger_from_env,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq, PartialOrd)]
struct Definition {
    name: String,
    block: usize,
}

#[derive(Default)]
struct ReachingDefs;

impl DataflowPass<HashSet<Definition>> for ReachingDefs {
    fn init(&self, func: &BBFunction) -> HashSet<Definition> {
        func.args
            .iter()
            .map(|arg| Definition {
                block: 0,
                name: arg.name.clone(),
            })
            .collect()
    }

    fn meet(&self, in_vals: &[HashSet<Definition>]) -> HashSet<Definition> {
        // The meet in reaching definitions is set union
        in_vals.iter().flatten().cloned().collect()
    }

    fn transfer(&self, block: &BasicBlock, in_val: &HashSet<Definition>) -> HashSet<Definition> {
        // Set of defined names in this block
        let defines: HashSet<_> = block.instrs.iter().filter_map(|insn| insn.dest()).collect();

        // Kill all definitions in in_vals that write to this name
        let mut out_vals: HashSet<_> = in_val
            .iter()
            .filter(|def| !defines.contains(&def.name))
            .cloned()
            .collect();

        // Add definitions defined in the block
        out_vals.extend(defines.into_iter().map(|name| Definition {
            name,
            block: block.idx,
        }));

        out_vals
    }
}

/// Dominator set node used to display graphs
/// For dominator sets
#[derive(Clone)]
pub struct GraphNode {
    defs: HashSet<Definition>,
}

impl DataflowLabel for GraphNode {
    fn in_label(&self, _: &CFG) -> Option<String> {
        None
    }

    fn out_label(&self, cfg: &CFG) -> Option<String> {
        // Create a set of definitions
        let defs: Vec<(_, _, _)> = self
            .defs
            .iter()
            .map(|Definition { block, name }| {
                log::debug!("Finding {} in {}", name, block);
                let block = cfg.get(*block);
                log::debug!("Finding {} in {}", name, block);
                let last_def = block
                    .instrs
                    .iter()
                    .rev()
                    .find(|instr| match instr.dest() {
                        Some(ref s) => s == name,
                        None => false,
                    })
                    .map(|i| i.value_str().unwrap())
                    .unwrap_or("?".to_owned());

                (name, block, last_def)
            })
            .collect();

        // Collect into a hashmap
        let mut name_map: HashMap<String, Vec<_>> = HashMap::new();

        for (def, block, val) in defs {
            let other_defs = name_map.entry(def.clone()).or_default();

            other_defs.push(format!("[.{}: {}]", block.label_or_default(), val));
        }

        // Convert to a vec and sort everything
        let mut names = name_map
            .into_iter()
            .map(|(name, mut defs)| {
                defs.sort_unstable();
                format!("{} = {}", name, defs.join(", "))
            })
            .collect_vec();

        names.sort_unstable();

        // Return a string representation of the set
        Some(format!("{}\\l", names.join("\\l")))
    }
}

impl From<HashSet<Definition>> for GraphNode {
    fn from(defs: HashSet<Definition>) -> Self {
        Self { defs }
    }
}

struct Drawer;

impl AnalysisPass for Drawer {
    fn program(&mut self, prog: &bril_rs::Program) -> Result<(), String> {
        let call_graph = CallGraph::new(prog.clone());

        let dot =
            draw_dataflow::<ReachingDefs, HashSet<Definition>, GraphNode>(call_graph, true, false);

        println!("{}", dot);

        Ok(())
    }
}

fn main() {
    setup_logger_from_env();
    run_analysis(Drawer);
}

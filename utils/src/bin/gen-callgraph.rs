use std::str::FromStr;

use argh::FromArgs;
use bril_rs::Function;
use std::default::Default;
use utils::{
    draw, run_analysis, setup_logger, AnalysisPass, CallGraph, Dataflow, DataflowPass,
    DominatorPass, DominatorSetNode, DominatorTree, CFG,
};

pub enum SubgraphTypes {
    None,
    CFG,
    DominatorTree,
    DominatorSets,
}

impl FromStr for SubgraphTypes {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "none" => Ok(Self::None),
            "cfg" => Ok(Self::CFG),
            "dominatortree" | "domtree" => Ok(Self::DominatorTree),
            "dominatorsets" | "domsets" => Ok(Self::DominatorSets),
            _ => Err(format!("Unknown subgraph type: {}", s)),
        }
    }
}

/// Draw the call graph of a Bril program.
#[derive(FromArgs)]
struct Options {
    /// log level
    #[argh(option, short = 'l', default = "log::LevelFilter::Info")]
    log: log::LevelFilter,
    /// subgraph type to draw
    #[argh(option, short = 's', default = "SubgraphTypes::None")]
    subgraph: SubgraphTypes,
}

struct CallDrawer {
    sg_ty: SubgraphTypes,
}

impl CallDrawer {
    pub fn new(sg_ty: SubgraphTypes) -> Self {
        Self { sg_ty }
    }
}

impl AnalysisPass for CallDrawer {
    fn program(&mut self, prog: &bril_rs::Program) -> Result<(), String> {
        let call_graph = CallGraph::new(prog.clone());

        let strict = false;

        let dot = match self.sg_ty {
            SubgraphTypes::None => draw::<Function>(call_graph, true, strict),
            SubgraphTypes::CFG => draw::<CFG>(call_graph, true, strict),
            SubgraphTypes::DominatorTree => draw::<DominatorTree>(call_graph, true, strict),
            SubgraphTypes::DominatorSets => {
                // Collect all the dominators
                let dominators = prog
                    .functions
                    .iter()
                    .map(|f| DominatorPass.cfg(CFG::from(f.clone())))
                    .map(<Dataflow<DominatorSetNode>>::from)
                    .collect();
                draw((call_graph, dominators), true, strict)
            }
        };

        println!("{}", dot);

        Ok(())
    }
}

fn main() {
    let args: Options = argh::from_env();
    setup_logger(args.log);
    run_analysis(CallDrawer::new(args.subgraph));
}

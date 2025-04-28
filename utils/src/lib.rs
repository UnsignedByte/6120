mod extensions;
mod graph;
mod logger;
mod macros;
mod misc;
mod passes;
mod reps;

pub use bril_rs;
pub use extensions::{InstrExt, LiteralExt};
pub use graph::{GraphLike, draw};
pub use logger::{LogArgs, setup_logger, setup_logger_from_env};
pub use misc::HashableLiteral;
pub(crate) use passes::DataflowNode;
pub use passes::{
    AnalysisPass, CanonicalizeLiterals, Dataflow, DataflowLabel, DataflowPass, DominatorPass,
    DominatorSetNode, FunctionPass, Pass, RemoveUnlabeledBlocks, draw_dataflow, run_analysis,
    run_passes,
};
pub use reps::{BBFunction, BasicBlock, CFG, CallGraph, ControlFlow, DominatorTree};

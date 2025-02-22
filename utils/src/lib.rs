mod extensions;
mod graph;
mod logger;
mod misc;
mod passes;
mod reps;

pub(crate) use passes::DominatorPass;

pub use extensions::{InstrExt, LiteralExt};
pub use graph::{draw, GraphLike};
pub use logger::{setup_logger, setup_logger_from_env, LogArgs};
pub use misc::HashableLiteral;
pub use passes::{
    run_analysis, run_passes, AnalysisPass, CanonicalizeLiterals, Dataflow, DataflowPass,
    FunctionPass, Pass,
};
pub use reps::{BBFunction, BasicBlock, CallGraph, DominatorTree, CFG};

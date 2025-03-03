mod analysis_pass;
mod dataflow_pass;
mod function_pass;
mod impls;
mod pass;

pub use analysis_pass::{AnalysisPass, run_analysis};
pub(crate) use dataflow_pass::DataflowNode;
pub use dataflow_pass::{Dataflow, DataflowLabel, DataflowPass, draw_dataflow};
pub use function_pass::FunctionPass;
pub use impls::{CanonicalizeLiterals, DominatorPass, DominatorSetNode, RemoveUnlabeledBlocks};
pub use pass::{Pass, run_passes};

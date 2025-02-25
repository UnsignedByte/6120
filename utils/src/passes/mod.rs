mod analysis_pass;
mod dataflow_pass;
mod function_pass;
mod impls;
mod pass;

pub use analysis_pass::{run_analysis, AnalysisPass};
pub(crate) use dataflow_pass::DataflowNode;
pub use dataflow_pass::{Dataflow, DataflowLabel, DataflowPass};
pub use function_pass::FunctionPass;
pub use impls::{CanonicalizeLiterals, DominatorPass, DominatorSetNode};
pub use pass::{run_passes, Pass};

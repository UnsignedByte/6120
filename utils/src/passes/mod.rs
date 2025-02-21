mod analysis_pass;
mod dataflow_pass;
mod function_pass;
mod impls;
mod pass;

pub use analysis_pass::{run_analysis, AnalysisPass};
pub use dataflow_pass::DataflowPass;
pub use function_pass::FunctionPass;
pub use impls::CanonicalizeLiterals;
pub use pass::{run_passes, Pass};

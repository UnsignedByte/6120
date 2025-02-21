use utils::{draw, run_analysis, setup_logger_from_env, AnalysisPass, CallGraph};

#[derive(Default)]
struct CFGDrawer;

impl AnalysisPass for CFGDrawer {
    fn program(&mut self, prog: &bril_rs::Program) {
        let call_graph = CallGraph::new(prog.clone());

        let cfgs: Vec<_> = vec![Box::new(call_graph)];
        let dot = draw(&cfgs, true, true);

        println!("{}", dot);
    }
}

fn main() {
    setup_logger_from_env();
    run_analysis(CFGDrawer);
}

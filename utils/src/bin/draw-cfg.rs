use bril_rs::Function;
use utils::{draw, run_analysis, AnalysisPass, CFG};

#[derive(Default)]
struct CFGDrawer {
    cfgs: Vec<CFG>,
}

impl AnalysisPass for CFGDrawer {
    fn function(&mut self, func: &Function) {
        let cfg = CFG::new(func.clone().into());

        self.cfgs.push(cfg);
    }

    fn finish(&mut self) {
        let cfgs = std::mem::take(&mut self.cfgs);

        let cfgs: Vec<_> = cfgs.into_iter().map(Box::new).collect();
        let dot = draw(&cfgs, true, true);

        println!("{}", dot);
    }
}

fn main() {
    run_analysis(CFGDrawer::default());
}

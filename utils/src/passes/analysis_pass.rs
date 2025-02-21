use bril_rs::{load_abstract_program_from_read, Function, Program};

pub fn run_analysis(mut analysis: impl AnalysisPass) {
    let input = std::io::stdin();

    // Read stdin and parse it into a Program using serde
    let prog: Program = load_abstract_program_from_read(input.lock())
        .try_into()
        .unwrap();

    analysis.run(&prog);
}

pub trait AnalysisPass {
    /// Function to be called on each function in the program
    fn function(&mut self, _func: &Function) {}

    /// Function to be called to analyze an entire program
    fn program(&mut self, _prog: &Program) {}

    /// Analysis reporting and the like
    fn finish(&mut self) {}

    fn run(&mut self, prog: &Program) {
        self.program(prog);

        for func in &prog.functions {
            self.function(func);
        }

        self.finish();
    }
}

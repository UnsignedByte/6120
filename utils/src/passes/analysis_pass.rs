use bril_rs::{load_abstract_program_from_read, Function, Program};

pub fn run_analysis(mut analysis: impl AnalysisPass) {
    let input = std::io::stdin();

    // Read stdin and parse it into a Program using serde
    let prog: Program = load_abstract_program_from_read(input.lock())
        .try_into()
        .unwrap();

    analysis.run(&prog).unwrap_or_else(|e| {
        eprintln!("Analysis failed with error: {}", e);
        std::process::exit(1);
    });
}

pub trait AnalysisPass {
    /// Function to be called on each function in the program
    fn function(&mut self, _func: &Function) -> Result<(), String> {
        Ok(())
    }

    /// Function to be called to analyze an entire program
    fn program(&mut self, _prog: &Program) -> Result<(), String> {
        Ok(())
    }

    /// Analysis reporting and the like
    fn finish(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn run(&mut self, prog: &Program) -> Result<(), String> {
        self.program(prog)?;

        for func in &prog.functions {
            self.function(func)?;
        }

        self.finish()
    }
}

use bril_rs::{load_abstract_program_from_read, output_program, Function, Program};

pub fn run_passes(passes: &mut [Box<dyn Pass>]) {
    let input = std::io::stdin();

    // Read stdin and parse it into a Program using serde
    let mut prog: Program = load_abstract_program_from_read(input.lock())
        .try_into()
        .unwrap();

    // Run each pass on the program
    for pass in passes {
        prog = pass.run(prog);
    }

    output_program(&prog);
}

pub trait Pass {
    /// Function to be called on each function in the program
    fn function(&mut self, func: Function) -> Function {
        func
    }

    /// Function to be called before the pass is run
    fn before(&mut self, prog: Program) -> Program {
        prog
    }

    /// Function to be called after the pass is run
    fn after(&mut self, prog: Program) -> Program {
        prog
    }

    fn run(&mut self, prog: Program) -> Program {
        let mut prog = self.before(prog);

        prog.functions = prog
            .functions
            .into_iter()
            .map(|func| self.function(func))
            .collect();

        self.after(prog)
    }
}

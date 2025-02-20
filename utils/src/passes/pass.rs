use bril_rs::{load_abstract_program_from_read, output_program, Function, Program};

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

    fn run(&mut self, input: impl std::io::Read) {
        // Read stdin and parse it into a Program using serde
        let prog: Program = load_abstract_program_from_read(input).try_into().unwrap();

        let mut prog = self.before(prog);

        prog.functions = prog
            .functions
            .into_iter()
            .map(|func| self.function(func))
            .collect();

        let prog = self.after(prog);
        output_program(&prog);
    }
}

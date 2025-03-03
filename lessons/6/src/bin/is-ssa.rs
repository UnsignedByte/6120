use std::collections::HashSet;

use bril_rs::{Code, Function};
use utils::{AnalysisPass, InstrExt, run_analysis};

struct SSACheck;

impl AnalysisPass for SSACheck {
    fn function(&mut self, func: &Function) -> Result<(), String> {
        // Check that the function is in SSA form
        let mut vars = HashSet::new();
        for instr in &func.instrs {
            if let Code::Instruction(instr) = instr {
                if let Some(dest) = instr.dest() {
                    if vars.contains(&dest) {
                        return Err(format!("Variable {} was assigned more than once", dest));
                    }
                    vars.insert(dest.clone());
                }
            }
        }

        Ok(())
    }
}

fn main() {
    run_analysis(SSACheck);
}

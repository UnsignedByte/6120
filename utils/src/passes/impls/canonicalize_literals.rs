use bril_rs::{Code, Instruction};

use crate::{LiteralExt, Pass};

/// Pass to canonicalize literals to the right type
pub struct CanonicalizeLiterals;

impl Pass for CanonicalizeLiterals {
    fn function(&mut self, mut func: bril_rs::Function) -> bril_rs::Function {
        func.instrs = func
            .instrs
            .into_iter()
            .map(|instr| {
                if let Code::Instruction(Instruction::Constant {
                    value,
                    const_type,
                    dest,
                    op,
                    pos,
                }) = instr
                {
                    // Make sure the literal matches the type of the destination
                    Code::Instruction(Instruction::Constant {
                        value: value.implicit_cast(&const_type),
                        const_type,
                        dest: dest.clone(),
                        op,
                        pos,
                    })
                } else {
                    instr
                }
            })
            .collect();

        func
    }
}

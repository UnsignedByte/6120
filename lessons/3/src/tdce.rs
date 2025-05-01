use std::collections::HashSet;

use bril_rs::{Code, Function, Instruction};
use itertools::Itertools;
use utils::{BasicBlock, FunctionPass, Pass};

pub struct TDCEPass;

impl Pass for TDCEPass {
    fn function(&mut self, func: Function) -> Function {
        self.func(func.into()).into()
    }
}

impl FunctionPass for TDCEPass {
    fn before(&mut self, func: utils::BBFunction) -> utils::BBFunction {
        // Perform global DCE to remove totally unused instructions
        let mut changed = true;
        let mut func = Function::from(func);
        while changed {
            changed = false;

            // Collect all values that are read in the function
            let read: HashSet<_> = func
                .instrs
                .iter()
                .filter_map(|instr| {
                    if let Code::Instruction(
                        Instruction::Value { args, .. } | Instruction::Effect { args, .. },
                    ) = instr
                    {
                        Some(args)
                    } else {
                        None
                    }
                })
                .flatten()
                .cloned()
                .collect();

            func.instrs = func
                .instrs
                .clone()
                .into_iter()
                .filter(|instr| {
                    match instr {
                        Code::Instruction(
                            Instruction::Value { dest, .. } | Instruction::Constant { dest, .. },
                        ) => {
                            // Keep the instruction if the destination is read
                            if read.contains(dest) {
                                true
                            } else {
                                changed = true;
                                false
                            }
                        }
                        _ => true,
                    }
                })
                .collect();
        }

        func.into()
    }

    fn basic_block(&mut self, bb: BasicBlock) -> BasicBlock {
        let mut written_unread = HashSet::new();

        // Iterate in reverse to discard writes that occur
        // without a subsequent read
        let mut instrs = bb
            .iter()
            .cloned()
            .rev()
            .filter(|instr| {
                let live = if let Instruction::Value { dest, .. }
                | Instruction::Constant { dest, .. } = instr
                {
                    // Insert the destination as a new write.
                    // If the insertion was not new, this instruction is dead.
                    written_unread.insert(dest.clone())
                } else {
                    true
                };

                // Loop through the arguments of the instruction and remove them from the set
                // of written values
                // If this instruction is dead, treat it like it doesn't read any values
                if live {
                    if let Instruction::Value { args, .. } | Instruction::Effect { args, .. } =
                        instr
                    {
                        for arg in args {
                            written_unread.remove(arg);
                        }
                    }
                }

                live
            })
            .collect_vec();

        instrs.reverse();

        BasicBlock::new(bb.idx, bb.label, instrs)
    }
}

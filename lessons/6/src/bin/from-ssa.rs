use std::collections::HashMap;

use bril_rs::{Code, EffectOps, Function, Instruction, ValueOps};
use utils::{InstrExt, Pass, pass_pipeline, setup_logger_from_env};

struct FromSSA;

impl Pass for FromSSA {
    fn function(&mut self, mut func: Function) -> Function {
        // Because this is SSA, each variable is written only once so we can simply get its type!
        let types: HashMap<_, _> = func
            .instrs
            .iter()
            .filter_map(|instr| {
                if let Code::Instruction(insn) = instr {
                    insn.dest()
                        .map(|dest| (dest.clone(), insn.get_type().unwrap()))
                } else {
                    None
                }
            })
            .collect();

        func.instrs = func
            .instrs
            .iter()
            .filter_map(|code| {
                if let Code::Instruction(instr) = code {
                    match &instr {
                        Instruction::Effect {
                            args,
                            op: EffectOps::Set,
                            ..
                        } => {
                            // replace set dest src;
                            // with    dest: ty = id src;
                            let [dest, src] = args.as_slice() else {
                                unreachable!()
                            };

                            Some(Instruction::Value {
                                dest: dest.clone(),
                                op: ValueOps::Id,
                                args: vec![src.clone()],
                                funcs: vec![],
                                labels: vec![],
                                pos: None,
                                op_type: types.get(src).unwrap().clone(),
                            })
                        }
                        Instruction::Value {
                            op: ValueOps::Get, ..
                        } => {
                            // We can simply remove the gets
                            None
                        }
                        _ => Some(instr.clone()),
                    }
                    .map(Code::Instruction)
                } else {
                    Some(code.clone())
                }
            })
            .collect();

        func
    }
}

fn main() {
    setup_logger_from_env();
    pass_pipeline!(FromSSA);
}

use crate::interp::{State, get_arg};
use bril_rs::{Argument, Code, EffectOps, Function, Instruction, Type, ValueOps};
use brilirs::basic_block::NumifiedInstruction;
use lesson_3::{LVNPass, TDCEPass};
use std::collections::HashSet;
use utils::{BBFunction, FunctionPass, InstrExt};
pub struct Trace {
    prefix: Vec<String>,
    max_len: usize,
    instrs: Vec<Instruction>,
    return_destinations: Vec<(String, Type)>,
    done: bool,
}

impl Trace {
    pub fn new(max_len: usize) -> Self {
        Trace {
            prefix: vec![],
            max_len,
            instrs: vec![],
            return_destinations: vec![],
            done: false,
        }
    }

    pub fn done(&self) -> bool {
        self.done
    }

    fn prefix_name(name: &str, prefix: &[String]) -> String {
        if prefix.is_empty() {
            name.to_string()
        } else {
            format!("__trace_{}_{}", prefix.join("_"), name)
        }
    }

    fn prefix_instr(&self, instr: &Instruction) -> Instruction {
        let mut instr = instr.clone();

        if let Some(dest) = instr.dest() {
            instr.set_dest(Self::prefix_name(&dest, &self.prefix));
        }

        if let Some(args) = instr.args() {
            instr.set_args(
                args.into_iter()
                    .map(|arg| Self::prefix_name(&arg, &self.prefix))
                    .collect(),
            );
        }

        instr
    }

    pub fn push(
        &mut self,
        instr: &Instruction,
        numified: &NumifiedInstruction,
        state: &State,
    ) -> Result<(), String> {
        eprintln!("Tracing instruction: {}", instr);
        if self.instrs.len() >= self.max_len {
            self.done = true;
            return Err("Trace is full".to_string());
        }

        if self.done {
            return Err("Trace is already done".to_string());
        }

        self.done = true;

        let instr = self.prefix_instr(instr);

        self.instrs.extend(match &instr {
            Instruction::Effect { op, args: vars, .. } => match op {
                EffectOps::Branch => {
                    // Add a guard instruction to the trace
                    let x = get_arg::<bool>(&state.env, 0, &numified.args);

                    // If x is false we need to create a `not arg` instruction.
                    // Otherwise, just do ID x

                    let condval = format!("__trace{}_cond", self.instrs.len());

                    let cond_instr = if x {
                        Instruction::Value {
                            op: ValueOps::Id,
                            args: vec![vars[0].clone()],
                            dest: condval.clone(),
                            op_type: Type::Bool,
                            pos: None,
                            funcs: vec![],
                            labels: vec![],
                        }
                    } else {
                        Instruction::Value {
                            op: ValueOps::Not,
                            args: vec![vars[0].clone()],
                            dest: condval.clone(),
                            op_type: Type::Bool,
                            pos: None,
                            funcs: vec![],
                            labels: vec![],
                        }
                    };

                    vec![
                        cond_instr,
                        // guard __tracei_cond;
                        Instruction::Effect {
                            op: EffectOps::Guard,
                            args: vec![condval],
                            labels: vec!["__trace_failed".to_string()],
                            funcs: vec![],
                            pos: None,
                        },
                    ]
                }
                EffectOps::Return => {
                    self.prefix.pop();

                    let (return_dest, return_type) = self.return_destinations.pop().unwrap();

                    vec![Instruction::Value {
                        op: ValueOps::Id,
                        args: vec![vars[0].clone()],
                        dest: return_dest,
                        op_type: return_type,
                        pos: None,
                        funcs: vec![],
                        labels: vec![],
                    }]
                }
                EffectOps::Call => {
                    // Effect only calls cannot be pure
                    return Err("Attempted to trace non-pure instruction".to_string());
                }
                EffectOps::Jump => vec![],
                EffectOps::Print => {
                    return Err("Attempted to trace non-pure instruction".to_string());
                }
                EffectOps::Nop | EffectOps::Set => vec![instr],
                EffectOps::Speculate | EffectOps::Commit | EffectOps::Guard => unimplemented!(),
                EffectOps::Store | EffectOps::Free => {
                    return Err("Memory operations cannot be traced".to_string());
                }
            },
            Instruction::Value {
                op: ValueOps::Call,
                funcs,
                args,
                dest,
                op_type,
                ..
            } => {
                // Add the function name to the prefix
                self.prefix.push(funcs[0].clone());
                self.return_destinations
                    .push((dest.clone(), op_type.clone()));

                // Bind the arguments to the function

                let callee_func = state.prog.get(numified.funcs[0]).unwrap();

                // Create a number of commands arg = x;
                callee_func
                    .args
                    .iter()
                    .zip(args)
                    .map(|(arg, var)| {
                        let argname = Self::prefix_name(&arg.name, &self.prefix);

                        Instruction::Value {
                            op: ValueOps::Id,
                            args: vec![var.clone()],
                            dest: argname.clone(),
                            op_type: arg.arg_type.clone(),
                            pos: None,
                            funcs: vec![],
                            labels: vec![],
                        }
                    })
                    .collect()
            }
            _ => {
                if !instr.is_pure() {
                    return Err("Attempted to trace non-pure instruction".to_string());
                }

                vec![instr]
            }
        });

        self.done = false;

        Ok(())
    }

    /// Simplify the trace using LVN and TDCE, and return the resultant instructions
    pub fn take(self) -> Vec<Code> {
        let instrs = self.instrs;

        // Find all variables that are read before written, and treat them as arguments
        let mut written = HashSet::new();
        let mut args: Vec<Argument> = vec![];

        for instr in &instrs {
            if let Some(instr_args) = instr.args() {
                for arg in instr_args {
                    if !written.contains(&arg) {
                        args.push(Argument {
                            name: arg,
                            arg_type: instr.get_type().unwrap(),
                        });
                    }
                }
            }

            if let Some(dest) = instr.dest() {
                written.insert(dest.clone());
            }
        }

        // We want to preserve every name that wasn't intermediately generated (starting with __trace)
        // So we can temporarily put them all into a print to prevent them from being optimized away
        let mut names = HashSet::new();
        for instr in &instrs {
            if let Some(dest) = instr.dest() {
                if !dest.starts_with("__trace") {
                    names.insert(dest.clone());
                }
            }
        }

        let instrs = std::iter::once(Code::Instruction(Instruction::Effect {
            op: EffectOps::Speculate,
            args: vec![],
            funcs: vec![],
            labels: vec![],
            pos: None,
        }))
        .chain(instrs.into_iter().map(Code::Instruction))
        .chain([
            // commit
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Commit,
                args: vec![],
                funcs: vec![],
                labels: vec![],
                pos: None,
            }),
            // Jump to continuation instruction
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Jump,
                labels: vec!["__trace_succeeded".to_owned()],
                funcs: vec![],
                args: vec![],
                pos: None,
            }),
            // Failure label
            Code::Label {
                label: "__trace_failed".to_owned(),
                pos: None,
            },
            Code::Instruction(Instruction::Effect {
                op: EffectOps::Print,
                args: names.into_iter().collect(),
                labels: vec![],
                funcs: vec![],
                pos: None,
            }),
        ])
        .collect();

        let func = Function {
            args,
            instrs,
            name: "__trace".to_owned(),
            pos: None,
            return_type: None,
        };

        let func = BBFunction::from(func);

        let func = LVNPass::default().func(func);
        let func = TDCEPass.func(func);

        let mut instrs = Function::from(func).instrs;

        instrs.pop(); // Remove the print instruction

        instrs
    }
}

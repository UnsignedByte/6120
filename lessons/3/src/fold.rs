use std::collections::HashMap;

use bril_rs::{Instruction, Literal, ValueOps};

pub trait Foldable<K, V> {
    fn fold(&self, f: impl Fn(&K) -> Option<V>) -> Option<(K, Option<V>)>;
}

/// A table of values that to be used for folding
#[derive(Default)]
pub struct ValueTable {
    map: HashMap<String, Literal>,
}

impl ValueTable {
    pub fn get(&self, k: &String) -> Option<&Literal> {
        self.map.get(k)
    }

    pub fn intern(&mut self, i: &Instruction) {
        if let Some((k, Some(v))) = i.fold(|k| self.map.get(k).cloned()) {
            self.map.insert(k, v);
        }
    }
}

impl Foldable<String, Literal> for Instruction {
    fn fold(&self, f: impl Fn(&String) -> Option<Literal>) -> Option<(String, Option<Literal>)> {
        match self {
            Instruction::Constant { dest, value, .. } => Some((dest.clone(), Some(value.clone()))),
            Instruction::Value { dest, op, args, .. } => Some((
                dest.clone(),
                if let Some(args) = args.iter().map(f).collect::<Option<Vec<_>>>() {
                    match op {
                        ValueOps::Id => {
                            let [ref a] = args[..] else {
                                panic!("Id expects one argument, got {:?}", args);
                            };
                            Some(a.clone())
                        }
                        ValueOps::Add
                        | ValueOps::Sub
                        | ValueOps::Mul
                        | ValueOps::Div
                        | ValueOps::Eq
                        | ValueOps::Gt
                        | ValueOps::Ge
                        | ValueOps::Lt
                        | ValueOps::Le => {
                            let [Literal::Int(a), Literal::Int(b)] = args[..] else {
                                panic!("Invalid arguments for binary arithmetic operation, expected 2 ints, got {:?}", args);
                            };
                            Some(match op {
                                ValueOps::Add => Literal::Int(a + b),
                                ValueOps::Sub => Literal::Int(a - b),
                                ValueOps::Mul => Literal::Int(a * b),
                                ValueOps::Div => Literal::Int(a / b),
                                ValueOps::Eq => Literal::Bool(a == b),
                                ValueOps::Gt => Literal::Bool(a > b),
                                ValueOps::Ge => Literal::Bool(a >= b),
                                ValueOps::Lt => Literal::Bool(a < b),
                                ValueOps::Le => Literal::Bool(a <= b),
                                _ => unreachable!(),
                            })
                        }
                        ValueOps::And | ValueOps::Or => {
                            let [Literal::Bool(a), Literal::Bool(b)] = args[..] else {
                                panic!("Invalid arguments for binary boolean operation, expected 2 bools, got {:?}", args);
                            };
                            Some(Literal::Bool(match op {
                                ValueOps::And => a && b,
                                ValueOps::Or => a || b,
                                _ => unreachable!(),
                            }))
                        }
                        ValueOps::Not => {
                            let [Literal::Bool(a)] = args[..] else {
                                panic!(
                                    "Invalid arguments for not operation, expected bool, got {:?}",
                                    args
                                );
                            };
                            Some(Literal::Bool(!a))
                        }
                        ValueOps::Fadd
                        | ValueOps::Fsub
                        | ValueOps::Fmul
                        | ValueOps::Fdiv
                        | ValueOps::Feq
                        | ValueOps::Fgt
                        | ValueOps::Fge
                        | ValueOps::Flt
                        | ValueOps::Fle => {
                            let [Literal::Float(a), Literal::Float(b)] = args[..] else {
                                panic!("Invalid arguments for binary arithmetic operation, expected 2 floats, got {:?}", args);
                            };
                            Some(match op {
                                ValueOps::Fadd => Literal::Float(a + b),
                                ValueOps::Fsub => Literal::Float(a - b),
                                ValueOps::Fmul => Literal::Float(a * b),
                                ValueOps::Fdiv => Literal::Float(a / b),
                                ValueOps::Feq => Literal::Bool(a == b),
                                ValueOps::Fgt => Literal::Bool(a > b),
                                ValueOps::Fge => Literal::Bool(a >= b),
                                ValueOps::Flt => Literal::Bool(a < b),
                                ValueOps::Fle => Literal::Bool(a <= b),
                                _ => unreachable!(),
                            })
                        }
                        ValueOps::Ceq
                        | ValueOps::Cgt
                        | ValueOps::Cge
                        | ValueOps::Clt
                        | ValueOps::Cle => {
                            let [Literal::Char(a), Literal::Char(b)] = args[..] else {
                                panic!("Invalid arguments for binary arithmetic operation, expected 2 chars, got {:?}", args);
                            };
                            Some(match op {
                                ValueOps::Ceq => Literal::Bool(a == b),
                                ValueOps::Cgt => Literal::Bool(a > b),
                                ValueOps::Cge => Literal::Bool(a >= b),
                                ValueOps::Clt => Literal::Bool(a < b),
                                ValueOps::Cle => Literal::Bool(a <= b),
                                _ => unreachable!(),
                            })
                        }
                        _ => None,
                    }
                } else {
                    None
                },
            )),
            Instruction::Effect { .. } => None,
        }
    }
}

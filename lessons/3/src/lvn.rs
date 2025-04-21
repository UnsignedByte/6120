use crate::Foldable;
use bril_rs::{Argument, ConstOps, Function, Instruction, Type, ValueOps};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};
use std::fmt::{Debug, Display};
use utils::{BasicBlock, FunctionPass, HashableLiteral, InstrExt, Pass};

/// A value interned in the LVN table
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum LVNValue {
    /// A Constant literal
    Literal(HashableLiteral),
    /// A Value operation
    Op(ValueOps, Type, Vec<usize>),
    /// Unknown unique value
    Unknown(usize),
}

impl Display for LVNValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LVNValue::Literal(l) => Display::fmt(&format!("{} {}", l.get_type(), l), f),
            LVNValue::Op(value_ops, _, items) => Display::fmt(
                &format!(
                    "{} {}",
                    value_ops,
                    items
                        .iter()
                        .map(|idx| format!("<{}>", idx))
                        .collect::<Vec<_>>()
                        .join(" ")
                ),
                f,
            ),
            LVNValue::Unknown(_) => Display::fmt("?", f),
        }
    }
}

/// The value table for the LVN pass
#[derive(Default)]
struct LVNTable {
    vid: usize,
    table: Vec<(String, LVNValue)>,
    vtable: HashMap<LVNValue, usize>,
    ntable: HashMap<String, usize>,
}

impl LVNTable {
    fn new(args: &Vec<Argument>) -> Self {
        let mut table = Self::default();
        for arg in args {
            let name = arg.name.clone();
            let value = table.unique_value();
            table.table.push((name.clone(), value.clone()));
            table.ntable.insert(name, table.table.len() - 1);
        }
        table
    }

    fn idx(&self, name: &str) -> Option<usize> {
        self.ntable.get(name).copied()
    }

    fn idx_or_insert(&mut self, name: &str) -> usize {
        if let Some(&idx) = self.ntable.get(name) {
            idx
        } else {
            let idx = self.table.len();
            let val = self.unique_value();
            self.table.push((name.to_owned(), val));
            self.ntable.insert(name.to_owned(), idx);
            idx
        }
    }

    fn unique_value(&mut self) -> LVNValue {
        let vid = self.vid;
        self.vid += 1;
        LVNValue::Unknown(vid)
    }

    fn value(&self, idx: usize) -> &LVNValue {
        &self.table[idx].1
    }

    fn representative(&self, idx: usize) -> &String {
        &self.table[idx].0
    }

    /// Intern a value into the table
    /// Returns [None] if there is no value to be interned
    /// Returns [Some((bool, usize))] where the first value is true if the value already exists
    pub fn intern(&mut self, instr: &Instruction) -> Option<(bool, usize)> {
        // Try to intern the value
        let value = instr.fold(|k| {
            self.idx(k).and_then(|idx| match self.value(idx) {
                LVNValue::Literal(l) => Some(l.clone().into()),
                _ => None,
            })
        });

        // If this instruction actually generates a value
        value.map(|(name, value)| {
            let value = match value {
                Some(v) => LVNValue::Literal(v.into()),
                None => {
                    if instr.is_pure() {
                        let Instruction::Value {
                            op, args, op_type, ..
                        } = instr.clone()
                        else {
                            panic!("Expected Value instruction, got {:?}", instr);
                        };

                        if op == ValueOps::Id {
                            // If the operation is an identity, just return the argument
                            let [ref arg] = args[..] else {
                                panic!("Id expects one argument, got {:?}", args);
                            };
                            self.value(self.idx(arg).unwrap()).clone()
                        } else {
                            // Convert the arguments to indexes
                            let mut args: Vec<_> =
                                args.iter().map(|arg| self.idx(arg).unwrap()).collect();

                            if instr.is_commutative() {
                                args.sort_unstable();
                            }

                            LVNValue::Op(op, op_type, args)
                        }
                    } else {
                        // Non-pure instructions generate unknown values
                        self.unique_value()
                    }
                }
            };

            // Check if the value already exists
            if let Some(&idx) = self.vtable.get(&value) {
                self.ntable.insert(name.clone(), idx);
                (true, idx)
            } else {
                let idx = self.table.len();
                self.table.push((name.clone(), value.clone()));
                self.vtable.insert(value, idx);
                self.ntable.insert(name, idx);
                (false, idx)
            }
        })
    }

    /// Transform an instruction using the table
    pub fn transform(&mut self, mut instr: Instruction) -> Instruction {
        // Transform the arguments of the instruction to their representative values
        match instr {
            Instruction::Constant { .. } => instr,
            Instruction::Value { .. } | Instruction::Effect { .. } => {
                // Replace the arguments with their representative values
                let args = instr
                    .args()
                    .unwrap()
                    .iter()
                    .map(|arg| {
                        let idx = self.idx_or_insert(arg);
                        self.representative(idx).clone()
                    })
                    .collect();

                // Create a new instruction with the transformed arguments
                instr.set_args(args);

                instr
            }
        }
    }

    /// Build a reference instruction to a value
    fn build_ref(&self, dest: &str) -> Option<Instruction> {
        self.idx(dest).and_then(|idx| match self.value(idx) {
            LVNValue::Literal(l) => Some(Instruction::Constant {
                dest: dest.to_owned(),
                const_type: l.get_type(),
                value: l.clone().into(),
                op: ConstOps::Const,
                pos: None,
            }),
            LVNValue::Op(_, ty, _) => Some(Instruction::Value {
                dest: dest.to_owned(),
                op: ValueOps::Id,
                args: vec![self.representative(idx).clone()],
                funcs: vec![],
                labels: vec![],
                pos: None,
                op_type: ty.clone(),
            }),
            _ => None,
        })
    }

    /// Clobber an index in the table with a new name
    pub fn clobber(&mut self, idx: usize, name: &str) {
        self.table[idx].0 = name.to_owned();
        self.ntable.insert(name.to_owned(), idx);
    }
}

impl Debug for LVNTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_map()
            .entries(
                self.table
                    .iter()
                    .map(|(name, value)| (name, value.clone()))
                    .enumerate(),
            )
            .finish()
    }
}

impl Display for LVNTable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{:>4} | {:<10} | {:<15} | {:<15}",
            "Idx", "Rep", "Value", "Other Names"
        )?;
        writeln!(f, "{:-<4}-+-{:-<10}-+-{:-<15}-+-{:-<15}", "", "", "", "")?;

        // Create a vector of all the mappings to a given idx
        let mut idx_map = vec![Vec::new(); self.table.len()];
        for (name, idx) in &self.ntable {
            idx_map[*idx].push(name.clone());
        }

        for (idx, (name, value)) in self.table.iter().enumerate() {
            writeln!(
                f,
                "{:>4} | {:<10} | {:<15} | {:<15}",
                idx,
                name,
                value,
                idx_map[idx].join(", ")
            )?;
        }
        Ok(())
    }
}

#[derive(Default)]
pub struct LVNPass {
    nid: usize,
    table: LVNTable,
    names: HashSet<String>,
    args: Vec<Argument>,
}

impl LVNPass {
    pub fn unique_name(&mut self, pref: &str) -> String {
        loop {
            let name = format!("_{}_{}", pref, self.nid);
            self.nid += 1;

            if !self.names.contains(&name) {
                break name;
            }
        }
    }
}

impl Pass for LVNPass {
    fn function(&mut self, func: Function) -> Function {
        self.func(func.into()).into()
    }
}

impl FunctionPass for LVNPass {
    fn before(&mut self, func: utils::BBFunction) -> utils::BBFunction {
        self.names
            .extend(func.args.iter().map(|arg| arg.name.clone()));

        self.names.extend(
            func.iter()
                .flat_map(|bb| bb.iter().filter_map(InstrExt::dest)),
        );

        self.args = func.args.clone();

        func
    }

    fn basic_block(&mut self, bb: utils::BasicBlock) -> utils::BasicBlock {
        self.table = LVNTable::new(&self.args);
        // Map of the instruction index to the last write of the variable
        let last_write_map: HashMap<_, _> = bb
            .iter()
            .enumerate()
            .filter_map(|(idx, instr)| instr.dest().map(|dest| (dest.clone(), idx)))
            .collect();

        log::debug!("Last write map: {:#?}", last_write_map);

        // Loop through instructions
        let instrs = bb
            .iter()
            .cloned()
            .enumerate()
            .map(|(i, instr)| {
                log::debug!("Original: {}", instr);
                let instr = self.table.transform(instr);
                let instr = if let Some((new, idx)) = self.table.intern(&instr) {
                    log::debug!("\n{}", self.table);
                    let dest = instr.dest().unwrap();
                    if new {
                        self.table.build_ref(&dest).unwrap_or(instr)
                    } else {
                        let last_write = last_write_map
                            .get(&instr.dest().unwrap())
                            .copied()
                            .unwrap_or_default();

                        let mut instr = instr;
                        // If this is not the last write, clobber the value
                        if last_write > i {
                            let new_name = self.unique_name(&dest);
                            log::debug!("Clobbering {} with {}", instr.dest().unwrap(), new_name);
                            self.table.clobber(idx, &new_name);
                            instr.set_dest(new_name);
                        }

                        match self.table.value(idx) {
                            LVNValue::Literal(l) => Instruction::Constant {
                                dest: instr.dest().unwrap(),
                                const_type: instr.get_type().unwrap(),
                                value: l.clone().into(),
                                op: ConstOps::Const,
                                pos: instr.get_pos(),
                            },
                            LVNValue::Op(_, _, _) | LVNValue::Unknown(_) => instr,
                        }
                    }
                } else {
                    instr
                };

                log::debug!("Transformed: {}", instr);
                instr
            })
            .collect_vec();

        BasicBlock::new(bb.idx, bb.label, instrs)
    }
}

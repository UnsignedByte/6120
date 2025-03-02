use bril_rs::{Code, EffectOps, Instruction};
use derivative::Derivative;
use graphviz_rust::{
    dot_generator::{attr, id},
    dot_structures::{Attribute, Id},
};
use std::fmt::{Debug, Display};

/// Represents the final control flow of a basic block.
#[derive(Debug, Derivative, Clone, PartialEq, Eq)]
#[derivative(Default)]
pub enum ControlFlow {
    Jump(String),
    Branch(String, String),
    Return(String),
    #[derivative(Default)]
    Fallthrough,
}

impl From<ControlFlow> for Option<Instruction> {
    fn from(value: ControlFlow) -> Self {
        match value {
            ControlFlow::Jump(l) => Some(Instruction::Effect {
                op: EffectOps::Jump,
                args: vec![],
                funcs: vec![],
                labels: vec![l],
                pos: None,
            }),
            ControlFlow::Branch(t, f) => Some(Instruction::Effect {
                op: EffectOps::Branch,
                args: vec![],
                funcs: vec![],
                labels: vec![t, f],
                pos: None,
            }),
            ControlFlow::Return(val) => Some(Instruction::Effect {
                op: EffectOps::Return,
                args: vec![val],
                funcs: vec![],
                labels: vec![],
                pos: None,
            }),
            ControlFlow::Fallthrough => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub idx: usize,
    pub label: Option<String>,
    instrs: Vec<Instruction>,
    control_flow: Option<Instruction>,
}

impl BasicBlock {
    pub fn new(idx: usize, label: Option<String>, instrs: Vec<Instruction>) -> Self {
        let mut bb = Self {
            idx,
            label,
            instrs: vec![],
            control_flow: Default::default(),
        };

        bb.extend(instrs);
        bb
    }

    pub fn is_empty(&self) -> bool {
        self.instrs.is_empty()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = &Instruction> {
        self.instrs.iter().chain(self.control_flow.iter())
    }

    pub fn iter_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Instruction> {
        self.instrs.iter_mut().chain(self.control_flow.iter_mut())
    }

    pub fn control_flow(&self) -> Option<&Instruction> {
        self.control_flow.as_ref()
    }

    pub fn is_entry(&self) -> bool {
        self.idx == 0
    }

    pub fn label_or_default(&self) -> &str {
        if let Some(label) = &self.label {
            label
        } else {
            match self.is_entry() {
                true => "entry",
                false => "?",
            }
        }
    }

    pub fn node_label(&self) -> String {
        format!(r#""{}""#, self.label_or_default())
    }

    pub fn node_attrs(&self) -> Vec<Attribute> {
        let mut attrs = vec![attr!("label", &self.node_label()), attr!("shape", "oval")];

        if self.is_entry() {
            attrs.push(attr!("color", "blue"));
            attrs.push(attr!("rank", "source"));
        }

        attrs
    }

    /// Push an instruction to a basic block.
    /// If the instruction is a control flow instruction, set the control flow.
    /// This way, instructions are pushed before the final control flow instruction.
    pub fn push(&mut self, instr: Instruction) {
        match instr {
            Instruction::Effect {
                op: EffectOps::Jump | EffectOps::Branch | EffectOps::Return,
                ..
            } => {
                if self.control_flow.is_some() {
                    panic!("Cannot have multiple control flow instructions in a basic block");
                }

                self.control_flow = Some(instr)
            }
            _ => {
                self.instrs.push(instr);
            }
        }
    }

    pub fn insert(&mut self, idx: usize, instr: Instruction) {
        match instr {
            Instruction::Effect {
                op: EffectOps::Jump | EffectOps::Branch | EffectOps::Return,
                ..
            } => {
                if self.control_flow.is_some() {
                    panic!("Cannot have multiple control flow instructions in a basic block");
                }

                if self.instrs.len() == idx {
                    panic!(
                        "Control flow instruction must be the last instruction in a basic block"
                    );
                }

                self.control_flow = Some(instr)
            }
            _ => {
                self.instrs.insert(idx, instr);
            }
        }
    }

    pub fn extend(&mut self, instrs: impl IntoIterator<Item = Instruction>) {
        for instr in instrs {
            self.push(instr);
        }
    }

    // Flatten the basic block into a vector of instructions.
    pub fn flatten(self) -> Vec<Code> {
        let mut instrs = match self.label {
            Some(label) => vec![Code::Label { label, pos: None }],

            None => vec![],
        };

        instrs.extend(self.instrs.into_iter().map(Code::Instruction));
        instrs.extend(self.control_flow.map(Code::Instruction));

        instrs
    }
}

impl IntoIterator for BasicBlock {
    type Item = Instruction;
    type IntoIter = std::vec::IntoIter<Instruction>;

    fn into_iter(self) -> Self::IntoIter {
        self.instrs.into_iter()
    }
}

impl Display for BasicBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(label) = &self.label {
            writeln!(f, ".{label}:")?;
        }

        for instr in &self.instrs {
            writeln!(f, "\t{}", instr)?;
        }

        Ok(())
    }
}

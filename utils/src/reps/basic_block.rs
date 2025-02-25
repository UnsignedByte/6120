use std::fmt::{Debug, Display};

use bril_rs::Instruction;
use graphviz_rust::{
    dot_generator::{attr, id},
    dot_structures::{Attribute, Id},
};

#[derive(Debug, Clone)]
pub struct BasicBlock {
    pub idx: usize,
    pub label: Option<String>,
    pub instrs: Vec<Instruction>,
}

impl BasicBlock {
    pub fn is_empty(&self) -> bool {
        self.instrs.is_empty()
    }

    pub fn iter(&self) -> std::slice::Iter<Instruction> {
        self.instrs.iter()
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

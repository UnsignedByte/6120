use bril_rs::Instruction;

pub struct BasicBlock {
    pub is_entry: bool,
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

    pub fn label_or_default(&self) -> &str {
        if let Some(label) = &self.label {
            label
        } else {
            match self.is_entry {
                true => "entry",
                false => "?",
            }
        }
    }
}

impl IntoIterator for BasicBlock {
    type Item = Instruction;
    type IntoIter = std::vec::IntoIter<Instruction>;

    fn into_iter(self) -> Self::IntoIter {
        self.instrs.into_iter()
    }
}

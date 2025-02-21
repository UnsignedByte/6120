use std::collections::HashMap;

use bril_rs::{Argument, Code, EffectOps, Function, Instruction, Type};

use crate::basic_block::BasicBlock;

/// Function representation using basic blocks.
pub struct BBFunction {
    pub name: String,
    pub args: Vec<Argument>,
    pub blocks: Vec<BasicBlock>,
    pub return_type: Option<Type>,
    name_map: HashMap<String, usize>,
}

impl BBFunction {
    pub fn new(func: Function) -> Self {
        func.into()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn get_block_idx(&self, label: &str) -> Option<usize> {
        self.name_map.get(label).copied()
    }
}

impl From<Function> for BBFunction {
    fn from(func: Function) -> Self {
        let mut blocks = Vec::new();
        let mut curr_block = None;

        for instr in func.instrs {
            match instr {
                Code::Label { label, .. } => {
                    if let Some(block) = curr_block {
                        blocks.push(block);
                    }
                    curr_block = Some(BasicBlock {
                        is_entry: false,
                        label: Some(label),
                        instrs: Vec::new(),
                    });
                }
                Code::Instruction(i) => {
                    curr_block = match curr_block {
                        Some(mut block) => {
                            block.instrs.push(i.clone());
                            if let Instruction::Effect {
                                op: EffectOps::Jump | EffectOps::Branch | EffectOps::Return,
                                ..
                            } = i
                            {
                                blocks.push(block);
                                None
                            } else {
                                Some(block)
                            }
                        }
                        None => Some(BasicBlock {
                            is_entry: false,
                            label: None,
                            instrs: vec![i],
                        }),
                    }
                }
            }
        }

        if let Some(block) = curr_block {
            blocks.push(block);
        }

        if !blocks.is_empty() {
            blocks[0].is_entry = true;
        }

        let name_map = blocks
            .iter()
            .enumerate()
            .filter_map(|(i, block)| block.label.as_ref().map(|label| (label.clone(), i)))
            .collect();

        BBFunction {
            name: func.name,
            args: func.args,
            blocks,
            return_type: func.return_type,
            name_map,
        }
    }
}

impl From<BBFunction> for Function {
    fn from(bb_func: BBFunction) -> Self {
        Function {
            args: bb_func.args,
            instrs: bb_func
                .blocks
                .into_iter()
                .flat_map(|block| {
                    block
                        .label
                        .map(|label| Code::Label { label, pos: None })
                        .into_iter()
                        .chain(block.instrs.into_iter().map(Code::Instruction))
                })
                .collect(),
            name: bb_func.name,
            pos: None,
            return_type: bb_func.return_type,
        }
    }
}

use crate::reps::basic_block::BasicBlock;
use bril_rs::{Argument, Code, EffectOps, Function, Instruction, Type};
use std::collections::HashMap;

/// Function representation using basic blocks.
#[derive(Debug, Clone)]
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

    pub fn iter(&self) -> impl Iterator<Item = &BasicBlock> {
        self.blocks.iter()
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = &mut BasicBlock> {
        self.blocks.iter_mut()
    }

    pub fn get(&self, idx: usize) -> &BasicBlock {
        &self.blocks[idx]
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
                    curr_block = Some(BasicBlock::new(blocks.len(), Some(label), vec![]));
                }
                Code::Instruction(i) => {
                    if let Instruction::Effect {
                        op: EffectOps::Jump | EffectOps::Branch | EffectOps::Return,
                        ..
                    } = i
                    {
                        if let Some(mut block) = curr_block {
                            block.push(i.clone());
                            blocks.push(block);
                        } else {
                            // There was no last block, but this block is a single control flow instruction
                            // So it is a single-instruction block
                            blocks.push(BasicBlock::new(blocks.len(), None, vec![i]));
                        }

                        // Reset the current block
                        curr_block = None;
                    } else if let Some(block) = &mut curr_block {
                        block.push(i.clone());
                    } else {
                        curr_block = Some(BasicBlock::new(blocks.len(), None, vec![i]));
                    };
                }
            }
        }

        if let Some(block) = curr_block {
            blocks.push(block);
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
                .flat_map(BasicBlock::flatten)
                .collect(),
            name: bb_func.name,
            pos: None,
            return_type: bb_func.return_type,
        }
    }
}

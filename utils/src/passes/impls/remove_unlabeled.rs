use crate::{BBFunction, Pass};

/// Pass to canonicalize literals to the right type
pub struct RemoveUnlabeledBlocks;

impl Pass for RemoveUnlabeledBlocks {
    fn function(&mut self, func: bril_rs::Function) -> bril_rs::Function {
        let mut func = BBFunction::from(func);

        // Remove blocks that have Label = none that are not the entry block
        func.blocks
            .retain(|block| block.idx == 0 || block.label.is_some());

        // Redo the block indices
        for (idx, block) in func.blocks.iter_mut().enumerate() {
            block.idx = idx;
        }

        func.into()
    }
}

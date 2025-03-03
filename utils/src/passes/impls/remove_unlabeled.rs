use crate::{BBFunction, Pass};

/// Pass to canonicalize literals to the right type
pub struct RemoveUnlabeledBlocks;

impl Pass for RemoveUnlabeledBlocks {
    fn function(&mut self, func: bril_rs::Function) -> bril_rs::Function {
        let func = BBFunction::from(func);

        // Remove blocks that have Label = none that are not the entry block
        let func = func.with_blocks(|blocks| {
            blocks
                .into_iter()
                .filter(|block| block.idx == 0 || block.label.is_some())
                .collect()
        });

        func.into()
    }
}

use crate::{BBFunction, BasicBlock};

pub trait FunctionPass {
    fn basic_block(&mut self, bb: BasicBlock) -> BasicBlock {
        bb
    }
    fn before(&mut self, func: BBFunction) -> BBFunction {
        func
    }
    fn after(&mut self, func: BBFunction) -> BBFunction {
        func
    }

    fn func(&mut self, func: BBFunction) -> BBFunction {
        let func = self.before(func);

        let func =
            func.with_blocks(|blocks| blocks.into_iter().map(|bb| self.basic_block(bb)).collect());

        self.after(func)
    }
}

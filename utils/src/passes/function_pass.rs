use crate::{BBFunction, BasicBlock};

pub trait FunctionPass {
    fn basic_block(&mut self, bb: &mut BasicBlock);
    fn before(&mut self, func: &mut BBFunction);
    fn after(&mut self, func: &mut BBFunction);

    fn run(&mut self, func: &mut BBFunction) {
        self.before(func);
        for bb in &mut func.blocks {
            self.basic_block(bb);
        }
        self.after(func);
    }
}

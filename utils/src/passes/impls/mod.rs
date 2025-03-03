mod canonicalize_literals;
mod dominators;
mod remove_unlabeled;

pub use canonicalize_literals::CanonicalizeLiterals;
pub use dominators::{DominatorPass, DominatorSetNode};
pub use remove_unlabeled::RemoveUnlabeledBlocks;

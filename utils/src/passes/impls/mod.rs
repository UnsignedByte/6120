mod canonicalize_literals;
mod dominators;

pub(crate) use dominators::DominatorPass;

pub use canonicalize_literals::CanonicalizeLiterals;

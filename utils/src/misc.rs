use std::{
    fmt::{Debug, Display},
    hash::{self},
};

use bril_rs::{Literal, Type};

/// Unsafe wrapper around a literal to allow hashing
#[derive(PartialEq, Clone)]
pub struct HashableLiteral {
    val: Literal,
}

impl HashableLiteral {
    /// A helper function to get the type of literal values
    #[must_use]
    pub const fn get_type(&self) -> Type {
        self.val.get_type()
    }
}

impl From<Literal> for HashableLiteral {
    fn from(val: Literal) -> Self {
        Self { val }
    }
}

impl From<HashableLiteral> for Literal {
    fn from(v: HashableLiteral) -> Self {
        v.val
    }
}

impl hash::Hash for HashableLiteral {
    fn hash<H: hash::Hasher>(&self, state: &mut H) {
        match self.val {
            Literal::Int(i) => {
                state.write_u8(0);
                i.hash(state)
            }
            Literal::Bool(b) => {
                state.write_u8(1);
                b.hash(state)
            }
            Literal::Float(f) => {
                state.write_u8(2);
                f.to_bits().hash(state)
            }
            Literal::Char(c) => {
                state.write_u8(3);
                c.hash(state)
            }
        }
    }
}

impl Eq for HashableLiteral {}

impl Debug for HashableLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.val)
    }
}

impl Display for HashableLiteral {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.val)
    }
}

use bril_rs::{EffectOps, Instruction, Literal, Type, ValueOps};

pub trait InstrExt {
    fn args(&self) -> Option<Vec<String>>;
    fn dest(&self) -> Option<String>;
    fn get_type(&self) -> Option<Type>;
    fn set_args(&mut self, args: Vec<String>);
    fn set_dest(&mut self, dest: String);
    fn branch(&self) -> Option<Vec<String>>;
    fn is_commutative(&self) -> bool;
    fn is_pure(&self) -> bool;
}

impl InstrExt for Instruction {
    fn args(&self) -> Option<Vec<String>> {
        match self {
            Instruction::Value { args, .. } | Instruction::Effect { args, .. } => {
                Some(args.clone())
            }
            Instruction::Constant { .. } => None,
        }
    }

    fn dest(&self) -> Option<String> {
        match self {
            Instruction::Value { dest, .. } | Instruction::Constant { dest, .. } => {
                Some(dest.clone())
            }
            Instruction::Effect { .. } => None,
        }
    }

    fn get_type(&self) -> Option<Type> {
        match self {
            Instruction::Constant { const_type: ty, .. }
            | Instruction::Value { op_type: ty, .. } => Some(ty.clone()),
            Instruction::Effect { .. } => None,
        }
    }

    fn set_args(&mut self, args: Vec<String>) {
        match self {
            Instruction::Value { args: a, .. } | Instruction::Effect { args: a, .. } => {
                *a = args;
            }
            Instruction::Constant { .. } => {}
        }
    }

    fn set_dest(&mut self, dest: String) {
        match self {
            Instruction::Value { dest: d, .. } | Instruction::Constant { dest: d, .. } => {
                *d = dest;
            }
            Instruction::Effect { .. } => {}
        }
    }

    fn branch(&self) -> Option<Vec<String>> {
        match self {
            Instruction::Effect {
                op: EffectOps::Jump | EffectOps::Branch,
                labels,
                ..
            } => Some(labels.clone()),
            Instruction::Effect {
                op: EffectOps::Return,
                ..
            } => Some(vec![]),
            _ => None,
        }
    }

    fn is_commutative(&self) -> bool {
        matches!(
            self,
            Instruction::Value {
                op: ValueOps::Add
                    | ValueOps::Mul
                    | ValueOps::And
                    | ValueOps::Or
                    | ValueOps::Eq
                    | ValueOps::Fadd
                    | ValueOps::Fmul
                    | ValueOps::Feq
                    | ValueOps::Ceq,
                ..
            }
        )
    }

    fn is_pure(&self) -> bool {
        match self {
            Instruction::Constant { .. } => true,
            Instruction::Value {
                op:
                    ValueOps::Add
                    | ValueOps::Mul
                    | ValueOps::Sub
                    | ValueOps::Div
                    | ValueOps::And
                    | ValueOps::Or
                    | ValueOps::Not
                    | ValueOps::Eq
                    | ValueOps::Lt
                    | ValueOps::Gt
                    | ValueOps::Le
                    | ValueOps::Ge
                    | ValueOps::Id
                    | ValueOps::Fadd
                    | ValueOps::Fsub
                    | ValueOps::Fmul
                    | ValueOps::Fdiv
                    | ValueOps::Feq
                    | ValueOps::Flt
                    | ValueOps::Fgt
                    | ValueOps::Fle
                    | ValueOps::Fge
                    | ValueOps::Ceq
                    | ValueOps::Clt
                    | ValueOps::Cgt
                    | ValueOps::Cle
                    | ValueOps::Cge,
                ..
            } => true,
            Instruction::Value { .. } => false, // Unknown value operations are assumed to be impure
            Instruction::Effect { .. } => false,
        }
    }
}

pub trait LiteralExt {
    /// Implicitly cast the literal to the given type
    fn implicit_cast(&self, ty: &Type) -> Self;
}

impl LiteralExt for Literal {
    fn implicit_cast(&self, ty: &Type) -> Self {
        match (self, ty) {
            (Literal::Int(i), Type::Int) => Literal::Int(*i),
            (Literal::Bool(b), Type::Bool) => Literal::Bool(*b),
            (Literal::Int(i), Type::Float) => Literal::Float(*i as f64),
            (Literal::Float(f), Type::Float) => Literal::Float(*f),
            (Literal::Char(c), Type::Char) => Literal::Char(*c),
            _ => unreachable!(),
        }
    }
}

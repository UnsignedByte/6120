use bril_rs::{Instruction, ValueOps};
use itertools::Itertools;
use std::{
    collections::{HashMap, HashSet},
    fmt::{Debug, Display},
};
use utils::{
    AnalysisPass, BBFunction, BasicBlock, CFG, CallGraph, CanonicalizeLiterals, DataflowLabel,
    DataflowPass, HashableLiteral, InstrExt, Pass, draw_dataflow, run_analysis,
    setup_logger_from_env,
};

#[derive(Clone, PartialEq, Eq, Hash)]
enum Expr {
    /// Constant expression
    Const(HashableLiteral),
    /// Any expression
    Op(ValueOps, Vec<String>),
}

impl Expr {
    fn contains(&self, arg: &str) -> bool {
        match self {
            Expr::Const(_) => false,
            Expr::Op(_, args) => args.contains(&arg.to_string()),
        }
    }
}

impl Expr {
    fn new(instr: &Instruction) -> Option<Self> {
        if instr.is_pure() {
            match instr {
                Instruction::Value { args, op, .. } => Some(Expr::Op(*op, args.clone())),
                Instruction::Constant { value, .. } => Some(Expr::Const(value.clone().into())),
                _ => None,
            }
        } else {
            None
        }
    }
}

impl Display for Expr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Expr::Const(c) => Display::fmt(c, f),
            Expr::Op(op, args) => write!(f, "{} {}", op, args.join(" ")),
        }
    }
}

#[derive(Clone, PartialEq, Eq)]
enum Set {
    /// Set of all items
    Full,
    /// Set of finite items
    Finite(HashSet<Expr>),
}

impl Set {
    fn empty() -> Self {
        Set::Finite(HashSet::new())
    }

    fn full() -> Self {
        Set::Full
    }

    fn intersect(&self, other: &Self) -> Self {
        match (self, other) {
            (Set::Full, _) => other.clone(),
            (_, Set::Full) => self.clone(),
            (Set::Finite(a), Set::Finite(b)) => Set::Finite(a.intersection(b).cloned().collect()),
        }
    }

    fn insert(&mut self, expr: Expr) {
        match self {
            Set::Full => {}
            Set::Finite(set) => {
                set.insert(expr);
            }
        }
    }
}

impl Debug for Set {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Set::Full => Display::fmt("T", f),
            Set::Finite(set) => {
                write!(f, "{{{}}}", set.iter().map(|e| format!("{}", e)).join(", "))
            }
        }
    }
}

#[derive(Default)]
struct AvailableExpr;

impl DataflowPass<Set> for AvailableExpr {
    fn entry(&self, _: &BBFunction) -> Set {
        // Entry block is empty
        Set::empty()
    }
    fn init(&self, _: &BBFunction) -> Set {
        Set::full()
    }

    fn meet(&self, in_vals: &[Set]) -> Set {
        // Set intersection
        in_vals
            .iter()
            .fold(Set::full(), |acc, val| acc.intersect(val))
    }

    fn transfer(&self, block: &BasicBlock, in_val: &Set) -> Set {
        let mut out_vals = in_val.clone();

        for instr in block.iter() {
            if let Some(e) = Expr::new(instr) {
                out_vals.insert(e);
            }

            if let Some(dest) = instr.dest() {
                // Remove all expressions that contain the destination
                if let Set::Finite(set) = &mut out_vals {
                    set.retain(|expr| !expr.contains(&dest));
                }
            }
        }

        out_vals
    }
}

/// Dominator set node used to display graphs
/// For dominator sets
#[derive(Clone)]
pub struct GraphNode {
    exprs: Set,
}

impl DataflowLabel for GraphNode {
    fn in_label(&self, _: &CFG) -> Option<String> {
        // Create a set of variables
        let mut vars = match self.exprs {
            Set::Finite(ref set) => set.iter().map(|expr| format!("{}", expr)).collect_vec(),

            Set::Full => return Some("T\\l".to_string()),
        };

        vars.sort_unstable();

        // Return a string representation of the set
        Some(format!("In: \\{{{}\\}}\\l", vars.join(", ")))
    }

    fn out_label(&self, _: &CFG) -> Option<String> {
        // Create a set of variables
        let mut vars = match self.exprs {
            Set::Finite(ref set) => set.iter().map(|expr| format!("{}", expr)).collect_vec(),

            Set::Full => return Some("T\\l".to_string()),
        };

        vars.sort_unstable();

        // Return a string representation of the set
        Some(format!("Out: \\{{{}\\}}\\l", vars.join(", ")))
    }
}

impl From<Set> for GraphNode {
    fn from(exprs: Set) -> Self {
        Self { exprs }
    }
}

struct Drawer;

impl AnalysisPass for Drawer {
    fn program(&mut self, prog: &bril_rs::Program) -> Result<(), String> {
        let canonical = CanonicalizeLiterals.run(prog.clone());
        let call_graph = CallGraph::new(canonical.clone());

        let dot = draw_dataflow::<AvailableExpr, Set, GraphNode>(call_graph, true, false);

        println!("{}", dot);

        Ok(())
    }
}

fn main() {
    setup_logger_from_env();
    run_analysis(Drawer);
}

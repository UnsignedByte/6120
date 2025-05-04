use itertools::Itertools;
use std::{collections::HashMap, fmt::Display, hash::Hash};
use utils::{
    AnalysisPass, BBFunction, BasicBlock, CFG, CallGraph, CanonicalizeLiterals, DataflowLabel,
    DataflowPass, Foldable, HashableLiteral, Pass, draw_dataflow, run_analysis,
    setup_logger_from_env,
};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
enum Value {
    Const(HashableLiteral),
    Any,
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Any => "T".fmt(f),
            Value::Const(l) => l.fmt(f),
        }
    }
}

type Val = HashMap<String, Value>;

#[derive(Default)]
struct ConstProp;

impl DataflowPass<Val> for ConstProp {
    fn init(&self, _: &BBFunction) -> Val {
        HashMap::default()
    }

    fn meet(&self, in_vals: &[Val]) -> Val {
        let mut out_vals = HashMap::new();

        // For every key, if it has multiple different bindings, set it to Any
        // Otherwise, set it to the value
        for (name, bind) in in_vals.iter().flat_map(|v| v.iter()) {
            if let Some(v) = out_vals.get(name) {
                if v != bind {
                    out_vals.insert(name.clone(), Value::Any);
                }
            } else {
                out_vals.insert(name.clone(), bind.clone());
            }
        }

        out_vals
    }

    fn transfer(&self, block: &BasicBlock, in_val: &Val) -> Val {
        let mut out_vals = in_val.clone();

        for insn in block.iter() {
            if let Some((dest, val)) = insn.fold(|arg| {
                in_val.get(arg).and_then(|v| match v {
                    Value::Const(c) => Some(c.clone().into()),
                    Value::Any => None,
                })
            }) {
                out_vals.insert(
                    dest,
                    match val {
                        Some(v) => Value::Const(v.into()),
                        None => Value::Any,
                    },
                );
            }
        }

        out_vals
    }
}

/// Dominator set node used to display graphs
/// For dominator sets
#[derive(Clone)]
pub struct GraphNode {
    vars: Val,
}

impl DataflowLabel for GraphNode {
    fn in_label(&self, _: &CFG) -> Option<String> {
        None
    }

    fn out_label(&self, _: &CFG) -> Option<String> {
        // Create a set of variables
        let mut vars = self
            .vars
            .iter()
            .map(|(k, v)| format!("{} = {}", k, v))
            .collect_vec();

        vars.sort_unstable();

        // Return a string representation of the set
        Some(format!("{}\\l", vars.join("\\l")))
    }
}

impl From<Val> for GraphNode {
    fn from(vars: Val) -> Self {
        Self { vars }
    }
}

struct Drawer;

impl AnalysisPass for Drawer {
    fn program(&mut self, prog: &bril_rs::Program) -> Result<(), String> {
        let canonical = CanonicalizeLiterals.run(prog.clone());
        let call_graph = CallGraph::new(canonical);

        let dot = draw_dataflow::<ConstProp, Val, GraphNode>(call_graph, true, false);

        println!("{}", dot);

        Ok(())
    }
}

fn main() {
    setup_logger_from_env();
    run_analysis(Drawer);
}

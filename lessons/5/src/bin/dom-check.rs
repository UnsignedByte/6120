use std::collections::HashSet;

use utils::{run_analysis, setup_logger_from_env, AnalysisPass, DominatorTree, CFG};

struct DomChecker;

impl DomChecker {
    /// Recursive DFS to generate acyclic paths
    fn check_paths_rec(
        pos: usize,
        target: usize,
        cfg: &CFG,
        doms: &HashSet<usize>,
        visited: &mut [bool],
    ) -> Result<(), String> {
        if pos == target {
            // Make sure we've visited all dominators
            match doms.iter().all(|&dom| visited[dom]) {
                true => Ok(()),
                false => Err(format!(
                    "Not all dominators visited for path from {} to {}",
                    pos, target
                )),
            }
        } else {
            for &next in &cfg.succs(pos) {
                if !visited[next] {
                    visited[next] = true;
                    let ret = Self::check_paths_rec(next, target, cfg, doms, visited);
                    visited[next] = false;

                    // Propagate error here to make sure visited is reset
                    ret?;
                }
            }

            Ok(())
        }
    }

    /// Check that all paths from a block to a dominator are acyclic
    pub fn check_paths(target: usize, cfg: &CFG, doms: &HashSet<usize>) -> Result<(), String> {
        let mut visited = vec![false; cfg.len()];
        visited[0] = true;

        Self::check_paths_rec(0, target, cfg, doms, &mut visited)
    }
}

impl AnalysisPass for DomChecker {
    fn function(&mut self, func: &bril_rs::Function) -> Result<(), String> {
        let tree = DominatorTree::from(func.clone());

        for i in 0..tree.cfg.len() {
            let doms = tree.dominators(i);
            Self::check_paths(i, &tree.cfg, &doms)?;
        }

        Ok(())
    }
}

fn main() {
    setup_logger_from_env();
    run_analysis(DomChecker);
}

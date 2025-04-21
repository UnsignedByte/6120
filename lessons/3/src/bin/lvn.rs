use lesson_3::LVNPass;
use utils::{CanonicalizeLiterals, run_passes, setup_logger_from_env};

fn main() {
    setup_logger_from_env();
    run_passes(&mut [Box::new(CanonicalizeLiterals), Box::new(LVNPass::default())]);
}

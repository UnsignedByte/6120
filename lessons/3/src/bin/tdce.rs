use lesson_3::TDCEPass;
use utils::run_passes;

fn main() {
    run_passes(&mut [Box::new(TDCEPass)]);
}

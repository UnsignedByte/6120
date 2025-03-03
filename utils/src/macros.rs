/// A macro to run a pass pipeline on a BRIL program
/// Takes a number of structs that implement `Pass` and runs them in order
/// ```
/// pass_pipeline!(Pass1, Pass2, Pass3);
/// ```
/// is equivalent to
/// ```
/// run_passes(&mut [Box::new(Pass1), Box::new(Pass2), Box::new(Pass3)]);
/// ```
#[macro_export]
macro_rules! pass_pipeline {
    ($($pass:ident),*) => {
        $crate::run_passes(&mut [$(Box::new($pass)),*]);
    };
}

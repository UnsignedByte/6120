add_llvm_pass_plugin(InductionVariableEliminationPass
    # List your source files here.
    src/InductionVariableElimination.cpp
)

add_llvm_pass_plugin(LoopInvariantCodeMotionPass
    src/LoopInvariantCodeMotion.cpp
)
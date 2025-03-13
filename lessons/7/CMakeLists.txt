add_llvm_pass_plugin(SkeletonPass
    # List your source files here.
    src/Skeleton.cpp
)

add_llvm_pass_plugin(BlockDisplayPass
    src/BlockDisplay.cpp
)

add_library(BlockDisplayRT
    src/BlockDisplayRT.c
)
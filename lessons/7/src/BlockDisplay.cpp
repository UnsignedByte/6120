#include "llvm/Pass.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Support/raw_ostream.h"

using namespace llvm;

namespace {

struct PhiDisplayPass : public PassInfoMixin<PhiDisplayPass> {
    PreservedAnalyses run(Module &M, ModuleAnalysisManager &AM) {
        LLVMContext &Ctx = M.getContext();
        FunctionCallee BlockDisplayRT = M.getOrInsertFunction(
            "__print_block",
            Type::getVoidTy(Ctx),
            Type::getInt32Ty(Ctx),
            Type::getInt32Ty(Ctx)
        );

        for (auto &F : M) {
            int i = 0;

            for (auto &BB : F) {

                const auto &instrs = BB.instructionsWithoutDebug();
                int len = std::distance(instrs.begin(), instrs.end());

                IRBuilder<> Builder(&*BB.getFirstInsertionPt());
                Builder.CreateCall(BlockDisplayRT, {
                    Builder.getInt32(i),
                    Builder.getInt32(len)
                });
            }

            i++;
        }
        return PreservedAnalyses::all();
    };
};

}

extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
    return {
        .APIVersion = LLVM_PLUGIN_API_VERSION,
        .PluginName = "Phi Display pass",
        .PluginVersion = "v0.1",
        .RegisterPassBuilderCallbacks = [](PassBuilder &PB) {
            PB.registerPipelineStartEPCallback(
                [](ModulePassManager &MPM, OptimizationLevel Level) {
                    MPM.addPass(PhiDisplayPass());
                });
        }
    };
}
#include "llvm/Pass.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/IR/Dominators.h"
#include "llvm/Analysis/LoopInfo.h"

using namespace llvm;

namespace {

struct InductionVariableEliminationPass : public PassInfoMixin<InductionVariableEliminationPass> {
    PreservedAnalyses run(Module &M, ModuleAnalysisManager &AM) {

        for (auto &F : M) {
            errs() << "Analyzing function " << F.getName() << "\n";

            // Exit if the function is empty
            if (F.empty()) {
                continue;
            }

            DominatorTree DT = llvm::DominatorTree();
            DT.recalculate(F);
            errs() << "Calculated Dominator Tree for function " << F.getName() << "\n";

            auto LI = new llvm::LoopInfoBase<BasicBlock, Loop>();
            LI->releaseMemory();
            errs() << "Analyzing loops for function " << F.getName() << "\n";
            LI->analyze(DT);

            errs() << "Found " << LI->getLoopsInPreorder().size() << " loops\n";

            for (auto &L : LI->getLoopsInPreorder()) {
                auto *H = L->getHeader();
                errs() << "Found loop header: " << H->getName() << "\n";

                // Find phi nodes in the header.

                for (auto &I : *H) {
                    errs() << "Found instruction: " << I << "\n";
                    if (auto *PN = dyn_cast<PHINode>(&I)) {
                        errs() << "Found PHI node: " << *PN << "\n";

                        // Check if the PHI node is an induction variable.
                        
                        // Check if the PHI node has two incoming values.
                        if (PN->getNumIncomingValues() != 2) {
                            continue;
                        }

                        // One of the incoming values should be defined outside the loop.
                        auto *first = PN->getIncomingValue(0);
                        auto *second = PN->getIncomingValue(1);

                        auto *firstBB = PN->getIncomingBlock(0);
                        auto *secondBB = PN->getIncomingBlock(1);

                        if (L->contains(firstBB) && !L->contains(secondBB)) {
                            errs() << "first is defined inside the loop\n";
                        } else if (L->contains(secondBB) && !L->contains(firstBB)) {
                            errs() << "second is defined inside the loop\n";
                            // swap the incoming values
                            std::swap(first, second);
                        } else {
                            errs() << "Both incoming values are defined inside the loop\n";
                            continue;
                        }

                        // first is the potential induction variable.

                        // We are looking for something that looks like
                        // a = phi [?, %entry] [b, %body]
                        // b = a + c
                        // where a is the induction variable and c is loop invariant.

                        auto *b = first;
                        auto *a = PN;

                        
                    }
                }
            }
        }

        errs() << "Done\n";

        return PreservedAnalyses::all();
    };
};

}

extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
    return {
        .APIVersion = LLVM_PLUGIN_API_VERSION,
        .PluginName = "Induction Variable Elimination",
        .PluginVersion = "v0.1",
        .RegisterPassBuilderCallbacks = [](PassBuilder &PB) {
            PB.registerPipelineStartEPCallback(
                [](ModulePassManager &MPM, OptimizationLevel Level) {
                    MPM.addPass(InductionVariableEliminationPass());
                });
        }
    };
}
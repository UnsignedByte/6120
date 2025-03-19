#include "llvm/Pass.h"
#include "llvm/IR/Module.h"
#include "llvm/IR/IRBuilder.h"
#include "llvm/Passes/PassBuilder.h"
#include "llvm/Passes/PassPlugin.h"
#include "llvm/Support/raw_ostream.h"
#include "llvm/IR/Dominators.h"
#include "llvm/Analysis/LoopInfo.h"
#include "llvm/Analysis/DDG.h"
#include "llvm/Analysis/AliasAnalysis.h"
#include "llvm/Analysis/AssumptionCache.h"
#include "llvm/Analysis/BasicAliasAnalysis.h"
#include "llvm/Analysis/LoopInfo.h"
#include "llvm/Analysis/ScalarEvolution.h"
#include "llvm/Analysis/TargetLibraryInfo.h"
#include "llvm/AsmParser/Parser.h"
#include "llvm/IR/Dominators.h"
#include "llvm/IR/Module.h"
#include "llvm/Support/SourceMgr.h"
#include <map>


using namespace llvm;

namespace {

struct LICMPass : public PassInfoMixin<LICMPass> {
    PreservedAnalyses run(Module &M, ModuleAnalysisManager &AM) {

        for (auto &F : M) {
            errs() << "Analyzing function " << F.getName() << "\n";

            // Exit if the function is empty
            if (F.empty()) {
                continue;
            }
            
            llvm::TargetLibraryInfoImpl TLII;
            llvm::TargetLibraryInfo TLI(TLII);
            llvm::AssumptionCache AC(F);
            llvm::DominatorTree DT(F);
            llvm::LoopInfo LI(DT);
            llvm::ScalarEvolution SE(F, TLI, AC, DT, LI);
            llvm::AAResults AA(TLI);
            llvm::DependenceInfo DI(&F, &AA, &SE, &LI);

            errs() << "Found " << LI.getLoopsInPreorder().size() << " loops\n";



            for (auto &L : LI.getLoopsInPreorder()) {
                DataDependenceGraph DDG(*L, LI, DI);

                auto H = L->getHeader();
                errs() << "Found loop header: " << H->getName() << "\n";

                // Find phi nodes in the header.
                // create a preheader
                if (!L->getLoopPreheader()) {
                    auto PH = H->splitBasicBlockBefore(H->getFirstNonPHI());
                    errs() << "Created preheader: " << PH->getName() << "\n";
                }

                std::map<Value*, bool> invariant;

                // All instructions outside the loop are loop invariant
                for (auto &BB : F) {
                    if (!L->contains(&BB)) {
                        for (auto &I : BB) {
                            invariant[&I] = true;
                        }
                    }
                }

                // Loop through all instructions and figure out whether they are loop invariant
                for (auto &BB : L->getBlocks()) {
                    bool changed = true;
                    while (changed) {
                        changed = false;
                        for (auto &I : *BB) {
                            if (invariant.count(&I) && invariant[&I]) {
                                continue;
                            }

                            errs() << "Found Instruction " << I << "\n";

                            if (isa<PHINode>(&I) // Phi nodes cannot be moved
                                || I.mayHaveSideEffects() // instructions with side effects cannot be moved
                                || I.isTerminator() // Terminator instruction cannot be moved
                            ) {
                                invariant[&I] = false;
                                continue;
                            }

                            bool inv = true;

                            for (auto &O : I.operands()) {
                                if (auto *OI = dyn_cast<Instruction>(O)) {
                                    if (!invariant.count(OI) || !invariant[OI]) {
                                        inv = false;
                                        break;
                                    }
                                } else {
                                    errs() << "Operand " << *O << " was not an instruction!\n";
                                }
                            }

                            changed |= inv;

                            invariant[&I] = inv;
                        }
                    }
                }

                for (auto const &[V, inv] : invariant) {

                    if (auto *I = dyn_cast<Instruction>(V)) {
                        if (inv && L->contains(I)) {
                            
                            errs() << "Found Invariant \t" <<  *I << "\n";

                            // Ensure the instruction dominates all exit blocks
                            bool dominatesAllExits = true;
                            for (auto *ExitBB : L->getBlocks()) {
                                if (L->isLoopExiting(ExitBB)) {
                                    errs() <<
                                        "Checking that it dominates " << *ExitBB->getTerminator() << "\n";

                                    if (!DT.dominates(I, ExitBB->getTerminator())) {
                                        dominatesAllExits = false;
                                        break;
                                    }
                                }
                            }

                            if (dominatesAllExits) {
                                // Move the invariant instruction to the preheader
                                auto *Preheader = L->getLoopPreheader();
                                I->moveBefore(Preheader->getTerminator());
                                errs() << "Moved Invariant Instruction to Preheader: " << *I << "\n";
                            }
                        }
                    }
                }
            }
        }

        return PreservedAnalyses::none();
    };
};

}

extern "C" LLVM_ATTRIBUTE_WEAK ::llvm::PassPluginLibraryInfo
llvmGetPassPluginInfo() {
    return {
        .APIVersion = LLVM_PLUGIN_API_VERSION,
        .PluginName = "Loop Invariant Code Motion",
        .PluginVersion = "v0.1",
        .RegisterPassBuilderCallbacks = [](PassBuilder &PB) {
            PB.registerPipelineStartEPCallback(
                [](ModulePassManager &MPM, OptimizationLevel Level) {
                    MPM.addPass(LICMPass());
                });
        }
    };
}
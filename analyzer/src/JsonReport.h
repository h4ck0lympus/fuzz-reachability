#pragma once

#include "CallGraph.h"
#include "Metrics.h"
#include "Reachability.h"
#include "llvm/ADT/DenseSet.h"
#include "llvm/IR/Module.h"
#include "llvm/Support/raw_ostream.h"
#include <string>
#include <vector>

namespace reach {
// Emit the machine-readable reachability report (spec section 5).
// flowTargets carries the address-flow evidence used to annotate per-function
// confidence (see computeAddressFlowTargets). metrics carries the per-function
// structural and graph-derived signals (see computeMetrics).
void writeJson(llvm::raw_ostream &os, llvm::Module &m, const CallGraph &g,
               const ReachResult &res, llvm::StringRef backend,
               const std::vector<std::string> &entries,
               const llvm::DenseSet<llvm::Function *> &flowTargets,
               const llvm::DenseMap<llvm::Function *, FuncMetrics> &metrics);
}

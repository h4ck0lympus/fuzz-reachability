#pragma once

#include "llvm/IR/Function.h"
#include "llvm/IR/InstrTypes.h"
#include "llvm/IR/Module.h"
#include <vector>

namespace reach {

// Pluggable backend for resolving indirect call targets. The type-based
// resolver is the default; AnyResolver (--indirect-any) is a debug variant.
struct IndirectResolver {
  virtual ~IndirectResolver() = default;
  // Precompute over the whole module once before resolve() calls.
  virtual void prepare(llvm::Module &m) = 0;
  // Candidate callees for one indirect call site (over-approximation).
  virtual std::vector<llvm::Function *> resolve(llvm::CallBase &cb) = 0;
};

} // namespace reach

#pragma once

#include "IndirectResolver.h"
#include "TypeBasedResolver.h"
#include "llvm/ADT/DenseSet.h"
#include <memory>

namespace reach {

struct SVFState; // pimpl: hides all SVF types from the rest of the analyzer

// Optional backend (--backend=svf): SVF Andersen points-to gives per-callsite
// callee sets, narrower than the type-based over-approximation. SVF
// under-approximates function pointers that escape through memory, so resolve()
// augments each callee set with the type-matched functions whose address
// escapes into memory (`MemEscaped`) and falls back to the full type-based set
// for any callsite SVF leaves unresolved. See docs/svf-notes.md.
class SVFResolver : public IndirectResolver {
public:
  SVFResolver();
  ~SVFResolver() override;
  void prepare(llvm::Module &m) override;
  std::vector<llvm::Function *> resolve(llvm::CallBase &cb) override;

private:
  llvm::Module *Mod = nullptr;
  TypeBasedResolver Fallback;
  llvm::DenseSet<llvm::Function *> MemEscaped;
  std::unique_ptr<SVFState> State;
};

} // namespace reach

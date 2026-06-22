#include "SVFResolver.h"

#ifdef REACHABILITY_ENABLE_SVF

#include "SVF-LLVM/LLVMModule.h"
#include "SVF-LLVM/SVFIRBuilder.h"
#include "Util/ExtAPI.h"
#include "Util/Options.h"
#include "WPA/Andersen.h"
#include "llvm/ADT/SmallPtrSet.h"
#include "llvm/IR/Constants.h"
#include "llvm/IR/GlobalAlias.h"
#include "llvm/IR/GlobalVariable.h"
#include "llvm/IR/InstIterator.h"
#include "llvm/IR/Instructions.h"

using namespace llvm;

namespace reach {

namespace {

void collectFnRefs(Value *v, SmallPtrSetImpl<Value *> &visited,
                   SmallPtrSetImpl<Function *> &out) {
  if (!v || !visited.insert(v).second)
    return;
  v = v->stripPointerCasts();
  if (auto *f = dyn_cast<Function>(v)) {
    if (!f->isDeclaration())
      out.insert(f);
    return;
  }
  if (auto *ga = dyn_cast<GlobalAlias>(v))
    return collectFnRefs(ga->getAliasee(), visited, out);
  if (auto *ce = dyn_cast<ConstantExpr>(v)) {
    for (Use &u : ce->operands())
      collectFnRefs(u.get(), visited, out);
    return;
  }
  if (auto *ca = dyn_cast<ConstantAggregate>(v)) {
    for (Use &u : ca->operands())
      collectFnRefs(u.get(), visited, out);
    return;
  }
  if (auto *sel = dyn_cast<SelectInst>(v)) {
    collectFnRefs(sel->getTrueValue(), visited, out);
    collectFnRefs(sel->getFalseValue(), visited, out);
    return;
  }
  if (auto *phi = dyn_cast<PHINode>(v)) {
    for (Value *iv : phi->incoming_values())
      collectFnRefs(iv, visited, out);
    return;
  }
  if (auto *iv = dyn_cast<InsertValueInst>(v)) {
    collectFnRefs(iv->getAggregateOperand(), visited, out);
    collectFnRefs(iv->getInsertedValueOperand(), visited, out);
    return;
  }
}

void collectMemEscaped(Module &m, DenseSet<Function *> &out) {
  SmallPtrSet<Function *, 16> esc;
  SmallPtrSet<Value *, 16> visited;
  auto scan = [&](Value *v) {
    visited.clear();
    collectFnRefs(v, visited, esc);
  };
  for (GlobalVariable &g : m.globals())
    if (g.hasInitializer())
      scan(g.getInitializer());
  for (Function &f : m) {
    if (f.isDeclaration())
      continue;
    for (Instruction &i : instructions(f)) {
      if (auto *st = dyn_cast<StoreInst>(&i))
        scan(st->getValueOperand());
      else if (auto *cb = dyn_cast<CallBase>(&i)) {
        for (const Use &a : cb->args())
          scan(a.get());
      } else if (auto *ret = dyn_cast<ReturnInst>(&i))
        scan(ret->getReturnValue());
    }
  }
  out.insert(esc.begin(), esc.end());
}

} // namespace

struct SVFState {
  SVF::PointerAnalysis *pta = nullptr;
};

SVFResolver::SVFResolver() : State(std::make_unique<SVFState>()) {}

void SVFResolver::prepare(Module &m) {
  Mod = &m;
  Fallback.prepare(m); // type-based soundness net
  collectMemEscaped(m, MemEscaped);

  // Silence SVF's statistics dump so it never pollutes our JSON on stdout.
  const_cast<::Option<bool> &>(SVF::Options::PStat).setValue(false);

#ifdef REACHABILITY_SVF_EXTAPI
  SVF::ExtAPI::setExtBcPath(REACHABILITY_SVF_EXTAPI);
#endif

  // Build SVF over our in-memory module so getCallICFGNode maps our calls.
  SVF::LLVMModuleSet::buildSVFModule(m);
  SVF::SVFIRBuilder builder;
  SVF::SVFIR *pag = builder.build();
  State->pta = SVF::AndersenWaveDiff::createAndersenWaveDiff(pag);
}

std::vector<Function *> SVFResolver::resolve(CallBase &cb) {
  SVF::PointerAnalysis *pta = State->pta;
  auto *ms = SVF::LLVMModuleSet::getLLVMModuleSet();
  SVF::CallICFGNode *cs = ms->getCallICFGNode(&cb);

  std::vector<Function *> out;
  SmallPtrSet<Function *, 8> seen;
  if (cs && pta->getCallGraph()->hasIndCSCallees(cs))
    for (const SVF::FunObjVar *f : pta->getIndCSCallees(cs))
      if (Function *lf = Mod->getFunction(f->getName()))
        if (seen.insert(lf).second)
          out.push_back(lf);

  if (out.empty())
    return Fallback.resolve(cb); // SVF resolved nothing -> full type-based net

  for (Function *f : Fallback.resolve(cb))
    if (MemEscaped.count(f) && seen.insert(f).second)
      out.push_back(f);
  return out;
}

SVFResolver::~SVFResolver() { SVF::LLVMModuleSet::releaseLLVMModuleSet(); }

} // namespace reach

#endif // REACHABILITY_ENABLE_SVF

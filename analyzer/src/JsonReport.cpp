#include "JsonReport.h"
#include "Demangle.h"
#include "Toolchain.h"
#include "llvm/IR/DebugInfoMetadata.h"
#include "llvm/Support/JSON.h"
#include <algorithm>

using namespace llvm;

namespace reach {

namespace {
const char *viaStr(Via v) {
  switch (v) {
  case Via::Direct:
    return "direct";
  case Via::Indirect:
    return "indirect";
  case Via::Both:
    return "both";
  }
  return "direct";
}

// Reachability confidence. `high`: reached by a concrete direct edge (or a
// root). `medium`: reached only indirectly, but the address has value-flow
// evidence of being callable (reaches an indirect callee, or escapes to
// unanalyzable code). `low`: reached only by a type match, with no flow
// evidence -- the likely-spurious surface of the over-approximation. This is a
// triage hint, not a verdict: it never removes a function from the reachable
// set, and a target whose address is laundered through integer arithmetic
// (ptrtoint/inttoptr) can legitimately rate `low`.
const char *confidenceStr(Via via, bool hasFlow) {
  if (via != Via::Indirect)
    return "high";
  return hasFlow ? "medium" : "low";
}

// Emit one function object. `via` is null for unreachable functions.
void emitFn(json::OStream &J, Function *f, const Via *via,
            const DenseSet<Function *> &flow) {
  J.object([&] {
    J.attribute("mangled", f->getName());
    J.attribute("demangled", demangle(f->getName()));
    if (DISubprogram *sp = f->getSubprogram()) {
      J.attribute("file", sp->getFilename());
      J.attribute("line", (int64_t)sp->getLine());
    } else {
      J.attribute("file", nullptr);
      J.attribute("line", nullptr);
    }
    if (via) {
      J.attribute("via", viaStr(*via));
      J.attribute("indirect_only", *via == Via::Indirect);
      J.attribute("confidence", confidenceStr(*via, flow.count(f)));
    }
  });
}
} // namespace

void writeJson(raw_ostream &os, Module &m, const CallGraph &, const ReachResult &res,
               StringRef backend, const std::vector<std::string> &entries,
               const DenseSet<Function *> &flowTargets) {
  // Defined functions, partitioned into reachable / unreachable, sorted by name.
  std::vector<std::pair<Function *, Via>> reachable;
  std::vector<Function *> unreachable;
  for (Function &f : m) {
    if (f.isDeclaration())
      continue;
    auto it = res.reached.find(&f);
    if (it != res.reached.end())
      reachable.push_back({&f, it->second});
    else
      unreachable.push_back(&f);
  }
  auto byName = [](Function *a, Function *b) { return a->getName() < b->getName(); };
  std::sort(reachable.begin(), reachable.end(),
            [&](auto &a, auto &b) { return byName(a.first, b.first); });
  std::sort(unreachable.begin(), unreachable.end(), byName);

  int64_t indirectOnly = 0;
  int64_t lowConfidence = 0;
  for (auto &[f, via] : reachable) {
    if (via == Via::Indirect) {
      ++indirectOnly;
      if (!flowTargets.count(f))
        ++lowConfidence;
    }
  }

  json::OStream J(os, 2);
  J.object([&] {
    J.attribute("llvm_version", std::to_string(linkedLLVMMajor()));
    J.attribute("backend", backend);
    J.attributeArray("entries", [&] {
      for (const auto &e : entries)
        J.value(e);
    });
    J.attributeObject("summary", [&] {
      J.attribute("defined", (int64_t)(reachable.size() + unreachable.size()));
      J.attribute("reachable", (int64_t)reachable.size());
      J.attribute("indirect_only", indirectOnly);
      J.attribute("low_confidence", lowConfidence);
      J.attribute("unreachable", (int64_t)unreachable.size());
    });
    J.attributeArray("reachable", [&] {
      for (auto &[f, via] : reachable)
        emitFn(J, f, &via, flowTargets);
    });
    J.attributeArray("unreachable_defined", [&] {
      for (Function *f : unreachable)
        emitFn(J, f, nullptr, flowTargets);
    });
  });
  os << "\n";
}

} // namespace reach

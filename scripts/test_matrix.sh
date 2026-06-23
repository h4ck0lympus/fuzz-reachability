#!/usr/bin/env bash
# LLVM version-compatibility matrix: build + test the analyzer against every
# installed llvm-config-NN with NN >= MIN_LLVM. Prints a matrix and exits
# non-zero if any build/test fails -- this is the early-warning system for
# breakage on future LLVMs.
#
#   scripts/test_matrix.sh
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
MIN_LLVM=21
PY="$ROOT/.venv/bin/python"
PATH="$(go env GOPATH 2>/dev/null)/bin:$PATH"  # gllvm, if installed
export PATH

declare -A CORE
versions=()
overall=0

for cfg in $(ls /usr/bin/llvm-config-* 2>/dev/null | sort -V); do
  major="$($cfg --version 2>/dev/null | cut -d. -f1)"
  [[ "$major" =~ ^[0-9]+$ ]] || continue
  [ "$major" -ge "$MIN_LLVM" ] || continue
  versions+=("$major")
  echo "=== LLVM $major ==="

  # --- build + analyzer behavior tests (no external toolchain needed) ---
  if make -C "$ROOT/analyzer" LLVM_CONFIG="llvm-config-$major" BUILD="build/$major" \
        >"/tmp/matrix-core-$major.log" 2>&1; then
    if REACHABILITY_ANALYZER="$ROOT/analyzer/build/$major/reachability-analyzer" \
       "$PY" -m pytest "$ROOT/driver/tests/test_analyzer_core.py" -q \
       -p no:cacheprovider >"/tmp/matrix-coretest-$major.log" 2>&1; then
      CORE[$major]="PASS"
    else
      CORE[$major]="TEST-FAIL"; overall=1
    fi
  else
    CORE[$major]="BUILD-FAIL"; overall=1
  fi
done

echo
printf "%-8s %-12s\n" "LLVM" "result"
printf "%-8s %-12s\n" "----" "------"
for v in "${versions[@]}"; do
  printf "%-8s %-12s\n" "$v" "${CORE[$v]}"
done
echo
[ "$overall" -eq 0 ] && echo "matrix: all PASS" || echo "matrix: FAILURES (see /tmp/matrix-*.log)"
exit "$overall"

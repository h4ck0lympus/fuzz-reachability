#!/usr/bin/env bash
# Print the LLVM major to build the analyzer against: the newest installed major
# >= MIN_LLVM (21). Newer LLVM tools read older bitcode but not newer, so the
# newest toolchain is the safest default -- in particular it can read rustc's
# bitcode (a too-old LLVM cannot). Falls back to MIN_LLVM when no suitable
# llvm-config is found.
set -uo pipefail
MIN_LLVM="${MIN_LLVM:-21}"

newest=""
for cfg in $(ls /usr/bin/llvm-config-* 2>/dev/null | sort -V); do
  m="${cfg##*/llvm-config-}"
  [[ "$m" =~ ^[0-9]+$ ]] || continue
  [ "$m" -ge "$MIN_LLVM" ] || continue
  newest="$m"
done

if [ -n "$newest" ]; then
  echo "$newest"
else
  echo "$MIN_LLVM"
fi

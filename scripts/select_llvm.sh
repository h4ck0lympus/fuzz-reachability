#!/usr/bin/env bash
# Print the LLVM major to build the analyzer against: prefer MIN_LLVM (21) when
# an llvm-config for it is installed, otherwise the newest installed major above
# it. Falls back to MIN_LLVM when no suitable llvm-config is found.
set -uo pipefail
MIN_LLVM="${MIN_LLVM:-21}"

newest=""
for cfg in $(ls /usr/bin/llvm-config-* 2>/dev/null | sort -V); do
  m="${cfg##*/llvm-config-}"
  [[ "$m" =~ ^[0-9]+$ ]] || continue
  [ "$m" -ge "$MIN_LLVM" ] || continue
  if [ "$m" -eq "$MIN_LLVM" ]; then
    echo "$MIN_LLVM"
    exit 0
  fi
  newest="$m"
done

if [ -n "$newest" ]; then
  echo "$newest"
else
  echo "$MIN_LLVM"
fi

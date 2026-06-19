#!/usr/bin/env bash
# Print the LLVM major to build the analyzer against: the smallest installed
# llvm-config-N (N >= MIN_LLVM) whose FULL version is >= rustc's bundled LLVM
# full version. Same major is not enough -- a distro LLVM that is an older patch
# release than rustc's cannot read rustc's bitcode (llvm-link: "Invalid record").
# Falls back to the highest installed major (>= MIN_LLVM), else MIN_LLVM.
set -uo pipefail
MIN_LLVM="${MIN_LLVM:-21}"

ver() { grep -oE '[0-9]+(\.[0-9]+){0,2}' | head -n1; }
ge() { [ "$(printf '%s\n%s\n' "$2" "$1" | sort -V | head -n1)" = "$2" ]; }

rustc_ver="$(rustc -vV 2>/dev/null | sed -n 's/^LLVM version:[[:space:]]*//p' | ver)"

best=""
chosen=""
for cfg in $(ls /usr/bin/llvm-config-* 2>/dev/null | sort -V); do
  m="${cfg##*/llvm-config-}"
  [[ "$m" =~ ^[0-9]+$ ]] || continue
  [ "$m" -ge "$MIN_LLVM" ] || continue
  best="$m"
  full="$("$cfg" --version 2>/dev/null | ver)"
  [ -n "$full" ] || continue
  if [ -z "$rustc_ver" ] || ge "$full" "$rustc_ver"; then
    chosen="$m"
    break
  fi
done

if [ -n "$chosen" ]; then
  echo "$chosen"
elif [ -n "$best" ]; then
  echo "$best"
else
  echo "$MIN_LLVM"
fi

#!/bin/bash

test -z "$AFL_PATH" && AFL_PATH=/prg/dev
PATH=$AFL_PATH:$PATH

AFL_LLVM_ALLOWLIST=`pwd`/reached.txt cargo ziggy build --no-honggfuzz

mkdir -p in
echo > in/in

afl-fuzz -i in -o out -V 60 -- target/afl/debug/rust_ziggy_indirect_calls

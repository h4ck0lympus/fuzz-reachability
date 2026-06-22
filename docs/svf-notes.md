# SVF backend — build spike notes (Milestone 5)

## Outcome: SUCCESS

SVF builds cleanly against the **system LLVM 21.1.8** and works as a real
`--backend=svf`, passing the same soundness invariant as the type-based backend
on every fixture.

## Why it worked

The top risk in the spec (§8) was SVF lagging LLVM releases. As of the pinned
commit, **SVF master already targets LLVM 21.1.x** (`setup.sh` references
`llvm-21.1.0.obj`), which matches our system LLVM 21.1.8 at the major+minor
level. One small source patch is applied to SVF's type inference — see gotcha
6 below.

## Build recipe (`scripts/build_svf.sh`)

- **SVF commit:** `795fd5cbbc2e5e343277c391300ba1d1d9903a73` (master, 2026-06-09).
- **LLVM:** system `/usr/lib/llvm-21` (NOT SVF's bundled download) — preserves the
  single-shared-LLVM prerequisite, since clang/rustc/analyzer all use 21.1.8.
- **Z3:** prebuilt `z3-4.8.8-x64-ubuntu-16.04` downloaded to `third_party/z3.obj`
  (the version SVF's own `build.sh` uses). No system Z3 required.
- Outputs `libSvfCore.a`, `libSvfLLVM.a`, `extapi.bc`, and a CMake config
  (`SVF::SvfCore`, `SVF::SvfLLVM`) under `third_party/SVF/install`.

> **Build system note:** the analyzer now builds with a plain Makefile (no
> CMake). Enable SVF with `make build-svf LLVM_MAJOR=21` (or
> `make -C analyzer SVF=1 BUILD=build-svf`). The Makefile links the SVF static
> archives + the LLVM dylib + z3 directly. SVF's *own* vendored build
> (`scripts/build_svf.sh`) still uses SVF's CMake internally — that is an
> external dependency, not part of our build. See also
> [`llvm-support.md`](llvm-support.md) for the current per-version status.

## Analyzer integration (`analyzer/src/SVFResolver.cpp`)

- Enabled via `REACHABILITY_ENABLE_SVF` (set by `make SVF=1`), which links the
  SVF static archives, the LLVM dylib, and z3.
- `SVFResolver` builds SVF over our **in-memory** module
  (`LLVMModuleSet::buildSVFModule(Module&)`) so `getCallICFGNode` maps our exact
  call instructions, runs `AndersenWaveDiff`, and reads per-callsite callees via
  `getIndCSCallees`. Callees map back to `llvm::Function*` by name.
- **Soundness:** SVF reliably tracks function pointers that flow through SSA
  values, direct call arguments, and returns, but its points-to
  **under-approximates** pointers that escape through memory — a function
  address stored into a struct field or a global table and later loaded and
  called indirectly. (Found in libtiff: `PackBitsPreEncode` is wired up by
  `TIFFInitPackBits` storing it into `tif->tif_preencode`, reached through the
  `_TIFFBuiltinCODECS` table; raw SVF dropped it and reported it unreachable.)
  So `SVFResolver` augments every per-callsite SVF set with the type-matched
  functions whose address escapes into memory (a `store`, a global initializer,
  a call argument, or a return — collected once in `prepare()`), and falls back
  to the **full** type-based set for any callsite SVF leaves unresolved. SVF
  therefore never misses a memory-resident target, while staying narrower than
  type-based for the pointers it tracks precisely (those are decided by SVF
  alone). The regression fixture `fixtures/c_codec_table` reproduces the dropped
  target and is asserted sound under both backends.

## Gotchas handled

1. **zstd export quirk** — LLVM 21's `LLVMSupport` link interface references a
   `zstd::libzstd_shared` target that Ubuntu's libzstd ships no config for; the
   analyzer's CMake defines it from `libzstd.so.1`.
2. **`z3::libz3` target** — created by SVF's bundled `FindZ3` (point it at
   `third_party/z3.obj` via `Z3_DIR`); do NOT also define it manually or CMake
   errors on the duplicate.
3. **`LLVM` target** — SVF's exported targets reference a target literally named
   `LLVM`; the analyzer defines it as an imported dylib (`libLLVM.so`). When SVF
   is on we link the LLVM dylib instead of the component static libs to keep a
   single LLVM in the process.
4. **`extapi.bc` at runtime** — set via `ExtAPI::setExtBcPath()` from the
   compile-time `REACHABILITY_SVF_EXTAPI` path, so no `SVF_DIR` env is required.
5. **stdout pollution** — SVF dumps statistics to stdout by default; disabled via
   `Options::PStat=false` so the analyzer's JSON stays clean on stdout.
6. **`ObjTypeInference` abort on void/non-returning callees** — during
   `SVFIRBuilder::build()`, SVF's object type inference back-traces a value to a
   `call` and follows the callee's return value to find the underlying
   allocation. Both back-tracers (`bwfindAllocOfVar`, `fwFindAllocOrClsNameSources`
   in `svf-llvm/lib/ObjTypeInference.cpp`) ran
   `ABORT_IFNOT(retInst && retInst->getReturnValue(), "not return inst?")`, an
   unconditional `abort()` (SIGABRT → analyzer exits -6). It fires whenever the
   callee's exit block has no value-returning `ret` (returns `void`, ends in
   `unreachable`/infinite loop, or is noreturn but not annotated — the
   `doesNotReturn()` guard misses those, and `LLVMModule.cpp` falls back to the
   last block as exit BB regardless). Patched to skip gracefully (`if (retInst &&
   retInst->getReturnValue()) insert...`): such a call simply isn't an allocation
   source, which is sound for this best-effort inference. The patch is tracked as
   `scripts/svf-objtypeinference-abort.patch` and reapplied idempotently by
   `scripts/build_svf.sh` after every `git checkout` (it skips when already
   applied), so a fresh clone or hard-reset of the vendored, gitignored SVF tree
   self-heals on the next `make build-svf`. To add further SVF source patches,
   drop a `scripts/svf-*.patch` file — the build script applies them all.

## If SVF ever stops building against the pinned LLVM

The `IndirectResolver` interface and the type-based backend are fully
independent of SVF. Build without SVF (`make build`, i.e. `SVF=0`, the default)
and `--backend=svf` returns a clear "SVF backend not available" error (exit 2).
No silent degradation. This is exactly what happens on LLVM 22/23 today — see
[`llvm-support.md`](llvm-support.md).

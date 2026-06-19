# Reachability for a ziggy fuzz harness

A [ziggy](https://github.com/srlabs/ziggy) harness is an ordinary Rust **binary
crate**: the fuzz loop lives in `ziggy::fuzz!(|data: &[u8]| { … })` *inside*
`fn main()`. There is no `LLVMFuzzerTestOneInput`. The entry point is therefore
the **Rust `main`**, and the rest of the merged-bitcode pipeline is identical to
the normal Rust flow ([README](../README.md) §3).

Only two things are ziggy-specific:

1. You emit bitcode for a **bin** crate (the harness) and all its dependencies.
2. You root the analysis at the **mangled Rust `main`** — *not* the bare `main`
   symbol (the C-ABI shim, which dead-ends in precompiled `std`; see the gotcha
   below).

## TL;DR

Here `reachability-analyzer` is the built binary
(`analyzer/build/reachability-analyzer`, or `$REACHABILITY_ANALYZER`), and
`llvm-link-22` / `llvm-nm-22` are the LLVM tools matching the analyzer's major
(≥ rustc's LLVM — see step 2).

```bash
cd <harness>                      # the ziggy bin crate directory

# 1. Emit per-crate bitcode (the final link may fail; only the .bc matter).
#    Add any rustflags the project's .cargo/config.toml requires (see below).
RUSTFLAGS="--emit=llvm-bc -Cembed-bitcode=yes -Ccodegen-units=1" cargo build

# 2. Merge with an llvm-link whose LLVM major matches the analyzer (>= rustc's).
llvm-link-22 target/debug/deps/*.bc -o merged.bc

# 3. Find the mangled Rust main (the symbol that demangles to <crate>::main).
llvm-nm-22 --defined-only target/debug/deps/<bin>-*.bc | grep ' T ' | grep main

# 4. Analyze, rooted at that symbol.
reachability-analyzer merged.bc --entry '<mangled main>' \
  --out reach.json --reached-out reached.txt --not-reached-out not_reached.txt
```

## Step by step

### 1. Emit bitcode

Same `RUSTFLAGS` as any Rust target. Two project-specific wrinkles:

- **Custom rustflags.** A setting of `RUSTFLAGS` in the environment *replaces*
  (does not merge with) a `rustflags` array in the project's
  `.cargo/config.toml`. If the project needs flags to build (e.g. aptos sets
  `rustflags = ["--cfg", "tokio_unstable"]`), include them yourself:

  ```bash
  RUSTFLAGS="--cfg tokio_unstable --emit=llvm-bc -Cembed-bitcode=yes -Ccodegen-units=1" cargo build
  ```

- **Where the `.bc` land.** They go to `<target>/debug/deps/*.bc`. For a crate
  that is its own package or is `exclude`d from a workspace, that is
  `<harness>/target/debug/deps`. For a **workspace member**, cargo writes to the
  *workspace* target (`<workspace>/target/debug/deps`); point cargo at a local
  dir with `CARGO_TARGET_DIR=$PWD/target` if you want it next to the harness.

If you rebuild with different flags, cargo keeps the **old** per-crate `.bc`
(one extra hash per build). Linking duplicates fails (`llvm-link`: redefined
symbol), so start from a clean `deps/` (or keep only the newest `.bc` per crate)
before merging.

### 2. Merge

Use an `llvm-link` whose LLVM major matches the analyzer and is **≥ rustc's**
bundled LLVM (the analyzer reads the merged module; see
[`llvm-support.md`](llvm-support.md)). On a box where the default toolchain is
LLVM 22, use `llvm-link-22`.

### 3. Find the Rust `main` entry — and avoid the C `main` shim

`--entry` matches the **exact mangled symbol** (`Module::getFunction`), so you
must pass `main`'s mangled name. A Rust bin produces *two* `main`-ish symbols:

| symbol | what it is | use as entry? |
|--------|------------|---------------|
| `main` | C-ABI shim that calls `std::rt::lang_start` | **No** — `lang_start` lives in precompiled `std` (a declaration in the bitcode), so the BFS dead-ends almost immediately (≈2 functions reachable). |
| `_ZN…<crate>…main…E` / `_RNvC…<crate>…main` | the real Rust `main` (your harness body, incl. the `ziggy::fuzz!` closure) | **Yes.** |

Find it by listing defined text symbols and picking the one that demangles to
`<crate>::main`:

```bash
llvm-nm-22 --defined-only target/debug/deps/<bin>-*.bc | grep ' T ' | grep main
# then confirm with the analyzer's demangler:
reachability-analyzer --selftest-demangle '<symbol>'
```

The crate name is the **bin target's** name with `-` → `_` (e.g. a `[[bin]]
name = "global-storage"` → `global_storage`); for a default-named bin it is the
package name. So the symbol looks like
`_ZN14global_storage4main17h…E` (legacy) or `_RNvCs…_14global_storage4main`
(v0 — when the project builds with `-Csymbol-mangling-version=v0`).

If you guess wrong and the entry does not resolve, the analyzer prints the
defined symbols whose names contain your string — a handy way to discover the
exact mangled `main`.

### 4. Analyze

```bash
reachability-analyzer merged.bc \
  --entry '_ZN14global_storage4main17h…E' \
  --out reach.json --reached-out reached.txt --not-reached-out not_reached.txt
```

Output is the usual triple (JSON report + sancov allow/ignore lists; see
[README §Output](../README.md#output)). The `ziggy::fuzz!` closure
(`<crate>::main::{{closure}}`) is reached from `main` and pulls in the whole
per-input code path.

## Using the driver instead

`reachability run --lang rust --project <harness> --entry '<mangled main>' --out reach.json`
does steps 1–4 in one shot. Caveats for ziggy/large projects:

- It sets its own `RUSTFLAGS` (no project `--cfg` flags) and globs **all**
  `<project>/target/debug/deps/*.bc`, so it needs a project that builds under
  those flags and a clean `deps/` (one `.bc` per crate). When either does not
  hold, use the manual steps above.
- It globs `<project>/target/...`; for a workspace **member** export
  `CARGO_TARGET_DIR=<project>/target` first so the bitcode lands where the glob
  looks.

## Worked example: move-smith `global-storage`

`~/aptos/move-smith/fuzz-ziggy` is a ziggy harness (`[[bin]] name =
"global-storage"`, `exclude`d from the aptos workspace, so it has its own
`target/`). aptos needs `--cfg tokio_unstable`.

```bash
cd ~/aptos/move-smith/fuzz-ziggy
RUSTFLAGS="--cfg tokio_unstable --emit=llvm-bc -Cembed-bitcode=yes -Ccodegen-units=1" cargo build
llvm-link-22 target/debug/deps/*.bc -o /tmp/merged.bc        # keep one .bc per crate
llvm-nm-22 --defined-only target/debug/deps/global_storage-*.bc | grep ' T ' | grep main
#   T _ZN14global_storage4main17hcc7e51cc3974f743E              <- the Rust main
#   t _ZN14global_storage4main28_$u7b$…closure…$u7d$…E          <- the ziggy::fuzz! body
reachability-analyzer /tmp/merged.bc \
  --entry _ZN14global_storage4main17hcc7e51cc3974f743E \
  --out /tmp/reach.json --reached-out /tmp/reached.txt --not-reached-out /tmp/not_reached.txt
```

Result (type-based backend): **227,492 reachable / 296,199 defined**
(7,690 indirect-only, 68,707 unreachable). The reachable set includes the
`ziggy::fuzz!` closure and the full execution stack it drives —
`msmith::…::execute_without_save`, `TransactionalInputBuilder`,
`TransactionalExecutor`, `TransactionalResult::is_bug`, etc. Rooting at the bare
`main` shim instead yields only 2 — the lang_start gotcha above.

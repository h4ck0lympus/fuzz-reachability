# Reachability for a ziggy fuzz harness

A [ziggy](https://github.com/srlabs/ziggy) harness is an ordinary Rust **binary
crate**: the fuzz loop lives in `ziggy::fuzz!(|data: &[u8]| { … })` *inside*
`fn main()`. There is no `LLVMFuzzerTestOneInput`, so the entry point is the Rust
`main`. Everything else matches the normal Rust flow
([README §A Rust target](../README.md#a-rust-target)).

## The driver does it in one command

`--lang ziggy` acquires the harness bitcode and roots at `main` automatically:

```bash
reachability run --lang ziggy --project <harness> --out reach.json
```

Rooting at `main` is enough: the analyzer resolves the token `main` to the real
Rust `main` (your harness body, including the `ziggy::fuzz!` closure), so you
never type a mangled symbol. The bare C-ABI `main` shim on its own would
dead-end in precompiled `std` — rooting at `main` avoids that by matching the
Rust `main` too.

## Caveats for real projects

The driver sets its own `RUSTFLAGS` and links **every** `.bc` under
`<project>/target/debug/deps/`. Two situations need a hand:

- **The project needs custom rustflags to build.** Setting `RUSTFLAGS` in the
  environment *replaces* (does not merge with) a `rustflags` array in
  `.cargo/config.toml`. If the project requires flags such as
  `--cfg tokio_unstable`, the driver's flags won't include them — build manually
  (below).
- **The harness is a workspace member.** Cargo writes a member's bitcode to the
  *workspace* target, not the harness's own. Export
  `CARGO_TARGET_DIR=<project>/target` so the bitcode lands where the driver
  looks. (A crate that is its own package, or `exclude`d from the workspace,
  already has its own `target/`.)

## Manual path

Use this when the driver's fixed rustflags don't fit. Here `llvm-link-22` is the
LLVM tool whose major matches the analyzer (and is ≥ rustc's LLVM — see
[`llvm-support.md`](llvm-support.md)).

```bash
cd <harness>

# 1. Emit per-crate bitcode. Add any rustflags the project needs to build.
#    The final link may fail under --emit=llvm-bc; only the .bc files matter.
RUSTFLAGS="--cfg tokio_unstable --emit=llvm-bc -Cembed-bitcode=yes -Ccodegen-units=1" cargo build

# 2. Merge. Start from a clean deps/ — a rebuild with different flags leaves
#    stale .bc behind, and linking duplicate symbols fails.
llvm-link-22 target/debug/deps/*.bc -o merged.bc

# 3. Analyze, rooted at the Rust main.
reachability-analyzer merged.bc --entry main \
  --out reach.json --reached-out reached.txt --not-reached-out not_reached.txt
```

The output is the usual triple — JSON report plus the sancov allow/ignore lists
([README §Output](../README.md#output)).

## Worked example: move-smith `global-storage`

`~/aptos/move-smith/fuzz-ziggy` is a ziggy harness (`[[bin]] name =
"global-storage"`, excluded from the aptos workspace, so it has its own
`target/`). aptos needs `--cfg tokio_unstable`, so build it manually:

```bash
cd ~/aptos/move-smith/fuzz-ziggy
RUSTFLAGS="--cfg tokio_unstable --emit=llvm-bc -Cembed-bitcode=yes -Ccodegen-units=1" cargo build
llvm-link-22 target/debug/deps/*.bc -o /tmp/merged.bc
reachability-analyzer /tmp/merged.bc --entry main \
  --out /tmp/reach.json --reached-out /tmp/reached.txt --not-reached-out /tmp/not_reached.txt
```

`--entry main` resolves to `global_storage::main`. The type-based backend reports
**229,419 reachable / 296,199 defined**, including the `ziggy::fuzz!` closure and
the full execution stack it drives (`execute_without_save`,
`TransactionalInputBuilder`, `TransactionalExecutor`, …). Rooting at the bare
`main` shim instead would yield only 2 functions — the dead-end described above.

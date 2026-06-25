# rust_ziggy_indirect_calls

A cargo-ziggy Rust fuzz target intended to stress CFG/callgraph recovery for source-level and machine-level indirect calls.

## Build

```sh
cargo install ziggy cargo-afl honggfuzz
cargo ziggy build
```

`cargo ziggy build` produces both an AFL++ binary (`target/afl/`) and a Honggfuzz
binary (`target/honggfuzz/`); pass `--no-honggfuzz` to build only the AFL++ one.
The `libc` dependency supplies the PLT, `dlopen`/`dlsym`, and signal edges, and
`build.rs` passes `-rdynamic` so the `dlsym` path resolves its own exported
target at runtime.

## Run

```sh
cargo ziggy fuzz -i in
```

`cargo ziggy fuzz` runs AFL++ and Honggfuzz in parallel over a shared corpus,
seeded from `in/`. Reproduce a single input with
`cargo ziggy run -i <file>`. The `in/seed_all` corpus file contains a byte
sequence that walks the main dispatcher through all 44 selector buckets; the
fuzzers mutate from there.

## Roots

There are several reachability roots that converge on `harness_entry`:

* `main` via the `ziggy::fuzz!` closure (the cargo-ziggy entry).
* `LLVMFuzzerTestOneInput` (exported), so a tool rooted at that symbol works unchanged.
* `.init_array` constructor `staticinit_ctor`, run by the loader before `main`; a tool should treat `.init_array` / `llvm.global_ctors` entries as roots.

## Naming

* `redherings_*`: callback-/dispatch-looking paths that are static/direct in Rust, plus precision-trap baits (address-taken-but-uncalled, same-signature, uninstantiated-vtable, dead-branch) that must never be reported as reached.
* `unreachable_*`: exported functions that should not be reached from `harness_entry` / `ziggy::fuzz!`.
* Other functions are named by call type: `fnptr_*`, `dyn_fn_*`, `trait_object_*`, `raw_waker_*`, `dyn_future_*`, etc.

## Mechanism coverage

The harness exercises:

* raw function pointer calls
* static function pointer tables
* function pointers stored in structs
* `Option<fn>` dispatch
* enum-contained function pointers
* method pointers / UFCS function pointers
* `extern "C"` function pointer callbacks
* function pointer callback parameters
* `&dyn Fn`, `Box<dyn Fn>`, `Box<dyn FnMut>`, `Box<dyn FnOnce>`
* callback registries containing boxed `dyn Fn`
* trait object method calls via `Box<dyn Trait>` and `Arc<dyn Trait>`
* trait object calls stored inside another object
* `dyn Iterator::next`
* `dyn Read::read`
* `dyn fmt::Write`
* `dyn Debug` formatting
* `dyn Error::source`
* `dyn Any` type-id/downcast checks
* dynamic drop through `Box<dyn Trait>`
* `Pin<Box<dyn Future>>` polling
* `RawWakerVTable` clone/wake/drop function pointers
* boxed `FnOnce` through `std::thread::spawn`
* non-capturing closures coerced to bare `fn` pointers
* runtime `HashMap<u8, fn>` dispatch tables
* function pointers laundered through `*const ()` via `transmute`
* function pointers laundered through `usize` (integer) via `transmute`
* machine-level indirect `call` through a register via inline `asm!` (x86_64)
* trait upcasting (`dyn Derived` → `dyn Base`) with supertrait vtable dispatch
* panic unwinding with `Drop` glue run during the unwind, caught by `catch_unwind`
* PLT / external-symbol edges into libc (`abs`, `strlen`)
* `dlopen(self)` + `dlsym` string-resolved call (target transmuted from the symbol pointer)
* OS signal handler (`extern "C"`) registered with `signal` and invoked by the kernel on `raise`
* `OnceLock::get_or_init` lazy one-time closure (fires on the first iteration only)
* `.init_array` pre-`main` constructor

The `transmute`, `usize`-launder, inline-`asm!`, and `dlsym` paths are
machine-level indirect calls whose targets are not recoverable from Rust types
alone. The PLT, signal, and `.init_array` paths are control transfers crossing
the module / OS / loader boundary.

## Red herrings and precision traps

Reached-but-static dispatch (must NOT be modelled as indirect): generic static
dispatch, monomorphized closure calls, enum match dispatch with direct targets,
macro-generated direct calls, and generic sort callbacks.

Indirect-call over-approximation baits (must NOT be reported as reached):

* address-taken function pointers stored in a static table whose slots are never invoked
* functions with the same `fn(u64, u8) -> u64` signature as real targets, address-taken but never stored in a called pointer
* an `impl trait_object_Op` on a type never coerced to `dyn` (no vtable emitted; CHA includes it, RTA/instantiation-aware excludes it)
* a call site in a branch that is provably never taken (`if black_box(false)`); the trivially dead `if false` variant is dropped by the compiler before bitcode emission

`unreachable_*` exported functions have no incoming edge from either root and
must be excluded despite surviving in the IR.

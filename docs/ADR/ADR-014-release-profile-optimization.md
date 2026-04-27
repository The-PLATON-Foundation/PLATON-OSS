---
date: 2026-04-26
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-014: Aggressive Cargo Release Profile (LTO + codegen-units=1 + strip)

## Context

Platon is a performance-critical VM kernel. Production execution latency targets are < 30μs per command (warm path). The Cargo release profile controls the trade-off between compile time and runtime performance. The default `[profile.release]` (opt-level=2, no LTO, codegen-units=16) leaves significant performance on the table for a crate of Platon's size and call patterns.

The primary deployment model is Docker containers built in CI, so compile time is paid once per release — not per developer iteration.

## Decision

```toml
[profile.release]
opt-level      = 3
lto            = true
codegen-units  = 1
strip          = true
```

**`opt-level = 3`**: Maximum LLVM optimisation. Enables auto-vectorisation, aggressive inlining, loop unrolling. Measured improvement over `opt-level = 2`: ~15% faster execution loop on the AVAP `addVar` benchmark.

**`lto = true`** (thin LTO): Link-Time Optimisation allows the linker to inline across crate boundaries. Critical for Platon: the execution loop in `platon/src/lib.rs` calls into `platon-core` functions (`VMState::pop`, `VMState::push`, `read_u32`) on every instruction. Without LTO these are cross-crate calls; with LTO they are inlined.

**`codegen-units = 1`**: Disables parallel code generation. With multiple codegen units, LLVM cannot optimise across unit boundaries. Combined with LTO, this gives LLVM the full picture for inlining decisions. Cost: longer compile time (~2× vs 16 units).

**`strip = true`**: Remove debug symbols from the final `.so`. Reduces wheel size by ~40% (the PyO3 extension module is a shared library). Debug symbols are not useful in production without matching `.dSYM` / `.pdb` files.

## Alternatives Considered

**Default release profile** (`opt-level=2`, no LTO, `codegen-units=16`): Faster compile (~30s vs ~60s). Rejected — measured 20–30% slower execution loop. For a kernel executing millions of instructions, this is a significant regression.

**`lto = "thin"`** (explicit thin LTO): Same as `lto = true` in Cargo's current default mapping. No difference in practice — retained as `true` for readability.

**`lto = "fat"` (full LTO)**: More aggressive cross-crate optimisation than thin LTO. Compile time increases to ~90s. Rejected — the performance gain vs thin LTO is marginal for Platon's crate graph (only two crates: platon + platon-core).

**Profile-Guided Optimisation (PGO)**: Collect profiling data from representative workloads, use it to guide branch prediction and inlining. Rejected for now — requires maintaining a representative benchmark corpus and a two-stage build pipeline. High maintenance cost; deferred to post-1.0.

**`debug = true` in release**: Keep symbols for profiling. Rejected — significantly increases wheel size and cannot be selectively stripped after the fact. Instead, use `RUSTFLAGS="-g" maturin develop` for profiling builds (documented in PERFORMANCE.md).

## Consequences

- `maturin build --release` for `platon` takes ~60s; for `avap-isa` (larger codebase) ~150s
- This is acceptable because: (a) builds happen in CI, not in the container at startup; (b) pre-built wheels are the target distribution model (see ROADMAP 0.4.0)
- The current Docker-build-at-startup workaround takes ~215s total — this is a known P0 blocker, not a consequence of the release profile choice
- `strip = true` reduces the compiled `.so` from ~3MB to ~1.8MB — meaningful for container layer caching
- Developers using `maturin develop` (without `--release`) get the default dev profile (fast compile, no optimisation) — correct trade-off for inner-loop development

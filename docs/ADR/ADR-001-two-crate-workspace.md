---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-001: Two-Crate Cargo Workspace (platon-core + platon)

## Context

The Platon kernel requires:
1. Rust types shared between the VM kernel and external ISA crates (`Value`, `VMState`, `ISAProvider`)
2. Python bindings via PyO3

The naive approach — a single crate — creates a hard dependency on PyO3 for every ISA implementation. On macOS, this causes a double-linking error: when both `platon` and `avap-isa` link PyO3's Python symbols, the linker produces conflicting symbol definitions at runtime (`dyld: Symbol not found`).

## Decision

Split into two crates within a single Cargo workspace:

```
platon/                 (workspace root)
├── Cargo.toml          [workspace] + [package] platon cdylib
├── src/lib.rs          PyO3 bindings only
└── platon-core/
    ├── Cargo.toml      [package] platon-core rlib, no PyO3
    └── src/lib.rs      Value, VMState, ISAProvider, InstructionSet
```

`platon-core` is a pure Rust `rlib` with zero non-std dependencies. `platon` depends on `platon-core` and PyO3. External ISA crates depend only on `platon-core` — they never see PyO3.

## Alternatives Considered

**Single crate**: All code in one repo. Rejected — any ISA implementation would drag in PyO3 as a dependency, causing double-linking on macOS and coupling ISA development to Python toolchain availability.

**Three separate repos**: `platon-core`, `platon`, and ISAs as fully independent repos. Considered but rejected for the initial version — the workspace simplifies local development and the Cargo path dependency `platon-core = { path = "../platon-core" }` works identically in CI and Docker. Can be split later if governance requires it.

**Dynamic library (cdylib) for platon-core**: Expose `platon-core` as a dynamic library so ISAs can link at runtime. Rejected — adds complexity, breaks Rust's ownership model across FFI boundaries, and is unnecessary when ISAs are compiled together with the kernel.

## Consequences

- `platon-core` is usable as a Rust library without any Python toolchain
- ISA implementations can be written and tested in pure Rust
- Adding `#[pyo3]` annotations to `platon-core` types is prohibited
- `cargo build` for `platon-core` alone takes ~2s; `platon` (with PyO3) takes ~45s
- The workspace Cargo.lock is shared between both crates

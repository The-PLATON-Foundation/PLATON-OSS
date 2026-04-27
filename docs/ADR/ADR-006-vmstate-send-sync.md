---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-006: VMState Implements Send + Sync via unsafe impl

## Context

`VMState` contains `registry_ptr: *mut ()` — a raw pointer. Raw pointers are `!Send` and `!Sync` in Rust by default because the compiler cannot verify thread safety for raw pointers. However, `Arc<dyn ISAProvider>` requires `ISAProvider: Send + Sync`, which in turn requires `VMState` to be `Send + Sync` (ISA handlers take `&mut VMState`).

## Decision

Use `unsafe impl Send for VMState` and `unsafe impl Sync for VMState` with the following safety invariant:

> `VMState` is accessed from a single thread within a single `execute()` call. The Python GIL is held for the entire duration of `execute()`. `registry_ptr` points to a `Py<PyDict>` that is valid while the GIL is held.

```rust
// Safety: VMState is only used single-threaded within a single execute() call.
unsafe impl Send for VMState {}
unsafe impl Sync for VMState {}
```

## Alternatives Considered

**Wrap registry_ptr in a newtype that is Send + Sync**: e.g. `struct OpaquePtr(*mut ())` with `unsafe impl Send/Sync`. Same result — the unsafety moves to the newtype. No practical benefit.

**Use AtomicPtr instead of raw pointer**: `AtomicPtr<()>` is `Send + Sync` without `unsafe impl`. Rejected — atomic operations add unnecessary overhead to what is a single-threaded access pattern; does not actually improve safety since we still need `unsafe` to dereference.

**Remove registry_ptr, pass registry as argument to execute()**: Clean solution but requires changing `InstructionFn` signature to include a registry argument — breaking change for all ISAs (see ADR-005).

## Consequences

- The safety invariant must be maintained by every future change that uses `registry_ptr` from a background thread
- If Platon ever supports multi-threaded execution (parallel ISA handler dispatch), this `unsafe impl` must be removed and replaced with a proper concurrent data structure
- The invariant is documented in both the source and in SECURITY.md

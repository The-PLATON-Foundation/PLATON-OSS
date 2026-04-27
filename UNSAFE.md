# Unsafe Code Inventory

*Author: Rafael Ruiz, CTO — The Platon Foundation*
*Last updated: 2026-03-22*

This document is the authoritative record of all `unsafe` code in the Platon kernel. Any PR that adds `unsafe` must update this document.

---

## Policy

1. Every `unsafe` block must have a `// SAFETY:` comment in source
2. Every `unsafe` block must be listed here with its invariant
3. `unsafe impl` requires the same documentation
4. PRs adding `unsafe` without this document update are automatically rejected

---

## Inventory

### 1. `platon/src/lib.rs` — `register_isa()`: Fat Pointer Reconstruction

```rust
// SAFETY: avap-isa stored Arc<dyn ISAProvider> as a fat pointer (data+vtable)
// and leaked it as (u64, u64) via transmute + Arc::into_raw.
// We reverse that here. Invariant: _get_arc_ptr() is called exactly once
// per register_isa() call. The Arc refcount has been incremented before
// leaking. Arc::from_raw takes ownership and will decrement on drop.
let provider: Arc<dyn ISAProvider> = unsafe {
    let raw = std::mem::transmute::<(u64, u64), *const dyn ISAProvider>(ptrs);
    Arc::from_raw(raw)
};
```

**Invariant:** `_get_arc_ptr()` on the ISA object must:
1. Clone the `Arc` (incrementing refcount)
2. Call `Arc::into_raw()` (leaking, refcount held)
3. Return the fat pointer via `transmute`

Violation: calling `_get_arc_ptr()` twice for the same `register_isa()` call would double the refcount, leaking the Arc.

---

### 2. `platon/src/lib.rs` — `execute()`: Registry Cleanup

```rust
// SAFETY: registry_ptr was set by Box::into_raw(Box::new(Py<PyDict>))
// earlier in this same execute() call. It is freed exactly once here.
// If execute() returns early via ?, the cleanup block still runs because
// it is unconditional at the end of the function.
if !self.state.registry_ptr.is_null() {
    unsafe {
        let _ = Box::from_raw(self.state.registry_ptr as *mut Py<PyDict>);
    }
    self.state.registry_ptr = std::ptr::null_mut();
}
```

**Invariant:** `registry_ptr` is set to `null_mut()` initially and only set to a non-null value by `Box::into_raw` in `execute()`. The cleanup block runs unconditionally at the end of `execute()`. No other code sets or reads `registry_ptr` between these two points.

**Known risk:** If `execute()` panics before reaching the cleanup block, the pointer is leaked. PyO3 catches Rust panics at the Python boundary, but the memory is not freed in that case. Mitigation: add `scopeguard` in a future version.

---

### 3. `platon-core/src/lib.rs` — `VMState`: unsafe impl Send + Sync

```rust
// Safety: VMState is only used single-threaded within a single execute() call.
// The Python GIL is held for the entire duration of execute().
// registry_ptr points to a Py<PyDict> that is valid while the GIL is held.
unsafe impl Send for VMState {}
unsafe impl Sync for VMState {}
```

**Invariant:** `VMState` is never shared between threads. A single `VM` instance handles one `execute()` call at a time (enforced by `&mut self` in `execute()`). The GIL is held by the calling thread for the duration.

**Future risk:** If Platon ever supports parallel ISA handler dispatch (e.g. for a vectorised ISA), this `unsafe impl` must be removed and `VMState` must be redesigned for concurrent access.

---

## Audit History

| Date | Auditor | Finding |
|---|---|---|
| 2026-03-22 | Rafael Ruiz | Initial inventory — 3 unsafe blocks, all documented |

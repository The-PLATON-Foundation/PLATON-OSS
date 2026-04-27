---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-004: Arc<dyn ISAProvider> Transfer via Fat Pointer (u64, u64)

## Context

`vm.register_isa(isa_obj)` receives a Python object and must extract an `Arc<dyn ISAProvider>` from it. PyO3 cannot directly represent Rust trait objects (`dyn Trait`) as Python types, because `dyn Trait` is a fat pointer (data pointer + vtable pointer) with no stable FFI representation.

## Decision

Use a `(u64, u64)` tuple as the transfer encoding for `Arc<dyn ISAProvider>`:

**ISA side** (`_get_arc_ptr` in Python-exposed ISA struct):
```rust
fn _get_arc_ptr(&self) -> (u64, u64) {
    let arc: Arc<dyn ISAProvider> = self.inner.clone(); // increments refcount
    let raw: *const dyn ISAProvider = Arc::into_raw(arc); // leaks, refcount held
    unsafe { std::mem::transmute::<*const dyn ISAProvider, (u64, u64)>(raw) }
}
```

**Kernel side** (`register_isa` in VM):
```rust
let ptrs = ptr_val.extract::<(u64, u64)>(py)?;
let provider: Arc<dyn ISAProvider> = unsafe {
    let raw = std::mem::transmute::<(u64, u64), *const dyn ISAProvider>(ptrs);
    Arc::from_raw(raw) // takes ownership, refcount already held
};
self.isa = Some(provider);
```

**Safety invariant**: `_get_arc_ptr()` must be called exactly once per `register_isa()` call. The Arc refcount is incremented before `into_raw` and decremented when `self.isa` is dropped. This is upheld by the single-call pattern in the Language Server.

## Alternatives Considered

**`Box<dyn Any>` + downcast**: Store the ISA as `Box<dyn Any>` in Python (PyO3 `#[pyclass]`), extract via `downcast_ref`. Rejected — `Box<dyn Any>` requires `'static` and makes cross-crate ISA registration impossible without a shared type registry.

**Raw `*const ()` (data pointer only)**: Pass only the data pointer, hardcode the vtable. Rejected — vtables are not stable across compilations; any change to `ISAProvider` would silently break without a compile error.

**Custom FFI struct**: Define a `#[repr(C)]` struct with function pointers matching `ISAProvider`. Rejected — reimplements Rust's vtable mechanism manually; maintenance burden; unsafe without the compiler's help.

**PyO3 `#[pyclass(unsendable)]`**: Wrap `Arc<dyn ISAProvider>` directly in a `#[pyclass]`. Rejected — PyO3 0.20 does not support `dyn Trait` as a `#[pyclass]` field without `unsendable`, which prevents the Arc from crossing thread boundaries.

## Consequences

- This is the most complex `unsafe` block in the codebase — requires careful documentation
- The fat pointer encoding is specific to Rust's current ABI; upgrading the Rust edition could theoretically change it (in practice, Rust fat pointer layout has been stable since 1.0)
- Must be revisited when upgrading PyO3 to 0.22 (`_get_arc_ptr` API may be replaceable with a safer abstraction)
- Any ISA that forgets to call `_get_arc_ptr` correctly leaks an Arc (refcount never decremented)

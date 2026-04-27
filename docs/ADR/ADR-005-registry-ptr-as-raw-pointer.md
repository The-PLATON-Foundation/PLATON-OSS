---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-005: NativeRegistry Exposed to ISA via Raw Pointer

## Context

ISA handlers for `CALL_EXT` need access to the `NativeRegistry` (a map of `func_id → Python callable`). However, `VMState` is defined in `platon-core`, which has no PyO3 dependency. `NativeRegistry` contains `PyObject` values and cannot be stored in `platon-core` without introducing a PyO3 dependency.

## Decision

Store the registry in `VMState` as a raw `*mut ()` pointer:

```rust
pub struct VMState {
    // ...
    pub registry_ptr: *mut (),  // opaque Python GIL pointer
}
```

Before `execute()`, `platon` builds a `Py<PyDict>` mapping `func_id (u32) → callable (PyObject)` from the `NativeRegistry` and stores it via `Box::into_raw`:

```rust
let func_dict = PyDict::new(py);
for (id, func) in r.functions.lock().unwrap().iter() {
    let _ = func_dict.set_item(*id, func.clone_ref(py));
}
let raw = Box::new(Py::from(func_dict));
self.state.registry_ptr = Box::into_raw(raw) as *mut ();
```

After `execute()` (including error paths), the pointer is freed:

```rust
if !self.state.registry_ptr.is_null() {
    unsafe { let _ = Box::from_raw(self.state.registry_ptr as *mut Py<PyDict>); }
    self.state.registry_ptr = std::ptr::null_mut();
}
```

ISA handlers access it as:

```rust
let dict = unsafe { &*(py_ctx as *const Py<PyDict>) };
```

## Alternatives Considered

**Generic VMState**: `VMState<R: Registry>` with a generic registry parameter. Rejected — makes all downstream types generic; PyO3 cannot expose generic structs to Python; significantly complicates the codebase.

**Separate registry argument to InstructionFn**: Add a `registry: *mut ()` parameter to the handler signature. Rejected — every handler must accept it even if unused; changes the `InstructionFn` type signature (breaking change for all ISAs).

**Lazy GIL acquisition inside handler**: Handler calls `Python::with_gil` and accesses `NativeRegistry` through a thread-local. Rejected — thread-locals complicate testing and are unreliable in async contexts.

**Store Arc<Mutex<PyObject>> in VMState**: Wrap the registry in a Rust-owned smart pointer. Rejected — `PyObject` is not `Send` without the GIL; `Arc<Mutex<PyObject>>` requires the GIL to be held to access, adding lock overhead to every `CALL_EXT`.

## Consequences

- `registry_ptr` is `*mut ()` — no type information at the pointer level
- The ISA must know the concrete type (`Py<PyDict>`) — this is documented in the ISA contract
- Memory safety depends on `platon` always freeing the pointer after `execute()`; a panic mid-execution would leak it (mitigated by PyO3 catching Rust panics at the boundary)
- Future: consider `scopeguard` to ensure cleanup even on panic

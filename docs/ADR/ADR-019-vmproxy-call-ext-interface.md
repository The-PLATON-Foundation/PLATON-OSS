---
date: 2026-04-26
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-019: _VmProxy as CALL_EXT Callback Interface

## Context

When a `CALL_EXT` instruction dispatches to a Python callable registered in `NativeRegistry`, that callable needs access to the current VM execution context — specifically:

1. **`globals`**: The ISA callback may need to read or write named variables (e.g., a function that modifies the program's working state)
2. **`registry`**: The callback may need to call other registered commands or access registry attributes set by the Language Server (e.g., `registry.task`, `registry.user`)

The callback is a plain Python function with a signature like `fn(vm, stack_args) -> None`. What should `vm` be?

## Decision

Define a `_VmProxy` PyO3 class that wraps a snapshot of `globals` and a reference to the `NativeRegistry`:

```rust
#[pyclass(name = "_VmProxy")]
pub struct VmProxy {
    #[pyo3(get, set)] pub globals:  PyObject,   // snapshot of VMState.globals as PyDict
    pub registry: NativeRegistry,
}
```

Python API for callbacks:

```python
def my_handler(vm, stack_args):
    task_info = vm.registry.task          # dynamic attr from registry
    value = vm.globals.get("myVar")       # read VM global
    vm.globals["result"] = Value(42)      # write VM global (see below)
    # call another registered function:
    other_id = vm.registry.get_id_by_name("otherCmd")
```

`_VmProxy.__setattr__` and `__getattr__` delegate to `registry.attrs` — allowing the Language Server to set attributes on the registry object and have them appear on the proxy.

## Alternatives Considered

**Pass `vm.globals` directly as a Python dict**: `fn(globals_dict, stack_args)`. Simpler but lossy — the callback cannot access `registry` attributes without a separate argument. Adds a second parameter for the registry, making the signature `fn(globals, registry, stack_args)` — less ergonomic and changes the contract every time new context is needed.

**Pass a plain Python dict with all context**: `fn({"globals": ..., "registry": ..., "task": ...}, stack_args)`. Flexible but untyped — no IDE autocompletion, no structured access.

**Wrap the entire `VM` object**: Pass the `VM` PyO3 object itself. Rejected — would allow callbacks to call `vm.execute()` recursively (re-entrant execution), which is not thread-safe and could violate the single-threaded invariant of `VMState`. `_VmProxy` is a read/write-limited view that prevents re-entrant execution.

**Async callback protocol**: Pass an async context and `await` callbacks. Rejected — PyO3 0.22 async support is experimental; requires an async runtime throughout the ISA; incompatible with the synchronous execution model.

## Consequences

- `_VmProxy` is the stable callback API — AVAP Language Server code is written against `vm.globals` and `vm.registry`
- `globals` is a `PyDict` **snapshot** taken before the callback is called — writes to `vm.globals` inside the callback are to the snapshot, not directly to `VMState.globals`. The ISA handler is responsible for syncing the snapshot back to `VMState` after the callback returns (documented in the CALL_EXT protocol in `docs/spec/vm-execution-model.md`)
- `_VmProxy` begins with underscore to signal it is not part of the public Python API — users should not instantiate it directly
- `registry.attrs` shared mutable state via `Arc<Mutex<HashMap<String, PyObject>>>` — thread-safe for the Language Server's multi-threaded registration, though callbacks run single-threaded under the GIL
- Any future context needed by callbacks (e.g., execution metadata, trace IDs) can be added to `_VmProxy` without changing the callback signature, since new attributes appear via `__getattr__`

---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-007: Dedicated Conector Namespaces in VMState

## Context

AVAP commands write results to two runtime namespaces: `conector.variables` (working variables) and `conector.results` (API response outputs). The Language Server needs to read these back after execution to build the API response.

The initial approach was to inject these namespaces as `Value::Dict` entries in `VMState.globals` (`__conector__`). However, `Value` has copy semantics — any `SET_ITEM` operation on a dict that was loaded from globals operates on a copy, not the original. The writes were silently lost.

## Decision

Add two dedicated `HashMap<String, Value>` fields to `VMState`:

```rust
pub struct VMState {
    // ... existing fields ...
    pub results:       HashMap<String, Value>,
    pub conector_vars: HashMap<String, Value>,
}
```

Expose them from the `VM` Python class after `execute()`:

```python
vm.conector_vars  # dict[str, Value]
vm.results        # dict[str, Value]
```

ISAs write to these fields directly. The avap-isa uses a marker-based routing scheme (see avap-isa ADR-006) to intercept `SET_ITEM` calls on the conector namespace and redirect them to `VMState.conector_vars` and `VMState.results`.

## Alternatives Considered

**Reference-counted Value::Dict**: Change `Value::Dict` to use `Arc<Mutex<Vec<(String, Value)>>>` for shared mutable access. Rejected — breaks `Send + Sync` (or requires `Arc<Mutex>` overhead on every access); fundamentally changes the value semantics contract and breaks all existing ISA match arms.

**Post-execution diff**: Compare globals before and after execution to detect changes. Rejected — cannot distinguish a variable named `__conector__` from an actual conector mutation; fragile.

**Add STORE_CVAR / STORE_RESULT opcodes to the kernel**: New opcodes that write directly to `conector_vars` and `results`. This is the correct long-term solution (see RFC-001). Deferred because it requires changes to `opcodes.json`, the compiler, and the ISA — more scope than this iteration.

**Return namespace dicts from execute()**: `vm.execute()` returns a tuple `(result, conector_vars, results)`. Rejected — changes the Python API significantly; callers that don't need namespaces are penalised.

## Consequences

- `VMState` grows two new fields — small memory overhead per execution (~56 bytes each for empty HashMaps)
- ISA implementations that use conector namespaces must write to `state.conector_vars` / `state.results` directly — cannot use the generic `SET_ITEM` handler
- The marker routing in avap-isa is a workaround for the absence of dedicated opcodes — documented as technical debt in ADR-006 of avap-isa
- This design is forward-compatible: when STORE_CVAR / STORE_RESULT opcodes are added, the fields remain unchanged, only the write path changes

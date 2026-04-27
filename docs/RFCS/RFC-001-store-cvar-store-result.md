# RFC-001: STORE_CVAR and STORE_RESULT Dedicated Opcodes

**Author:** Rafael Ruiz, CTO — The Platon Foundation
**Date:** 2026-03-22
**Status:** Draft
**Supersedes:** ADR-007 (Conector Namespaces) — partially

---

## Summary

Add two new opcodes to the AVAP ISA that write directly to `VMState.conector_vars` and `VMState.results`, eliminating the current marker-based routing scheme described in avap-isa ADR-006.

---

## Motivation

The current implementation uses string markers (`"__CONECTOR_VARS__"`, `"__CONECTOR_RESULTS__"`) pushed onto the stack by `LOAD_CONECTOR` + `GET_ATTR`. `SET_ITEM` detects these markers and routes writes to `VMState` directly. This is a workaround for `Value`'s copy semantics and has three problems:

1. **Fragile** — any variable named `__CONECTOR_VARS__` would be incorrectly intercepted
2. **Non-obvious** — the routing is invisible in the bytecode; a disassembler shows `SET_ITEM` with no indication it writes to VMState
3. **Wasteful** — `LOAD_CONECTOR` + `GET_ATTR "variables"` + `LOAD key` + `LOAD val` + `SET_ITEM` is 5 instructions for what should be 2

---

## Proposed Design

### New Opcodes

Add to `opcodes.json`:

```json
"STORE_CVAR": {
  "opcode": 42,
  "args": 1,
  "description": "Pop value, store in VMState.conector_vars[const[name_idx]]"
},
"STORE_RESULT": {
  "opcode": 43,
  "args": 1,
  "description": "Pop value, store in VMState.results[const[name_idx]]"
}
```

### Handler Implementation (avap-isa)

```rust
fn h_store_cvar(s: &mut VMState, c: &[u8], ip: &mut usize, _: *mut ()) -> Result<(), ISAError> {
    let idx  = read_u32(c, ip)? as usize;
    let name = s.get_const_str(idx).ok_or("STORE_CVAR: not a string")?;
    let val  = s.pop().ok_or("STORE_CVAR: empty stack")?;
    s.conector_vars.insert(name, val);
    Ok(())
}

fn h_store_result(s: &mut VMState, c: &[u8], ip: &mut usize, _: *mut ()) -> Result<(), ISAError> {
    let idx  = read_u32(c, ip)? as usize;
    let name = s.get_const_str(idx).ok_or("STORE_RESULT: not a string")?;
    let val  = s.pop().ok_or("STORE_RESULT: empty stack")?;
    s.results.insert(name, val);
    Ok(())
}
```

### Compiler Change (compiler.py)

```python
# Before (5 instructions):
# self.conector.variables[target] = value
# → LOAD_CONECTOR, GET_ATTR "variables", LOAD target, LOAD value, SET_ITEM

# After (2 instructions):
# → LOAD value, STORE_CVAR target
if node is Assign and target is conector.variables[key]:
    emit(LOAD, value)
    emit(STORE_CVAR, key)
```

---

## Migration Plan

1. Add opcodes to `opcodes.json` (new opcode bytes — no existing bytecode affected)
2. Add handlers to `avap-isa/src/lib.rs`
3. Update `compiler.py` to emit new opcodes
4. Recompile all 27 AVAP commands in Definition Server
5. Remove marker routing from `h_load_conector`, `h_get_attr`, `h_set_item`, `h_call_method`

The marker routing can remain as fallback for backwards compatibility until all bytecode is recompiled.

---

## Impact

| Component | Change |
|---|---|
| `platon-core` | None — `VMState.conector_vars` and `VMState.results` already exist |
| `avap-isa` | Add 2 handlers, remove marker routing (~40 lines net reduction) |
| `compiler.py` | Detect conector assignment patterns, emit new opcodes |
| `opcodes.json` | Add 2 entries |
| Existing bytecode | Unaffected (old opcodes still valid) |

---

## Open Questions

1. Should `STORE_CVAR` / `STORE_RESULT` accept a runtime key (from stack) or only a constant key? Constant is simpler and covers all current use cases.
2. Should `GET_CVAR` be added for reading conector variables? Currently `LOAD_CONECTOR` + `GET_ATTR` + `CALL_METHOD "get"` handles this — a `GET_CVAR` opcode would simplify it.

---
date: 2026-04-26
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-018: NativeRegistry Dispatch via u32 Function ID

## Context

The `CALL_EXT` opcode allows ISA handlers to call back into Python-registered functions during execution. This is Platon's extension point for host-provided operations (HTTP requests, database queries, AVAP built-in commands). The dispatch mechanism must:

1. Be callable from inside a Rust instruction handler (hot path)
2. Be fast: typical AVAP commands execute 5–20 `CALL_EXT` calls per script
3. Not require string interning or name-based lookups inside the execution loop
4. Be serializable in AVBC bytecode (the function identifier must fit in a `u32` argument)

## Decision

Use a numeric `u32` function ID as the stable identifier. Python callers register functions with both a name (for human readability) and an explicit `func_id` (for bytecode dispatch):

```python
registry = NativeRegistry()
registry.register_command(name="addVar", func_id=1, py_func=add_var_handler)
registry.register_command(name="getTask", func_id=2, py_func=get_task_handler)
```

`NativeRegistry` stores two maps:
- `functions: Arc<Mutex<HashMap<u32, PyObject>>>` — ID → callable
- `names_to_ids: Arc<Mutex<HashMap<String, u32>>>` — name → ID (for tooling)

Before `execute()`, `platon` converts `functions` into a `Py<PyDict>` (Python dict keyed by integer `u32`). This dict is stored as `VMState.registry_ptr` (see ADR-005). ISA handlers look up by integer key:

```python
# Inside the Python dict (registry_ptr), keyed by u32:
{1: add_var_handler, 2: get_task_handler}
```

`CALL_EXT` reads the `func_id` from bytecode (4-byte `u32`), looks it up in the dict, and calls the Python object.

## Alternatives Considered

**String name dispatch in bytecode**: Encode the function name as a constant pool string reference. The `CALL_EXT` opcode reads a const index, looks up the name string, then looks up the handler. Rejected — double indirection (const pool + name map) on every call; string comparison is slower than integer lookup; constant pool strings are a finite resource (256 entries in some ISAs).

**Python object reference directly in VMState**: Store `Arc<Mutex<HashMap<u32, Py<PyObject>>>>` in `VMState` directly. Rejected — `Py<PyObject>` is not `Send` without the GIL; `Arc<Mutex<...>>` would require acquiring a Rust mutex on every `CALL_EXT` call (in addition to the Python GIL already held). The pre-execution conversion to `Py<PyDict>` avoids this.

**Enum-based dispatch**: Define a Rust enum of all known external functions. Rejected — requires recompiling `platon-core` for every new function; breaks the extensibility principle.

**Function pointer table (u8 opcode-style)**: 256-entry array indexed by `u8`. Rejected — 256 slots insufficient for AVAP's function set (potentially hundreds of built-in commands); `u32` gives 4 billion possible IDs.

**Name interning with integer handles**: Intern string names at registration time, use intern IDs in bytecode. Effectively equivalent to `u32` func_id — chosen `u32` directly for simplicity and explicitness.

## Consequences

- `func_id` values must be agreed upon between the AVAP compiler and the Language Server at registration time — they are not auto-assigned. This is by design: bytecode files embedded `func_id` values, so the mapping must be stable across sessions
- `registry.get_id_by_name("addVar")` → `1` provides name→id lookup for tooling (disassembler, debugger) without affecting the dispatch path
- `Arc<Mutex<>>` wrapping in `NativeRegistry` allows the registry to be shared across Python threads (the Language Server may register commands from multiple threads). The mutex is held only during registration, not during dispatch
- Registration logs `[PLATON] Registered: {name} [ID: {id}]` to stdout — useful during startup, negligible overhead
- If a `func_id` is not in the registry when `CALL_EXT` executes, the ISA handler returns an `ISAError` — the error message identifies the missing ID

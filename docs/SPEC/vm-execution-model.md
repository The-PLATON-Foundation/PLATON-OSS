# VM Execution Model

*Author: Rafael Ruiz, CTO — The Platon Foundation*
*Version: 1.0 — 2026-03-22*

---

## Architecture

Platon is a **register-free stack machine**. All operations consume values from the stack and push results back. No named registers.

---

## Lifecycle

```
VM(timeout, max_instr)
  → register_isa(isa)       ISAProvider stored as Arc<dyn ISAProvider>
  → load(bytecode)          Header parsed, constants loaded, code copied
  → globals = {...}         Optional: inject initial state
  → execute(registry)       Main loop
  → globals / conector_vars / results   Read back after execute()
```

---

## Execution Loop

```
loop:
  if ip >= code.len() → break (fell off end)
  if elapsed > timeout → TimeoutError
  if count >= max_instr → VMError
  opcode = code[ip++]
  if isa.is_halt(opcode) → break
  handler = isa.get(opcode) or VMError("unknown opcode")
  handler(state, code, &mut ip, py_ctx) or VMError(err)
  count++

return stack.last() or Null
```

---

## VMState Fields

| Field | Type | Description |
|---|---|---|
| `stack` | `Vec<Value>` | Operand stack |
| `globals` | `HashMap<String,Value>` | Named variables |
| `constants` | `Vec<Value>` | Constant pool (read-only) |
| `try_stack` | `Vec<usize>` | Exception handler IPs |
| `results` | `HashMap<String,Value>` | ISA result output |
| `conector_vars` | `HashMap<String,Value>` | ISA conector namespace |
| `registry_ptr` | `*mut ()` | Opaque ptr to `Py<PyDict>` |
| `debug` | `bool` | Per-instruction trace |

---

## Exception Handling

```
PUSH_TRY handler_ip  → push handler_ip onto try_stack
  ... guarded code ...
POP_TRY              → pop (normal exit from try)
JMP end

handler_ip:          ← ISA error lands here when try_stack non-empty
  ... handle error (error msg on TOS as Str) ...
end:
```

When `handler(...)` returns `Err` and `try_stack` is non-empty:
1. Push error message as `Value::Str` onto stack
2. `ip = try_stack.pop()`
3. Continue execution

---

## CALL_EXT Protocol

```
1. Read func_id (u32) from bytecode
2. Lookup func_id in registry_ptr (Py<PyDict>)
3. Call Python: func(vm_proxy, py_stack)
4. Sync modified globals back to VMState
```

`py_ctx` is `*const Python<'_>`. Handlers use `Python::with_gil(|py| {...})`.

---

## Limits

| Limit | Default | API |
|---|---|---|
| Timeout | 5.0s | `VM(timeout=N)` |
| Max instructions | 100,000 | `VM(max_instr=N)` |
| Stack depth | OS limit | Future: `VM(max_stack=N)` |

---

## Thread Safety

A single `VM` instance is **not thread-safe**. Create one VM per execution. `VMState` is `Send + Sync` because the kernel guarantees single-threaded access within `execute()` (GIL held throughout).

---

## Debug Output

```python
vm.debug = True
# [PLATON-VM] execute() — ISA: AVAP-ISA v0.1.0, 230 bytes
# [PLATON-VM] 0x71 LOAD_TASK ip=0
# [PLATON-VM] HALT — 39 instr in 0.130ms
```

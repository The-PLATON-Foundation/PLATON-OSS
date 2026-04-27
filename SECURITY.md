# Security Policy

*Author: Rafael Ruiz, CTO — The Platon Foundation*

---

## Supported Versions

| Version | Supported |
|---|---|
| 0.3.x | ✅ |
| < 0.3 | ❌ |

---

## Reporting a Vulnerability

**Do not open a public GitHub issue.**

Email: **security@101obex.com**

Include:
- Description of the vulnerability
- Minimal reproduction steps
- Potential impact
- Your name/handle (optional, for acknowledgment)

Response within **72 hours**. Critical patches within **14 days**.

---

## Security Model

### Execution Isolation

The Platon VM provides **soft isolation**:

- Instruction handlers operate exclusively on `VMState`
- Handlers cannot access host filesystem, network, or OS directly
- Wall-clock timeout and instruction count limit prevent infinite loops and CPU exhaustion
- The `CALL_EXT` opcode calls back into Python — isolation quality depends on what the caller registers in `NativeRegistry`

Platon does **not** provide:
- Memory isolation between concurrent executions on the same VM instance
- Bytecode signature verification (caller responsibility)
- Capability-based sandboxing (planned for a future version)

### `unsafe` Inventory

All `unsafe` blocks are listed here. Any PR adding `unsafe` not on this list requires security review.

| Location | Reason | Invariant |
|---|---|---|
| `platon/src/lib.rs` `register_isa()` | Fat pointer transmute `(u64,u64)` → `*const dyn ISAProvider` | `avap-isa` leaks an `Arc<dyn ISAProvider>` with refcount incremented. Platon reconstructs via `Arc::from_raw`. Invariant: pointer is valid and was produced by `Arc::into_raw` in the same session. |
| `platon/src/lib.rs` `execute()` | `Box::from_raw(registry_ptr)` cleanup | `registry_ptr` was set by `Box::into_raw(Box::new(Py<PyDict>))` earlier in the same `execute()` call. Freed exactly once. |
| `platon-core/src/lib.rs` | `unsafe impl Send/Sync for VMState` | VMState is only accessed from a single thread within a single `execute()` call. The GIL is held for the duration. |

### Known Limitations

| Limitation | Impact | Mitigation |
|---|---|---|
| No stack depth limit | Malicious bytecode could overflow the Rust stack via deep recursion patterns | Deploy with OS stack size limits; add stack depth counter in a future version |
| Integer arithmetic is wrapping | `i64::MAX + 1` wraps to `i64::MIN` silently | Expected behavior for the current ISA; document in language spec |
| `registry_ptr` is a raw pointer | If `execute()` panics mid-run, the pointer is leaked | The Rust panic boundary is caught by PyO3; consider `scopeguard` in a future version |
| `registry_ptr` GIL access pattern | `py_ctx` passed as `*const Python<'_>` cast to `*mut ()` — ISA handlers call `Python::with_gil` instead of unsafe assume_gil_acquired | Revisit with PyO3 0.22 pyo3::marker::Python lifetime improvements |

---

## Acknowledgments

*(None yet — be the first)*

# ISA Contract

*Author: Rafael Ruiz, CTO — The Platon Foundation*
*Version: 1.0 — 2026-03-22*

---

## Overview

The Platon kernel defines no opcodes. All execution semantics are provided by an external ISA that implements the `ISAProvider` trait from `platon-core`.

---

## ISAProvider Trait

```rust
pub trait ISAProvider: Send + Sync {
    fn name(&self)            -> &str;
    fn version(&self)         -> (u8, u8, u8);
    fn instruction_set(&self) -> &InstructionSet;
}
```

| Method | Contract |
|---|---|
| `name()` | Human-readable ISA identifier (e.g. `"AVAP-ISA"`) |
| `version()` | Semantic version `(major, minor, patch)` |
| `instruction_set()` | Reference to the populated `InstructionSet` |

---

## InstructionFn

Every opcode handler must match this signature:

```rust
pub type InstructionFn = fn(
    state:  &mut VMState,   // mutable execution state
    code:   &[u8],          // full instruction stream (for reading args)
    ip:     &mut usize,     // current instruction pointer (advance past args)
    py_ctx: *mut (),        // opaque GIL pointer (cast to Python<'_> if needed)
) -> Result<(), ISAError>;
```

### Handler Responsibilities

- Read any arguments from `code` using `read_u32(code, ip)` (advances `ip`)
- Operate on `state.stack`, `state.globals`, `state.constants`, etc.
- Return `Ok(())` on success or `Err(ISAError)` on failure
- Never panic — return `Err` instead

### Python GIL Access

For handlers that need to call Python (e.g. `CALL_EXT`):

```rust
fn h_call_ext(s: &mut VMState, c: &[u8], ip: &mut usize, py_ctx: *mut ()) -> Result<(), ISAError> {
    Python::with_gil(|py| {
        // py is valid — GIL acquired
        // ... call Python callables ...
    });
    Ok(())
}
```

`py_ctx` is a hint that the GIL is held by the calling thread. `Python::with_gil` is safe to call even if the GIL is already held (it is a no-op in that case).

---

## InstructionSet Registration

```rust
let mut isa = InstructionSet::new(0xFF); // 0xFF = halt opcode

isa.register(
    InstructionMeta { opcode: 0x01, name: "PUSH", n_u32_args: 1 },
    h_push,
);
// ... register all opcodes ...
```

`n_u32_args` is metadata for disassemblers — the kernel does not use it to parse arguments.

---

## Python Exposure Protocol

To expose an ISA to Python via PyO3:

```rust
#[pymethods]
impl MyISAPy {
    fn _get_arc_ptr(&self) -> (u64, u64) {
        // Clone Arc (increments refcount), then leak as fat pointer
        let arc_clone: Arc<dyn ISAProvider> = self.inner.clone();
        let raw: *const dyn ISAProvider = Arc::into_raw(arc_clone);
        // Transmute fat pointer (data_ptr, vtable_ptr) to (u64, u64)
        unsafe { std::mem::transmute::<*const dyn ISAProvider, (u64, u64)>(raw) }
    }
}
```

The Platon kernel reconstructs the `Arc` in `vm.register_isa()`:

```rust
let ptrs = ptr_val.extract::<(u64, u64)>(py)?;
let provider: Arc<dyn ISAProvider> = unsafe {
    let raw = std::mem::transmute::<(u64, u64), *const dyn ISAProvider>(ptrs);
    Arc::from_raw(raw)
};
```

This is the only `unsafe` boundary in the ISA registration flow. The invariant is that `_get_arc_ptr()` is called exactly once per `register_isa()` call and the Arc refcount has been incremented before leaking.

---

## ISA Versioning

The `version()` method returns `(major, minor, patch)`. The AVBC header stores `major<<8 | minor` in the `isa_version` field.

The kernel does not enforce version compatibility — callers are responsible for checking that the loaded bytecode was compiled with a compatible ISA version.

---

## Reference Implementation

See [avap-isa](https://github.com/avapcloud/avap-isa) for the reference implementation (AVAP language ISA, 56 opcodes).

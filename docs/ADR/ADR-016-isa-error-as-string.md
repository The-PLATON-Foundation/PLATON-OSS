---
date: 2026-04-26
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-016: ISAError as String (Zero-Dependency Error Type)

## Context

Every instruction handler in `platon-core` returns `Result<(), ISAError>`. The error type must be defined in `platon-core`, which has a strict zero-dependency policy (no `std` extensions, no external crates). The type must be:

1. Producible from any ISA without importing types from `platon-core`'s dependencies
2. Convertible to a Python exception by `platon` without additional FFI plumbing
3. Lightweight — returned on every instruction dispatch even in the non-error path (as `Ok(())`)

## Decision

```rust
pub type ISAError = String;
```

`ISAError` is a type alias for `String`. Instruction handlers return error messages directly:

```rust
fn my_handler(state: &mut VMState, code: &[u8], ip: &mut usize, _py: *mut ())
    -> Result<(), ISAError>
{
    let idx = read_u32(code, ip)?;  // read_u32 returns Err(ISAError) on truncation
    let val = state.get_const(idx as usize)
        .ok_or_else(|| format!("PUSH: constant index {} out of range", idx))?;
    state.push(val.clone());
    Ok(())
}
```

In `platon/src/lib.rs`, the ISA error is converted to a Python `VMError` at the boundary:

```rust
(instr.handler)(&mut self.state, &self.code, &mut self.ip, py_ctx)
    .map_err(|e| VMError::new_err(e))?;
```

## Alternatives Considered

**Custom error enum** (`enum ISAError { OutOfBounds, UnknownOpcode, ... }`): Type-safe, allows pattern matching in the kernel. Rejected — the kernel never pattern-matches on ISAError variants; all errors are propagated directly to Python as `VMError`. A typed enum would require maintaining variant parity between platon-core and every ISA crate, with no benefit.

**`Box<dyn std::error::Error>`**: Ergonomic with the `?` operator and compatible with the ecosystem. Rejected — `Box<dyn Error>` is `!Send` without a `Send` bound, complicating the `ISAProvider: Send + Sync` requirement; also requires heap allocation on every error path.

**`anyhow::Error`**: The standard ecosystem error type with context chains. Rejected — introduces `anyhow` as a dependency into `platon-core`, violating the zero-dependency policy. The benefit (error context chaining) does not justify this for a type whose only consumer is `platon/src/lib.rs`.

**`thiserror`-derived enum**: Zero-overhead enum with `Display` impl via proc-macro. Rejected — same problem as custom enum, plus a proc-macro dependency.

**`&'static str`**: Zero heap allocation for static error messages. Rejected — dynamic error messages (e.g., "constant index 42 out of range") require formatting; `&'static str` cannot hold formatted data.

## Consequences

- ISA handlers produce error messages with `format!(...)` — ergonomic and zero-friction
- Error strings are heap-allocated only on the error path — `Ok(())` is a zero-cost return
- No structured error introspection: callers cannot distinguish "out of bounds" from "type error" without parsing the message string. This is acceptable because all ISAErrors propagate to Python as `VMError(message)` — Python callers only see the string
- If future tooling needs error codes (e.g., for telemetry), the `ISAError` type alias can be changed to a newtype wrapping `(u32, String)` — a single-file change in `platon-core` — without breaking ISA source compatibility (only recompilation required)

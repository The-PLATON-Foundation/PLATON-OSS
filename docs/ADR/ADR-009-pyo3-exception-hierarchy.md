---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-009: Custom Exception Hierarchy (VMError, TimeoutError)

## Context

The VM needs to signal different failure modes to Python callers: normal execution errors (unknown opcode, ISA handler failure, instruction limit) versus timeout. Callers need to distinguish between these to implement fallback logic and observability.

## Decision

Define two custom exceptions via `pyo3::create_exception!`:

```rust
pyo3::create_exception!(platon, VMError,      PyException, "Base VM error.");
pyo3::create_exception!(platon, TimeoutError, VMError,     "Execution timeout.");
```

`TimeoutError` is a subclass of `VMError`. Python callers can catch either:

```python
try:
    result = vm.execute()
except TimeoutError:
    log.warning("execution timed out")
except VMError as e:
    log.error(f"VM error: {e}")
```

All ISA handler errors propagate as `VMError`. The timeout check raises `TimeoutError`.

## Alternatives Considered

**Use built-in Python exceptions**: `RuntimeError`, `TimeoutError` (stdlib). Rejected — stdlib `TimeoutError` is a subclass of `OSError` which implies OS-level timeout; semantic mismatch. `RuntimeError` is too broad — callers cannot distinguish VM errors from other runtime errors.

**Single VMError with an error code attribute**: `VMError(code=TIMEOUT, message=...)`. Considered — cleaner for programmatic handling. Rejected for now — adds complexity; exception subclassing achieves the same with no extra code.

**Return error objects instead of raising**: `execute()` returns `(Value, Optional[VMError])`. Rejected — breaks Python's exception model; callers must check return type on every call.

## Consequences

- `from platon import VMError, TimeoutError` is part of the public API
- Any new failure mode (e.g. stack overflow) should be added as a new `VMError` subclass, not by repurposing `VMError`
- Both exceptions are exported from the `platon` module in `#[pymodule]`

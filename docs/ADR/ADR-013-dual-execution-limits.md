---
date: 2026-04-26
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-013: Dual Execution Safety Limits (Wall-Clock Timeout + Instruction Count)

## Context

Platon executes untrusted bytecode from AVAP programs. Without explicit limits, a buggy or adversarial program can cause two distinct denial-of-service patterns:

1. **CPU exhaustion**: an infinite loop in pure bytecode (`JMP 0` forever) — no I/O, no Python callbacks, runs at millions of instructions/second
2. **Wall-clock hang**: a loop that calls `CALL_EXT` repeatedly, each call blocking in Python (network I/O, sleep, mutex contention) — low instruction count, but unbounded real time

A single limit catches only one class of attack. Both are required for robust isolation.

## Decision

Expose two independent, configurable limits on the `VM` constructor:

```python
vm = VM(timeout=5.0, max_instr=100_000)
```

**`timeout: f64`** (default 5.0 seconds): Wall-clock limit checked at the top of every iteration of the execution loop using `std::time::Instant::elapsed()`. Raises `TimeoutError` (subclass of `VMError`) when exceeded.

**`max_instr: u64`** (default 100,000): Instruction counter incremented after each successful handler call. Raises `VMError("Instruction limit exceeded")` when reached.

```rust
let start = Instant::now();
let mut count = 0u64;
loop {
    if start.elapsed().as_secs_f64() > self.timeout {
        return Err(TimeoutError::new_err("VM Timeout"));
    }
    if count >= self.max_instr {
        return Err(VMError::new_err("Instruction limit exceeded"));
    }
    // ... dispatch ...
    count += 1;
}
```

Both limits are checked before executing each instruction — no instruction can partially execute past either limit.

## Alternatives Considered

**Timeout only**: Catches wall-clock hangs but not CPU-bound loops at high instruction rates. A 5-second timeout allows ~7.5M instructions at 1.5M instr/s — acceptable in isolation but provides no defense against sustained CPU saturation across many concurrent VMs.

**Instruction count only**: Catches CPU-bound loops but not I/O-bound hangs via `CALL_EXT`. A Python callback that sleeps for 60 seconds would pass the instruction count limit easily (only 1 instruction per call).

**OS-level preemption (SIGALRM / threads)**: Set an OS timer or run execution in a separate thread with a join timeout. Rejected — SIGALRM is not safe to use from a Python extension module (PyO3 does not support signal handling across the FFI); threading adds synchronization complexity and makes `VMState` genuinely multi-threaded (currently `unsafe impl Send/Sync` relies on single-threaded access).

**Async cancellation**: Use Rust async (tokio) with `tokio::time::timeout`. Rejected — requires an async runtime throughout the stack; PyO3 integration with async runtimes adds significant complexity; the blocking execution model is simpler and sufficient.

**Fuel-based model**: Each instruction consumes "fuel"; the VM stops when fuel runs out. Effectively equivalent to `max_instr`. Naming retained as implementation-agnostic phrasing in ROADMAP.

## Consequences

- `TimeoutError` and `VMError("Instruction limit exceeded")` are distinct — callers can react differently (retry vs. hard-fail)
- `Instant::elapsed()` is called once per instruction — overhead is ~1ns per call on modern hardware, negligible against typical handler work (~100–500ns)
- The defaults (5s, 100K) are conservative for AVAP command execution (typically 30–60 instructions, < 1ms). Production callers should tune based on their SLO
- Neither limit prevents memory exhaustion from a stack-growing loop — a `Value::List` pushed on every iteration will grow until OOM. This is a known gap (no `max_stack` or `max_memory` limit yet — see ROADMAP 0.4.0)
- Callers can set `max_instr=u64::MAX` or `timeout=f64::INFINITY` to disable either limit individually — no forced minimum is imposed

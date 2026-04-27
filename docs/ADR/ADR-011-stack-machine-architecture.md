---
date: 2026-04-26
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-011: Stack Machine Architecture (Register-Free)

## Context

The Platon VM requires a fundamental execution model — a decision that shapes every ISA opcode, the bytecode encoding, the VMState layout, and the ergonomics of writing instruction handlers. The two canonical choices are:

- **Stack machine**: operands are pushed/popped from an implicit stack; instructions operate on the top-of-stack
- **Register machine**: instructions name explicit source/destination registers; operands are addressed by register number

This decision was made implicitly when designing `VMState` and `InstructionFn`, but its consequences are pervasive and warrant explicit documentation.

## Decision

Platon is a **stack machine**. All execution state lives in `VMState.stack: Vec<Value>`. Instruction handlers:

1. Pop their operands from the stack
2. Compute a result
3. Push the result back

No named registers exist. The instruction pointer (`VM.ip: usize`) and the stack are the only implicit state beyond globals and constants.

```rust
// Typical ISA handler pattern (e.g. ADD):
let b = state.pop().unwrap_or(Value::Null);
let a = state.pop().unwrap_or(Value::Null);
let result = /* a + b */;
state.push(result);
```

## Alternatives Considered

**Register machine (like Lua 5.0+, LLVM IR)**: Instructions encode source/destination register numbers explicitly. Rejected — requires a fixed-size register file in `VMState` (or dynamic allocation), adds register-address bytes to every instruction, and complicates ISA handler signatures (`handler(state, code, ip, py_ctx)` would need a register window). The benefit (fewer instructions for complex operations) is marginal for the short AVAP command sequences Platon executes.

**Accumulator machine (one implicit accumulator register)**: Single register, all instructions operate on it. Rejected — doesn't generalize to AVAP's need for multi-value stack manipulation (e.g., building a dict from N key-value pairs without explicit temporaries).

**Hybrid (stack + local variable slots)**: Stack for expressions, fixed-size slots for locals (like CPython). Considered but deferred — `VMState.globals` already serves as the named-variable namespace, making a separate local-variable array redundant for the current AVAP command model.

**Direct-threaded dispatch**: Encode handler function pointers directly in the bytecode. Rejected — breaks the ISA plugin model (handler addresses differ per process invocation); incompatible with serialized bytecode files.

## Consequences

- Bytecode is compact: most instructions need only an opcode byte (no register addresses)
- ISA handlers are simple: `pop`, compute, `push` is the universal pattern
- AVAP's 56 opcodes map naturally: all expression evaluation uses the stack; named state goes in globals
- No fixed stack frame size: `VMState.stack` grows dynamically (known limitation: no stack depth limit — see SECURITY.md)
- Deep expression trees require proportionally deep stack usage — not a concern for current AVAP command workloads
- Adding a register model in the future would be a breaking change to `VMState` and all ISA handlers

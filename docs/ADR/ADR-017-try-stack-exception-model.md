---
date: 2026-04-26
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-017: try_stack Cooperative Exception Model

## Context

AVAP programs need structured exception handling — the ability to guard a block of bytecode and recover from errors without propagating them to the Python caller as a `VMError`. The kernel must provide a mechanism for ISAs to implement this, while keeping the kernel itself opcode-agnostic.

The challenge: ISA handlers return `Result<(), ISAError>`. Without any kernel-level support, any `Err` immediately terminates execution and raises a Python `VMError`. There is no way for an ISA to mark a region of bytecode as "guarded" without the kernel knowing about it.

## Decision

Add `try_stack: Vec<usize>` to `VMState`. This is a stack of bytecode instruction-pointer offsets — each entry is the IP of an error handler for the current try-guarded region.

**Kernel behaviour** (in the execution loop):

```rust
(instr.handler)(&mut self.state, &self.code, &mut self.ip, py_ctx)
    .map_err(|e| {
        if let Some(handler_ip) = self.state.try_stack.pop() {
            // Error caught: push error message, jump to handler
            self.state.push(Value::Str(e));
            self.ip = handler_ip;
            None  // continue execution
        } else {
            Some(VMError::new_err(e))  // propagate to Python
        }
    })?;
```

**ISA bytecode pattern** (AVAP assembly pseudo-code):

```
PUSH_TRY  handler_ip   ; push handler_ip onto try_stack
  ... guarded code ...
POP_TRY               ; pop try_stack (normal exit from try block)
JMP       end_ip

handler_ip:            ; execution resumes here on error
  ... error msg is Value::Str on top of stack ...

end_ip:
```

The `PUSH_TRY` and `POP_TRY` opcodes are defined by the ISA (not the kernel). The kernel only inspects `try_stack` when an instruction handler returns `Err`.

## Alternatives Considered

**Python-side exception handling only**: Callers wrap `vm.execute()` in a Python `try/except VMError`. Rejected — this terminates the entire execution; there is no way to recover and continue executing subsequent bytecode instructions. AVAP's `try/catch` blocks need to catch errors mid-program and continue.

**Kernel-defined PUSH_TRY/POP_TRY opcodes**: The kernel handles these directly, not via ISA handlers. Rejected — breaks the language-agnostic kernel principle. Different ISAs may encode handler IPs differently (absolute vs. relative) or may use different exception models entirely.

**Setjmp/longjmp analogy via Rust panics**: Use `std::panic::catch_unwind` to catch ISA panics. Rejected — panics are not a designed control flow mechanism in Rust; catching them reliably from PyO3 is undefined behaviour; performance overhead of `catch_unwind` per instruction is unacceptable.

**Error channel**: A separate `VMState.last_error: Option<ISAError>` field; ISA handlers return `Ok(())` and set the error field. Rejected — requires every ISA handler to check the error field after every instruction call; equivalent in complexity but less idiomatic than Rust's `Result`.

**Two-phase dispatch**: After each instruction, check if a flag is set. If set, route to a handler. Rejected — equivalent to `try_stack.pop()` but without the explicit stack structure needed for nested try blocks.

## Consequences

- `try_stack` supports arbitrarily nested try/catch blocks (each `PUSH_TRY` pushes a new entry; `POP_TRY` pops one)
- The kernel adds O(1) overhead per instruction (`try_stack.last()` check in the error path — not the hot path)
- On normal execution (no errors), `try_stack` is empty and the error check branch is not taken
- ISA authors must pair every `PUSH_TRY` with either a `POP_TRY` (normal path) or rely on the handler popping (error path). An unbalanced try_stack after execution is a bug in the ISA — the kernel does not validate this
- The error message is pushed as `Value::Str` onto the stack — handler bytecode accesses it via a `POP` or `LOAD_TOS` opcode
- This model is forward-compatible: future kernel versions can inspect the try_stack depth without changing the ISA protocol

---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-010: Debug Trace via vm.debug Flag (stdout)

## Context

During development and production debugging, engineers need to trace instruction execution: which opcodes ran, at what IP, how many instructions, and how long it took. This needs to be toggleable — always-on tracing is too expensive in production.

## Decision

Add a `debug: bool` field to `VMState`, toggled via the `vm.debug` Python property. When `True`, the execution loop prints to stdout:

```
[PLATON-VM] execute() — ISA: AVAP-ISA v0.1.0, 230 bytes
[PLATON-VM] 0x71 LOAD_TASK ip=0
[PLATON-VM] 0x01 PUSH ip=1
...
[PLATON-VM] HALT — 39 instr in 0.130ms
```

Format: `[PLATON-VM] 0xNN OPCODE_NAME ip=NNN`

## Alternatives Considered

**Python logging module**: Emit via `log::info!` and a Python logging bridge. Rejected — adds a logging crate dependency and PyO3 interop complexity; stdout is sufficient for development debugging.

**Structured JSON trace**: Emit each instruction as a JSON object. Considered for future production use. Deferred — the debug flag is for development; production observability needs a separate design (metrics, tracing spans).

**Callback-based trace**: `vm.on_instruction = lambda opcode, ip: ...`. More flexible. Rejected for now — significant overhead (Python callback per instruction); breaks the execution loop's hot path.

## Consequences

- Debug output goes to stdout — in Docker this appears in container logs
- Performance overhead when `debug=True`: ~1 `println!` per instruction (~1μs each)
- The format string is not stable — tooling should not parse it
- `debug=False` (default) has zero overhead — the `if self.state.debug` branch is predicted-not-taken

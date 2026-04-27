# Roadmap

*Author: Rafael Ruiz, CTO — The Platon Foundation*
*Last updated: 2026-03-22*

---

## Version 0.4.0 — Production Readiness

**Target:** Q2 2026

| Item | Priority | ADR/RFC |
|---|---|---|
| PyO3 upgrade to 0.22 | P0 | — |
| Pre-built wheels in CI (eliminate build-at-startup) | P0 | — |
| Formal test suite: unit + integration + regression | P0 | — |
| `STORE_CVAR` / `STORE_RESULT` dedicated opcodes | P1 | RFC-001 |
| Stack depth limit (`VM(max_stack=N)`) | P1 | — |
| `scopeguard` for `registry_ptr` cleanup on panic | P1 | — |

---

## Version 0.5.0 — Observability & Tooling

**Target:** Q3 2026

| Item | Priority |
|---|---|
| Structured execution trace (JSON, not println) | P1 |
| `platon disasm <file>` — AVBC disassembler CLI | P1 |
| Execution metrics: instruction count, time, stack depth | P1 |
| `Value::Bytes` variant for binary data | P2 |
| `GET_CVAR` opcode for reading conector variables | P2 |

---

## Version 1.0.0 — Stable API

**Target:** Q4 2026

| Item | Priority |
|---|---|
| Stable Python API (semver guarantee) | P0 |
| Stable `ISAProvider` trait (semver guarantee) | P0 |
| Stable AVBC bytecode format v1.0 | P0 |
| ISA version negotiation in `vm.load()` | P1 |
| Second ISA implementation (proof of language-agnosticism) | P1 |
| Formal AVBC specification document (RFC) | P1 |

---

## Version 2.0.0 — Multi-language & Concurrency

**Target:** 2027

| Item | Notes |
|---|---|
| Parallel ISA handler dispatch | Requires redesigning VMState for concurrent access |
| WASM target | `platon-core` as `wasm32-unknown-unknown` |
| Capability-based sandboxing | Replace NativeRegistry with a permission system |
| JIT compilation hooks | Optional JIT backend via `ISAProvider::jit_compile()` |

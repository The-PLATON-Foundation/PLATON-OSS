# Architecture Decision Records — Platon

*Author: Rafael Ruiz, CTO — The Platon Foundation*

---

## Index

| ID | Title | Status | Date |
|---|---|---|---|
| [ADR-001](ADR-001-two-crate-workspace.md) | Two-Crate Cargo Workspace (platon-core + platon) | Accepted | 2026-03-22 |
| [ADR-002](ADR-002-value-as-enum.md) | Value as a Tagged-Union Enum with Copy Semantics | Accepted | 2026-03-22 |
| [ADR-003](ADR-003-isa-provider-trait.md) | ISAProvider as a Runtime Trait Object | Accepted | 2026-03-22 |
| [ADR-004](ADR-004-fat-pointer-protocol.md) | Arc<dyn ISAProvider> Transfer via Fat Pointer | Accepted | 2026-03-22 |
| [ADR-005](ADR-005-registry-ptr-as-raw-pointer.md) | NativeRegistry Exposed to ISA via Raw Pointer | Accepted | 2026-03-22 |
| [ADR-006](ADR-006-vmstate-send-sync.md) | VMState Implements Send + Sync via unsafe impl | Accepted | 2026-03-22 |
| [ADR-007](ADR-007-conector-namespaces.md) | Dedicated Conector Namespaces in VMState | Accepted | 2026-03-22 |
| [ADR-008](ADR-008-bytecode-loader.md) | Bytecode Loaded and Parsed in vm.load() | Accepted | 2026-03-22 |
| [ADR-009](ADR-009-pyo3-exception-hierarchy.md) | Custom Exception Hierarchy (VMError, TimeoutError) | Accepted | 2026-03-22 |
| [ADR-010](ADR-010-debug-trace.md) | Debug Trace via vm.debug Flag | Accepted | 2026-03-22 |
| [ADR-011](ADR-011-stack-machine-architecture.md) | Stack Machine Architecture (Register-Free) | Accepted | 2026-04-26 |
| [ADR-012](ADR-012-avbc-binary-format.md) | AVBC Binary Container Format Design | Accepted | 2026-04-26 |
| [ADR-013](ADR-013-dual-execution-limits.md) | Dual Execution Safety Limits (timeout + max_instr) | Accepted | 2026-04-26 |
| [ADR-014](ADR-014-release-profile-optimization.md) | Aggressive Cargo Release Profile (LTO + codegen-units=1) | Accepted | 2026-04-26 |
| [ADR-015](ADR-015-maturin-build-backend.md) | Maturin as Python Extension Build Backend | Accepted | 2026-04-26 |
| [ADR-016](ADR-016-isa-error-as-string.md) | ISAError as String (Zero-Dependency Error Type) | Accepted | 2026-04-26 |
| [ADR-017](ADR-017-try-stack-exception-model.md) | try_stack Cooperative Exception Model | Accepted | 2026-04-26 |
| [ADR-018](ADR-018-native-registry-u32-dispatch.md) | NativeRegistry Dispatch via u32 Function ID | Accepted | 2026-04-26 |
| [ADR-019](ADR-019-vmproxy-call-ext-interface.md) | _VmProxy as CALL_EXT Callback Interface | Accepted | 2026-04-26 |

---

## Format

Each ADR contains:
- **Context** — what situation led to the decision
- **Decision** — what was decided and how it is implemented
- **Alternatives Considered** — what was evaluated and rejected, and why
- **Consequences** — what changes as a result, including negative consequences

## Process

1. Before making a significant architectural change, create a new ADR file: `ADR-NNN-short-title.md`
2. Add it to this index
3. Open a Pull Request — the ADR is discussed and approved in review
4. Once merged, an ADR is **immutable** — its status changes from `Proposed` → `Accepted` → `Superseded` (never deleted)
5. If a decision is reversed, create a new ADR that supersedes the old one

## Status Values

| Status | Meaning |
|---|---|
| `Proposed` | Under discussion, not yet approved |
| `Accepted` | Approved and implemented |
| `Superseded` | Replaced by a newer ADR (link to successor) |
| `Deprecated` | No longer relevant but kept for history |
| `Rejected` | Considered but not adopted |

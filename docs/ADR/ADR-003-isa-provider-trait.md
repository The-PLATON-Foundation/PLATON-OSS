---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-003: ISAProvider as a Runtime Trait Object

## Context

The original Platon design had all opcodes compiled directly into the VM kernel. This made the VM tied to a specific language (AVAP). The kernel needs to support multiple languages without being recompiled.

## Decision

Define `ISAProvider` as a Rust trait in `platon-core`:

```rust
pub trait ISAProvider: Send + Sync {
    fn name(&self)            -> &str;
    fn version(&self)         -> (u8, u8, u8);
    fn instruction_set(&self) -> &InstructionSet;
}
```

ISAs are stored and dispatched via `Arc<dyn ISAProvider>`. Registration happens at runtime via `vm.register_isa(isa_obj)`. The kernel dispatch loop calls `isa.instruction_set().get(opcode)?.handler(...)` for every instruction.

`InstructionSet` is a `HashMap<u8, Instruction>` built at ISA construction time. The dispatch lookup is O(1).

## Alternatives Considered

**Static dispatch (generics)**: `VM<I: ISAProvider>` — monomorphisation, no virtual dispatch overhead. Rejected — makes the Python-facing `VM` type non-generic (PyO3 cannot expose generic structs). Would require separate Python classes per ISA.

**C-style function pointer table**: Raw `fn` pointers in a 256-element array indexed by opcode. Rejected — not type-safe; requires `unsafe` for every dispatch; harder to document and test per-opcode.

**Dynamic loading (dlopen)**: ISA as a `.so` file loaded at runtime. Rejected — adds significant complexity, platform-specific behaviour, and breaks Rust's memory safety guarantees across the FFI boundary.

**Enum-based dispatch**: `enum ISA { Avap(AvapISA), Other(OtherISA) }`. Rejected — requires recompiling the kernel to add any new ISA; defeats the extensibility goal.

## Consequences

- Every ISA must implement `Send + Sync` — restricts what ISAs can hold (no `Rc<RefCell>`)
- `dyn ISAProvider` has vtable overhead (~1ns per dispatch) — negligible compared to instruction handler work
- Adding a method to `ISAProvider` is a breaking change for all ISA implementations — requires a new ADR and version bump
- The fat pointer transfer protocol (see ADR-004) is the direct consequence of this design

---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-002: Value as a Tagged-Union Enum with Copy Semantics

## Context

The VM needs a single type to represent all runtime values: integers, floats, strings, collections, and null. The design of this type has significant implications for performance, memory safety, and ISA ergonomics.

## Decision

Define `Value` as a Rust `enum` with value semantics (derives `Clone`, no `Rc`/`Arc`):

```rust
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Dict(Vec<(String, Value)>),
    Iter(Vec<Value>, usize),
}
```

All operations on `Value` use explicit `.clone()` — there are no implicit shared references.

`Dict` is implemented as `Vec<(String, Value)>` (not `HashMap`) to preserve insertion order and avoid the overhead of hashing for the small dict sizes typical in AVAP command execution (< 20 keys).

## Alternatives Considered

**Reference-counted values (`Rc<RefCell<...>>` or `Arc<Mutex<...>>`)**: Would enable reference semantics — mutations to a dict via `SET_ITEM` would be visible to all holders. Rejected — breaks `Send + Sync`, required for `Arc<dyn ISAProvider>`. Also significantly complicates the borrow checker interactions in instruction handlers.

**Boxed heap allocation for all variants**: `Box<dyn ValueTrait>`. Rejected — pointer indirection on every stack operation; worse cache performance; no `PartialEq` without additional trait bounds.

**HashMap for Dict**: Better lookup performance. Rejected — breaks insertion order, which matters for JSON serialization in AVAP commands. Vec<(String, Value)> is measurably faster for < 30 keys due to cache locality.

**Separate stack type for primitives**: Store `Int`, `Float`, `Bool` inline on the stack without heap allocation, use `Box<Value>` for heap types. Rejected — premature optimisation; adds complexity to instruction handlers; value semantics already avoids heap allocation for primitives.

## Consequences

- `SET_ITEM` on a `Dict` that was cloned from globals modifies the copy, not the original — this is the root cause of the conector mutation problem (see ADR-006 in avap-isa)
- All value moves are explicit — instruction handlers have clear ownership semantics
- No garbage collector required — values are freed when they drop off the stack
- Adding a new variant (`Value::Bytes`, `Value::Tuple`) is a breaking change for all ISA match arms
- `Dict` lookup is O(n) — acceptable for current use cases, documented as a known limitation

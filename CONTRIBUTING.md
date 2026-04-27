# Contributing to Platon

---

## Code of Conduct

This project follows the [Contributor Covenant](https://www.contributor-covenant.org/) v2.1. Report violations to security@101obex.com.

---

## Ways to Contribute

- **Bug reports** — GitHub Issue with `bug` label and minimal reproduction
- **Performance improvements** — benchmarks required (before/after)
- **New `Value` variants** — requires RFC discussion first (breaking change)
- **ISA contract changes** — requires ADR (breaking change for all ISA implementors)
- **Documentation** — always welcome

---

## Development Setup

```bash
# Clone
git clone https://github.com/avapcloud/platon
cd platon

# Build
maturin develop

# Verify
python3 -c "from platon import VM, Value; print('OK')"

# Lint
cargo clippy -- -D warnings
cargo fmt --check
```

---

## Project Structure

```
platon/
├── src/lib.rs        PyO3 bindings — VM, NativeRegistry, Value, exceptions
└── platon-core/
    └── src/lib.rs    Pure Rust — Value, VMState, ISAProvider, InstructionSet
```

**Rule**: `platon-core` must never depend on PyO3. If you need Python types in a new feature, they belong in `platon/src/lib.rs`.

---

## Key Invariants

These must hold after every change:

1. `platon-core` has zero non-std dependencies
2. `unsafe` blocks must have a `// SAFETY:` comment explaining the invariant
3. `VMState` is `Send + Sync` — justify any change that affects this
4. `Value` is `Clone` — new variants must implement Clone
5. `ISAProvider` is `Send + Sync` — required for Arc<dyn ISAProvider>
6. The AVBC bytecode format is forwards-compatible — new header fields go in the reserved region

---

## Adding a New Value Variant

`Value` is the core data type. Adding a variant is a **breaking change** for every ISA implementation — they must handle the new variant in their `match` arms.

Process:
1. Open a GitHub Discussion proposing the new variant
2. Get maintainer approval
3. Update `Value` in `platon-core/src/lib.rs`
4. Update all `match` arms in `platon-core` (type_name, is_truthy, eq_val, get_item, set_item, contains, to_string_repr)
5. Update `core_from_py` and `core_to_py` in `platon/src/lib.rs`
6. Bump minor version
7. Notify downstream ISA maintainers

---

## Changing the ISAProvider Contract

Any change to the `ISAProvider` trait or `InstructionFn` signature is a breaking change. Requires:
1. ADR document
2. Coordination with `avap-isa` maintainers
3. Major version bump

---

## Pull Request Requirements

- `cargo test` passes
- `cargo clippy -- -D warnings` passes
- `cargo fmt` applied
- `CHANGELOG.md` updated under `[Unreleased]`
- New `unsafe` blocks have `// SAFETY:` comments
- Breaking changes have an ADR

---

## Commit Messages

[Conventional Commits](https://www.conventionalcommits.org/):

```
feat(core): add Value::Bytes variant
fix(vm): prevent double-free of registry_ptr on execute() error
perf(vm): avoid Vec allocation in execute() hot loop
docs(adr): add ADR-010 for Value::Bytes
chore(deps): update pyo3 to 0.22
```

---

## Questions

Open a GitHub Discussion or contact dev@101obex.com.

---

*Author: Rafael Ruiz, CTO — The Platon Foundation*

# Changelog

All notable changes to Platon are documented here.

This project follows [Semantic Versioning](https://semver.org/) and [Keep a Changelog](https://keepachangelog.com/en/1.0.0/).

---

## [Unreleased]

### Planned
- PyO3 upgrade to 0.22
- `STORE_CVAR` / `STORE_RESULT` dedicated opcodes (replace marker routing)
- Pre-built wheels in CI (eliminate build-at-startup)
- Formal test suite: unit, integration, regression

---

## [0.3.0] — 2026-03-22

### Added
- `VMState.conector_vars` — dedicated namespace for ISA conector variable writes
- `VMState.results` — dedicated namespace for ISA result writes
- `vm.conector_vars` Python getter — exposes conector_vars after execute()
- `vm.results` Python getter — exposes results after execute()
- `vm.register_isa()` — runtime ISA registration via `ISAProvider` trait
- Fat pointer protocol: `(u64, u64)` encoding for `Arc<dyn ISAProvider>` transfer across PyO3 boundary
- `core_from_py`: dict-before-list check to prevent Python dicts being extracted as key-only lists

### Changed
- `VMState` now has two additional fields: `results` and `conector_vars`
- `vm.execute()` builds `Py<PyDict>` from `NativeRegistry` for `CALL_EXT` dispatch (instead of storing raw `NativeRegistry` pointer)
- Registry cleanup now frees `Py<PyDict>` (not `NativeRegistry`) after execute()

### Fixed
- Python `dict` objects being incorrectly converted to `Value::List` (keys only) in `core_from_py`

---

## [0.2.0] — 2026-03-22

### Added
- Three-crate workspace: `platon-core` (rlib) + `platon` (cdylib)
- `ISAProvider` trait in `platon-core`
- `InstructionSet` — opcode → handler dispatch table
- `NativeRegistry` — Python callable registry for `CALL_EXT`
- `VmProxy` — proxy object passed to native callbacks
- `VMError` / `TimeoutError` Python exceptions
- Configurable timeout (seconds) and max instruction count

### Changed
- ISA removed from kernel — all opcodes provided externally via `ISAProvider`

---

## [0.1.0] — 2026-03-21

### Added
- Initial Rust VM kernel with hardcoded AVAP ISA
- PyO3 Python bindings
- `Value` enum: Null, Bool, Int, Float, Str, List, Dict, Iter
- `VMState`: stack, globals, constants, try_stack
- AVBC bytecode loader (128-byte header + constant pool + code)
- Basic execution loop with timeout and instruction limit

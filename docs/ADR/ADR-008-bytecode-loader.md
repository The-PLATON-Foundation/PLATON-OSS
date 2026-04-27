---
date: 2026-03-22
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-008: Bytecode Loaded and Parsed in vm.load(), Not execute()

## Context

AVBC bytecode must be parsed before execution: the header must be validated, the constant pool decoded, and the instruction stream extracted. This can happen either in `load()` (eager) or in `execute()` (lazy).

## Decision

Parse and validate fully in `vm.load(bytecode: &PyBytes)`:

1. Validate magic bytes (`b"AVBC"`)
2. Read header fields: code_size, const_count, entry_point
3. Decode the constant pool into `VMState.constants` (Vec<Value>)
4. Copy the instruction stream to `VM.code` (Vec<u8>)
5. Set `VM.ip = entry_point`

`execute()` begins immediately at `VM.ip` with no parsing work.

## Alternatives Considered

**Lazy parsing in execute()**: Defer constant pool decoding to the first access. Rejected — complicates the execution loop with per-constant error handling; errors are easier to diagnose at load time.

**Memory-mapped bytecode**: Parse directly from a memory-mapped file without copying. Rejected — adds platform-specific complexity; the bytecode files are small (< 5KB typical) and a Vec<u8> copy is negligible.

**Streaming parse**: Parse the constant pool on demand during execution. Rejected — requires bounds checking on every `PUSH const_idx`; simpler and safer to pre-decode.

**Store raw bytecode, decode on execute()**: Keep `VM.code` as the raw bytes including header and constant pool. Rejected — the constant pool would need to be re-decoded on every `execute()` call if the VM is reused.

## Consequences

- `vm.load()` is a fallible operation — callers must handle `PyValueError`
- The constant pool is eagerly heap-allocated on load — acceptable for typical bytecode sizes
- `vm.load()` and `vm.execute()` can be called independently — allows pre-loading bytecode and executing it multiple times with different globals (useful for benchmarking)
- Constants are indexed by `u32` in the instruction stream — index bounds checking uses `constants.get(idx)` which returns `None` safely

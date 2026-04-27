---
date: 2026-04-26
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-012: AVBC Binary Container Format Design

## Context

Platon executes compiled programs stored in a bytecode container. The container format defines the contract between any AVAP compiler and the VM kernel. Key requirements:

1. Parseable without external dependencies (~50 lines of Rust)
2. Self-describing: the container tells the parser everything it needs to know
3. Forward-compatible: future header fields must not break existing parsers
4. Efficient: no seek operations required, linear single-pass parse
5. Unambiguous magic: prevent accidental execution of non-bytecode data

## Decision

Define AVBC (AVAP Virtual Bytecode), a binary format with three sections:

```
┌─────────────────────────────────┐
│  Header (128 bytes, fixed)      │
├─────────────────────────────────┤
│  Constant Pool (variable)       │
├─────────────────────────────────┤
│  Instruction Stream (variable)  │
└─────────────────────────────────┘
```

**Header** (128 bytes, all integers little-endian):

| Offset | Size | Field | Description |
|---|---|---|---|
| 0 | 4 | magic | `b"AVBC"` — rejects non-bytecode files |
| 4 | 2 | isa_version | `major<<8 \| minor` — for future ISA negotiation |
| 6 | 2 | flags | Reserved = 0 |
| 8 | 4 | code_size | Instruction stream length in bytes |
| 12 | 4 | const_count | Number of constant pool entries |
| 16 | 4 | entry_point | IP offset into instruction stream |
| 20 | 108 | reserved | Zero-padded; new fields go here |

The 128-byte fixed header size provides 108 bytes of reserved space. New fields can be added within the reserved region without breaking parsers that read only the fields they know.

**Constant Pool**: type-tagged records immediately after the header. Each entry: 1 tag byte + type-specific payload.

**Instruction Stream**: raw opcode bytes + u32 arguments (little-endian). The kernel does not know argument counts — the ISA handler reads them via `read_u32(code, ip)`.

## Alternatives Considered

**JSON bytecode**: Human-readable, debuggable without tooling. Rejected — parsing overhead is orders of magnitude higher; string interning for opcode names adds complexity; not suitable for a kernel targeting < 30μs execution latency.

**MessagePack / protobuf**: Established binary serialization with schema. Rejected — introduces a parsing library dependency into `platon-core`, which must remain zero-dependency. Both formats also lack the fixed-header / reserved-region forward-compat model.

**WASM binary format reuse**: WebAssembly uses a well-specified binary format. Considered briefly — WASM's format is designed for a specific type system and instruction set (LEB128 encoding, section-based structure) that would be overcomplicated for Platon's needs and would force AVAP's compiler to produce WASM-compatible output.

**ELF-inspired format**: Variable-length sections with a section table. Rejected — adds a two-pass parse (read section table, then seek to each section); unnecessary complexity given Platon bytecode files are always < 64KB.

**Variable-length header**: Header size stored in the file itself. Rejected — creates a chicken-and-egg parse problem (you need to know where the header ends to read its length field). Fixed 128 bytes eliminates this entirely.

## Consequences

- `vm.load()` validates the `b"AVBC"` magic and rejects foreign files immediately
- `vm.load()` is linear-scan, single-pass, no seeks — predictable latency
- The 108-byte reserved region gives ample space for future fields (checksums, debug info offset, ISA UUID)
- `code_size` + `const_count` + `entry_point` allow full parse without executing anything — important for offline validation tooling
- `isa_version` in the header enables future ISA version negotiation in `vm.load()` (planned for 1.0.0, see ROADMAP)
- The instruction stream is opaque to the kernel — the ISA handler drives `ip` forward, so new argument encodings don't require kernel changes
- A corrupted constant pool produces a `PyValueError` at load time, not at execute time — errors are caught early

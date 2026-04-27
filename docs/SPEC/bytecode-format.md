# AVBC Bytecode Format Specification

*Author: Rafael Ruiz, CTO — The Platon Foundation*
*Version: 1.0 — 2026-03-22*

---

## Overview

AVBC (AVAP Virtual Bytecode) is the container format for compiled programs executed by the Platon VM. It is a binary format with a fixed 128-byte header followed by a constant pool and an instruction stream.

Designed to be:
- **Simple** — parseable without external libraries in ~50 lines of code
- **Forwards-compatible** — new header fields go in the reserved region
- **Self-describing** — header contains all metadata needed to parse the file

---

## File Structure

```
┌─────────────────────────────────┐
│  Header (128 bytes, fixed)      │
├─────────────────────────────────┤
│  Constant Pool (variable)       │
├─────────────────────────────────┤
│  Instruction Stream (variable)  │
└─────────────────────────────────┘
```

---

## Header (128 bytes, all integers little-endian)

```
Offset  Size  Type    Field          Description
──────  ────  ──────  ─────────────  ───────────────────────────────────────────
0       4     u8[4]   magic          Always b'AVBC' (0x41 0x56 0x42 0x43)
4       2     u16     isa_version    major<<8 | minor  (e.g. 0x0100 = v1.0)
6       2     u16     flags          Reserved = 0
8       4     u32     code_size      Byte length of the instruction stream
12      4     u32     const_count    Number of entries in the constant pool
16      4     u32     entry_point    IP offset into instruction stream
20      108   u8[]    reserved       Zero-padded. Future fields here.
```

---

## Constant Pool

Immediately follows the 128-byte header. Each entry is a type-tagged record:

| Tag | Type | Payload |
|---|---|---|
| `0x01` | Null | (none) |
| `0x02` | Int | 8 bytes, i64 LE |
| `0x03` | Float | 8 bytes, f64 LE (IEEE 754) |
| `0x04` | String | 4-byte length u32 LE + UTF-8 bytes |
| `0x05` | Bool true | (none) |
| `0x06` | Bool false | (none) |

Constants are indexed 0-based and referenced by ISA instructions.

---

## Instruction Stream

Immediately follows the constant pool. Each instruction:

```
1 byte    opcode
N × 4 bytes  arguments (u32 LE each)
```

N is defined by the ISA for each opcode. The kernel does not know argument counts — the ISA handler reads them via `read_u32(code, ip)`.

---

## Minimal Example

AVBC file that pushes 42 and halts (AVAP ISA):

```
Header (128 bytes):
  41 56 42 43        magic "AVBC"
  01 00              isa_version 0.1
  00 00              flags 0
  06 00 00 00        code_size 6
  01 00 00 00        const_count 1
  00 00 00 00        entry_point 0
  [108 zeros]

Constant pool (9 bytes):
  02                 tag Int
  2a 00 00 00 00 00 00 00   value 42

Instruction stream (6 bytes):
  01 00 00 00 00     PUSH const[0]
  ff                 HALT
```

Total: **143 bytes**.

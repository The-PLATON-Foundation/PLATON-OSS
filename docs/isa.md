# PLATON Instruction Set Architecture (ISA) v1.0

This document specifies the bytecode format and the instruction set for the PLATON Virtual Machine. PLATON is a **stack-based** VM, meaning most instructions operate on a Last-In, First-Out (LIFO) data structure.

---

## 1. Bytecode Portable Format (BPF)

Every bytecode line consists of three main sections:

| Section | Size | Description |
| :--- | :--- | :--- |
| **Header** | 128 bytes | Metadata, versioning, and integrity hashes. |
| **Constant Pool** | Variable | A table of literal values (Strings, Ints, etc.). |
| **Code Section** | Variable | The sequence of opcodes to be executed. |

### 1.1 Header Specification
| Offset | Size | Type | Field |
| :--- | :--- | :--- | :--- |
| 0 | 4 | Magic | `0x41 0x56 0x42 0x43` ("AVBC") |
| 4 | 2 | uint16 | Major Version |
| 6 | 2 | uint16 | Minor Version |
| 8 | 4 | uint32 | Code Section Size (bytes) |
| 12 | 4 | uint32 | Constant Pool Count |
| 16 | 4 | uint32 | Entry Point (Instruction Offset) |

---

## 2. Instruction Set

Instructions consist of a 1-byte **Opcode** followed by zero or more arguments.

### 2.1 Stack Operations
| Opcode | Mnemonic | Args | Description |
| :--- | :--- | :--- | :--- |
| `0x00` | **NOP** | - | No operation. |
| `0x01` | **PUSH** | `u32` | Pushes constant at `index` onto the stack. |
| `0x02` | **POP** | - | Removes the top element from the stack. |
| `0x03` | **DUP** | - | Duplicates the top element. |

### 2.2 Arithmetic & Logic
| Opcode | Mnemonic | Args | Description |
| :--- | :--- | :--- | :--- |
| `0x10` | **ADD** | - | `b = pop(), a = pop() -> push(a + b)` |
| `0x11` | **SUB** | - | `b = pop(), a = pop() -> push(a - b)` |
| `0x12` | **MUL** | - | `b = pop(), a = pop() -> push(a * b)` |
| `0x13` | **DIV** | - | `b = pop(), a = pop() -> push(a / b)` |
| `0x20` | **EQ** | - | `b = pop(), a = pop() -> push(a == b)` |
| `0x21` | **LT** | - | `b = pop(), a = pop() -> push(a < b)` |

### 2.3 Control Flow
| Opcode | Mnemonic | Args | Description |
| :--- | :--- | :--- | :--- |
| `0x30` | **JMP** | `u32` | Sets IP to the given instruction offset. |
| `0x31** | **JMP_IF** | `u32` | Jumps if top of stack is `True`. |
| `0x32** | **JMP_IF_NOT** | `u32` | Jumps if top of stack is `False`. |

### 2.4 Variable Management
*Note: Variable names are stored as strings in the Constant Pool.*

| Opcode | Mnemonic | Args | Description |
| :--- | :--- | :--- | :--- |
| `0x40` | **LOAD** | `u32` | Resolves name at `const_idx` and pushes its value. |
| `0x41` | **STORE** | `u32` | Pops value and stores it in variable named at `const_idx`. |

### 2.5 System & FFI
| Opcode | Mnemonic | Args | Description |
| :--- | :--- | :--- | :--- |
| `0x60` | **CALL_EXT** | `u32` | Calls Host Native Function registered at `index`. |
| `0xFF` | **HALT** | - | Stops VM execution immediately. |

---

## 3. Value System Types
When serializing constants or handling stack values, the following type tags are used:

- `0x00`: Null
- `0x01`: Boolean (1 byte)
- `0x02`: Integer (8 bytes, signed)
- `0x03`: Float (8 bytes, IEEE 754)
- `0x04`: String (4 bytes length + UTF-8 data)
- `0x05`: Array (4 bytes count + items)
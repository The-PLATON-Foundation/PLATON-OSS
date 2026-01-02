from enum import IntEnum

class Opcode(IntEnum):
    """PLATON Instruction Set Architecture (ISA) v1.0"""
    # Stack Operations
    NOP = 0x00
    PUSH = 0x01       # Args: [u32: const_idx]
    POP = 0x02
    DUP = 0x03

    # Arithmetic & Logic
    ADD = 0x10
    SUB = 0x11
    MUL = 0x12
    DIV = 0x13
    EQ = 0x20
    LT = 0x21

    # Control Flow
    JMP = 0x30        # Args: [u32: target_ip]
    JMP_IF = 0x31     # Args: [u32: target_ip]
    JMP_IF_NOT = 0x32 # Args: [u32: target_ip]

    # Variable Management
    LOAD = 0x40       # Args: [u32: const_idx (name)]
    STORE = 0x41      # Args: [u32: const_idx (name)]

    # System & FFI
    CALL_EXT = 0x60   # Args: [u32: func_idx]
    HALT = 0xFF
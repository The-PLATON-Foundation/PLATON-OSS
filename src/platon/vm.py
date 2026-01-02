import time
import struct
from typing import List, Dict, Any, Optional
from .opcodes import Opcode
from .values import Value, NullValue, IntValue, FloatValue, StringValue, BoolValue
from .registry import NativeRegistry
from .exceptions import TimeoutError, VMError 

class VM:
    def __init__(self, timeout: float = 5.0, max_instr: int = 100000):
        self.stack: List[Value] = []
        self.globals: Dict[str, Value] = {}
        self.constants: List[Value] = []
        self.code: List[int] = []
        self.ip = 0
        self.timeout = timeout
        self.max_instr = max_instr
        
        self.registry: Optional[NativeRegistry] = None
        self.context: Any = None

    def load(self, bytecode: bytes):
        magic, _, _, c_size, c_count, entry = struct.unpack('<4sHHIII', bytecode[0:20])
        if magic != b'AVBC': raise ValueError("Invalid Bytecode")
        
        offset = 128
        self.constants = []
        for _ in range(c_count):
            tag = bytecode[offset]
            if tag == 0x02: # Int
                v = struct.unpack('<q', bytecode[offset+1:offset+9])[0]
                self.constants.append(IntValue(v)); offset += 9
            elif tag == 0x03: # Float
                v = struct.unpack('<d', bytecode[offset+1:offset+9])[0]
                self.constants.append(FloatValue(v)); offset += 9
            elif tag == 0x04: # String
                ln = struct.unpack('<I', bytecode[offset+1:offset+5])[0]
                v = bytecode[offset+5:offset+5+ln].decode('utf-8')
                self.constants.append(StringValue(v)); offset += 5 + ln
            else:
                self.constants.append(NullValue()); offset += 1
        
        self.code = list(bytecode[offset:offset+c_size])
        self.ip = entry

    def execute(self, registry: NativeRegistry = None, context: Any = None):
        self.registry = registry
        self.context = context
        # CAMBIO CLAVE: Usar perf_counter para precisión de microsegundos
        start = time.perf_counter() 
        instr_count = 0
        
        while self.ip < len(self.code):
            # Chequeo de seguridad
            if (time.perf_counter() - start) > self.timeout: 
                raise TimeoutError("VM Timeout")
            
            if instr_count >= self.max_instr: 
                raise RuntimeError("Instruction Limit")
            
            op = self.code[self.ip]
            self.ip += 1
            
            if op == Opcode.HALT:
                break
            elif op == Opcode.PUSH:
                idx = struct.unpack('<I', bytes(self.code[self.ip:self.ip+4]))[0]
                self.stack.append(self.constants[idx]); self.ip += 4
            elif op == Opcode.LOAD:
                idx = struct.unpack('<I', bytes(self.code[self.ip:self.ip+4]))[0]
                name = self.constants[idx].value
                self.stack.append(self.globals.get(name, NullValue()))
                self.ip += 4
            elif op == Opcode.STORE:
                idx = struct.unpack('<I', bytes(self.code[self.ip:self.ip+4]))[0]
                name = self.constants[idx].value
                self.globals[name] = self.stack.pop()
                self.ip += 4
            elif op == Opcode.ADD:
                b = self.stack.pop(); a = self.stack.pop()
                self.stack.append(Value.from_python(a.value + b.value))
            elif op == Opcode.CALL_EXT:
                if not self.registry: raise RuntimeError("No NativeRegistry attached")
                func_id = struct.unpack('<I', bytes(self.code[self.ip:self.ip+4]))[0]
                self.ip += 4
                func = self.registry.get_function(func_id)
                result = func(self.context, self.stack)
                if result is not None:
                    self.stack.append(result if isinstance(result, Value) else Value.from_python(result))
            
            instr_count += 1
        
        return self.stack[-1] if self.stack else NullValue()
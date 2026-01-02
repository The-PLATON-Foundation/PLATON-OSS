import struct
from typing import List, Any, Dict
from .opcodes import Opcode

class Compiler:
    def __init__(self):
        self.constants: List[Any] = []
        self.code = bytearray()

    def _add_constant(self, value: Any) -> int:
        if value in self.constants: return self.constants.index(value)
        self.constants.append(value)
        return len(self.constants) - 1

    def emit(self, opcode: Opcode, arg: int = None):
        self.code.append(opcode)
        if arg is not None:
            self.code.extend(struct.pack('<I', arg))

    def compile_ast(self, ast_nodes: List[Dict[str, Any]]) -> bytes:
        for node in ast_nodes:
            self.process_node(node)
        return self.finalize()

    def process_node(self, node: Dict[str, Any]):
        t = node['type']
        
        if t == 'addVar':
            # 1. Primero procesamos el valor (esto hace PUSH al stack)
            val_node = node['properties'][1]
            if isinstance(val_node, (int, float, str)):
                idx_val = self._add_constant(val_node)
                self.emit(Opcode.PUSH, idx_val)
            else:
                self.process_node(val_node)
            
            # 2. Luego guardamos (esto hace POP del stack)
            idx_name = self._add_constant(node['properties'][0])
            self.emit(Opcode.STORE, idx_name)

        elif t == 'literal':
            idx = self._add_constant(node['value'])
            self.emit(Opcode.PUSH, idx)

        elif t == 'variable':
            idx = self._add_constant(node['name'])
            self.emit(Opcode.LOAD, idx)

        elif t == 'binary_op':
            # Para operaciones binarias: Izquierda, Derecha, y luego Operación
            self.process_node(node['left'])
            self.process_node(node['right'])
            ops = {'+': Opcode.ADD, '-': Opcode.SUB, '*': Opcode.MUL, '/': Opcode.DIV, '==': Opcode.EQ}
            self.emit(ops[node['operator']])

    def process_expression(self, val: Any):
        if isinstance(val, dict): self.process_node(val)
        else:
            idx = self._add_constant(val)
            self.emit(Opcode.PUSH, idx)

    def finalize(self) -> bytes:
        self.emit(Opcode.HALT)
        # Header 128 bytes
        header = struct.pack('<4sHHIII', b'AVBC', 1, 0, len(self.code), len(self.constants), 0)
        header = header.ljust(128, b'\x00')
        
        # Pool
        pool = bytearray()
        for c in self.constants:
            if isinstance(c, int): pool.extend(b'\x02' + struct.pack('<q', c))
            elif isinstance(c, float): pool.extend(b'\x03' + struct.pack('<d', c))
            elif isinstance(c, str):
                data = c.encode('utf-8')
                pool.extend(b'\x04' + struct.pack('<I', len(data)) + data)
        return bytes(header + pool + self.code)
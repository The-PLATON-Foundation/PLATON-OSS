from avap_core import VM, Compiler

# Compilar código
compiler = Compiler()
compiler.compile_expression({
    'type': 'assignment',
    'target': 'result',
    'value': {
        'type': 'binary_op',
        'operator': '+',
        'left': {'type': 'literal', 'value': 10},
        'right': {'type': 'literal', 'value': 20}
    }
})

bytecode = compiler.finalize()

# Ejecutar
vm = VM()
vm.load(bytecode)
result = vm.execute()

print(f"Result: {result.value}")  # 30
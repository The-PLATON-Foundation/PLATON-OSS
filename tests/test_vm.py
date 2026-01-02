import pytest
from platon.vm import VM
from platon.compiler import Compiler
from platon.exceptions import TimeoutError # IMPORTANTE: Usar el de PLATON
from platon.values import IntValue

def test_arithmetic_execution():
    """Prueba que 10 + 20 resulte en 30 en la VM"""
    compiler = Compiler()
    # Cambiamos valores planos por diccionarios 'literal'
    ast = [{
        'type': 'addVar',
        'properties': ['counter', {
            'type': 'binary_op',
            'operator': '+',
            'left': {'type': 'literal', 'value': 10},
            'right': {'type': 'literal', 'value': 20}
        }]
    }]
    
    bytecode = compiler.compile_ast(ast)
    vm = VM()
    vm.load(bytecode)
    vm.execute()
    
    assert vm.globals['counter'].value == 30

def test_vm_timeout():
    """Prueba que el sandbox detiene ejecuciones infinitas"""
    from platon.opcodes import Opcode
    compiler = Compiler()
    
    # Creamos un bucle infinito: JMP a la posición 0
    compiler.emit(Opcode.JMP, 0)
    bytecode = compiler.finalize()
    
    # Timeout ultra bajo (1 microsegundo)
    vm = VM(timeout=0.000001) 
    vm.load(bytecode)
    
    with pytest.raises(TimeoutError):
        vm.execute()

def test_string_concatenation():
    """Prueba que el core maneja strings correctamente"""
    compiler = Compiler()
    ast = [{
        'type': 'addVar',
        'properties': ['msg', {
            'type': 'binary_op',
            'operator': '+',
            'left': {'type': 'literal', 'value': "Hello "},
            'right': {'type': 'literal', 'value': "PLATON"}
        }]
    }]
    
    bytecode = compiler.compile_ast(ast)
    vm = VM()
    vm.load(bytecode)
    vm.execute()
    
    assert vm.globals['msg'].value == "Hello PLATON"
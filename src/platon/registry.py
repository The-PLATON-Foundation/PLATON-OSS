from typing import Callable, Dict, List, Any
from .values import Value, NullValue
from .exceptions import VMError

# Tipo para funciones nativas: (contexto, argumentos) -> Value
NativeFunc = Callable[[Any, List[Value]], Value]

class NativeRegistry:
    def __init__(self):
        self._functions: Dict[int, NativeFunc] = {}
        self._names: Dict[str, int] = {}
        self._next_id: int = 0

    def register(self, name: str, func: NativeFunc, func_id: int = None) -> int:
        if func_id is None:
            func_id = self._next_id
            self._next_id += 1
            
        self._functions[func_id] = func
        self._names[name] = func_id
        return func_id

    def get_function(self, func_id: int) -> NativeFunc:
        if func_id not in self._functions:
            raise VMError(f"Native function with ID {func_id} not found")
        return self._functions[func_id]

    def get_id(self, name: str) -> int:
        if name not in self._names:
            raise VMError(f"Native function '{name}' is not registered")
        return self._names[name]
from typing import Any

class Value:
    __slots__ = ('value', 'type')
    def __init__(self, value: Any, type_name: str):
        self.value = value
        self.type = type_name

    @classmethod
    def from_python(cls, obj: Any):
        if obj is None: return NullValue()
        if isinstance(obj, bool): return BoolValue(obj)
        if isinstance(obj, int): return IntValue(obj)
        if isinstance(obj, float): return FloatValue(obj)
        if isinstance(obj, str): return StringValue(obj)
        raise TypeError(f"Unsupported type: {type(obj)}")

    def to_python(self): return self.value

class NullValue(Value):
    def __init__(self): super().__init__(None, "null")

class BoolValue(Value):
    def __init__(self, v: bool): super().__init__(v, "bool")

class IntValue(Value):
    def __init__(self, v: int): super().__init__(v, "int")

class FloatValue(Value):
    def __init__(self, v: float): super().__init__(v, "float")

class StringValue(Value):
    def __init__(self, v: str): super().__init__(v, "string")
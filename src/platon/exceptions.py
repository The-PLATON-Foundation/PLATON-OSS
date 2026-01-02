class VMError(Exception):
    """Base error for all PLATON VM related issues."""
    pass

class TimeoutError(VMError):
    """Raised when the execution exceeds the time limit."""
    pass

class MemoryLimitError(VMError):
    """Raised when the VM exceeds its memory allocation."""
    pass
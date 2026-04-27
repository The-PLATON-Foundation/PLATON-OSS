# -*- coding: utf-8 -*-
"""
Platon - Language-agnostic virtual machine kernel (Rust).
"""
try:
    from .platon import (
        VM, NativeRegistry, Value,
        VMError, TimeoutError,
        __version__,
    )
except ImportError as e:
    raise ImportError(
        "Platon native extension not found. Run 'maturin develop'.\n"
        f"Original error: {e}"
    ) from e

__all__ = ["VM", "NativeRegistry", "Value", "VMError", "TimeoutError", "__version__"]

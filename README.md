# PLATON Kernel 🌌

![CI](https://github.com/user/platon/actions/workflows/ci.yml/badge.svg)
![License](https://img.shields.io/badge/license-MIT-blue)
![Python](https://img.shields.io/badge/python-3.10+-blue)

**PLATON** is a high-performance, syntax-agnostic virtual language kernel. It allows developers to build their own secure, sandboxed Domain Specific Languages (DSLs) by providing a robust Bytecode VM and resource-constrained runtime.

## Key Features
- **Agnostic Architecture**: Decouples language syntax from execution.
- **Secure by Design**: Built-in resource limiting (Memory/CPU/Timeout).
- **Portable Bytecode**: Compiles to a standard binary format.
- **Extensible**: Easily register native functions via Python or Rust (Phase 2).

## Quick Start
1. **Clone & Install**:
   ```bash
   pip install -r requirements.txt
   ```

2. **Compile a Script**:
    ```python
    from platon import Compiler
    # ... logic ...
    ````
## Roadmap
Check our ROADMAP.md for the MVP plan.

## Contributing
Please read CONTRIBUTING.md for details on our code of conduct and the process for submitting pull requests.


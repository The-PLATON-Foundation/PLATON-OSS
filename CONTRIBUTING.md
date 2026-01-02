# Contributing to PLATON Kernel

First of all, thank you for taking the time to contribute!.

As an Open Source project, we rely on collaborators to help us build a safe, fast, and universal virtual language kernel.

## [Our Vision]
The core objective of PLATON is to provide a universal, high-performance kernel that enables anyone to build their own virtual languages. We aim to abstract the complexity of bytecode execution, memory management, and sandboxing, providing a "virtual engine" that powers diverse DSLs.

We are currently in *Phase 1* (Python Bootstrapping) to validate the ISA and logic, with the ultimate goal of migrating the core to Rust (*Phase 2*) for production-grade performance.

## Why PLATON?

PLATON is built on the premise that execution logic should be decoupled from syntax. Our vision is to provide a universal kernel that serves as the "CPU" for your own virtual languages.

- **Language Creator Toolkit**: Use PLATON as the foundation to build rules engines, automation scripts, or custom domain languages without reinventing the VM.
- **Universal Kernel**: A single, secure core that can interpret any language compiled to its standard Bytecode Portable Format (BPF).

---

## Code of Conduct
By participating, you are expected to uphold our standards of inclusivity and respect. Please report any inappropriate behavior to the project maintainers.


## How Can I Contribute?

### 1. Reporting Bugs
- Check if the bug has already been reported in the [Issues](https://github.com/youruser/platon-kernel/issues) section.
- If not, open a new issue using the Bug Report template, including steps to reproduce it.

### 2. Suggesting Features
- We love new Opcodes and DSL ideas!
- Please open an issue describing the use-case and the proposed bytecode structure.

### 3. Submitting Pull Requests
1. Fork the repo and create your branch from `main`.
2. If you've added code that should be tested, add tests!
3. Ensure the test suite passes.
4. Make sure your code adheres to the **Ruff** linting standards.

---

## Development Setup

### Python Setup

1. Create a virtual environment:
   ```bash
   python -m venv venv
   source venv/bin/activate # On Windows: venv\Scripts\activate
   ```
   
2. Install dev dependencies:
    ```bash
    pip install -r requirements-dev.txt
    ```

### Running Tests

    
    pytest tests/
    
### Code Quality (Linting)

We use Ruff for fast linting and formatting:

    
    ruff check .      # Check for errors
    ruff format .     # Auto-format code
    

## Physical Layout & Bytecode Standards

All contributions must respect the BPF (Bytecode Portable Format) specification:

- Little-Endian for all numeric values.

- Header Size: is fixed at 128 bytes.

- Opcodes: must be added to opcodes.py and documented in the wiki.

## Rust Migration (Phase 2)

We are planning to rewrite the VM core in Rust using Maturin for Python bindings. If you have Rust expertise, please look for issues labeled rust-port.

---

Thank you for helping us build the future of secure virtual languages!
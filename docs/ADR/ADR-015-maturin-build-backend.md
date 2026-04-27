---
date: 2026-04-26
status: Accepted
author: Rafael Ruiz, CTO — The Platon Foundation
---

# ADR-015: Maturin as Python Extension Build Backend

## Context

Platon is a hybrid Rust+Python project: a Rust library compiled as a Python extension module (`.so` / `.pyd`). Building such a hybrid requires tooling that:

1. Compiles Rust with Cargo and produces a shared library
2. Packages the shared library as a Python wheel (`.whl`)
3. Handles platform-specific ABI naming (`platon.cpython-311-x86_64-linux-gnu.so`)
4. Integrates with `pip install` and standard Python build frontends (PEP 517/518)
5. Supports `manylinux` for PyPI distribution

## Decision

Use **Maturin 1.5+** as the PEP 517 build backend, configured in `pyproject.toml`:

```toml
[build-system]
requires      = ["maturin>=1.5,<2.0"]
build-backend = "maturin"

[tool.maturin]
features      = ["pyo3/extension-module"]
```

Maturin automatically:
- Detects `crate-type = ["cdylib"]` in `Cargo.toml`
- Compiles with Cargo (inheriting `[profile.release]`)
- Names the output correctly for the target Python ABI
- Builds a compliant `.whl` with `pip install target/wheels/*.whl`
- Supports `maturin develop` for in-place development installs (no wheel building)

## Alternatives Considered

**setuptools-rust**: Integrates Rust compilation into setuptools via a custom `build_ext`. Requires writing a `setup.py` / `setup.cfg` with explicit Rust extension configuration. Rejected — setuptools-rust predates PEP 517; its build model is more complex for a Cargo workspace; Maturin's wheel naming and manylinux support are significantly more mature.

**PyO3-build-config + manual Cargo invocation**: Call `cargo build --release` manually and copy the `.so` file. Rejected — requires manual ABI naming, no wheel packaging, no `pip install` integration. Suitable for local hacking but not for CI or distribution.

**milksnake (Instabot)**: Legacy approach using CFFI + Rust. Rejected — not PyO3-native; requires writing a Python CFFI shim layer on top of Rust; no longer actively maintained.

**Pyo3-pack (historical name for Maturin)**: Maturin was previously called pyo3-pack. The current name is Maturin — no distinction.

## Consequences

- **Development workflow**: `maturin develop` compiles Rust (debug profile) and installs in-place — typically 2–10s for incremental builds
- **CI/CD workflow**: `maturin build --release` produces a `.whl` in `target/wheels/`; `pip install target/wheels/*.whl --force-reinstall` installs it
- **Docker startup blocker**: Currently, the Docker container runs `maturin build --release` at startup (~60s). This is a P0 blocker addressed in ROADMAP 0.4.0 by pre-building wheels in CI and including them in the image
- **Version bound**: `maturin>=1.5,<2.0` pins to the 1.x major series. Maturin 2.0 (if released) may have breaking changes to `pyproject.toml` configuration — the upper bound prevents silent breakage
- **manylinux**: `maturin build --release --manylinux auto` produces a `manylinux2014` wheel compatible with most Linux distributions — required for PyPI distribution
- **Python 3.11+ only**: `requires-python = ">=3.11"` in `pyproject.toml` is enforced by Maturin at wheel build time — incompatible Python versions produce an explicit error

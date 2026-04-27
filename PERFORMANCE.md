# Performance

*Author: Rafael Ruiz, CTO — The Platon Foundation*
*Last updated: 2026-03-22*

---

## Observed Execution Times

Measured on Apple M2 (8-core), Docker Linux container, Release build (`opt-level = 3, lto = true`).

| Command | Instructions | Time (first run) | Time (warm) |
|---|---|---|---|
| `addVar` | 39 | 0.295ms | 0.026ms |
| `addVar` (2nd call) | 39 | 0.054ms | 0.026ms |
| Typical AVAP command | 30–60 | 0.05–0.3ms | 0.02–0.08ms |

The first-run overhead (~0.3ms) includes GIL acquisition and PyO3 bridge initialisation. Subsequent calls on the same VM instance are consistently ~0.03ms.

---

## Bottlenecks

### 1. Build Time (Critical for Production)

| Stage | Time |
|---|---|
| `maturin build --release` (platon) | ~60s |
| `maturin build --release` (avap-isa) | ~150s |
| `pip install` (wheel) | ~5s |
| **Total container startup** | **~215s (3.5 min)** |

This is **the primary production blocker**. Every container restart or rollout incurs 3.5 minutes of downtime. The fix is pre-building wheels in CI and including them in the Docker image (`COPY wheels/ /wheels/ && pip install /wheels/*.whl`).

### 2. vm.globals Property

`vm.globals` creates a new `PyDict` on every access — it iterates `VMState.globals` and boxes each `Value` as a `PyValue`. In the Language Server sync loop, this is called once per execution. Acceptable currently (~0.05ms for 10 variables). If the namespace grows to 100+ variables, consider a lazy snapshot.

### 3. Dict Lookup (O(n))

`Value::Dict` uses `Vec<(String, Value)>` with linear scan. For dicts with > 30 keys, this degrades. AVAP commands typically have < 20 keys in any single dict. If larger dicts are needed, `Value::Dict` should switch to `IndexMap` (preserves order, O(1) lookup).

### 4. Value Cloning

Every stack operation that reads a value clones it (`load_global` returns `Value` by value). For small values (Null, Bool, Int, Float) this is cheap. For `List` and `Dict`, cloning is O(n). Avoid large collection manipulation in tight loops.

---

## Profiling

```bash
# Build with symbols
RUSTFLAGS="-g" maturin develop

# Profile with py-spy
pip install py-spy
py-spy record -o profile.svg -- python3 your_benchmark.py

# Cargo flamegraph (Rust-level)
cargo install flamegraph
cargo flamegraph --bin your_binary
```

---

## Benchmarking the Execution Loop

```python
import time
from platon import VM
from avap_isa import AvapISA

vm = VM(timeout=5.0)
vm.register_isa(AvapISA())
vm.load(avbc_bytecode)
vm.debug = False

N = 10_000
start = time.perf_counter()
for _ in range(N):
    vm.load(avbc_bytecode)  # reset state
    vm.execute()
elapsed = time.perf_counter() - start
print(f"{N} executions: {elapsed*1000:.1f}ms total, {elapsed/N*1e6:.1f}μs/call")
```

---

## Targets (Future)

| Metric | Current | Target |
|---|---|---|
| Container startup (with pre-built wheels) | 215s | < 10s |
| Execution latency p50 (warm) | ~30μs | < 20μs |
| Execution latency p99 (warm) | ~300μs | < 100μs |
| Max instructions/second (single thread) | ~1.5M | > 5M |

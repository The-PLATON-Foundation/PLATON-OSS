# Value Type System

*Author: Rafael Ruiz, CTO — The Platon Foundation*
*Version: 1.0 — 2026-03-22*

---

## Variants

| Variant | Rust | Python | Truthy |
|---|---|---|---|
| `Null` | — | `None` | Never |
| `Bool(b)` | `bool` | `bool` | `b == true` |
| `Int(i)` | `i64` | `int` | `i != 0` |
| `Float(f)` | `f64` | `float` | `f != 0.0` |
| `Str(s)` | `String` | `str` | non-empty |
| `List(v)` | `Vec<Value>` | `list` | non-empty |
| `Dict(d)` | `Vec<(String,Value)>` | `dict` | non-empty |
| `Iter(v,i)` | `(Vec<Value>, usize)` | iterator | `i < v.len()` |

---

## Semantics

**Null** — absence of value. Returned by `LOAD` for undefined variables, `GET_ITEM` for missing keys.

**Bool** — distinct from Int. `Bool(true) != Int(1)` under `eq_val`.

**Int** — wrapping arithmetic. `IS_INSTANCE` for `float` also matches `Int` (permissive coercion).

**Dict** — ordered `Vec<(String, Value)>`. O(n) lookup. Preserves insertion order. Suitable for small dicts (< 20 keys typical in AVAP).

**Iter** — snapshot iterator. `GET_ITER` copies the source collection. Mutations to source after iteration begins are not reflected.

---

## Python ↔ Value Conversion

### Python → Value (priority order)

1. `None` → `Null`
2. `bool` → `Bool` (before `int` — Python bool is int subclass)
3. `int` → `Int`
4. `float` → `Float`
5. `str` → `Str`
6. `Value` → unwrap
7. `dict` → `Dict` (before list — dicts are iterable)
8. iterable → `List`

### Value → Python

Direct mapping. `Iter(v, i)` → `list` of remaining items from index `i`.

---

## Equality

Strict type equality — no cross-type coercion:

```
Null == Null         → true
Bool(true) == Bool(true) → true
Int(1) == Bool(true) → false  (different types)
Int(1) == Float(1.0) → false  (different types)
```

---

## Known Limitations

- No `Bytes` type
- No `Tuple` — `BUILD_TUPLE` produces `List`
- Value semantics — `Dict` and `List` are always copied. `SET_ITEM` on a copy does not affect the original in globals.
- `Dict` lookup is O(n)

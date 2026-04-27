// ─────────────────────────────────────────────────────────────────────────────
// platon-core unit tests
//
// Run with:  cargo test -p platon-core
//
// Coverage targets:
//   Value         — all variants, is_truthy, eq_val, get_item, set_item,
//                   contains, to_string_repr
//   VMState       — push/pop/peek, globals, constants, try_stack,
//                   conector_vars, results
//   InstructionSet — register, get, is_halt, len
//   read_u32       — valid, truncated
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod value_type_tests {
    use crate::Value;

    // ── type_name ────────────────────────────────────────────────────────────

    #[test]
    fn type_name_all_variants() {
        assert_eq!(Value::Null.type_name(),                  "null");
        assert_eq!(Value::Bool(true).type_name(),            "bool");
        assert_eq!(Value::Int(0).type_name(),                "int");
        assert_eq!(Value::Float(0.0).type_name(),            "float");
        assert_eq!(Value::Str("".to_string()).type_name(),   "string");
        assert_eq!(Value::List(vec![]).type_name(),          "list");
        assert_eq!(Value::Dict(vec![]).type_name(),          "dict");
        assert_eq!(Value::Iter(vec![], 0).type_name(),       "iterator");
    }

    // ── is_truthy ────────────────────────────────────────────────────────────

    #[test]
    fn null_is_falsy() {
        assert!(!Value::Null.is_truthy());
    }

    #[test]
    fn bool_truthy() {
        assert!(Value::Bool(true).is_truthy());
        assert!(!Value::Bool(false).is_truthy());
    }

    #[test]
    fn int_truthy() {
        assert!(Value::Int(1).is_truthy());
        assert!(Value::Int(-1).is_truthy());
        assert!(!Value::Int(0).is_truthy());
    }

    #[test]
    fn float_truthy() {
        assert!(Value::Float(0.1).is_truthy());
        assert!(Value::Float(-0.1).is_truthy());
        assert!(!Value::Float(0.0).is_truthy());
    }

    #[test]
    fn str_truthy() {
        assert!(Value::Str("x".to_string()).is_truthy());
        assert!(!Value::Str("".to_string()).is_truthy());
    }

    #[test]
    fn list_truthy() {
        assert!(Value::List(vec![Value::Int(1)]).is_truthy());
        assert!(!Value::List(vec![]).is_truthy());
    }

    #[test]
    fn dict_truthy() {
        assert!(Value::Dict(vec![("k".to_string(), Value::Int(1))]).is_truthy());
        assert!(!Value::Dict(vec![]).is_truthy());
    }

    #[test]
    fn iter_truthy_when_items_remain() {
        // Iter with items remaining → truthy
        assert!(Value::Iter(vec![Value::Int(1)], 0).is_truthy());
        // Iter exhausted (i >= len) → falsy
        assert!(!Value::Iter(vec![Value::Int(1)], 1).is_truthy());
        // Empty iter → falsy
        assert!(!Value::Iter(vec![], 0).is_truthy());
    }

    // ── eq_val ───────────────────────────────────────────────────────────────

    #[test]
    fn null_equals_null() {
        assert!(Value::Null.eq_val(&Value::Null));
    }

    #[test]
    fn bool_equality() {
        assert!(Value::Bool(true).eq_val(&Value::Bool(true)));
        assert!(Value::Bool(false).eq_val(&Value::Bool(false)));
        assert!(!Value::Bool(true).eq_val(&Value::Bool(false)));
    }

    #[test]
    fn int_equality() {
        assert!(Value::Int(42).eq_val(&Value::Int(42)));
        assert!(!Value::Int(42).eq_val(&Value::Int(43)));
    }

    #[test]
    fn str_equality() {
        assert!(Value::Str("hello".to_string()).eq_val(&Value::Str("hello".to_string())));
        assert!(!Value::Str("hello".to_string()).eq_val(&Value::Str("world".to_string())));
    }

    /// CRITICAL: Bool(true) must NOT equal Int(1). Python semantics differ but
    /// Platon uses strict type equality for correctness and predictability.
    #[test]
    fn bool_not_equal_to_int() {
        assert!(!Value::Bool(true).eq_val(&Value::Int(1)));
        assert!(!Value::Bool(false).eq_val(&Value::Int(0)));
    }

    #[test]
    fn int_not_equal_to_float() {
        assert!(!Value::Int(1).eq_val(&Value::Float(1.0)));
    }

    #[test]
    fn null_not_equal_to_bool_false() {
        assert!(!Value::Null.eq_val(&Value::Bool(false)));
    }

    #[test]
    fn cross_type_never_equal() {
        // Primitivos de distintos tipos con valor "cero" — nunca deben ser iguales entre sí
        let vals: Vec<(&str, Value)> = vec![
            ("null",   Value::Null),
            ("bool",   Value::Bool(false)),
            ("int",    Value::Int(0)),
            ("float",  Value::Float(0.0)),
            ("string", Value::Str("".to_string())),
        ];
        for (i, (name_a, a)) in vals.iter().enumerate() {
            for (j, (name_b, b)) in vals.iter().enumerate() {
                if i == j {
                    assert!(a.eq_val(b),
                        "Same type should equal itself: {}", name_a);
                } else {
                    assert!(!a.eq_val(b),
                        "Different types should never be equal: {} vs {}", name_a, name_b);
                }
            }
        }
    }

    // ── get_item ─────────────────────────────────────────────────────────────

    #[test]
    fn dict_get_existing_key() {
        let d = Value::Dict(vec![
            ("x".to_string(), Value::Int(42)),
            ("y".to_string(), Value::Str("hello".to_string())),
        ]);
        assert_eq!(d.get_item(&Value::Str("x".to_string())), Some(Value::Int(42)));
        assert_eq!(d.get_item(&Value::Str("y".to_string())),
            Some(Value::Str("hello".to_string())));
    }

    #[test]
    fn dict_get_missing_key_returns_none() {
        let d = Value::Dict(vec![("x".to_string(), Value::Int(1))]);
        assert_eq!(d.get_item(&Value::Str("missing".to_string())), None);
    }

    #[test]
    fn dict_get_requires_str_key() {
        let d = Value::Dict(vec![("0".to_string(), Value::Int(1))]);
        // Int key on Dict → None (not panic)
        assert_eq!(d.get_item(&Value::Int(0)), None);
    }

    #[test]
    fn list_get_by_int_index() {
        let l = Value::List(vec![Value::Int(10), Value::Int(20), Value::Int(30)]);
        assert_eq!(l.get_item(&Value::Int(0)), Some(Value::Int(10)));
        assert_eq!(l.get_item(&Value::Int(2)), Some(Value::Int(30)));
    }

    #[test]
    fn list_get_out_of_bounds_returns_none() {
        let l = Value::List(vec![Value::Int(1)]);
        assert_eq!(l.get_item(&Value::Int(99)), None);
        assert_eq!(l.get_item(&Value::Int(-1)), None);
    }

    #[test]
    fn null_get_item_returns_none() {
        assert_eq!(Value::Null.get_item(&Value::Str("k".to_string())), None);
    }

    // ── set_item ─────────────────────────────────────────────────────────────

    #[test]
    fn dict_set_new_key() {
        let mut d = Value::Dict(vec![]);
        d.set_item(Value::Str("k".to_string()), Value::Int(99));
        assert_eq!(d.get_item(&Value::Str("k".to_string())), Some(Value::Int(99)));
    }

    #[test]
    fn dict_set_existing_key_overwrites() {
        let mut d = Value::Dict(vec![("k".to_string(), Value::Int(1))]);
        d.set_item(Value::Str("k".to_string()), Value::Int(2));
        assert_eq!(d.get_item(&Value::Str("k".to_string())), Some(Value::Int(2)));
        // Only one entry — no duplicates
        if let Value::Dict(pairs) = &d {
            assert_eq!(pairs.len(), 1);
        }
    }

    #[test]
    fn dict_set_preserves_insertion_order() {
        let mut d = Value::Dict(vec![]);
        d.set_item(Value::Str("a".to_string()), Value::Int(1));
        d.set_item(Value::Str("b".to_string()), Value::Int(2));
        d.set_item(Value::Str("c".to_string()), Value::Int(3));
        if let Value::Dict(pairs) = &d {
            assert_eq!(pairs[0].0, "a");
            assert_eq!(pairs[1].0, "b");
            assert_eq!(pairs[2].0, "c");
        }
    }

    #[test]
    fn list_set_by_index() {
        let mut l = Value::List(vec![Value::Int(0), Value::Int(0), Value::Int(0)]);
        l.set_item(Value::Int(1), Value::Int(99));
        assert_eq!(l.get_item(&Value::Int(1)), Some(Value::Int(99)));
        // Other items unchanged
        assert_eq!(l.get_item(&Value::Int(0)), Some(Value::Int(0)));
        assert_eq!(l.get_item(&Value::Int(2)), Some(Value::Int(0)));
    }

    #[test]
    fn set_item_on_null_is_noop() {
        let mut v = Value::Null;
        v.set_item(Value::Str("k".to_string()), Value::Int(1));
        // Should not panic, should not change value
        assert!(matches!(v, Value::Null));
    }

    // ── contains ─────────────────────────────────────────────────────────────

    #[test]
    fn list_contains() {
        let l = Value::List(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
        assert!(l.contains(&Value::Int(2)));
        assert!(!l.contains(&Value::Int(99)));
    }

    #[test]
    fn dict_contains_key() {
        let d = Value::Dict(vec![("x".to_string(), Value::Int(1))]);
        assert!(d.contains(&Value::Str("x".to_string())));
        assert!(!d.contains(&Value::Str("y".to_string())));
    }

    #[test]
    fn str_contains_substring() {
        let s = Value::Str("hello world".to_string());
        assert!(s.contains(&Value::Str("world".to_string())));
        assert!(!s.contains(&Value::Str("xyz".to_string())));
    }

    #[test]
    fn null_contains_nothing() {
        assert!(!Value::Null.contains(&Value::Null));
    }

    // ── to_string_repr ───────────────────────────────────────────────────────

    #[test]
    fn string_repr_primitives() {
        assert_eq!(Value::Null.to_string_repr(),             "None");
        assert_eq!(Value::Bool(true).to_string_repr(),       "True");
        assert_eq!(Value::Bool(false).to_string_repr(),      "False");
        assert_eq!(Value::Int(42).to_string_repr(),          "42");
        assert_eq!(Value::Int(-7).to_string_repr(),          "-7");
        assert_eq!(Value::Float(3.14).to_string_repr(),      "3.14");
        assert_eq!(Value::Str("hi".to_string()).to_string_repr(), "hi");
    }

    #[test]
    fn string_repr_list() {
        let l = Value::List(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(l.to_string_repr(), "[1, 2]");
    }

    #[test]
    fn string_repr_empty_collections() {
        assert_eq!(Value::List(vec![]).to_string_repr(), "[]");
        assert_eq!(Value::Dict(vec![]).to_string_repr(), "{}");
    }

    // ── clone ────────────────────────────────────────────────────────────────

    #[test]
    fn clone_is_independent_copy() {
        let mut original = Value::Dict(vec![("k".to_string(), Value::Int(1))]);
        let clone = original.clone();
        // Mutate original
        original.set_item(Value::Str("k".to_string()), Value::Int(999));
        // Clone is unaffected — value semantics
        assert_eq!(clone.get_item(&Value::Str("k".to_string())), Some(Value::Int(1)));
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// VMState tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod vmstate_tests {
    use crate::{Value, VMState};

    fn state() -> VMState { VMState::new() }

    // ── stack ────────────────────────────────────────────────────────────────

    #[test]
    fn push_pop_basic() {
        let mut s = state();
        s.push(Value::Int(1));
        s.push(Value::Int(2));
        assert_eq!(s.pop(), Some(Value::Int(2)));
        assert_eq!(s.pop(), Some(Value::Int(1)));
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn pop_empty_returns_none() {
        let mut s = state();
        assert_eq!(s.pop(), None);
    }

    #[test]
    fn peek_does_not_consume() {
        let mut s = state();
        s.push(Value::Int(42));
        assert_eq!(s.peek(), Some(&Value::Int(42)));
        assert_eq!(s.peek(), Some(&Value::Int(42))); // still there
        assert_eq!(s.pop(), Some(Value::Int(42)));
    }

    #[test]
    fn peek_empty_returns_none() {
        let s = state();
        assert_eq!(s.peek(), None);
    }

    #[test]
    fn peek_mut_allows_in_place_modification() {
        let mut s = state();
        s.push(Value::Int(0));
        if let Some(top) = s.peek_mut() {
            *top = Value::Int(99);
        }
        assert_eq!(s.pop(), Some(Value::Int(99)));
    }

    #[test]
    fn stack_lifo_order() {
        let mut s = state();
        for i in 0..10 {
            s.push(Value::Int(i));
        }
        for i in (0..10).rev() {
            assert_eq!(s.pop(), Some(Value::Int(i)));
        }
    }

    // ── globals ──────────────────────────────────────────────────────────────

    #[test]
    fn store_and_load_global() {
        let mut s = state();
        s.store_global("x".to_string(), Value::Int(42));
        assert_eq!(s.load_global("x"), Value::Int(42));
    }

    #[test]
    fn load_undefined_global_returns_null() {
        let s = state();
        assert!(matches!(s.load_global("undefined"), Value::Null));
    }

    #[test]
    fn store_global_overwrites() {
        let mut s = state();
        s.store_global("x".to_string(), Value::Int(1));
        s.store_global("x".to_string(), Value::Int(2));
        assert_eq!(s.load_global("x"), Value::Int(2));
    }

    #[test]
    fn multiple_globals_independent() {
        let mut s = state();
        s.store_global("a".to_string(), Value::Int(1));
        s.store_global("b".to_string(), Value::Str("hello".to_string()));
        s.store_global("c".to_string(), Value::Bool(true));
        assert_eq!(s.load_global("a"), Value::Int(1));
        assert_eq!(s.load_global("b"), Value::Str("hello".to_string()));
        assert_eq!(s.load_global("c"), Value::Bool(true));
    }

    // ── constants ────────────────────────────────────────────────────────────

    #[test]
    fn get_const_returns_reference() {
        let mut s = state();
        s.constants.push(Value::Int(10));
        s.constants.push(Value::Str("hello".to_string()));
        assert_eq!(s.get_const(0), Some(&Value::Int(10)));
        assert_eq!(s.get_const(1), Some(&Value::Str("hello".to_string())));
    }

    #[test]
    fn get_const_out_of_bounds_returns_none() {
        let s = state();
        assert_eq!(s.get_const(0), None);
        assert_eq!(s.get_const(999), None);
    }

    #[test]
    fn get_const_str_extracts_string() {
        let mut s = state();
        s.constants.push(Value::Str("hello".to_string()));
        assert_eq!(s.get_const_str(0), Some("hello".to_string()));
    }

    #[test]
    fn get_const_str_non_string_returns_none() {
        let mut s = state();
        s.constants.push(Value::Int(42));
        assert_eq!(s.get_const_str(0), None);
    }

    // ── try_stack ────────────────────────────────────────────────────────────

    #[test]
    fn try_stack_push_pop() {
        let mut s = state();
        s.try_stack.push(100);
        s.try_stack.push(200);
        assert_eq!(s.try_stack.pop(), Some(200));
        assert_eq!(s.try_stack.pop(), Some(100));
        assert_eq!(s.try_stack.pop(), None);
    }

    // ── conector namespaces ──────────────────────────────────────────────────

    #[test]
    fn conector_vars_starts_empty() {
        let s = state();
        assert!(s.conector_vars.is_empty());
        assert!(s.results.is_empty());
    }

    #[test]
    fn conector_vars_insert_and_read() {
        let mut s = state();
        s.conector_vars.insert("total".to_string(), Value::Int(42));
        assert_eq!(s.conector_vars.get("total"), Some(&Value::Int(42)));
    }

    #[test]
    fn results_insert_and_read() {
        let mut s = state();
        s.results.insert("output".to_string(), Value::Str("ok".to_string()));
        assert_eq!(s.results.get("output"), Some(&Value::Str("ok".to_string())));
    }

    #[test]
    fn conector_vars_and_results_are_independent() {
        let mut s = state();
        s.conector_vars.insert("x".to_string(), Value::Int(1));
        s.results.insert("x".to_string(), Value::Int(2));
        // Same key, different namespaces
        assert_eq!(s.conector_vars.get("x"), Some(&Value::Int(1)));
        assert_eq!(s.results.get("x"), Some(&Value::Int(2)));
    }

    // ── new() starts clean ───────────────────────────────────────────────────

    #[test]
    fn new_state_is_empty() {
        let s = state();
        assert!(s.stack.is_empty());
        assert!(s.globals.is_empty());
        assert!(s.constants.is_empty());
        assert!(s.try_stack.is_empty());
        assert!(s.conector_vars.is_empty());
        assert!(s.results.is_empty());
        assert!(!s.debug);
        assert!(s.registry_ptr.is_null());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// InstructionSet tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod instruction_set_tests {
    use crate::{InstructionSet, InstructionMeta, VMState, ISAError};

    fn noop(_s: &mut VMState, _c: &[u8], _ip: &mut usize, _: *mut ()) -> Result<(), ISAError> {
        Ok(())
    }

    fn fail(_s: &mut VMState, _c: &[u8], _ip: &mut usize, _: *mut ()) -> Result<(), ISAError> {
        Err("intentional failure".to_string())
    }

    #[test]
    fn register_and_get() {
        let mut isa = InstructionSet::new(0xFF);
        isa.register(InstructionMeta { opcode: 0x01, name: "PUSH", n_u32_args: 1 }, noop);
        let instr = isa.get(0x01);
        assert!(instr.is_some());
        assert_eq!(instr.unwrap().meta.name, "PUSH");
        assert_eq!(instr.unwrap().meta.n_u32_args, 1);
    }

    #[test]
    fn get_unknown_opcode_returns_none() {
        let isa = InstructionSet::new(0xFF);
        assert!(isa.get(0x42).is_none());
    }

    #[test]
    fn is_halt_true_for_halt_opcode() {
        let isa = InstructionSet::new(0xFF);
        assert!(isa.is_halt(0xFF));
    }

    #[test]
    fn is_halt_false_for_other_opcodes() {
        let isa = InstructionSet::new(0xFF);
        assert!(!isa.is_halt(0x00));
        assert!(!isa.is_halt(0x01));
        assert!(!isa.is_halt(0xFE));
    }

    #[test]
    fn len_tracks_registered_count() {
        let mut isa = InstructionSet::new(0xFF);
        assert_eq!(isa.len(), 0);
        assert!(isa.is_empty());
        isa.register(InstructionMeta { opcode: 0x01, name: "A", n_u32_args: 0 }, noop);
        assert_eq!(isa.len(), 1);
        assert!(!isa.is_empty());
        isa.register(InstructionMeta { opcode: 0x02, name: "B", n_u32_args: 0 }, noop);
        assert_eq!(isa.len(), 2);
    }

    #[test]
    fn handler_is_callable() {
        let mut isa = InstructionSet::new(0xFF);
        isa.register(InstructionMeta { opcode: 0x01, name: "FAIL", n_u32_args: 0 }, fail);
        let instr = isa.get(0x01).unwrap();
        let mut s = VMState::new();
        let result = (instr.handler)(&mut s, &[], &mut 0, std::ptr::null_mut());
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "intentional failure");
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// read_u32 tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod read_u32_tests {
    use crate::read_u32;

    #[test]
    fn reads_little_endian_u32() {
        let code: Vec<u8> = vec![0x01, 0x00, 0x00, 0x00]; // LE: 1
        let mut ip = 0;
        assert_eq!(read_u32(&code, &mut ip).unwrap(), 1);
        assert_eq!(ip, 4);
    }

    #[test]
    fn reads_large_value() {
        // 0x00_01_00_00 = 65536
        let code: Vec<u8> = vec![0x00, 0x00, 0x01, 0x00];
        let mut ip = 0;
        assert_eq!(read_u32(&code, &mut ip).unwrap(), 65536);
    }

    #[test]
    fn advances_ip_by_4() {
        let code: Vec<u8> = vec![
            0x01, 0x00, 0x00, 0x00,  // first u32 = 1
            0x02, 0x00, 0x00, 0x00,  // second u32 = 2
        ];
        let mut ip = 0;
        assert_eq!(read_u32(&code, &mut ip).unwrap(), 1);
        assert_eq!(read_u32(&code, &mut ip).unwrap(), 2);
        assert_eq!(ip, 8);
    }

    #[test]
    fn reads_at_offset() {
        let code: Vec<u8> = vec![0xFF, 0xFF, 0x07, 0x00, 0x00, 0x00];
        let mut ip = 2; // skip first 2 bytes
        assert_eq!(read_u32(&code, &mut ip).unwrap(), 7);
        assert_eq!(ip, 6);
    }

    #[test]
    fn truncated_returns_error() {
        let code: Vec<u8> = vec![0x01, 0x00, 0x00]; // only 3 bytes
        let mut ip = 0;
        assert!(read_u32(&code, &mut ip).is_err());
        // ip should not advance on error
        assert_eq!(ip, 0);
    }

    #[test]
    fn empty_code_returns_error() {
        let code: Vec<u8> = vec![];
        let mut ip = 0;
        assert!(read_u32(&code, &mut ip).is_err());
    }

    #[test]
    fn read_at_exact_boundary_succeeds() {
        let code: Vec<u8> = vec![0x05, 0x00, 0x00, 0x00];
        let mut ip = 0;
        assert_eq!(read_u32(&code, &mut ip).unwrap(), 5);
    }

    #[test]
    fn read_past_boundary_returns_error() {
        let code: Vec<u8> = vec![0x01, 0x00, 0x00, 0x00];
        let mut ip = 1; // only 3 bytes remaining, need 4
        assert!(read_u32(&code, &mut ip).is_err());
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// ISAProvider trait tests
// ─────────────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod isa_provider_tests {
    use crate::{ISAProvider, InstructionSet, InstructionMeta, VMState, ISAError};
    use std::sync::Arc;

    struct MinimalISA { isa: InstructionSet }

    fn h_nop(_: &mut VMState, _: &[u8], _: &mut usize, _: *mut ()) -> Result<(), ISAError> {
        Ok(())
    }

    impl MinimalISA {
        fn new() -> Self {
            let mut isa = InstructionSet::new(0xFF);
            isa.register(InstructionMeta { opcode: 0x00, name: "NOP", n_u32_args: 0 }, h_nop);
            Self { isa }
        }
    }

    impl ISAProvider for MinimalISA {
        fn name(&self)            -> &str         { "test-isa" }
        fn version(&self)         -> (u8, u8, u8) { (1, 2, 3) }
        fn instruction_set(&self) -> &InstructionSet { &self.isa }
    }

    #[test]
    fn isa_metadata() {
        let isa = MinimalISA::new();
        assert_eq!(isa.name(), "test-isa");
        assert_eq!(isa.version(), (1, 2, 3));
    }

    #[test]
    fn isa_instruction_set_accessible() {
        let isa = MinimalISA::new();
        assert_eq!(isa.instruction_set().len(), 1);
        assert!(isa.instruction_set().get(0x00).is_some());
    }

    #[test]
    fn isa_is_send_sync() {
        // Verifies the trait bound at compile time — ISAProvider must be Send + Sync
        // for Arc<dyn ISAProvider> to work.
        let isa = MinimalISA::new();
        let _arc: Arc<dyn ISAProvider> = Arc::new(isa);
    }
}

//! Platon kernel core — pure Rust, no Python dependency.
//!
//! Contains the fundamental types shared between:
//!   platon     (Python bindings)
//!   avap-isa   (AVAP instruction set)
//!   any other ISA crate
//!
//! Public API:
//!   Value           — runtime value enum
//!   VMState         — mutable execution state
//!   ISAProvider     — trait every ISA must implement
//!   InstructionSet  — opcode -> handler table
//!   InstructionMeta — opcode metadata

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Value
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Bool(bool),
    Int(i64),
    Float(f64),
    Str(String),
    List(Vec<Value>),
    Dict(Vec<(String, Value)>),
    Iter(Vec<Value>, usize),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Self::Null    => "null",   Self::Bool(_)  => "bool",
            Self::Int(_)  => "int",    Self::Float(_) => "float",
            Self::Str(_)  => "string", Self::List(_)  => "list",
            Self::Dict(_) => "dict",   Self::Iter(..) => "iterator",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Self::Null       => false,
            Self::Bool(b)    => *b,
            Self::Int(i)     => *i != 0,
            Self::Float(f)   => *f != 0.0,
            Self::Str(s)     => !s.is_empty(),
            Self::List(v)    => !v.is_empty(),
            Self::Dict(d)    => !d.is_empty(),
            Self::Iter(v, i) => *i < v.len(),
        }
    }

    pub fn eq_val(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null,     Self::Null)     => true,
            (Self::Bool(a),  Self::Bool(b))  => a == b,
            (Self::Int(a),   Self::Int(b))   => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            (Self::Str(a),   Self::Str(b))   => a == b,
            _ => false,
        }
    }

    pub fn get_item(&self, key: &Value) -> Option<Value> {
        match (self, key) {
            (Self::Dict(pairs), Self::Str(k)) =>
                pairs.iter().find(|(pk, _)| pk == k).map(|(_, v)| v.clone()),
            (Self::List(items), Self::Int(i)) =>
                items.get(*i as usize).cloned(),
            _ => None,
        }
    }

    pub fn set_item(&mut self, key: Value, value: Value) {
        match (self, key) {
            (Self::Dict(pairs), Value::Str(k)) => {
                if let Some(p) = pairs.iter_mut().find(|(pk, _)| *pk == k) {
                    p.1 = value;
                } else {
                    pairs.push((k, value));
                }
            }
            (Self::List(items), Value::Int(i)) => {
                if let Some(slot) = items.get_mut(i as usize) { *slot = value; }
            }
            _ => {}
        }
    }

    pub fn contains(&self, item: &Value) -> bool {
        match self {
            Self::List(items) => items.iter().any(|i| i.eq_val(item)),
            Self::Dict(pairs) => {
                if let Self::Str(k) = item {
                    pairs.iter().any(|(pk, _)| pk == k)
                } else { false }
            }
            Self::Str(s) => {
                if let Self::Str(sub) = item { s.contains(sub.as_str()) }
                else { false }
            }
            _ => false,
        }
    }

    pub fn to_string_repr(&self) -> String {
        match self {
            Self::Null     => "None".to_string(),
            Self::Bool(b)  => if *b { "True".to_string() } else { "False".to_string() },
            Self::Int(i)   => i.to_string(),
            Self::Float(f) => f.to_string(),
            Self::Str(s)   => s.clone(),
            Self::List(v)  => format!("[{}]",
                v.iter().map(|i| i.to_string_repr()).collect::<Vec<_>>().join(", ")),
            Self::Dict(d)  => format!("{{{}}}",
                d.iter().map(|(k, v)| format!("{:?}: {}", k, v.to_string_repr()))
                    .collect::<Vec<_>>().join(", ")),
            Self::Iter(..) => "<iterator>".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// VMState — mutable execution state passed to every instruction handler
// ---------------------------------------------------------------------------

pub struct VMState {
    pub stack:        Vec<Value>,
    pub globals:      HashMap<String, Value>,
    pub constants:    Vec<Value>,
    pub try_stack:    Vec<usize>,
    pub debug:        bool,
    // Call frame stack: each entry is the local scope of a function invocation.
    // PUSH_FRAME pushes a new HashMap; POP_FRAME removes it.
    // LOAD checks frames (innermost first) before falling back to globals.
    // STORE writes to the innermost frame when one is active, else to globals.
    pub frames:       Vec<HashMap<String, Value>>,
    // Special namespaces written by STORE_RESULT / SET_CONECTOR_VAR opcodes
    // These are exposed back to Python after execute() completes.
    pub results:      HashMap<String, Value>,
    pub conector_vars: HashMap<String, Value>,
    // Registry is stored as a raw pointer to avoid generic parameters.
    // The platon crate fills this in before execute().
    pub registry_ptr: *mut (),
}

// Safety: VMState is only used single-threaded within a single execute() call.
unsafe impl Send for VMState {}
unsafe impl Sync for VMState {}

impl VMState {
    pub fn new() -> Self {
        Self {
            stack:         Vec::new(),
            globals:       HashMap::new(),
            constants:     Vec::new(),
            try_stack:     Vec::new(),
            debug:         false,
            frames:        Vec::new(),
            results:       HashMap::new(),
            conector_vars: HashMap::new(),
            registry_ptr:  std::ptr::null_mut(),
        }
    }

    pub fn pop(&mut self)        -> Option<Value> { self.stack.pop() }
    pub fn push(&mut self, v: Value)              { self.stack.push(v); }
    pub fn peek(&self)           -> Option<&Value> { self.stack.last() }
    pub fn peek_mut(&mut self)   -> Option<&mut Value> { self.stack.last_mut() }

    pub fn load_global(&self, name: &str) -> Value {
        self.globals.get(name).cloned().unwrap_or(Value::Null)
    }
    pub fn store_global(&mut self, name: String, value: Value) {
        self.globals.insert(name, value);
    }
    pub fn get_const(&self, idx: usize) -> Option<&Value> {
        self.constants.get(idx)
    }
    pub fn get_const_str(&self, idx: usize) -> Option<String> {
        match self.constants.get(idx) {
            Some(Value::Str(s)) => Some(s.clone()),
            _ => None,
        }
    }
}

// ---------------------------------------------------------------------------
// ISA contract
// ---------------------------------------------------------------------------

/// Error type returned by instruction handlers.
/// We use a simple string to avoid pulling in PyO3 types here.
pub type ISAError = String;

/// The type signature for every instruction handler.
///
/// The `py_ctx` parameter is a raw pointer to a Python GIL context.
/// ISAs that need to call back into Python (e.g. CALL_EXT) cast this
/// to `Python<'_>` using `unsafe { Python::assume_gil_acquired() }`.
/// ISAs that don't need Python can ignore it.
pub type InstructionFn = fn(
    state:  &mut VMState,
    code:   &[u8],
    ip:     &mut usize,
    py_ctx: *mut (),       // opaque Python GIL pointer
) -> Result<(), ISAError>;

/// Per-opcode metadata — used for disassembly and debug output.
pub struct InstructionMeta {
    pub opcode:     u8,
    pub name:       &'static str,
    pub n_u32_args: u8,
}

pub struct Instruction {
    pub meta:    InstructionMeta,
    pub handler: InstructionFn,
}

/// The full instruction table for one ISA.
pub struct InstructionSet {
    instructions: HashMap<u8, Instruction>,
    halt_opcode:  u8,
}

impl InstructionSet {
    pub fn new(halt_opcode: u8) -> Self {
        Self { instructions: HashMap::new(), halt_opcode }
    }

    pub fn register(&mut self, meta: InstructionMeta, handler: InstructionFn) {
        self.instructions.insert(meta.opcode, Instruction { meta, handler });
    }

    pub fn get(&self, opcode: u8) -> Option<&Instruction> {
        self.instructions.get(&opcode)
    }

    pub fn is_halt(&self, opcode: u8) -> bool {
        opcode == self.halt_opcode
    }

    pub fn len(&self) -> usize { self.instructions.len() }
    pub fn is_empty(&self) -> bool { self.instructions.is_empty() }
}

/// Every ISA must implement this trait.
pub trait ISAProvider: Send + Sync {
    fn name(&self)             -> &str;
    fn version(&self)          -> (u8, u8, u8);
    fn instruction_set(&self)  -> &InstructionSet;
}

// ---------------------------------------------------------------------------
// Bytecode loader helpers
// ---------------------------------------------------------------------------

/// Read a u32 (little-endian) from `code` at `*ip` and advance ip by 4.
pub fn read_u32(code: &[u8], ip: &mut usize) -> Result<u32, ISAError> {
    if *ip + 4 > code.len() {
        return Err("Bytecode truncated reading u32".to_string());
    }
    let b: [u8; 4] = code[*ip..*ip + 4].try_into().unwrap();
    *ip += 4;
    Ok(u32::from_le_bytes(b))
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;

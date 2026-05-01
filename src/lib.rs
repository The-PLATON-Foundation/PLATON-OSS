//! Platon — Python bindings for the platon-core kernel.
//!
//! Exposes VM, NativeRegistry, Value, VMError, TimeoutError to Python.
//! ISAs are registered via vm.register_isa() before calling vm.execute().//!
//! PyO3 version: 0.22

use platon_core::{Value as CoreValue, VMState, ISAProvider};
use pyo3::prelude::*;
use pyo3::exceptions::{PyException, PyRuntimeError, PyValueError};
use pyo3::types::{PyBytes, PyDict, PyList};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pyo3::create_exception!(platon, VMError,      PyException, "Base VM error.");
pyo3::create_exception!(platon, TimeoutError, VMError,     "Execution timeout.");

// ---------------------------------------------------------------------------
// Re-export CoreValue as PyValue for Python
// ---------------------------------------------------------------------------

#[pyclass(name = "Value")]
#[derive(Clone)]
pub struct PyValue { pub inner: CoreValue }

impl PyValue {
    pub fn from_py_any(obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self { inner: core_from_py(obj)? })
    }
}

pub fn core_from_py(obj: &Bound<'_, PyAny>) -> PyResult<CoreValue> {
    if obj.is_none() { return Ok(CoreValue::Null); }
    if let Ok(b) = obj.extract::<bool>()   { return Ok(CoreValue::Bool(b)); }
    if let Ok(i) = obj.extract::<i64>()    { return Ok(CoreValue::Int(i)); }
    if let Ok(f) = obj.extract::<f64>()    { return Ok(CoreValue::Float(f)); }
    if let Ok(s) = obj.extract::<String>() { return Ok(CoreValue::Str(s)); }
    if let Ok(pv) = obj.extract::<PyValue>() { return Ok(pv.inner); }
    // Check dict BEFORE list — dicts are also iterable in Python and
    // would be incorrectly extracted as Vec<&PyAny> (just the keys)
    if let Ok(d) = obj.downcast::<PyDict>() {
        let mut pairs = Vec::new();
        for (k, v) in d.iter() {
            pairs.push((k.extract::<String>()?, core_from_py(&v)?));
        }
        return Ok(CoreValue::Dict(pairs));
    }
    if let Ok(list) = obj.extract::<Vec<Bound<'_, PyAny>>>() {
        let items: PyResult<Vec<_>> = list.iter().map(|i| core_from_py(i)).collect();
        return Ok(CoreValue::List(items?));
    }
    Err(PyValueError::new_err(format!("Unsupported type: {}", obj.get_type().name()?)))
}

pub fn core_to_py<'py>(v: &CoreValue, py: Python<'py>) -> Bound<'py, PyAny> {
    match v {
        CoreValue::Null     => py.None().into_bound(py),
        CoreValue::Bool(b)  => b.into_py(py).into_bound(py),
        CoreValue::Int(i)   => i.into_py(py).into_bound(py),
        CoreValue::Float(f) => f.into_py(py).into_bound(py),
        CoreValue::Str(s)   => s.into_py(py).into_bound(py),
        CoreValue::List(items) => {
            let v: Vec<Bound<'_, PyAny>> = items.iter().map(|i| core_to_py(i, py)).collect();
            PyList::new_bound(py, &v).into_any()
        }
        CoreValue::Dict(pairs) => {
            let d = PyDict::new_bound(py);
            for (k, v) in pairs { let _ = d.set_item(k, core_to_py(v, py)); }
            d.into_any()
        }
        CoreValue::Iter(items, idx) => {
            let v: Vec<Bound<'_, PyAny>> = items[*idx..].iter().map(|i| core_to_py(i, py)).collect();
            PyList::new_bound(py, &v).into_any()
        }
    }
}

#[pymethods]
impl PyValue {
    #[new]
    #[pyo3(signature = (value, _type_name=None))]
    fn new(value: &Bound<'_, PyAny>, _type_name: Option<String>) -> PyResult<Self> {
        Ok(Self { inner: core_from_py(value)? })
    }
    #[classmethod]
    fn from_python(_cls: &Bound<'_, pyo3::types::PyType>, obj: &Bound<'_, PyAny>) -> PyResult<Self> {
        Ok(Self { inner: core_from_py(obj)? })
    }
    fn to_python(&self, py: Python<'_>) -> PyObject {
        core_to_py(&self.inner, py).into()
    }
    #[getter] fn value(&self, py: Python<'_>) -> PyObject {
        core_to_py(&self.inner, py).into()
    }
    #[getter] fn type_(&self) -> &str { self.inner.type_name() }
    fn __repr__(&self) -> String { format!("<Value type={}>", self.inner.type_name()) }
}

// ---------------------------------------------------------------------------
// NativeRegistry
// ---------------------------------------------------------------------------

#[pyclass(name = "NativeRegistry")]
#[derive(Clone)]
pub struct NativeRegistry {
    pub functions:    Arc<Mutex<HashMap<u32, PyObject>>>,
    pub names_to_ids: Arc<Mutex<HashMap<String, u32>>>,
    pub attrs:        Arc<Mutex<HashMap<String, PyObject>>>,
}

#[pymethods]
impl NativeRegistry {
    #[new]
    fn new() -> Self {
        Self {
            functions:    Arc::new(Mutex::new(HashMap::new())),
            names_to_ids: Arc::new(Mutex::new(HashMap::new())),
            attrs:        Arc::new(Mutex::new(HashMap::new())),
        }
    }
    fn register_command(&self, name: String, func_id: u32, py_func: PyObject) -> PyResult<()> {
        self.functions.lock().unwrap().insert(func_id, py_func);
        self.names_to_ids.lock().unwrap().insert(name.clone(), func_id);
        //println!("[PLATON] Registered: {} [ID: {}]", name, func_id);
        Ok(())
    }
    fn get_id_by_name(&self, name: &str) -> i64 {
        self.names_to_ids.lock().unwrap()
            .get(name).copied().map(|i| i as i64).unwrap_or(-1)
    }
    #[getter]
    fn _names_to_ids(&self, py: Python<'_>) -> PyObject {
        let d = PyDict::new_bound(py);
        for (n, id) in self.names_to_ids.lock().unwrap().iter() {
            let _ = d.set_item(n, *id);
        }
        d.into()
    }
    fn __setattr__(&self, name: String, value: PyObject) -> PyResult<()> {
        self.attrs.lock().unwrap().insert(name, value); Ok(())
    }
    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        self.attrs.lock().unwrap().get(name)
            .map(|o| o.clone_ref(py))
            .ok_or_else(|| PyRuntimeError::new_err(
                format!("NativeRegistry has no attr '{}'", name)
            ))
    }
}

impl NativeRegistry {
    pub fn get_function(&self, func_id: u32) -> Option<PyObject> {
        Python::with_gil(|py| self.functions.lock().unwrap().get(&func_id).map(|o| o.clone_ref(py)))
    }
    pub fn get_name_for_id(&self, func_id: u32) -> Option<String> {
        self.names_to_ids.lock().unwrap().iter()
            .find(|(_, &id)| id == func_id).map(|(n, _)| n.clone())
    }
}

// ---------------------------------------------------------------------------
// VmProxy — passed to CALL_EXT callbacks as 'vm'
// ---------------------------------------------------------------------------

#[pyclass(name = "_VmProxy")]
pub struct VmProxy {
    #[pyo3(get, set)] pub globals: PyObject,
    pub registry: NativeRegistry,
}

impl Clone for VmProxy {
    fn clone(&self) -> Self {
        Python::with_gil(|py| Self {
            globals:  self.globals.clone_ref(py),
            registry: self.registry.clone(),
        })
    }
}

#[pymethods]
impl VmProxy {
    #[new]
    fn new(globals: PyObject, registry: NativeRegistry) -> Self { Self { globals, registry } }
    #[getter] fn get_registry(&self) -> NativeRegistry { self.registry.clone() }
    fn __setattr__(&self, name: String, value: PyObject) -> PyResult<()> {
        self.registry.attrs.lock().unwrap().insert(name, value); Ok(())
    }
    fn __getattr__(&self, py: Python<'_>, name: &str) -> PyResult<PyObject> {
        self.registry.attrs.lock().unwrap().get(name)
            .map(|o| o.clone_ref(py))
            .ok_or_else(|| PyRuntimeError::new_err(format!("_VmProxy has no attr '{}'", name)))
    }
}

// ---------------------------------------------------------------------------
// VM
// ---------------------------------------------------------------------------

#[pyclass(name = "VM")]
pub struct VM {
    code:      Vec<u8>,
    ip:        usize,
    state:     VMState,
    timeout:   f64,
    max_instr: u64,
    isa:       Option<Arc<dyn ISAProvider>>,
    context:   Option<PyObject>,
    registry:  Option<NativeRegistry>,
}

#[pymethods]
impl VM {
    #[new]
    #[pyo3(signature = (timeout=5.0, max_instr=100000))]
    fn new(timeout: f64, max_instr: u64) -> Self {
        Self {
            code: Vec::new(), ip: 0,
            state: VMState::new(),
            timeout, max_instr,
            isa: None, context: None, registry: None,
        }
    }

    fn load(&mut self, bytecode: &Bound<'_, PyBytes>) -> PyResult<()> {
        let data = bytecode.as_bytes();
        if data.len() < 128 { return Err(PyValueError::new_err("Bytecode too short")); }
        if &data[0..4] != b"AVBC" { return Err(PyValueError::new_err("Invalid magic")); }
        let code_size   = u32::from_le_bytes(data[8..12].try_into().unwrap())  as usize;
        let const_count = u32::from_le_bytes(data[12..16].try_into().unwrap()) as usize;
        let entry       = u32::from_le_bytes(data[16..20].try_into().unwrap()) as usize;
        let mut off = 128usize;
        self.state.constants = Vec::with_capacity(const_count);
        for _ in 0..const_count {
            if off >= data.len() { return Err(PyValueError::new_err("Bytecode truncated")); }
            let tag = data[off]; off += 1;
            match tag {
                0x02 => { let v=i64::from_le_bytes(data[off..off+8].try_into().unwrap());
                          self.state.constants.push(CoreValue::Int(v)); off+=8; }
                0x03 => { let v=f64::from_le_bytes(data[off..off+8].try_into().unwrap());
                          self.state.constants.push(CoreValue::Float(v)); off+=8; }
                0x04 => { let ln=u32::from_le_bytes(data[off..off+4].try_into().unwrap()) as usize;
                          off+=4;
                          let s=std::str::from_utf8(&data[off..off+ln])
                              .map_err(|e| PyValueError::new_err(format!("Bad UTF-8: {}",e)))?;
                          self.state.constants.push(CoreValue::Str(s.to_string())); off+=ln; }
                0x05 => { self.state.constants.push(CoreValue::Bool(true));  }
                0x06 => { self.state.constants.push(CoreValue::Bool(false)); }
                _    => { self.state.constants.push(CoreValue::Null); }
            }
        }
        self.code = data[off..off+code_size].to_vec();
        self.ip   = entry;
        Ok(())
    }

    fn register_isa(&mut self, isa_obj: PyObject, py: Python<'_>) -> PyResult<()> {
        let ptr_val = isa_obj
            .call_method0(py, "_get_arc_ptr")
            .map_err(|_| PyRuntimeError::new_err(
                "ISA object must implement _get_arc_ptr() -> (u64,u64)."
            ))?;
        let ptrs = ptr_val.extract::<(u64, u64)>(py)?;
        // SAFETY: avap-isa stored Arc<dyn ISAProvider> as a fat pointer (data+vtable)
        // and leaked it as (u64, u64) via transmute. We reverse that here.
        let provider: Arc<dyn ISAProvider> = unsafe {
            let raw: *const dyn ISAProvider = std::mem::transmute::<(u64, u64), *const dyn ISAProvider>(ptrs);
            Arc::from_raw(raw)
        };
        self.isa = Some(provider);
        Ok(())
    }

    #[pyo3(signature = (registry=None, context=None))]
    fn execute(
        &mut self,
        py:       Python<'_>,
        registry: Option<NativeRegistry>,
        context:  Option<PyObject>,
    ) -> PyResult<PyValue> {
        if let Some(r) = registry.clone() {
            self.registry = Some(r.clone());
            let func_dict = PyDict::new_bound(py);
            for (id, func) in r.functions.lock().unwrap().iter() {
                let _ = func_dict.set_item(*id, func.clone_ref(py));
            }
            let dict_py: Py<PyDict> = func_dict.into();
            let raw = Box::new(dict_py);
            self.state.registry_ptr = Box::into_raw(raw) as *mut ();
        }
        if let Some(c) = context { self.context = Some(c); }

        let isa = self.isa.as_ref()
            .ok_or_else(|| VMError::new_err(
                "No ISA registered. Call vm.register_isa(AvapISA()) before execute()."
            ))?
            .clone();

        let instruction_set = isa.instruction_set();
        let start = Instant::now();
        let mut count = 0u64;
        let py_ctx = (&py as *const Python<'_>) as *mut ();

        if self.state.debug {
            println!("[PLATON-VM] execute() — ISA: {} v{}.{}.{}, {} bytes",
                isa.name(),
                isa.version().0, isa.version().1, isa.version().2,
                self.code.len());
        }

        loop {
            if self.ip >= self.code.len() { break; }
            if start.elapsed().as_secs_f64() > self.timeout {
                return Err(TimeoutError::new_err("VM Timeout"));
            }
            if count >= self.max_instr {
                return Err(VMError::new_err("Instruction limit exceeded"));
            }
            let opcode = self.code[self.ip]; self.ip += 1;
            if instruction_set.is_halt(opcode) {
                if self.state.debug {
                    println!("[PLATON-VM] HALT — {} instr in {:.3}ms",
                        count, start.elapsed().as_secs_f64()*1000.0);
                }
                break;
            }
            let instr = instruction_set.get(opcode).ok_or_else(|| {
                VMError::new_err(format!("Unknown opcode 0x{:02X} at ip={} (ISA: {})",
                    opcode, self.ip-1, isa.name()))
            })?;
            if self.state.debug {
                println!("[PLATON-VM] 0x{:02X} {} ip={}", opcode, instr.meta.name, self.ip-1);
            }
            (instr.handler)(&mut self.state, &self.code, &mut self.ip, py_ctx)
                .map_err(|e| VMError::new_err(e))?;
            count += 1;
        }

        // SAFETY: registry_ptr was set by Box::into_raw above; freed exactly once here.
        if !self.state.registry_ptr.is_null() {
            unsafe { let _ = Box::from_raw(self.state.registry_ptr as *mut Py<PyDict>); }
            self.state.registry_ptr = std::ptr::null_mut();
        }

        let result = self.state.stack.last().cloned().unwrap_or(CoreValue::Null);
        Ok(PyValue { inner: result })
    }

    #[getter]
    fn globals(&self, py: Python<'_>) -> PyObject {
        let d = PyDict::new_bound(py);
        for (k, v) in &self.state.globals {
            let _ = d.set_item(k, Py::new(py, PyValue { inner: v.clone() }).unwrap().into_py(py));
        }
        d.into()
    }
    #[setter]
    fn set_globals(&mut self, py: Python<'_>, dict: &Bound<'_, PyDict>) -> PyResult<()> {
        self.state.globals.clear();
        for (k, v) in dict.iter() {
            let key: String = k.extract::<String>()?;
            let val = if let Ok(pv) = v.extract::<PyValue>() { pv.inner }
                      else { core_from_py(&v)? };
            self.state.globals.insert(key, val);
        }
        let _ = py; Ok(())
    }
    #[getter] fn get_debug(&self) -> bool { self.state.debug }
    #[setter] fn set_debug(&mut self, v: bool) { self.state.debug = v; }

    #[getter]
    fn conector_vars(&self, py: Python<'_>) -> PyObject {
        let d = PyDict::new_bound(py);
        for (k, v) in &self.state.conector_vars {
            let _ = d.set_item(k, Py::new(py, PyValue { inner: v.clone() }).unwrap().into_py(py));
        }
        d.into()
    }

    #[getter]
    fn results(&self, py: Python<'_>) -> PyObject {
        let d = PyDict::new_bound(py);
        for (k, v) in &self.state.results {
            let _ = d.set_item(k, Py::new(py, PyValue { inner: v.clone() }).unwrap().into_py(py));
        }
        d.into()
    }
    #[getter] fn get_registry(&self) -> Option<NativeRegistry> { self.registry.clone() }
    #[setter] fn set_registry(&mut self, r: NativeRegistry) { self.registry = Some(r.clone()); }
}

// ---------------------------------------------------------------------------
// Module entry point
// ---------------------------------------------------------------------------

#[pymodule]
fn platon(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<VM>()?;
    m.add_class::<NativeRegistry>()?;
    m.add_class::<PyValue>()?;
    m.add_class::<VmProxy>()?;
    m.add("VMError",      m.py().get_type_bound::<VMError>())?;
    m.add("TimeoutError", m.py().get_type_bound::<TimeoutError>())?;
    m.add("__version__",  "0.3.0")?;
    Ok(())
}
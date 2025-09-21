pub mod injection_simulation;
use injection_simulation::_injection_simulation_rust;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn rust_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_injection_simulation_rust, m)?)?;
    Ok(())
}

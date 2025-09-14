pub mod velocity_model;
use velocity_model::_single_source_co2_fill_rust;

use pyo3::prelude::*;

/// A Python module implemented in Rust.
#[pymodule]
fn rust_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_single_source_co2_fill_rust, m)?)?;
    Ok(())
}

pub mod constants;
pub mod datastucture;
pub mod utils;

pub mod injection_simulation;
use injection_simulation::_injection_simulation_rust;

use numpy::{PyArray3, PyReadonlyArray1, PyReadonlyArray3};
use pyo3::prelude::*;

/// Wrap the injection simulation function to be accessible from Python
#[pyfunction]
#[pyo3(signature = (reservoir_matrix, depths, max_column_height, source, total_snapshots = 100))]
#[allow(clippy::too_many_arguments)] // TODO: Handle this later
pub fn _injection_simulation_python_wrapper(
    py: Python<'_>,
    reservoir_matrix: PyReadonlyArray3<f64>,
    depths: PyReadonlyArray1<f64>,
    max_column_height: usize,
    source: (usize, usize, usize),
    total_snapshots: usize,
) -> PyResult<Py<PyArray3<i32>>> {
    let reservoir_matrix = reservoir_matrix.as_array();
    let depths = depths.as_array();

    // Call the Rust implementation of the injection simulation
    let snapshots = _injection_simulation_rust(
        reservoir_matrix,
        depths,
        max_column_height,
        source,
        total_snapshots,
    );

    // Return the snapshots as a Python array
    Ok(PyArray3::from_array(py, &snapshots).into())
}

/// A Python module implemented in Rust.
#[pymodule]
fn rust_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(_injection_simulation_python_wrapper, m)?)?;
    Ok(())
}

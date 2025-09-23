// Run using  cargo run --bin simulate from the rust_backend directory
// Remember to rename Cargo.toml.bak to Cargo.toml when debugging in Rust

use ndarray_npy::read_npy;
use numpy::ndarray::{Array1, Array2, Array3};

// Import some functions from the Rust backend
use rust_backend::injection_simulation::_injection_simulation_rust;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let depths: Array1<f64> = read_npy("../simulations/depths.npy")?;
    let caprock_matrix: Array3<f64> = read_npy("../simulations/caprock_matrix.npy")?;
    let bedrock_indices: Array2<i32> = read_npy("../simulations/bedrock_indices.npy")?;

    // Turn into usize
    let bedrock_indices = bedrock_indices.mapv(|x| x as usize);

    // Hardcoded source for testing
    let xi = 600;
    let yi = 200;
    let zi = 24;

    let source = (xi, yi, zi);
    let max_column_height = 10;
    let total_snapshots = 100;

    let _ = _injection_simulation_rust(
        caprock_matrix.view(),
        depths.view(),
        bedrock_indices.view(),
        max_column_height,
        source,
        total_snapshots,
    );

    Ok(())
}

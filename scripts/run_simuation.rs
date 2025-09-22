/// Utility for running the simulation within the Rust backend. Useful for testing and debugging.
use ndarray_npy::read_npy;
use numpy::ndarray::{Array1, Array3};

// Import some functions from the Rust backend

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // depths = np.load(simulations_dir / "depths.npy")
    // caprock_matrix = np.load(simulations_dir / "caprock_matrix.npy")
    let depths: Array1<f64> = read_npy("../../simulations/depths.npy")?;
    let caprock_matrix: Array3<f64> = read_npy("../../simulations/caprock_matrix.npy")?;

    // Hardcoded source for testing
    let xi = 600;
    let yi = 200;
    let zi = 24;

    let source = (xi, yi, zi);
    let max_column_height = 10;
    let total_snapshots = 100;

    // let _ = _injection_simulation_rust(
    //     caprock_matrix.view(),
    //     depths.view(),
    //     max_column_height,
    //     source,
    //     total_snapshots,
    // );

    Ok(())
}

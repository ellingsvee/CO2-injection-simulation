use pyo3::prelude::*;

use numpy::ndarray::Array3;
use numpy::{PyArray3, PyReadonlyArray1, PyReadonlyArray3};

use crate::constants::{VELOCITY_CAPROCK, VELOCITY_CO2, VELOCITY_RESERVOIR};
use crate::datastucture::DepthOrderedQueue;
use crate::utils::{is_inside_bounds, safe_indices};

// Spread directions for 8-connectivity
const SPREAD_DIRECTIONS: [(i32, i32); 8] = [
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1),
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1),
];

#[pyfunction]
#[pyo3(signature = (reservoir_matrix, depths, source, total_snapshots = 100))]
pub fn _injection_simulation_rust(
    py: Python<'_>,
    reservoir_matrix: PyReadonlyArray3<f64>,
    depths: PyReadonlyArray1<f64>,
    source: (usize, usize, usize),
    total_snapshots: usize,
) -> PyResult<Py<PyArray3<i32>>> {
    let reservoir_matrix = reservoir_matrix.as_array();
    let depths = depths.as_array();

    let (nx, ny, nz) = reservoir_matrix.dim();
    let (xi, yi, zi) = source;
    let mut zi = zi;

    // Create mutable copy of reservoir_matrix matrix
    let mut reservoir_matrix = reservoir_matrix.to_owned();
    let mut visited = Array3::<bool>::default((nx, ny, nz));
    let mut snapshots = Array3::<i32>::from_elem((nx, ny, nz), -1);

    // Calculate snapshot interval
    let n_total_reservoir_cells: usize = reservoir_matrix
        .iter()
        .filter(|&&val| val == VELOCITY_RESERVOIR)
        .count();
    let snapshot_interval = std::cmp::max(1, n_total_reservoir_cells / total_snapshots);

    // Validate source position
    if reservoir_matrix[[xi, yi, zi]] != VELOCITY_RESERVOIR {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Source must be in reservoir",
        ));
    }
    if zi > 0 && reservoir_matrix[[xi, yi, zi - 1]] != VELOCITY_CAPROCK {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Source must be just below caprock",
        ));
    }

    let mut snapshots_counter = 0;
    let mut cells_filled_since_snapshot = 0;

    while zi < nz {
        let mut queue = DepthOrderedQueue::new();

        if is_inside_bounds(xi as i32, yi as i32, zi as i32, nx, ny, nz) {
            queue.push(depths[zi], xi, yi, zi);
        }

        while let Some((xi_curr, yi_curr, zi_curr)) = queue.pop() {
            // Skip if already visited
            if visited[[xi_curr, yi_curr, zi_curr]] {
                continue;
            }

            // Mark as visited
            visited[[xi_curr, yi_curr, zi_curr]] = true;

            // Check if the cell can be filled with CO2
            if reservoir_matrix[[xi_curr, yi_curr, zi_curr]] == VELOCITY_RESERVOIR
                && (zi_curr == 0
                    || reservoir_matrix[[xi_curr, yi_curr, zi_curr - 1]] != VELOCITY_RESERVOIR)
            {
                reservoir_matrix[[xi_curr, yi_curr, zi_curr]] = VELOCITY_CO2;
                snapshots[[xi_curr, yi_curr, zi_curr]] = snapshots_counter;
                cells_filled_since_snapshot += 1;

                // Take snapshot based on number of cells filled
                if cells_filled_since_snapshot >= snapshot_interval {
                    snapshots_counter += 1;
                    cells_filled_since_snapshot = 0;
                }
            }

            // Check if CO2 can move upward (9-connectivity neighbors above)
            let mut added_above = false;

            // Check directly above first
            if zi_curr > 0 {
                let zi_above = zi_curr - 1;
                if reservoir_matrix[[xi_curr, yi_curr, zi_above]] == VELOCITY_RESERVOIR {
                    queue.push(depths[zi_above], xi_curr, yi_curr, zi_above);
                    added_above = true;
                }

                // Check 8-connected neighbors above
                for &(dx, dy) in &SPREAD_DIRECTIONS {
                    if let Some((x_new, y_new, z_new)) = safe_indices(
                        xi_curr as i32 + dx,
                        yi_curr as i32 + dy,
                        zi_above as i32,
                        nx,
                        ny,
                        nz,
                    ) {
                        if reservoir_matrix[[x_new, y_new, z_new]] == VELOCITY_RESERVOIR {
                            queue.push(depths[z_new], x_new, y_new, z_new);
                            added_above = true;
                        }
                    }
                }
            }

            // If can't move up, spread horizontally
            if !added_above {
                for &(dx, dy) in &SPREAD_DIRECTIONS {
                    if let Some((x_new, y_new, z_new)) = safe_indices(
                        xi_curr as i32 + dx,
                        yi_curr as i32 + dy,
                        zi_curr as i32,
                        nx,
                        ny,
                        nz,
                    ) {
                        if reservoir_matrix[[x_new, y_new, z_new]] != VELOCITY_CAPROCK {
                            queue.push(depths[z_new], x_new, y_new, z_new);
                        }
                    }
                }
            }
        }

        zi += 1;
    }

    // Convert result to Python array
    Ok(PyArray3::from_array(py, &snapshots).into())
}

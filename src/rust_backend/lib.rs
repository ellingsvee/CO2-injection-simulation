use pyo3::prelude::*;

// The algorithm
use numpy::ndarray::Array3;
use numpy::{PyArray3, PyReadonlyArray1, PyReadonlyArray2, PyReadonlyArray3};
use std::cmp::Ordering;
use std::collections::BinaryHeap;

// Velocity constants matching the Python implementation
const VELOCITY_CAPROCK: i32 = 2607;
const VELOCITY_RESERVOIR: i32 = 1500;
const VELOCITY_CO2: i32 = 300;

// Spread directions for 8-connectivity (including diagonals)
const SPREAD_DIRECTIONS: [(i32, i32); 8] = [
    (-1, 0),
    (1, 0),
    (0, -1),
    (0, 1), // Cardinal directions
    (-1, -1),
    (-1, 1),
    (1, -1),
    (1, 1), // Diagonal directions
];

// Priority queue element for the heap-based flood fill
#[derive(Clone)]
struct HeapElement {
    depth: f64,
    x: usize,
    y: usize,
    z: usize,
}

// Custom ordering for the heap (min-heap based on depth)
impl PartialEq for HeapElement {
    fn eq(&self, other: &Self) -> bool {
        self.depth == other.depth
    }
}

impl Eq for HeapElement {}

impl PartialOrd for HeapElement {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Reverse ordering for min-heap (BinaryHeap is max-heap by default)
        other.depth.partial_cmp(&self.depth)
    }
}

impl Ord for HeapElement {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).unwrap_or(Ordering::Equal)
    }
}

// Helper function for bounds checking
#[inline]
fn is_inside_bounds(x: i32, y: i32, z: i32, nx: usize, ny: usize, nz: usize) -> bool {
    x >= 0 && (x as usize) < nx && y >= 0 && (y as usize) < ny && z >= 0 && (z as usize) < nz
}

// Helper function to safely get array indices
#[inline]
fn safe_indices(
    x: i32,
    y: i32,
    z: i32,
    nx: usize,
    ny: usize,
    nz: usize,
) -> Option<(usize, usize, usize)> {
    if is_inside_bounds(x, y, z, nx, ny, nz) {
        Some((x as usize, y as usize, z as usize))
    } else {
        None
    }
}

#[pyfunction]
#[pyo3(signature = (injection_matrix, topography, depths, source, total_snapshots = 100))]
pub fn single_source_co2_fill_rust(
    py: Python<'_>,
    injection_matrix: PyReadonlyArray3<i32>,
    topography: PyReadonlyArray2<f64>, // Correct: 2D array with (nx, ny) shape
    depths: PyReadonlyArray1<f64>,
    source: (usize, usize),
    total_snapshots: usize,
) -> PyResult<Py<PyArray3<i32>>> {
    let injection_array = injection_matrix.as_array();
    let topography_array = topography.as_array();
    let depths_array = depths.as_array();

    let (nx, ny, nz) = injection_array.dim();
    let (xi, yi) = source;

    // Create mutable copy of injection matrix
    let mut injection = injection_array.to_owned();
    let mut visited = Array3::<bool>::default((nx, ny, nz));
    let mut snapshots = Array3::<i32>::from_elem((nx, ny, nz), -1);

    // Calculate snapshot interval
    let n_total_reservoir_cells: usize = injection_array
        .iter()
        .filter(|&&val| val == VELOCITY_RESERVOIR)
        .count();
    let snapshot_interval = std::cmp::max(1, n_total_reservoir_cells / total_snapshots);

    // Find injection start depth
    let depth_injection_start = topography_array[[xi, yi]];
    let mut zi = depths_array
        .iter()
        .enumerate()
        .min_by(|(_, &a), (_, &b)| {
            (a - depth_injection_start)
                .abs()
                .partial_cmp(&(b - depth_injection_start).abs())
                .unwrap()
        })
        .unwrap()
        .0
        + 1;

    // Validate source position
    if injection[[xi, yi, zi]] != VELOCITY_RESERVOIR {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Source must be in reservoir",
        ));
    }
    if zi > 0 && injection[[xi, yi, zi - 1]] != VELOCITY_CAPROCK {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Source must be just below caprock",
        ));
    }

    let mut snapshots_counter = 0;
    let mut cells_filled_since_snapshot = 0;

    while zi < nz {
        let mut heap = BinaryHeap::new();

        if is_inside_bounds(xi as i32, yi as i32, zi as i32, nx, ny, nz) {
            heap.push(HeapElement {
                depth: depths_array[zi],
                x: xi,
                y: yi,
                z: zi,
            });
        }

        while let Some(element) = heap.pop() {
            let (xi_curr, yi_curr, zi_curr) = (element.x, element.y, element.z);

            // Skip if already visited
            if visited[[xi_curr, yi_curr, zi_curr]] {
                continue;
            }

            // Mark as visited
            visited[[xi_curr, yi_curr, zi_curr]] = true;

            // Check if the cell can be filled with CO2
            if injection[[xi_curr, yi_curr, zi_curr]] == VELOCITY_RESERVOIR
                && (zi_curr == 0
                    || injection[[xi_curr, yi_curr, zi_curr - 1]] != VELOCITY_RESERVOIR)
            {
                injection[[xi_curr, yi_curr, zi_curr]] = VELOCITY_CO2;
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
                if injection[[xi_curr, yi_curr, zi_above]] == VELOCITY_RESERVOIR {
                    heap.push(HeapElement {
                        depth: depths_array[zi_above],
                        x: xi_curr,
                        y: yi_curr,
                        z: zi_above,
                    });
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
                        if injection[[x_new, y_new, z_new]] == VELOCITY_RESERVOIR {
                            heap.push(HeapElement {
                                depth: depths_array[z_new],
                                x: x_new,
                                y: y_new,
                                z: z_new,
                            });
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
                        if injection[[x_new, y_new, z_new]] != VELOCITY_CAPROCK {
                            heap.push(HeapElement {
                                depth: depths_array[z_new],
                                x: x_new,
                                y: y_new,
                                z: z_new,
                            });
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

/// A Python module implemented in Rust.
#[pymodule]
fn rust_backend(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(single_source_co2_fill_rust, m)?)?;
    Ok(())
}

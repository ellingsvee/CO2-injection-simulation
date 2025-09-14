use pyo3::prelude::*;

// The algorithm
use numpy::ndarray::Array3;
use numpy::{PyArray3, PyReadonlyArray1, PyReadonlyArray2, PyReadonlyArray3};
use ordered_float::OrderedFloat;
use std::collections::{BinaryHeap, HashMap, VecDeque};

// Velocity constants matching the Python implementation
const VELOCITY_CAPROCK: i32 = 2607;
const VELOCITY_RESERVOIR: i32 = 1500;
const VELOCITY_CO2: i32 = 300;

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

// Optimized data structure for depth-ordered processing
// Uses a heap for depth ordering and queues for cells at the same depth
struct DepthOrderedQueue {
    // Maps depth to queue of cells at that depth
    depth_queues: HashMap<OrderedFloat<f64>, VecDeque<(usize, usize, usize)>>,
    // Min-heap of depths (using Reverse for min-heap behavior)
    depth_heap: BinaryHeap<std::cmp::Reverse<OrderedFloat<f64>>>,
}

impl DepthOrderedQueue {
    fn new() -> Self {
        DepthOrderedQueue {
            depth_queues: HashMap::new(),
            depth_heap: BinaryHeap::new(),
        }
    }

    fn push(&mut self, depth: f64, x: usize, y: usize, z: usize) {
        let depth_key = OrderedFloat(depth);

        // Add to depth queue
        self.depth_queues
            .entry(depth_key)
            .or_default()
            .push_back((x, y, z));

        // Add depth to heap if not already present
        let reverse_depth = std::cmp::Reverse(depth_key);
        if !self.depth_heap.iter().any(|&d| d == reverse_depth) {
            self.depth_heap.push(reverse_depth);
        }
    }

    fn pop(&mut self) -> Option<(usize, usize, usize)> {
        while let Some(&std::cmp::Reverse(depth_key)) = self.depth_heap.peek() {
            if let Some(queue) = self.depth_queues.get_mut(&depth_key) {
                if let Some(cell) = queue.pop_front() {
                    return Some(cell);
                } else {
                    // Queue is empty, remove this depth
                    self.depth_queues.remove(&depth_key);
                    self.depth_heap.pop();
                }
            } else {
                // Shouldn't happen, but handle gracefully
                self.depth_heap.pop();
            }
        }
        None
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
pub fn _single_source_co2_fill_rust(
    py: Python<'_>,
    injection_matrix: PyReadonlyArray3<i32>,
    topography: PyReadonlyArray2<f64>,
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
        let mut queue = DepthOrderedQueue::new();

        if is_inside_bounds(xi as i32, yi as i32, zi as i32, nx, ny, nz) {
            queue.push(depths_array[zi], xi, yi, zi);
        }

        while let Some((xi_curr, yi_curr, zi_curr)) = queue.pop() {
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
                    queue.push(depths_array[zi_above], xi_curr, yi_curr, zi_above);
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
                            queue.push(depths_array[z_new], x_new, y_new, z_new);
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
                            queue.push(depths_array[z_new], x_new, y_new, z_new);
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

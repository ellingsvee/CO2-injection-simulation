use pyo3::prelude::*;

// The algorithm
use numpy::{PyArray1, PyReadonlyArray1, PyReadonlyArray2};
use ordered_float::OrderedFloat;
use std::collections::{BinaryHeap, HashMap, VecDeque};

// Velocity constants matching the Python implementation
const VELOCITY_CAPROCK: i32 = 2607;
const VELOCITY_RESERVOIR: i32 = 1500;
const VELOCITY_CO2: i32 = 300;

// 1D matrix wrapper for 3D indexing
struct Matrix3D1D {
    data: Vec<i32>,
    nx: usize,
    ny: usize,
    nz: usize,
}

impl Matrix3D1D {
    fn new(nx: usize, ny: usize, nz: usize) -> Self {
        Self {
            data: vec![0; nx * ny * nz],
            nx,
            ny,
            nz,
        }
    }

    fn from_vec(data: Vec<i32>, nx: usize, ny: usize, nz: usize) -> Self {
        assert_eq!(data.len(), nx * ny * nz);
        Self { data, nx, ny, nz }
    }

    #[inline]
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x * self.ny * self.nz + y * self.nz + z
    }

    #[inline]
    fn get(&self, x: usize, y: usize, z: usize) -> i32 {
        self.data[self.index(x, y, z)]
    }

    #[inline]
    fn set(&mut self, x: usize, y: usize, z: usize, value: i32) {
        let idx = self.index(x, y, z);
        self.data[idx] = value;
    }

    fn into_vec(self) -> Vec<i32> {
        self.data
    }
}

// 1D boolean matrix wrapper for visited tracking
struct BoolMatrix3D1D {
    data: Vec<bool>,
    nx: usize,
    ny: usize,
    nz: usize,
}

impl BoolMatrix3D1D {
    fn new(nx: usize, ny: usize, nz: usize) -> Self {
        Self {
            data: vec![false; nx * ny * nz],
            nx,
            ny,
            nz,
        }
    }

    #[inline]
    fn index(&self, x: usize, y: usize, z: usize) -> usize {
        x * self.ny * self.nz + y * self.nz + z
    }

    #[inline]
    fn get(&self, x: usize, y: usize, z: usize) -> bool {
        self.data[self.index(x, y, z)]
    }

    #[inline]
    fn set(&mut self, x: usize, y: usize, z: usize, value: bool) {
        let idx = self.index(x, y, z);
        self.data[idx] = value;
    }
}

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
#[pyo3(signature = (injection_matrix_flat, topography, depths, dimensions, source, total_snapshots = 100))]
pub fn _single_source_co2_fill_rust_1d(
    py: Python<'_>,
    injection_matrix_flat: PyReadonlyArray1<i32>,
    topography: PyReadonlyArray2<f64>,
    depths: PyReadonlyArray1<f64>,
    dimensions: (usize, usize, usize), // (nx, ny, nz)
    source: (usize, usize),
    total_snapshots: usize,
) -> PyResult<Py<PyArray1<i32>>> {
    let injection_flat = injection_matrix_flat.as_array();
    let topography_array = topography.as_array();
    let depths_array = depths.as_array();

    let (nx, ny, nz) = dimensions;
    let (xi, yi) = source;

    // Create 1D matrices from flat input
    let mut injection = Matrix3D1D::from_vec(injection_flat.to_vec(), nx, ny, nz);
    let mut visited = BoolMatrix3D1D::new(nx, ny, nz);
    let mut snapshots = Matrix3D1D::new(nx, ny, nz);
    
    // Initialize snapshots with -1 (unfilled)
    for x in 0..nx {
        for y in 0..ny {
            for z in 0..nz {
                snapshots.set(x, y, z, -1);
            }
        }
    }

    // Calculate snapshot interval
    let n_total_reservoir_cells: usize = injection_flat
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
    if injection.get(xi, yi, zi) != VELOCITY_RESERVOIR {
        return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
            "Source must be in reservoir",
        ));
    }
    if zi > 0 && injection.get(xi, yi, zi - 1) != VELOCITY_CAPROCK {
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
            if visited.get(xi_curr, yi_curr, zi_curr) {
                continue;
            }

            // Mark as visited
            visited.set(xi_curr, yi_curr, zi_curr, true);

            // Check if the cell can be filled with CO2
            if injection.get(xi_curr, yi_curr, zi_curr) == VELOCITY_RESERVOIR
                && (zi_curr == 0
                    || injection.get(xi_curr, yi_curr, zi_curr - 1) != VELOCITY_RESERVOIR)
            {
                injection.set(xi_curr, yi_curr, zi_curr, VELOCITY_CO2);
                snapshots.set(xi_curr, yi_curr, zi_curr, snapshots_counter);
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
                if injection.get(xi_curr, yi_curr, zi_above) == VELOCITY_RESERVOIR {
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
                        if injection.get(x_new, y_new, z_new) == VELOCITY_RESERVOIR {
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
                        if injection.get(x_new, y_new, z_new) != VELOCITY_CAPROCK {
                            queue.push(depths_array[z_new], x_new, y_new, z_new);
                        }
                    }
                }
            }
        }

        zi += 1;
    }

    // Convert result to 1D Python array
    let snapshots_vec = snapshots.into_vec();
    Ok(PyArray1::from_vec(py, snapshots_vec).into())
}

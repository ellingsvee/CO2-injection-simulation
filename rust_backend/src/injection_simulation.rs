use numpy::ndarray::{s, Array3, ArrayView1, ArrayView3};

use crate::constants::{VELOCITY_CAPROCK, VELOCITY_CO2, VELOCITY_RESERVOIR};
use crate::datastucture::DepthOrderedQueue;
use crate::utils::{
    find_closest_caprock_idx, find_height_to_caprock, is_caprock, is_empty, is_inside_bounds,
    safe_indices,
};

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

pub fn _injection_simulation_rust(
    reservoir_matrix: ArrayView3<f64>,
    depths: ArrayView1<f64>,
    max_column_height: usize,
    source: (usize, usize, usize),
    total_snapshots: usize,
) -> Array3<i32> {
    // let reservoir_matrix = reservoir_matrix.as_array();
    // let depths = depths.as_array();

    // Getting the dimensions
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
        panic!("Source must be in reservoir");
    }
    if zi > 0 && reservoir_matrix[[xi, yi, zi - 1]] != VELOCITY_CAPROCK {
        panic!("Source must be just below caprock");
    }

    let mut snapshots_counter = 0;
    let mut cells_filled_since_snapshot = 0;

    while zi < nz {
        println!("Current zi: {}", zi);

        let mut queue = DepthOrderedQueue::new();

        if is_inside_bounds(xi as i32, yi as i32, zi as i32, nx, ny, nz) {
            queue.push(depths[zi], (xi, yi, zi));
        }

        while let Some((xi_curr, yi_curr, zi_curr)) = queue.pop() {
            // Skip if already visited
            if visited[[xi_curr, yi_curr, zi_curr]] {
                continue;
            }

            // Mark as visited
            visited[[xi_curr, yi_curr, zi_curr]] = true;

            // Check if the cell can be filled with CO2
            if is_empty(reservoir_matrix[[xi_curr, yi_curr, zi_curr]])
                && (zi_curr == 0 || !is_empty(reservoir_matrix[[xi_curr, yi_curr, zi_curr - 1]]))
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
                if is_empty(reservoir_matrix[[xi_curr, yi_curr, zi_above]]) {
                    queue.push(depths[zi_above], (xi_curr, yi_curr, zi_above));
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
                        if is_empty(reservoir_matrix[[x_new, y_new, z_new]]) {
                            queue.push(depths[z_new], (x_new, y_new, z_new));
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
                        if !is_caprock(reservoir_matrix[[x_new, y_new, z_new]]) {
                            queue.push(depths[z_new], (x_new, y_new, z_new));
                        }
                    }
                }
            }

            let closest_caprock_idx = find_closest_caprock_idx(
                reservoir_matrix.slice(s![xi_curr, yi_curr, ..]), // Slice to get the z indices for (xi_curr, yi_curr)
                zi_curr,
            );

            // Check if the column height has reached the threshold where the caprock breaks
            if find_height_to_caprock(zi_curr, closest_caprock_idx) >= max_column_height {
                println!("Caprock breaks!");
                // Change the caprock cell from VELOCITY_CAPROCK to VELOCITY_RESERVOIR
                reservoir_matrix[[xi_curr, yi_curr, closest_caprock_idx]] = VELOCITY_RESERVOIR;

                // Add this cell to the heap
                queue.push(
                    depths[closest_caprock_idx],
                    (xi_curr, yi_curr, closest_caprock_idx),
                );
            }
        }

        zi += 1;
    }

    // Return the snapshots array
    snapshots
}

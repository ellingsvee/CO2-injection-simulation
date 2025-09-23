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

/// Validate that the initial source position is in the reservoir and just below caprock.
fn validate_initial_position(reservoir_matrix: &Array3<f64>, source: (usize, usize, usize)) {
    let (xi, yi, zi) = source;

    if reservoir_matrix[[xi, yi, zi]] != VELOCITY_RESERVOIR {
        panic!("Source must be in reservoir");
    }
    if zi > 0 && reservoir_matrix[[xi, yi, zi - 1]] != VELOCITY_CAPROCK {
        panic!("Source must be just below caprock");
    }
}

/// Compute the snapshot interval based on the total number of reservoir cells and desired total snapshots.
fn compute_snapshot_interval(reservoir_matrix: &Array3<f64>, total_snapshots: usize) -> usize {
    let n_total_reservoir_cells: usize = reservoir_matrix
        .iter()
        .filter(|&&val| val == VELOCITY_RESERVOIR)
        .count();
    std::cmp::max(1, n_total_reservoir_cells / total_snapshots)
}

/// Try to fill the cell with CO2 if it is empty and the cell below is not empty.
/// Update snapshots and counters accordingly.
fn try_to_fill_cell_with_co2(
    reservoir_matrix: &mut Array3<f64>,
    snapshots: &mut Array3<i32>,
    cell: (usize, usize, usize),
    snapshots_counter: &mut i32,
    cells_filled_since_snapshot: &mut usize,
    snapshot_interval: usize,
) {
    let (xi, yi, zi) = cell;

    // Check if the cell can be filled with CO2
    if is_empty(reservoir_matrix[[xi, yi, zi]])
        && (zi == 0 || !is_empty(reservoir_matrix[[xi, yi, zi - 1]]))
    {
        reservoir_matrix[[xi, yi, zi]] = VELOCITY_CO2;
        snapshots[[xi, yi, zi]] = *snapshots_counter;
        *cells_filled_since_snapshot += 1;

        // Take snapshot based on number of cells filled
        if *cells_filled_since_snapshot >= snapshot_interval {
            *snapshots_counter += 1;
            *cells_filled_since_snapshot = 0;
        }
    }
}

/// Add 8-connected neighbors to the queue if they are empty. Set cell_added to true if any cell is added.
fn add_to_8_connected_neighbors(
    queue: &mut DepthOrderedQueue,
    reservoir_matrix: &Array3<f64>,
    depths: &ArrayView1<f64>,
    current_cell: (usize, usize, usize),
    dims: (usize, usize, usize),
    cell_added: &mut bool,
) {
    let (xi_curr, yi_curr, zi_curr) = current_cell;
    let (nx, ny, nz) = dims;

    for &(dx, dy) in &SPREAD_DIRECTIONS {
        if let Some((x_new, y_new, z_new)) = safe_indices(
            xi_curr as i32 + dx,
            yi_curr as i32 + dy,
            zi_curr as i32,
            nx,
            ny,
            nz,
        ) {
            if is_empty(reservoir_matrix[[x_new, y_new, z_new]]) {
                queue.push(depths[z_new], (x_new, y_new, z_new));
                *cell_added = true;
            }
        }
    }
}

/// Check if the caprock breaks based on the column height of CO2. If it does, change the caprock cell to reservoir and add it to the queue.
fn try_to_break_caprock(
    queue: &mut DepthOrderedQueue,
    reservoir_matrix: &mut Array3<f64>,
    depths: &ArrayView1<f64>,
    current_cell: (usize, usize, usize),
    max_column_height: usize,
) {
    let (xi_curr, yi_curr, zi_curr) = current_cell;

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

pub fn _injection_simulation_rust(
    reservoir_matrix: ArrayView3<f64>,
    depths: ArrayView1<f64>,
    max_column_height: usize,
    source: (usize, usize, usize),
    total_snapshots: usize,
) -> Array3<i32> {
    // Getting the dimensions
    let (nx, ny, nz) = reservoir_matrix.dim();
    let (xi, yi, zi) = source;
    let mut zi = zi;

    // Create mutable copy of reservoir_matrix matrix
    let mut reservoir_matrix = reservoir_matrix.to_owned();
    let mut visited = Array3::<bool>::default((nx, ny, nz));
    let mut snapshots = Array3::<i32>::from_elem((nx, ny, nz), -1);

    // Calculate snapshot interval
    let snapshot_interval = compute_snapshot_interval(&reservoir_matrix, total_snapshots);

    // Validate source position
    validate_initial_position(&reservoir_matrix, source);

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

            // Check if the cell can be filled with CO2, and fill it if possible
            try_to_fill_cell_with_co2(
                &mut reservoir_matrix,
                &mut snapshots,
                (xi_curr, yi_curr, zi_curr),
                &mut snapshots_counter,
                &mut cells_filled_since_snapshot,
                snapshot_interval,
            );

            // Check if CO2 can move upward (9-connectivity neighbors above)
            let mut added_above = false;

            // Check directly above first
            if zi_curr > 0 {
                let zi_above = zi_curr - 1;
                if is_empty(reservoir_matrix[[xi_curr, yi_curr, zi_above]]) {
                    queue.push(depths[zi_above], (xi_curr, yi_curr, zi_above));
                    added_above = true;
                }

                add_to_8_connected_neighbors(
                    &mut queue,
                    &reservoir_matrix,
                    &depths,
                    (xi_curr, yi_curr, zi_above),
                    (nx, ny, nz),
                    &mut added_above,
                );
            }

            // If can't move up, spread horizontally
            if !added_above {
                let mut temp = false;
                add_to_8_connected_neighbors(
                    &mut queue,
                    &reservoir_matrix,
                    &depths,
                    (xi_curr, yi_curr, zi_curr),
                    (nx, ny, nz),
                    &mut temp,
                );
            }

            // Check the column height to see if the caprock breaks.
            try_to_break_caprock(
                &mut queue,
                &mut reservoir_matrix,
                &depths,
                (xi_curr, yi_curr, zi_curr),
                max_column_height,
            );
        }

        zi += 1;
    }

    // Return the snapshots array
    snapshots
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::datastucture::DepthOrderedQueue;
    use numpy::ndarray::{Array1, Array3};

    fn make_test_reservoir(nx: usize, ny: usize, nz: usize, fill: f64) -> Array3<f64> {
        Array3::<f64>::from_elem((nx, ny, nz), fill)
    }

    #[test]
    #[should_panic(expected = "Source must be in reservoir")]
    fn test_validate_initial_position_panics_if_not_reservoir() {
        let reservoir = make_test_reservoir(3, 3, 3, VELOCITY_CAPROCK);
        validate_initial_position(&reservoir, (1, 1, 1));
    }

    #[test]
    #[should_panic(expected = "Source must be just below caprock")]
    fn test_validate_initial_position_panics_if_not_below_caprock() {
        let mut reservoir = make_test_reservoir(3, 3, 3, VELOCITY_RESERVOIR);
        reservoir[[1, 1, 0]] = VELOCITY_RESERVOIR; // not caprock above
        validate_initial_position(&reservoir, (1, 1, 1));
    }

    #[test]
    fn test_compute_snapshot_interval() {
        let reservoir = make_test_reservoir(2, 2, 2, VELOCITY_RESERVOIR);
        assert_eq!(compute_snapshot_interval(&reservoir, 4), 2); // 8/4 = 2
        assert_eq!(compute_snapshot_interval(&reservoir, 20), 1); // max(1, ..)
    }

    #[test]
    fn test_try_to_fill_cell_with_co2_fills_correctly() {
        let mut reservoir = make_test_reservoir(2, 2, 2, VELOCITY_RESERVOIR);
        reservoir[[0, 0, 0]] = VELOCITY_CAPROCK; // caprock above (0,0,1)
        let mut snapshots = Array3::<i32>::from_elem((2, 2, 2), -1);
        let mut snapshots_counter = 0;
        let mut cells_filled_since_snapshot = 0;

        try_to_fill_cell_with_co2(
            &mut reservoir,
            &mut snapshots,
            (0, 0, 1),
            &mut snapshots_counter,
            &mut cells_filled_since_snapshot,
            1,
        );

        assert_eq!(reservoir[[0, 0, 1]], VELOCITY_CO2);
        assert_eq!(snapshots[[0, 0, 1]], 0);
        assert_eq!(snapshots_counter, 1); // snapshot interval hit
    }

    #[test]
    fn test_add_to_8_connected_neighbors() {
        let mut reservoir = make_test_reservoir(3, 3, 1, VELOCITY_RESERVOIR);
        reservoir[[1, 1, 0]] = VELOCITY_CO2; // already filled
        let depths = Array1::from(vec![0.0]);
        let mut queue = DepthOrderedQueue::new();
        let mut added = false;

        add_to_8_connected_neighbors(
            &mut queue,
            &reservoir,
            &depths.view(),
            (1, 1, 0),
            (3, 3, 1),
            &mut added,
        );

        // Should add some neighbors (all empty)
        assert!(added);
        assert!(!queue.is_empty());
        assert!(queue.len() == 8); // Note, the original cell is not added itself. Therefore 9 - 1 = 8
    }

    #[test]
    fn test_try_to_break_caprock() {
        let mut reservoir = make_test_reservoir(2, 2, 3, VELOCITY_RESERVOIR);
        reservoir[[0, 0, 1]] = VELOCITY_CAPROCK;
        let depths = Array1::from(vec![0.0, 1.0, 2.0]);
        let mut queue = DepthOrderedQueue::new();

        // Place CO2 below caprock
        reservoir[[0, 0, 2]] = VELOCITY_CO2;

        try_to_break_caprock(&mut queue, &mut reservoir, &depths.view(), (0, 0, 2), 1);

        // Caprock at [0,0,1] should have turned into reservoir
        assert_eq!(reservoir[[0, 0, 1]], VELOCITY_RESERVOIR);
        assert!(!queue.is_empty());
    }
}

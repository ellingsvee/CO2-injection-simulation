use crate::constants::{VELOCITY_CAPROCK, VELOCITY_RESERVOIR};
use numpy::ndarray::ArrayView1;

/// Helper function for bounds checking
#[inline]
pub fn is_inside_bounds(x: i32, y: i32, z: i32, nx: usize, ny: usize, nz: usize) -> bool {
    x >= 0 && (x as usize) < nx && y >= 0 && (y as usize) < ny && z >= 0 && (z as usize) < nz
}

/// Helper function to safely get array indices
#[inline]
pub fn safe_indices(
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

/// Helper function to check that the cell is caprock
#[inline]
pub fn is_caprock(val: f64) -> bool {
    val == VELOCITY_CAPROCK
}

/// Helper function to check that the cell is unfilled
#[inline]
pub fn is_empty(val: f64) -> bool {
    val == VELOCITY_RESERVOIR
}

/// Find the number of cells from the current index to the nearest caprock
#[inline]
pub fn find_height_to_caprock(zi: usize, caprock_idx: usize) -> usize {
    zi - caprock_idx
}

/// Find the index of the closest layer with VELOCITY_CAPROCK below or at zi
#[inline]
pub fn find_closest_caprock_idx(reservoir_matrix_column: ArrayView1<f64>, zi: usize) -> usize {
    reservoir_matrix_column
        .iter()
        .enumerate()
        .rfind(|&(_idx, &val)| val == VELOCITY_CAPROCK && _idx <= zi)
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    use numpy::ndarray::array;

    #[test]
    fn test_is_inside_bounds() {
        assert!(is_inside_bounds(0, 0, 0, 10, 10, 10));
        assert!(is_inside_bounds(9, 9, 9, 10, 10, 10));
        assert!(!is_inside_bounds(-1, 0, 0, 10, 10, 10));
        assert!(!is_inside_bounds(10, 0, 0, 10, 10, 10));
        assert!(!is_inside_bounds(0, 10, 0, 10, 10, 10));
        assert!(!is_inside_bounds(0, 0, 10, 10, 10, 10));
    }

    #[test]
    fn test_safe_indices() {
        assert_eq!(safe_indices(0, 0, 0, 10, 10, 10), Some((0, 0, 0)));
        assert_eq!(safe_indices(9, 9, 9, 10, 10, 10), Some((9, 9, 9)));
        assert_eq!(safe_indices(-1, 0, 0, 10, 10, 10), None);
        assert_eq!(safe_indices(10, 0, 0, 10, 10, 10), None);
    }

    #[test]
    fn test_is_caprock_and_is_empty() {
        assert!(is_caprock(VELOCITY_CAPROCK));
        assert!(!is_caprock(VELOCITY_RESERVOIR));

        assert!(is_empty(VELOCITY_RESERVOIR));
        assert!(!is_empty(VELOCITY_CAPROCK));
    }

    #[test]
    fn test_find_height_to_caprock() {
        assert_eq!(find_height_to_caprock(10, 7), 3);
        assert_eq!(find_height_to_caprock(5, 0), 5);
        assert_eq!(find_height_to_caprock(0, 0), 0);
    }

    #[test]
    fn test_find_closest_layer_idx() {
        let column = array![
            VELOCITY_RESERVOIR,
            VELOCITY_RESERVOIR,
            VELOCITY_CAPROCK,
            VELOCITY_RESERVOIR,
            VELOCITY_CAPROCK
        ];

        // should find the last caprock at or before zi = 4
        assert_eq!(find_closest_caprock_idx(column.view(), 4), 4);

        // should find caprock at index 2
        assert_eq!(find_closest_caprock_idx(column.view(), 3), 2);

        // no caprock before zi=1 → returns 0
        assert_eq!(find_closest_caprock_idx(column.view(), 1), 0);

        // zi at 0 → no caprock at/below, return 0
        assert_eq!(find_closest_caprock_idx(column.view(), 0), 0);
    }
}

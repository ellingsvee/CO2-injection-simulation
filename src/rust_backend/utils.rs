// Helper function for bounds checking
#[inline]
pub fn is_inside_bounds(x: i32, y: i32, z: i32, nx: usize, ny: usize, nz: usize) -> bool {
    x >= 0 && (x as usize) < nx && y >= 0 && (y as usize) < ny && z >= 0 && (z as usize) < nz
}

// Helper function to safely get array indices
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

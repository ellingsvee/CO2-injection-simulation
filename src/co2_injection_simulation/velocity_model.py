import numpy as np
from heapq import heappush, heappop
from typing import Tuple

from co2_injection_simulation import VELOCITY_RESERVOIR, VELOCITY_CO2, VELOCITY_CAPROCK
from co2_injection_simulation.rust_backend import (
    _single_source_co2_fill_rust_with_buckets,
)


def _single_source_co2_fill(
    injection_matrix: np.ndarray,  # (nx, ny, nz)
    topography: np.ndarray,  # (nx, ny)
    depths: np.ndarray,  # (nz,)
    source: Tuple[int, int],  # (x, y)
    total_snapshots: int = 100,  # Number of snapshots to capture
) -> np.ndarray:  # (nx, ny, nz, total_snapshots)
    """
    Simulate upward migration of CO2 through a caprock structure.

    Args:
        injection_matrix: The 3D matrix that will be filled with CO2. This matrix must be initialized by the map_topography_to_velocities-function.
        total_snapshots: Number of snapshots to capture during the filling process

    Returns:
        snapshots: ...
    """
    print("Debug: Starting CO2 migration simulation")
    # Retrieve the dimentions
    nx, ny, nz = injection_matrix.shape

    visited = np.zeros_like(injection_matrix, dtype=bool)

    # Calculate snapshot interval based on total reservoir cells
    n_total_reservoir_cells = np.sum(injection_matrix == VELOCITY_RESERVOIR)
    snapshot_interval = max(1, n_total_reservoir_cells // total_snapshots)
    print(f"Debug: Will take snapshot every {snapshot_interval} cells filled")

    # Store snapshots of the states
    snapshots = (
        np.ones_like(injection_matrix, dtype=int) * -1
    )  # Initialize with -1 (unfilled)

    # The first indexes will be at the source
    xi, yi = source

    # Find the z-index in the injection matrix where we start injection
    # Add one to get one cell deeper than caprock
    depth_injection_start = topography[source]
    # Find the closest depth index instead of exact match
    zi = np.argmin(np.abs(depths - depth_injection_start)) + 1

    assert injection_matrix[xi, yi, zi] == VELOCITY_RESERVOIR, (
        "Source must be in reservoir"
    )
    assert injection_matrix[xi, yi, zi - 1] == VELOCITY_CAPROCK, (
        "Source must be just below caprock"
    )

    # Spread directions
    SPREAD_DIRECTIONS = [
        (-1, 0),
        (1, 0),
        (0, -1),
        (0, 1),
        (-1, -1),
        (-1, 1),
        (1, -1),
        (1, 1),
    ]

    # Inline bounds checking function
    def is_inside_bounds(x, y, z):
        return 0 <= x < nx and 0 <= y < ny and 0 <= z < nz

    # Continue until the entire reservoir is filled
    snapshots_counter = 0
    cells_filled_since_snapshot = 0

    while zi < nz:
        print(f"Debug: Starting to fill at depth index {zi}/{nz}")
        # Create the heap. The parent will always have a smaller value at the first element in the tuple
        heap = []
        if is_inside_bounds(xi, yi, zi):
            heappush(heap, (depths[zi], xi, yi, zi))

        # Continue as long as the heap is not empty
        while heap:
            # Get the element with the smallest depth
            _, xi, yi, zi = heappop(heap)

            # Skip if already visited
            if visited[xi, yi, zi]:
                continue

            # Skip if out of bounds
            if not (0 <= xi < nx and 0 <= yi < ny and 0 <= zi < nz):
                continue

            # Mark as visited
            visited[xi, yi, zi] = True

            # Check if the cell can be filled with CO2
            if (
                injection_matrix[xi, yi, zi] == VELOCITY_RESERVOIR
                and injection_matrix[xi, yi, zi - 1]
                != VELOCITY_RESERVOIR  # Either caprock or already filled with CO2
            ):
                injection_matrix[xi, yi, zi] = VELOCITY_CO2
                snapshots[xi, yi, zi] = snapshots_counter
                cells_filled_since_snapshot += 1

                # Take snapshot based on number of cells filled
                if cells_filled_since_snapshot >= snapshot_interval:
                    snapshots_counter += 1
                    cells_filled_since_snapshot = 0
                    # print(f"Debug: Snapshot {snapshots_counter} taken at depth {depths[zi]:.3f}m")

            # Check if CO2 can move upward
            # 9-connectivity neighbors above
            added_above = False  # To see if we instead will spread horizonally
            for dx, dy in [(0, 0)] + SPREAD_DIRECTIONS:
                # If not caprock or already filled, we add to the heap
                if injection_matrix[
                    xi + dx, yi + dy, zi - 1
                ] == VELOCITY_RESERVOIR and is_inside_bounds(xi + dx, yi + dy, zi - 1):
                    heappush(heap, (depths[zi - 1], xi + dx, yi + dy, zi - 1))
                    added_above = True
            if not added_above:
                for dx, dy in SPREAD_DIRECTIONS:
                    if injection_matrix[
                        xi + dx, yi + dy, zi
                    ] != VELOCITY_CAPROCK and is_inside_bounds(xi + dx, yi + dy, zi):
                        heappush(heap, (depths[zi], xi + dx, yi + dy, zi))

        # Increase the depth when heap is empty
        zi += 1

    return snapshots


def single_source_co2_fill(
    injection_matrix: np.ndarray,  # (nx, ny, nz)
    topography: np.ndarray,  # (nx, ny)
    depths: np.ndarray,  # (nz,)
    source: Tuple[int, int],  # (x, y)
    total_snapshots: int = 100,  # Number of snapshots to capture
    rust_implementation: bool = False,
) -> np.ndarray:  # (nx, ny, nz, total_snapshots)
    if rust_implementation:
        # Ensure arrays have the correct data types for Rust
        injection_matrix_i32 = injection_matrix.astype(np.int32)
        topography_f64 = topography.astype(np.float64)
        depths_f64 = depths.astype(np.float64)

        # Ensure arrays are contiguous
        injection_matrix_i32 = np.ascontiguousarray(injection_matrix_i32)
        topography_f64 = np.ascontiguousarray(topography_f64)
        depths_f64 = np.ascontiguousarray(depths_f64)

        print("Debug: Using Rust implementation")
        return _single_source_co2_fill_rust_with_buckets(
            injection_matrix_i32, topography_f64, depths_f64, source, total_snapshots
        )

    else:
        return _single_source_co2_fill(
            injection_matrix, topography, depths, source, total_snapshots
        )

# from typing import Tuple
#
# import numpy as np
# from numpy.typing import NDArray
#
# from co2_injection_simulation.rust_backend import (
#     _leakage_model_rust,
# )
# from co2_injection_simulation import VELOCITY_CAPROCK, VELOCITY_RESERVOIR
#
#
# def leakage_model(
#     injection_matrix: NDArray[np.int32],  # (nx, ny, nz)
#     depths: NDArray[np.float64],  # (nz,)
#     source: Tuple[int, int, int],  # (x, y, z)
#     total_snapshots: int = 100,  # Number of snapshots to capture
# ) -> NDArray[np.int32]:  # (nx, ny, nz)
#     # Ensure arrays have the correct data types for Rust
#     injection_matrix_i32 = injection_matrix.astype(np.int32)
#     depths_f64 = depths.astype(np.float64)
#
#     # Ensure arrays are contiguous
#     injection_matrix_i32 = np.ascontiguousarray(injection_matrix_i32)
#     depths_f64 = np.ascontiguousarray(depths_f64)
#
#     assert injection_matrix[source] == VELOCITY_RESERVOIR, "Source must be in reservoir region"
#     assert injection_matrix[source[0], source[1], source[2] - 1] == VELOCITY_CAPROCK, "Source must be just below caprock"
#
#     print("Debug: Using Rust implementation")
#     return _leakage_model_rust(
#         injection_matrix_i32,
#         depths_f64,
#         source,
#         total_snapshots,
#     )

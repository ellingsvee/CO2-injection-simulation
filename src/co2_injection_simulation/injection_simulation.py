from typing import Tuple

import numpy as np
from numpy.typing import NDArray

from co2_injection_simulation.rust_backend import _injection_simulation_python_wrapper


def injection_simulation(
    reservoir_matrix: NDArray[np.float64],  # (nx, ny, nz)
    depths: NDArray[np.float64],  # (nz,)
    bedrock_indices: NDArray[np.int32],  # (nx, ny)
    max_column_height: int,
    source: Tuple[int, int, int],
    total_snapshots: int = 100,  # Number of snapshots to capture
) -> NDArray[np.int32]:  # (nx, ny, nz)
    # Assure the arrays are correct. Unsure if this really is needed.
    reservoir_matrix = reservoir_matrix.astype(np.float64)
    reservoir_matrix = np.ascontiguousarray(reservoir_matrix)
    depths = depths.astype(np.float64)
    depths = np.ascontiguousarray(depths)
    bedrock_indices = bedrock_indices.astype(np.int32)
    bedrock_indices = np.ascontiguousarray(bedrock_indices)

    snapshots = _injection_simulation_python_wrapper(
        reservoir_matrix=reservoir_matrix,
        depths=depths,
        bedrock_indices=bedrock_indices,
        max_column_height=max_column_height,
        source=source,
        total_snapshots=total_snapshots,
    )

    return snapshots

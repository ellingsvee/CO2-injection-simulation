from typing import Tuple

import numpy as np
from numpy.typing import NDArray

from co2_injection_simulation.rust_backend import _injection_simulation_rust


def injection_simulation(
    reservoir_matrix: NDArray[np.float64],  # (nx, ny, nz)
    depths: NDArray[np.float64],  # (nz,)
    source: Tuple[int, int, int],
    total_snapshots: int = 100,  # Number of snapshots to capture
) -> NDArray[np.float64]:  # (nx, ny, nz)
    reservoir_matrix = reservoir_matrix.astype(np.float64)
    reservoir_matrix = np.ascontiguousarray(reservoir_matrix)

    depths = depths.astype(np.float64)
    depths = np.ascontiguousarray(depths)

    snapshots = _injection_simulation_rust(
        reservoir_matrix=reservoir_matrix,
        depths=depths,
        source=source,
        total_snapshots=total_snapshots,
    )

    return snapshots

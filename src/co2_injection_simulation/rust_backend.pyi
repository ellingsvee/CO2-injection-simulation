from typing import Tuple

import numpy as np
from numpy.typing import NDArray

def _single_source_co2_fill_rust(
    injection_matrix: NDArray[np.int32],
    topography: NDArray[np.float64],
    depths: NDArray[np.float64],
    source: Tuple[int, int],
    total_snapshots: int = 100,
) -> NDArray[np.int32]: ...

def _single_source_co2_fill_rust_1d(
    injection_matrix_flat: NDArray[np.int32],
    topography: NDArray[np.float64],
    depths: NDArray[np.float64],
    dimensions: Tuple[int, int, int],
    source: Tuple[int, int],
    total_snapshots: int = 100,
) -> NDArray[np.int32]: ...

from typing import Tuple

import numpy as np
from numpy.typing import NDArray

def _injection_simulation_rust(
    reservoir_matrix: NDArray[np.float64],
    depths: NDArray[np.float64],
    max_column_height: int,
    source: Tuple[int, int, int],
    total_snapshots: int = 100,
) -> NDArray[np.int32]: ...

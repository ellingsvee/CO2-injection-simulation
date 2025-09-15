from typing import Tuple

import numpy as np
from numpy.typing import NDArray
from scipy.io import loadmat

from co2_injection_simulation import PROJECT_ROOT


def retrieve_sleipner_topography(
    nz: int = 100,
) -> Tuple[NDArray[np.float64], NDArray[np.float64]]:
    # Load the data

    mat_file = PROJECT_ROOT / "zzz.mat"  # Desktop
    data = loadmat(mat_file)

    # Unsure why we bother to add the thickness
    model_thickness = 0.2
    caprock_topography = np.flip(data["zzz"] + model_thickness)

    # Discretize the caprock topography to nz distinct values
    min_val = np.min(caprock_topography)
    max_val = np.max(caprock_topography)
    depths = np.linspace(min_val, max_val, nz + 1)
    caprock_topography_discrete = np.digitize(caprock_topography, depths[:-1])
    # Map bin indices back to representative values
    caprock_topography_discrete = depths[caprock_topography_discrete]

    return caprock_topography_discrete, depths

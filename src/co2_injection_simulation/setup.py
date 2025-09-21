import numpy as np
from numpy.typing import NDArray
from scipy.io import loadmat

from co2_injection_simulation import PROJECT_ROOT


def retrieve_sleipner_topography() -> NDArray[np.float64]:
    # Load the data

    mat_file = PROJECT_ROOT / "zzz.mat"  # Desktop
    data = loadmat(mat_file)

    # Unsure why we bother to add the thickness
    model_thickness = 0.2
    caprock_topography = np.flip(data["zzz"] + model_thickness)

    return caprock_topography


def build_sleipner_layers(
    displacements: NDArray[np.float64],
    base_topography: NDArray[np.float64],
    nz: int = 100,
):
    nrow, ncol = base_topography.shape
    N = displacements.shape[0]

    layers = np.zeros((nrow, ncol, N), dtype=np.float64)

    for i in range(N):
        layers[:, :, i] = base_topography + displacements[i]

    layers, depths = bin_depths(layers, nz)
    return layers, depths


def bin_depths(layers: NDArray[np.float64], nz: int):
    min_val = np.min(layers)
    max_val = np.max(layers)
    depths = np.linspace(min_val, max_val, nz + 1)
    binned_layers = np.empty_like(layers)
    for i in range(layers.shape[2]):
        depth_indices = np.searchsorted(depths, layers[:, :, i], side="left")
        binned_layers[:, :, i] = depths[depth_indices]
    return binned_layers, depths


if __name__ == "__main__":
    caprock_topography = retrieve_sleipner_topography()
    displacements = np.array([0.05, 0.1, 0.20])
    layers, depths = build_sleipner_layers(displacements, caprock_topography, nz=100)
    print(layers)
    print(depths)

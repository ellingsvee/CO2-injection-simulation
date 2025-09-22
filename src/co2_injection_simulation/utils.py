import numpy as np
from numpy.typing import NDArray

from co2_injection_simulation import VELOCITY_CAPROCK, VELOCITY_CO2, VELOCITY_RESERVOIR


def map_layers_to_caprock_matrix(
    layers: NDArray[np.float64], depths: NDArray[np.float64], layer_thickness: int = 1
) -> NDArray[np.float64]:
    nrow, ncol, N = layers.shape
    nz = depths.shape[0]

    caprock_matrix = np.full((nrow, ncol, nz), VELOCITY_RESERVOIR, dtype=np.float64)

    depths_expanded = depths[np.newaxis, np.newaxis, :]
    i_idx, j_idx = np.meshgrid(np.arange(nrow), np.arange(ncol), indexing="ij")
    for i in range(N):
        layer_expanded = layers[:, :, i][:, :, np.newaxis]
        idx = np.abs(depths_expanded - layer_expanded).argmin(axis=2)
        for offset in range(layer_thickness):
            caprock_indices = np.clip(idx + offset, 0, nz - 1)
            caprock_matrix[i_idx, j_idx, caprock_indices] = VELOCITY_CAPROCK

    return caprock_matrix


def get_reservoir_from_snapshot(
    caprock_matrix: NDArray[np.float64],
    snapshots: NDArray[np.float64],
    snapshot_value: int,
):
    reservoir_matrix = np.copy(caprock_matrix)
    indexes = np.where((snapshots <= snapshot_value) & (snapshots != -1))
    reservoir_matrix[indexes] = VELOCITY_CO2
    return reservoir_matrix


def find_z_index(depths: NDArray[np.float64], target_depth: np.float64) -> np.uint:
    return np.uint(np.argmin(np.abs(depths - target_depth)))


def compute_layers_idx(
    layers: NDArray[np.float64], depths: NDArray[np.float64]
) -> NDArray[np.uint]:
    diff = np.abs(layers[..., np.newaxis] - depths)
    layers_idx = np.argmin(diff, axis=-1).astype(np.int32)
    return layers_idx

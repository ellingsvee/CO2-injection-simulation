import numpy as np
from numpy.typing import NDArray

from co2_injection_simulation import (
    VELOCITY_CAPROCK,
    VELOCITY_CO2,
    VELOCITY_RESERVOIR,
)


def map_topography_to_velocities(
    caprock_topography: NDArray[np.float64],
    depths: NDArray[np.float64],
) -> NDArray[np.int32]:
    velocity_matrix = np.zeros(
        (caprock_topography.shape[0], caprock_topography.shape[1], depths.shape[0]),
        dtype=np.int32,
    )

    caprock_topography_expanded = caprock_topography[
        :, :, np.newaxis
    ]  # shape (rows, cols, 1)
    depth_expanded = depths[np.newaxis, np.newaxis, :]  # shape (1, 1, nz)
    mask = depth_expanded <= caprock_topography_expanded  # shape (rows, cols, nz)
    velocity_matrix[mask] = VELOCITY_CAPROCK
    velocity_matrix[~mask] = VELOCITY_RESERVOIR
    return velocity_matrix


def get_matrix_from_snapshot(
    caprock_topography: NDArray[np.float64],
    depths: NDArray[np.float64],
    snapshots: NDArray[np.int32],
    snapshot_value: int,
) -> NDArray[np.int32]:
    # Generate the velocity matrix
    injection_matrix = map_topography_to_velocities(caprock_topography, depths=depths)

    # Find the indexes where snapshot has value equal to the snapshot_value
    indexes = np.where((snapshots <= snapshot_value) & (snapshots != -1))

    # Set the velocity at these indexes to VELOCITY_CO2
    injection_matrix[indexes] = VELOCITY_CO2

    return injection_matrix

import numpy as np
from co2_injection_simulation import VELOCITY_CO2
from co2_injection_simulation import PROJECT_ROOT, VELOCITY_CAPROCK, VELOCITY_RESERVOIR

def map_topography_to_velocities(
    caprock_topography: np.ndarray,
    depths: np.ndarray,
) -> np.ndarray:
    velocity_matrix = np.zeros(
        (caprock_topography.shape[0], caprock_topography.shape[1], depths.shape[0])
    )

    caprock_topography_expanded = caprock_topography[
        :, :, np.newaxis
    ]  # shape (rows, cols, 1)
    depth_expanded = depths[np.newaxis, np.newaxis, :]  # shape (1, 1, nz)
    mask = depth_expanded <= caprock_topography_expanded  # shape (rows, cols, nz)
    velocity_matrix[mask] = VELOCITY_CAPROCK
    velocity_matrix[~mask] = VELOCITY_RESERVOIR
    return velocity_matrix

def get_matrix_from_snapshot(caprock_topography: np.ndarray, depths: np.ndarray,snapshots: np.ndarray, snapshot_value: int):
    # Generate the velocity matrix
    injection_matrix = map_topography_to_velocities(caprock_topography, depths=depths)

    # Find the indexes where snapshot has value equal to the snapshot_value
    indexes = np.where((snapshots <= snapshot_value) & (snapshots != -1))

    # Set the velocity at these indexes to VELOCITY_CO2
    injection_matrix[indexes] = VELOCITY_CO2

    return injection_matrix
    

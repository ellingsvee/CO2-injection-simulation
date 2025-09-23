import os
import time

import numpy as np

from co2_injection_simulation import PROJECT_ROOT, VELOCITY_CAPROCK, VELOCITY_RESERVOIR
from co2_injection_simulation.injection_simulation import injection_simulation
from co2_injection_simulation.utils import compute_layers_idx

if __name__ == "__main__":
    simulations_dir = PROJECT_ROOT / "simulations"
    os.makedirs(simulations_dir, exist_ok=True)
    layers = np.load(simulations_dir / "layers.npy")
    depths = np.load(simulations_dir / "depths.npy")
    caprock_matrix = np.load(simulations_dir / "caprock_matrix.npy")

    # NB: Have to make sure this is the same as in setup_domain.py
    layer_thickness = 5

    xi = caprock_matrix.shape[0] // 2
    yi = caprock_matrix.shape[1] // 2

    layers_idx = compute_layers_idx(layers, depths, layer_thickness=layer_thickness)
    zi = int(layers_idx[xi, yi, -1]) + 1

    print(f"Source coordinates: {(xi, yi, zi)}")

    assert caprock_matrix[xi, yi, zi] == VELOCITY_RESERVOIR, (
        "Error: Source not in reservoir"
    )
    assert caprock_matrix[xi, yi, zi - 1] == VELOCITY_CAPROCK, (
        "Error: Source not directly below caprock"
    )

    print("Running injection simulation:")
    start_time = time.time()
    snapshots = injection_simulation(
        reservoir_matrix=caprock_matrix,
        depths=depths,
        bedrock_indices=layers_idx[:, :, 0],
        max_column_height=20,
        source=(xi, yi, zi),
        total_snapshots=400,
    )
    end_time = time.time()
    print(f"Simulation completed in {end_time - start_time:.2f} seconds.")

    print("Saving snapshots:")
    np.save(simulations_dir / "snapshots.npy", snapshots)

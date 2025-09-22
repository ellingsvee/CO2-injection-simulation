import os
import time

import numpy as np

from co2_injection_simulation import PROJECT_ROOT, VELOCITY_CAPROCK, VELOCITY_RESERVOIR
from co2_injection_simulation.injection_simulation import injection_simulation
from co2_injection_simulation.utils import compute_layers_idx

simulations_dir = PROJECT_ROOT / "simulations"
os.makedirs(simulations_dir, exist_ok=True)
layers = np.load(simulations_dir / "layers.npy")
depths = np.load(simulations_dir / "depths.npy")
caprock_matrix = np.load(simulations_dir / "caprock_matrix.npy")
layers_idx = compute_layers_idx(layers, depths)


layer_thickness = 5
xi = caprock_matrix.shape[0] // 2
yi = caprock_matrix.shape[1] // 2
zi = int(layers_idx[xi, yi, -1]) + layer_thickness

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
    max_column_height=5,
    source=(xi, yi, zi),
    total_snapshots=200,
)
end_time = time.time()
print(f"Simulation completed in {end_time - start_time:.2f} seconds.")

print("Saving snapshots:")
np.save(simulations_dir / "snapshots.npy", snapshots)

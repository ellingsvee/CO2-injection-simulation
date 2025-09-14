from co2_injection_simulation.setup import (
    retrieve_sleipner_topography,
)
from co2_injection_simulation.utils import (
    map_topography_to_velocities,
)

from co2_injection_simulation import PROJECT_ROOT
from co2_injection_simulation.velocity_model import single_source_co2_fill
import numpy as np
import time


# Set constants
nz = 100
xy_extent = (3, 1)


# Retrieve the model
print("Debug: " + "Retireving Sleipner Model")

caprock_topography, depths = retrieve_sleipner_topography(nz=nz)
caprock_matrix = map_topography_to_velocities(caprock_topography, depths=depths)

# Run the simulation
tic = time.time()
snapshots = single_source_co2_fill(
    injection_matrix=caprock_matrix.copy(),
    topography=caprock_topography,
    depths=depths,
    source=(caprock_topography.shape[0] // 2, caprock_topography.shape[1] // 2),
    rust_implementation=True,  # Choose wether or not to use the rust_implementation
)
toc = time.time()
total_time = toc - tic
print("Debug: " + f"Simulation finished in {total_time:.2} sec!")

# Save the snapshots array
print("Debug: " + "Saving snapshots array")
np.save(PROJECT_ROOT / "simulations" / "flood_fill_snapshots.npy", snapshots)

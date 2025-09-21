import os

import numpy as np

from co2_injection_simulation import PROJECT_ROOT
from co2_injection_simulation.plot import plot_cross_section
from co2_injection_simulation.setup import (
    build_sleipner_layers,
    retrieve_sleipner_topography,
)
from co2_injection_simulation.utils import map_layers_to_caprock_matrix

if __name__ == "__main__":
    print("Setting up the matrices:")
    caprock_topography = retrieve_sleipner_topography()
    displacements = np.array([0.05, 0.1, 0.20])
    layers, depths = build_sleipner_layers(displacements, caprock_topography, nz=100)
    caprock_matrix = map_layers_to_caprock_matrix(
        layers=layers, depths=depths, layer_thickness=5
    )

    # Save the matrices
    print("Saving matrices:")
    simulations_dir = PROJECT_ROOT / "simulations"
    os.makedirs(simulations_dir, exist_ok=True)
    np.save(simulations_dir / "layers.npy", layers)
    np.save(simulations_dir / "depths.npy", depths)
    np.save(simulations_dir / "caprock_matrix.npy", caprock_matrix)

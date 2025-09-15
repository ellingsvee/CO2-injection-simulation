import os

import numpy as np

from co2_injection_simulation import PROJECT_ROOT
from co2_injection_simulation.plot import (
    plot_birdseye_animation,
    plot_cross_section_animation,
)
from co2_injection_simulation.setup import (
    retrieve_sleipner_topography,
)

# Load the snapshots from the simulation
print("Debug: " + "Loading snapshots array")
snapshots = np.load(PROJECT_ROOT / "simulations" / "snapshots.npy")

# Set constants
nz = 100
caprock_topography, depths = retrieve_sleipner_topography(nz=nz)

# Create and save the animation
plots_dir = PROJECT_ROOT / "plots"
os.makedirs(plots_dir, exist_ok=True)
plot_cross_section_animation(
    index=caprock_topography.shape[1] // 2,
    snapshots=snapshots,
    caprock_topography=caprock_topography,
    depths=depths,
    save_path=plots_dir / "cross_section.gif",
    show=False,
)
plot_birdseye_animation(
    snapshots=snapshots,
    caprock_topography=caprock_topography,
    depths=depths,
    save_path=plots_dir / "birdseye.gif",
    show=False,
)

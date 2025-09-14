from co2_injection_simulation.setup import (
    retrieve_sleipner_topography,
)
from co2_injection_simulation.utils import (
    map_topography_to_velocities,
)
from co2_injection_simulation.plot import (
    plot_birdseye,
    plot_cross_section,
    plot_cross_section_as_lineplot,
)

from co2_injection_simulation import PROJECT_ROOT
from co2_injection_simulation.velocity_model import single_source_co2_fill
import numpy as np


# Set constants
nz = 100
xy_extent = (3, 1)


# water_column = 0
# velocity_reservoir = 1500
# velocity_CO2 = 300
# velocity_caprock = 2607
# basement_depth = 0
# velocity_basement = 2000

# Retrieve the model
print("Debug: " + "Retireving Sleipner Model")

caprock_topography, depths = retrieve_sleipner_topography(nz=nz)
caprock_matrix = map_topography_to_velocities(caprock_topography, depths=depths)

# Some initial plotting of the setup
plot_birdseye(
    topography=caprock_topography,
    save_path=PROJECT_ROOT / "plots" / "caprock_birdseye.pdf",
    show=False,
)

plot_cross_section(
    velocity_matrix=caprock_matrix,
    index=caprock_topography.shape[1] // 2,
    depths=depths,
    save_path=PROJECT_ROOT / "plots" / "caprock_cross_section.pdf",
    show=False,
)

plot_cross_section_as_lineplot(
    topography=caprock_topography,
    index=caprock_topography.shape[1] // 2,
    save_path=PROJECT_ROOT / "plots" / "caprock_cross_section_lineplot.pdf",
    show=False,
)


snapshots = single_source_co2_fill(
    injection_matrix=caprock_matrix.copy(),
    topography=caprock_topography,
    depths=depths,
    source=(caprock_topography.shape[0] // 2, caprock_topography.shape[1] // 2),
    rust_implementation=True,
)

# Save the snapshots array
print("Debug: " + "Saving snapshots array")
np.save(PROJECT_ROOT / "simulations" / "flood_fill_snapshots.npy", snapshots)

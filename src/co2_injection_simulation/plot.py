import numpy as np
from matplotlib import pyplot as plt
from pathlib import Path
from typing import Union

from co2_injection_simulation import VELOCITY_CO2, VELOCITY_CAPROCK, VELOCITY_RESERVOIR
from co2_injection_simulation.utils import (
    get_matrix_from_snapshot,
)


def _save_or_show_plot(fig, save_path=None, show=True):
    """Helper function to either save or show a plot."""
    if save_path:
        fig.savefig(save_path, dpi=300, bbox_inches="tight")
        print(f"Plot saved to: {save_path}")

    if show:
        plt.show(block=False)
        plt.pause(0.001)

    if save_path and not show:
        plt.close(fig)


def plot_birdseye(
    topography: np.ndarray,
    save_path: Union[str, Path] = None,
    show: bool = True,
):
    # Create a 2D birds view representation
    fig = plt.figure(figsize=(10, 8))
    plt.imshow(topography, aspect=1, cmap="RdBu_r")
    plt.ylabel("x")
    plt.xlabel("y")
    plt.colorbar(label="Velocity (m/s)")

    _save_or_show_plot(fig, save_path, show)


def plot_cross_section(
    velocity_matrix: np.ndarray,  # 3D velocity matrix (shape: [y, x, depth])
    index: int,
    depths: np.ndarray,  # 1D array of physical depth values
    save_path: Union[str, Path] = None,
    show: bool = True,
):
    # Extract the cross section at the specified row (shape: [x, depth])
    cross_section = velocity_matrix[:, index, :]

    fig = plt.figure(figsize=(10, 6))
    # Transpose so x is horizontal, depth is vertical
    plt.imshow(
        cross_section.T,
        aspect="auto",
        cmap="RdBu_r",
        origin="upper",
        extent=(
            0,
            cross_section.shape[0],
            depths[-1],
            depths[0],
        ),  # x from 0 to width, y from min to max physical depth
    )

    plt.title("Cross Section of Velocity")
    plt.xlabel("Width (x)")
    plt.ylabel("Depth (m)")
    plt.colorbar(label="Velocity (m/s)")

    _save_or_show_plot(fig, save_path, show)


def plot_cross_section_as_lineplot(
    topography: np.ndarray,  # 2D matrix of the topography
    index: int,
    save_path: Union[str, Path] = None,
    show: bool = True,
):
    # Extract the cross section at the specified row
    cross_section = topography[:, index]

    # Create the plot
    fig, ax = plt.subplots(figsize=(10, 6))
    ax.plot(cross_section, color="blue", label="Topography")
    ax.fill_between(
        range(len(cross_section)), cross_section, color="lightblue", alpha=0.5
    )
    ax.set_title("Cross Section of Topography")
    ax.set_xlabel("Width")
    ax.set_ylabel("Elevation")
    ax.legend()
    ax.invert_yaxis()  # Flip the y-axis so higher depth is down

    _save_or_show_plot(fig, save_path, show)


def plot_birdseye_animation(
    snapshots: np.ndarray,
    caprock_topography: np.ndarray,
    depths: np.ndarray,
    save_path: Union[str, Path] = None,
    show: bool = True,
    interval: int = 100,
):
    from matplotlib.animation import FuncAnimation

    print("Debug: " + "Creating birdseye animation")

    fig, ax = plt.subplots(figsize=(10, 8))

    # Import velocity constants for proper scaling
    from co2_injection_simulation import VELOCITY_RESERVOIR

    # The first frame is the unfilled reservoir
    injection_matrix = get_matrix_from_snapshot(
        caprock_topography=caprock_topography,
        depths=depths,
        snapshots=snapshots,
        snapshot_value=-1,
    )

    # Set the cells with CO2 present to VELOCITY_CO2 to 1
    filled_birdseye = np.where(
        np.any(injection_matrix == VELOCITY_CO2, axis=2),
        VELOCITY_CO2,
        np.nan
    )
    im = ax.imshow(filled_birdseye, aspect=1, cmap="RdBu_r")
    plt.ylabel("x")
    plt.xlabel("y")


    def update(frame):
        print(f"Current frame: {frame}")
        # Retrieve the injection matrix at the current snapshot
        injection_matrix_frame = get_matrix_from_snapshot(
            caprock_topography=caprock_topography,
            depths=depths,
            snapshots=snapshots,
            snapshot_value=frame,
        )
        filled_birdseye_frame = np.where(
            np.any(injection_matrix_frame == VELOCITY_CO2, axis=2),
            VELOCITY_CO2,
            np.nan
        )
        im.set_data(filled_birdseye_frame)
        ax.set_title(f"Frame {frame + 1}")
        return [im]

    # The largest value stored in the snapshots matrix
    total_frames = snapshots.max() + 1
    print(f"Total frames: {total_frames}")
    ani = FuncAnimation(fig, update, frames=total_frames, interval=interval, blit=True)

    if save_path:
        ani.save(save_path, writer="pillow", dpi=100)
        print(f"Animation saved to: {save_path}")

    if show:
        plt.show()
    else:
        plt.close(fig)


def plot_cross_section_animation(
    index: int,
    snapshots: np.ndarray,
    caprock_topography: np.ndarray,
    depths: np.ndarray,
    save_path: Union[str, Path] = None,
    show: bool = True,
    interval: int = 100,
):
    from matplotlib.animation import FuncAnimation

    print("Debug: " + "Creating birdseye animation")

    fig, ax = plt.subplots(figsize=(10, 8))

    # The first frame is the unfilled reservoir
    injection_matrix = get_matrix_from_snapshot(
        caprock_topography=caprock_topography,
        depths=depths,
        snapshots=snapshots,
        snapshot_value=-1,
    )
    # Extract the cross section at the specified row (shape: [x, depth])
    cross_section = injection_matrix[:, index, :]

    # Transpose so x is horizontal, depth is vertical
    im = ax.imshow(
        cross_section.T,
        aspect="auto",
        # cmap="viridis",
        cmap="RdBu_r",
        origin="upper",
        vmin=min(VELOCITY_RESERVOIR, VELOCITY_CAPROCK, VELOCITY_CO2),
        vmax=max(VELOCITY_RESERVOIR, VELOCITY_CAPROCK, VELOCITY_CO2),
        extent=(
            0,
            cross_section.shape[0],
            depths[-1],
            depths[0],
        ),  # x from 0 to width, y from min to max physical depth
    )

    plt.xlabel("Width (x)")
    plt.ylabel("Depth (m)")
    plt.colorbar(im, label="Velocity (m/s)")

    def update(frame):
        print(f"Current frame: {frame}")
        # Retrieve the injection matrix at the current snapshot
        injection_matrix_frame = get_matrix_from_snapshot(
            caprock_topography=caprock_topography,
            depths=depths,
            snapshots=snapshots,
            snapshot_value=frame,
        )

        print(
            "CO2 cells in frame",
            frame,
            ":",
            np.sum(injection_matrix_frame == VELOCITY_CO2),
        )
        cross_section_frame = injection_matrix_frame[:, index, :]

        # Debug: Print unique values in this cross-section
        print(f"Cross-section unique values: {np.unique(cross_section_frame)}")

        im.set_data(cross_section_frame.T)
        # Ensure color limits are maintained
        im.set_clim(
            vmin=min(VELOCITY_RESERVOIR, VELOCITY_CAPROCK, VELOCITY_CO2),
            vmax=max(VELOCITY_RESERVOIR, VELOCITY_CAPROCK, VELOCITY_CO2),
        )
        ax.set_title(f"Frame {frame + 1}")
        return [im]

    # The largest value stored in the snapshots matrix
    total_frames = snapshots.max() + 1
    print(f"Total frames: {total_frames}")
    ani = FuncAnimation(fig, update, frames=total_frames, interval=interval, blit=True)

    if save_path:
        ani.save(save_path, writer="pillow", dpi=100)
        print(f"Animation saved to: {save_path}")

    if show:
        plt.show()
    else:
        plt.close(fig)


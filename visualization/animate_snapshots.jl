VELOCITY_RESERVOIR = 1500.0;
VELOCITY_CAPROCK = 2607.0;
VELOCITY_CO2 = 300.0;

using NPZ
caprock_matrix = npzread("simulations/caprock_matrix.npy");
depths = npzread("simulations/depths.npy");
snapshots = npzread("simulations/snapshots.npy");

function get_reservoir_from_snapshot(caprock_matrix::Array{Float64,3}, snapshots::Array{Int32,3}, snapshot_value::Int)
    velocity_matrix = copy(caprock_matrix)
    velocity_matrix[(snapshots.<=snapshot_value).&(snapshots.!=-1)] .= VELOCITY_CO2

    # Hacky solution to invert the depth axis
    velocity_matrix = velocity_matrix[:, :, end:-1:1]


    return velocity_matrix
end

using Makie
using GLMakie
GLMakie.activate!(inline=false);

fig = Figure(resolution=(800, 600));
ax = Axis3(fig[1, 1], aspect=:data);

current_volume = Observable(get_reservoir_from_snapshot(caprock_matrix, snapshots, -1));
plt = volume!(ax, current_volume;
    algorithm=:iso,
    isovalue=VELOCITY_CO2,         # replace with your VELOCITY_CAPROCK
    colormap=:viridis,
)

caprock_matrix = caprock_matrix[:, :, end:-1:1]
plt = volume!(ax, caprock_matrix;
    alpha=0.4,
    algorithm=:iso,
    isovalue=VELOCITY_CAPROCK,         # replace with your VELOCITY_CAPROCK
    # algorithm = :absorption,
    colormap=:viridis,
)

# For a slider to control the snapshot
n_snapshots = maximum(snapshots)
slider = Slider(fig[2, 1], range=0:n_snapshots, startvalue=0)
on(slider.value) do i
    current_volume[] = get_reservoir_from_snapshot(caprock_matrix, snapshots, i)
end
fig

# # To record an animation
# n_snapshots = maximum(snapshots)
# record(fig, "reservoir_animation.mp4", 1:n_snapshots) do i
#     current_volume[] = get_reservoir_from_snapshot(caprock_matrix, snapshots, i)
# end

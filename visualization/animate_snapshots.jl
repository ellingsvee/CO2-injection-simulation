VELOCITY_RESERVOIR = 1500.0;
VELOCITY_CAPROCK = 2607.0;
VELOCITY_CO2 = 300.0;

using NPZ
caprock_matrix = npzread("simulations/caprock_matrix.npy");
snapshots = npzread("simulations/snapshots.npy");

using Makie
using GLMakie
GLMakie.activate!(inline=false)

function get_reservoir_from_snapshot(caprock_matrix::Array{Float64,3}, snapshots::Array{Int32,3}, snapshot_value::Int)
    velocity_matrix = copy(caprock_matrix)
    velocity_matrix[(snapshots.<=snapshot_value).&(snapshots.!=-1)] .= VELOCITY_CO2
    return velocity_matrix
end

fig = Figure(resolution=(800, 600))
ax = Axis3(fig[1, 1])

current_volume = Observable(get_reservoir_from_snapshot(caprock_matrix, snapshots, -1))
# plot as volume
plt = volume!(ax, caprock_matrix;
    alpha=0.5,
    algorithm=:iso,
    isovalue=VELOCITY_CAPROCK,         # replace with your VELOCITY_CAPROCK
    colormap=:viridis,
    transformation=(:identity, :identity, x -> -x)
)
plt = volume!(ax, current_volume;
    algorithm=:iso,
    isovalue=VELOCITY_CO2,         # replace with your VELOCITY_CAPROCK
    colormap=:viridis,
    transformation=(:identity, :identity, x -> -x)
)


n_snapshots = maximum(snapshots)
slider = Slider(fig[2, 1], range=0:n_snapshots, startvalue=0)

on(slider.value) do i
    current_volume[] = get_reservoir_from_snapshot(caprock_matrix, snapshots, i)
end

# record(fig, "reservoir_animation.mp4", 1:length(reservoir_matrices)) do i
#     current_volume[] = reservoir_matrices[i]
# end

fig
VELOCITY_RESERVOIR = 1500.0;
VELOCITY_CAPROCK = 2607.0;
VELOCITY_CO2 = 300.0;

using NPZ
reservoir_matrix = npzread("simulations/caprock_matrix.npy");

using GLMakie
GLMakie.activate!(inline=false)
fig = Figure()
ax = Axis3(fig[1, 1], aspect=:data)
volume!(ax, reservoir_matrix; algorithm=:iso, isovalue=VELOCITY_CAPROCK, colormap=:viridis)
fig
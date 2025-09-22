#!/usr/bin/env bash
set -e  # exit immediately if a command fails

echo "=== Setting up domain ==="
uv run scripts/setup_domain.py

echo "=== Running simulation ==="
uv run scripts/simulation.py

echo "=== Visualizing snapshots ==="
julia visualization/animate_snapshots.jl

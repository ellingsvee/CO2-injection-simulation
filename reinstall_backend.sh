#!/usr/bin/env bash
set -e  # exit immediately if a command fails

echo "=== Sync ==="
uv sync --reinstall

echo "=== Build ==="
uv build

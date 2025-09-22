#!/usr/bin/env bash
set -e  # exit immediately if a command fails

cd rust_backend
mv Cargo.toml.bak Cargo.toml
cargo build
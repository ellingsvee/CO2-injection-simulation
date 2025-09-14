# CO2 injection simulation

A nearest-neighbour approach for simulating CO2 injection in a reservoir.

Uses Rust under the hood for efficient computations.

## Prerequisites

- Python 3.13+ (I suggest [Install Python](https://docs.astral.sh/uv/guides/install-python/) for installing Python using uv)
- Rust (See [Install Rust](https://www.rust-lang.org/tools/install))
- uv (See [uv installation](https://docs.astral.sh/uv/getting-started/installation/))

## How to use

1. **Clone this repo:**

   ```bash
   git clone git@github.com:ellingsvee/python-rust-template.git
   cd python-rust-template
   ```

2. **Install dependencies:**

   ```bash
   uv venv # Likely not needed, but just to be sure
   uv sync
   ```

3. **Run the simulation:**

   ```bash
   uv run simulation.py
   ```

4. Generate animations

   ```bash
   uv run generate_animations.py
   ```

## Making Changes

**Python code changes:**

- Edit files in `src/python_rust_template/`
- Changes are immediately available (editable install)

**Rust code changes (IMPORTANT!):**

- Edit files in `src/python_rust_template/rust_backend/`
- Rebuild the extension:

  ```bash
  uv sync --reinstall
  ```

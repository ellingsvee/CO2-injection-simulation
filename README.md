# Python Rust Template

A template for creating Python packages with Rust extensions using uv and maturin. This template provides a complete setup for building hybrid Python-Rust projects with modern tooling.

## Features

- Python package with Rust extensions using PyO3
- Modern build system with maturin
- Package management with uv (no pip required)
- Editable development installs
- Type hints support with py.typed
- Example functions demonstrating Python-Rust interop

## Prerequisites

- Python 3.13+
- Rust (install via [rustup](https://rustup.rs/))
- uv (install via `curl -LsSf https://astral.sh/uv/install.sh | sh`)

## Project Structure

```
├── pyproject.toml              # Python project configuration
├── README.md                   # This file
└── src/
    └── python_rust_template/   # Python package
        ├── __init__.py         # Package entry point
        ├── py.typed            # Type hints marker
        └── rust_backend/       # Rust extension module
            ├── Cargo.toml      # Rust project configuration
            └── lib.rs          # Rust source code
```

## Quick Start

1. **Clone this template:**
   ```bash
   git clone git@github.com:ellingsvee/python-rust-template.git
   cd python-rust-template
   ```

2. **Install dependencies and build:**
   ```bash
   uv sync
   ```

3. **Test the installation:**
   ```bash
   uv run python -c "import python_rust_template; print(python_rust_template.hello())"
   uv run python -c "import python_rust_template; print(python_rust_template.sum_as_string(5, 3))"
   uv run python -c "import python_rust_template; print(python_rust_template.multiply(2.5, 4.0))"
   ```

## Development Workflow

### Initial Setup
```bash
# Install dependencies and build the project
uv sync
```

### Making Changes

**Python code changes:**
- Edit files in `src/python_rust_template/`
- Changes are immediately available (editable install)

**Rust code changes:**
- Edit files in `src/python_rust_template/rust_backend/`
- Rebuild the extension:
  ```bash
  uv sync --reinstall
  ```

### Building for Distribution
```bash
# Build wheel and source distribution
uv build
```

### Running Tests
```bash
# Run your Python code
uv run python your_script.py

# Or start an interactive session
uv run python
```

## Customizing for Your Project

1. **Rename the package:**
   - Update `name` in `pyproject.toml`
   - Rename `src/python_rust_template/` directory
   - Update `module-name` and `manifest-path` in `pyproject.toml`
   - Update imports in `__init__.py`

2. **Add dependencies:**
   ```bash
   # Python dependencies
   uv add numpy pandas

   # Rust dependencies (edit Cargo.toml)
   # Add to [dependencies] section
   ```

3. **Add new Rust functions:**
   - Define functions in `lib.rs` with `#[pyfunction]`
   - Add them to the module with `m.add_function(wrap_pyfunction!(your_function, m)?)?`
   - Import them in `__init__.py`

## Configuration Files

### pyproject.toml
- Python project metadata and dependencies
- Maturin build configuration
- Points to Rust source location

### Cargo.toml
- Rust project configuration
- PyO3 dependency for Python bindings
- Library configuration for Python extension

## Troubleshooting

**Import errors after Rust changes:**
```bash
uv sync --reinstall
```

**Build errors:**
- Check that Rust code compiles: `cd src/python_rust_template/rust_backend && cargo check`
- Verify Python syntax in Python files
- Ensure all paths in `pyproject.toml` are correct

**Missing dependencies:**
```bash
# Check what's installed
uv tree

# Reinstall everything
uv sync --reinstall
```

## Example Usage

```python
import python_rust_template

# Python function
print(python_rust_template.hello())

# Rust functions
result = python_rust_template.sum_as_string(10, 20)
print(result)  # "The sum of 10 and 20 is 30"

product = python_rust_template.multiply(3.14, 2.0)
print(product)  # 6.28
```

## Advanced Features

- **Type hints:** Add type stubs for Rust functions in `*.pyi` files
- **Documentation:** Use `pyo3` doc comments that appear in Python help()
- **Error handling:** Use `PyResult<T>` for proper Python exception handling
- **Complex types:** Pass/return lists, dicts, custom classes between Python and Rust

## License

This template is provided under the MIT License. Customize as needed for your project.

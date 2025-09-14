from pathlib import Path

# Project root (assumes this file is in src/simple_injection_simulator)
PROJECT_ROOT = Path(__file__).resolve().parents[2]
CODE_DIR = Path(__file__).resolve().parent

# Some physical constants
VELOCITY_CAPROCK = 2607
VELOCITY_RESERVOIR = 1500
VELOCITY_CO2 = 300

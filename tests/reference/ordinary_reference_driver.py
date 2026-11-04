# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated entry to the independent ordinary-process binary producer."""

import sys
from pathlib import Path

# -I deliberately excludes the current directory and PYTHONPATH. Only this
# explicitly selected reference-tool directory enters the isolated interpreter.
sys.path.insert(0, str(Path(__file__).resolve().parent))

from oracles.ordinary_exchange import main

if __name__ == "__main__":
    main()

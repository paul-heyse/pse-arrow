# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated entry to the independent binary degeneracy reference."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from oracles.degeneracy import main

if __name__ == "__main__":
    main()

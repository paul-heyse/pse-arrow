# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Make the flat modules in `scripts/` importable by the library-catalog tests."""

import sys
from pathlib import Path

SCRIPTS = str(Path(__file__).resolve().parents[1])
if SCRIPTS not in sys.path:
    sys.path.insert(0, SCRIPTS)

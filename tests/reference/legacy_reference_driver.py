# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated entry to the named independent legacy JSON producers."""

import sys
from importlib import import_module
from pathlib import Path

if __name__ == "__main__":
    if sys.flags.isolated != 1 or len(sys.argv) != 4:
        message = "expected isolated oracle name, request and response paths"
        raise ValueError(message)
    name = sys.argv.pop(1)
    if name not in {"covariance", "degeneracy", "pid", "sensitivity"}:
        message = f"unknown reference oracle {name}"
        raise ValueError(message)
    # Explicitly admit only this reference-tool directory under -I.
    sys.path.insert(0, str(Path(__file__).resolve().parent))
    import_module(f"oracles.{name}").main()

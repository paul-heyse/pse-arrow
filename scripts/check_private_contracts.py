# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Validate source-only generated bootstrap records without package startup."""

from __future__ import annotations

import importlib.util
import sys
from pathlib import Path

import msgspec


def main() -> None:
    """Resolve every emitted annotation and verify the bootstrap import boundary."""
    for index, path in enumerate(sorted(Path(sys.argv[1]).glob("*.py"))):
        name = f"_private_contract_candidate_{index}"
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise RuntimeError("private contract candidate cannot be loaded")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        for exported in module.__all__:
            msgspec.inspect.type_info(getattr(module, exported))
    if any(
        name == "pse" or name.startswith(("pse.", "opentelemetry"))
        for name in sys.modules
    ):
        raise RuntimeError("private bootstrap contract imported native package or SDK")


if __name__ == "__main__":
    main()

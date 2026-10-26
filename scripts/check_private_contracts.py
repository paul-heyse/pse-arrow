# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Validate source-only generated bootstrap records without package startup."""

from __future__ import annotations

import importlib
import importlib.util
import sys
from pathlib import Path

import msgspec
from pydantic import BaseModel


def main() -> None:
    """Resolve every emitted annotation and verify the bootstrap import boundary."""
    candidate_root = Path(sys.argv[1]).resolve()
    # The closed host projection imports its real source-only Contract/default
    # owners even under Python -I. Keep every generated dependency on the same
    # unpublished candidate tree, never the checkout's previous generated files.
    source_root = Path(__file__).resolve().parents[1]
    sys.path.insert(0, str(source_root))
    package = importlib.import_module("scripts.generated")
    package.__dict__["__path__"] = [str(candidate_root)]
    for index, path in enumerate(sorted(candidate_root.glob("*.py"))):
        name = f"_private_contract_candidate_{index}"
        spec = importlib.util.spec_from_file_location(name, path)
        if spec is None or spec.loader is None:
            raise RuntimeError("private contract candidate cannot be loaded")
        module = importlib.util.module_from_spec(spec)
        sys.modules[name] = module
        spec.loader.exec_module(module)
        for exported in module.__all__:
            value = getattr(module, exported)
            if isinstance(value, type) and issubclass(value, BaseModel):
                value.model_rebuild(force=True)
                value.model_json_schema()
            else:
                msgspec.inspect.type_info(value)
    if any(
        name == "pse" or name.startswith(("pse.", "opentelemetry"))
        for name in sys.modules
    ):
        raise RuntimeError("private bootstrap contract imported native package or SDK")


if __name__ == "__main__":
    main()

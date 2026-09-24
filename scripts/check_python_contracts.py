# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Check the chosen generated contract tree without importing package startup."""

import argparse
import importlib
import sys
from pathlib import Path
from types import ModuleType


def main() -> None:
    """Select candidate sources before any generated module is loaded."""
    repository = Path(__file__).resolve().parents[1]
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=repository)
    candidate = parser.parse_args().root.resolve()
    contracts = candidate / "python/pse/contracts"
    if not (contracts / "__init__.py").is_file():
        parser.error(f"candidate contracts are absent: {contracts}")
    package = ModuleType("pse")
    package.__path__ = [str(candidate / "python/pse"), str(repository / "python/pse")]
    sys.modules["pse"] = package
    # Candidate contracts need only the stable scalar identity codecs. Loading the
    # full live gateway would couple pure regeneration to new workflow classes in
    # an extension which cannot be rebuilt until these contracts are generated.
    native = importlib.import_module("pse._native")
    boundary = ModuleType("pse._build")
    for name in (
        "semantic_id_from_hex",
        "semantic_id_to_hex",
        "content_hash_from_prefixed",
        "content_hash_to_prefixed",
    ):
        setattr(boundary, name, getattr(native, name))
    sys.modules["pse._build"] = boundary
    root = importlib.import_module("pse.contracts")
    if (
        root.__file__ is None
        or Path(root.__file__).resolve() != contracts / "__init__.py"
    ):
        parser.error("loaded contracts do not belong to the candidate tree")
    governance = importlib.import_module("pse.governance")
    governance.check(root)
    print(f"contract annotations: OK ({contracts})")


if __name__ == "__main__":
    main()

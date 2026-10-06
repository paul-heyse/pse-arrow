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
    parser.add_argument(
        "--documents",
        action="store_true",
        help="check the candidate document types against the checkout's contracts",
    )
    arguments = parser.parse_args()
    candidate = arguments.root.resolve()
    contracts = candidate / "python/pse/contracts"
    if arguments.documents:
        target = contracts / "documents/__init__.py"
    else:
        target = contracts / "__init__.py"
    if not target.is_file():
        parser.error(f"candidate contracts are absent: {target}")
    package = ModuleType("pse")
    package.__path__ = [str(candidate / "python/pse"), str(repository / "python/pse")]
    sys.modules["pse"] = package

    # This checks annotations, never scalar values. Importing the native extension
    # would couple pure generation to scientific libraries and to workflow classes
    # that cannot be rebuilt until these candidate contracts have been generated.
    # A forbidden invocation fails rather than substituting a second identity codec.
    def annotation_only_codec(_value: object) -> object:
        raise RuntimeError("annotation checking cannot execute native scalar codecs")

    boundary = ModuleType("pse._build")
    for name in (
        "semantic_id_from_hex",
        "semantic_id_to_hex",
        "content_hash_from_prefixed",
        "content_hash_to_prefixed",
    ):
        setattr(boundary, name, annotation_only_codec)
    sys.modules["pse._build"] = boundary
    root = importlib.import_module("pse.contracts")
    if arguments.documents:
        # The document types are checked against the checkout's registry contracts, from
        # the candidate's own directory.
        root.__path__ = [str(contracts), *root.__path__]
        root = importlib.import_module("pse.contracts.documents")
    if root.__file__ is None or Path(root.__file__).resolve() != target:
        parser.error("loaded contracts do not belong to the candidate tree")
    governance = importlib.import_module("pse.governance")
    governance.check(root)
    print(f"contract annotations: OK ({target.parent})")


if __name__ == "__main__":
    main()

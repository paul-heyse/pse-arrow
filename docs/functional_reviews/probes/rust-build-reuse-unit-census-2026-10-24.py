# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Unit-graph census for the accompanying Rust build-reuse review.

Reads Cargo `--unit-graph` JSON files produced without compiling, for example:

    scripts/pse-env -- cargo test --workspace --no-run --locked \
        --features pse-relations/force-validate --unit-graph -Z unstable-options > test-ws.json

The first file is the reference selection. For each file, report third-party library
packages that Cargo builds more than once within the selection. Same mode means a
host/target split: either the package's own features differ, or it inherits the split
from a dependency whose features differ (syn through proc-macro2). Differing modes are the
check/build split inherent to `cargo check`. For
each later file, also report packages whose feature sets differ from the reference,
which shows where workspace feature unification does not hold across selections.
It writes nothing and runs no Cargo command.
"""

from __future__ import annotations

import json
import sys
from collections import defaultdict
from pathlib import Path

LIBRARY_KINDS = {"lib", "rlib", "proc-macro"}
Key = tuple[str, str]


def feature_sets(path: Path) -> tuple[dict[Key, set[tuple[str, ...]]], dict[Key, int]]:
    graph = json.loads(path.read_text())
    sets: dict[Key, set[tuple[str, ...]]] = defaultdict(set)
    units: dict[Key, int] = defaultdict(int)
    for unit in graph["units"]:
        package_id: str = unit["pkg_id"]
        if unit["mode"] not in {"build", "check"} or package_id.startswith("path+"):
            continue
        if not LIBRARY_KINDS.intersection(unit["target"]["kind"]):
            continue
        key = (package_id.split("#")[-1], unit["mode"])
        sets[key].add(tuple(unit["features"]))
        units[key] += 1
    return sets, units


def split_keys(units: dict[Key, int]) -> list[Key]:
    return [key for key, count in units.items() if count > 1]


def report(path: Path, reference: dict[Key, set[tuple[str, ...]]] | None) -> None:
    sets, units = feature_sets(path)
    split = sorted(name for (name, _), count in units.items() if count > 1)
    inherited = sorted(name for (name, mode) in split_keys(units) if len(sets[(name, mode)]) == 1)
    modes: dict[str, set[str]] = defaultdict(set)
    for name, mode in sets:
        modes[name].add(mode)
    check_build = sorted(name for name, found in modes.items() if len(found) > 1)
    print(f"== {path.name}: third-party packages={len(modes)}")
    print(f"   host/target split ({len(split)}): {' '.join(split)}")
    print(f"     of which inherited from a dependency ({len(inherited)}): {' '.join(inherited)}")
    print(f"   check/build split ({len(check_build)})")
    if reference is not None:
        differing = sorted(
            name
            for key, found in sets.items()
            if key in reference and found != reference[key]
            for name in [f"{key[0]}[{key[1]}]"]
        )
        print(f"   feature sets differing from reference ({len(differing)}): {' '.join(differing)}")


def main(paths: list[Path]) -> None:
    reference, _ = feature_sets(paths[0])
    report(paths[0], None)
    for path in paths[1:]:
        report(path, reference)


if __name__ == "__main__":
    main([Path(argument) for argument in sys.argv[1:]])

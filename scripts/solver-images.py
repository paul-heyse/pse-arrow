#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Single authority for solver images and checked workflow literal projections."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MANIFEST = ROOT / ".github/setup/solver-images.json"
PREFIX = "ghcr.io/paul-heyse/pse-solvers:"


def validate(pins: dict[str, str]) -> None:
    if set(pins) != {"ci", "dev"}:
        raise ValueError("image manifest must contain ci and dev")
    trees = []
    for stage, ref in pins.items():
        match = re.fullmatch(
            re.escape(PREFIX) + stage + r"-([0-9a-f]{12})@sha256:[0-9a-f]{64}", ref
        )
        if not match:
            raise ValueError(
                f"{stage}: expected a recipe tree tag and immutable digest"
            )
        trees.append(match[1])
    if len(set(trees)) != 1:
        raise ValueError("ci and dev must come from the same recipe tree")


def projections(root: Path, pins: dict[str, str]) -> dict[Path, str]:
    targets = [
        *sorted((root / ".github/workflows").glob("*.yml")),
        root / ".devcontainer/devcontainer.json",
    ]
    result = {}
    for path in targets:
        original = path.read_text()
        updated = original
        for stage, ref in pins.items():
            pattern = (
                re.escape(PREFIX) + stage + r"-[0-9a-zA-Z.\-]+(?:@sha256:[0-9a-f]{64})?"
            )
            updated = re.sub(pattern, ref, updated)
        if original != updated:
            result[path] = updated
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="command", required=True)
    commands.add_parser("check")
    commands.add_parser("sync")
    ref = commands.add_parser("ref")
    ref.add_argument("stage", choices=("ci", "dev"))
    update = commands.add_parser("update")
    update.add_argument("--ci", required=True)
    update.add_argument("--dev", required=True)
    args = parser.parse_args()
    pins = json.loads(MANIFEST.read_text())
    if args.command == "update":
        pins = {"ci": args.ci, "dev": args.dev}
    validate(pins)
    if args.command == "ref":
        print(pins[args.stage])
        return 0
    changes = projections(ROOT, pins)
    if args.command == "check":
        for path in changes:
            print(f"solver pin drift: {path.relative_to(ROOT)}", file=sys.stderr)
        return int(bool(changes))
    if args.command == "update":
        MANIFEST.write_text(json.dumps(pins, indent=2) + "\n")
    for path, content in changes.items():
        path.write_text(content)
        print(f"updated {path.relative_to(ROOT)}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Detect hand edits to generated Python contracts (standard library only).

``python/pse/contracts/`` is written by ``pse-schema`` and committed. A hand edit
there survives until the next ``just codegen`` silently reverts it, taking the
fix with it, so the manifest is checked in pre-commit and in ``python / lint``.

``GENERATED.sha256`` lists one ``<sha256>  <name>`` line per file in the
directory, in the format ``sha256sum`` reads, excluding itself. Run with
``--write`` to regenerate it; the generator will do that itself from phase 1.
"""

import argparse
import hashlib
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[1]
CONTRACTS = REPO_ROOT / "python" / "pse" / "contracts"
MANIFEST = CONTRACTS / "GENERATED.sha256"
MANIFEST_NAME = MANIFEST.name

FIX = "edit the registry, then `just codegen`"


def digest(path: Path) -> str:
    """Return the sha256 of a file's bytes.

    Args:
        path: The file to hash.

    Returns:
        The lowercase hex digest.
    """
    return hashlib.sha256(path.read_bytes()).hexdigest()


def observed() -> dict[str, str]:
    """Hash every generated file currently in the contracts directory.

    Returns:
        A mapping of file name to hex digest, excluding the manifest itself.
    """
    return {
        path.name: digest(path)
        for path in sorted(CONTRACTS.iterdir())
        if path.is_file() and path.name != MANIFEST_NAME
    }


def recorded() -> dict[str, str]:
    """Parse the committed manifest.

    Returns:
        A mapping of file name to the digest the manifest records.
    """
    if not MANIFEST.is_file():
        return {}
    entries: dict[str, str] = {}
    for line in MANIFEST.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped or stripped.startswith("#"):
            continue
        checksum, _, name = stripped.partition("  ")
        entries[name.strip()] = checksum.strip()
    return entries


def render(entries: dict[str, str]) -> str:
    """Render a manifest body.

    Args:
        entries: File name to digest.

    Returns:
        The manifest text, newline-terminated.
    """
    lines = [f"{entries[name]}  {name}" for name in sorted(entries)]
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    """Compare the manifest against the directory.

    Args:
        argv: Command-line arguments; defaults to ``sys.argv[1:]``.

    Returns:
        0 when the manifest matches, 1 on drift.
    """
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--write",
        action="store_true",
        help="rewrite GENERATED.sha256 from the files on disk",
    )
    args = parser.parse_args(argv)

    current = observed()
    if args.write:
        MANIFEST.write_text(render(current), encoding="utf-8")
        print(f"wrote {MANIFEST.relative_to(REPO_ROOT)} ({len(current)} files)")
        return 0

    manifest = recorded()
    problems: list[str] = []
    for name in sorted(set(manifest) | set(current)):
        if name not in current:
            problems.append(f"  {name}: recorded in the manifest but missing on disk")
        elif name not in manifest:
            problems.append(f"  {name}: present on disk but not in the manifest")
        elif manifest[name] != current[name]:
            problems.append(f"  {name}: content differs from the manifest")

    if not problems:
        return 0

    print(
        f"generated contracts have drifted from {MANIFEST.relative_to(REPO_ROOT)}:",
        file=sys.stderr,
    )
    for problem in problems:
        print(problem, file=sys.stderr)
    print(FIX, file=sys.stderr)
    return 1


if __name__ == "__main__":
    raise SystemExit(main())

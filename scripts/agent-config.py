#!/usr/bin/env python3
# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Materialize shared skill aliases; native agent adapters are maintained separately."""
# ruff: noqa: N999 -- executable script name, not an importable module

from __future__ import annotations

import argparse
import shutil
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def skill_alias(root: Path, directory: str) -> None:
    alias = root / directory / "skills"
    target = root / ".codex/skills"
    alias.parent.mkdir(exist_ok=True)
    if alias.is_symlink():
        if alias.resolve() != target.resolve():
            raise ValueError(f"unexpected skill symlink: {alias}")
        return
    if alias.is_file():
        if alias.read_text().strip() != "../.codex/skills":
            raise ValueError(f"refusing to replace {alias}")
        alias.unlink()
    if not alias.exists():
        try:
            alias.symlink_to("../.codex/skills", target_is_directory=True)
        except OSError:
            pass  # Windows without symlink privileges uses a checked materialization.
        else:
            return
    # Keep shared library links as links; never materialize multi-GB bundles here.
    shutil.copytree(target, alias, dirs_exist_ok=True, symlinks=True)


def same_tree(left: Path, right: Path) -> bool:
    """Compare copies without traversing shared library directory links."""
    if left.is_symlink() or right.is_symlink():
        return left.is_symlink() and right.is_symlink() and left.resolve() == right.resolve()
    if left.is_dir() and right.is_dir():
        names = {path.name for path in left.iterdir()}
        return names == {path.name for path in right.iterdir()} and all(
            same_tree(left / name, right / name) for name in names
        )
    return left.is_file() and right.is_file() and left.read_bytes() == right.read_bytes()


def synchronize(root: Path, *, check: bool) -> list[str]:
    problems = []
    target = root / ".codex/skills"
    if not target.is_dir():
        return [".codex/skills must exist before materializing aliases"]
    for directory in (".claude", ".agents"):
        alias = root / directory / "skills"
        if check:
            if not same_tree(alias, target) and not (
                alias.is_symlink() and alias.resolve() == target.resolve()
            ):
                problems.append(f"skill alias drift: {directory}/skills; run just agent-config-sync")
        else:
            skill_alias(root, directory)
    return problems


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        problems = synchronize(ROOT, check=args.check)
    except (ValueError, OSError) as exc:
        problems = [str(exc)]
    for problem in problems:
        print(problem, file=sys.stderr)
    return int(bool(problems))


if __name__ == "__main__":
    sys.exit(main())

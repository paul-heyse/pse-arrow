# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Create a second working copy for a parallel agent.

``just worktree <name> [--ref REF] [--python] [--native]`` adds ``~/pse-arrow-wt/<name>``
on a new branch ``<name>`` from REF (default HEAD), copies the gitignored ``.envrc.local``,
allows direnv, links the selected skills and reports ``just doctor``. Uncommitted work in this checkout is never
carried over; the command says so. ``--python`` builds that checkout's own venv and dev
extension; ``--native`` builds the linked native extension instead. Nothing is shared
except the sccache compiler cache: each checkout keeps its own ``target/`` (ADR-0122).
"""

from __future__ import annotations

import argparse
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
WORKTREES = Path.home() / "pse-arrow-wt"


def run(command: list[str], cwd: Path) -> None:
    print("+ " + " ".join(command), flush=True)
    subprocess.run(command, cwd=cwd, check=True)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("name")
    parser.add_argument("--ref", default="HEAD")
    parser.add_argument("--python", action="store_true", help="build its venv and dev extension")
    parser.add_argument("--native", action="store_true", help="build the linked native extension")
    args = parser.parse_args(argv)
    target = WORKTREES / args.name
    if target.exists():
        parser.error(f"{target} already exists")
    commit = subprocess.run(
        ["git", "rev-parse", "--verify", f"{args.ref}^{{commit}}"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout.strip()
    print(f"worktree: {target} on new branch {args.name} from {args.ref} ({commit[:12]})")
    dirty = subprocess.run(
        ["git", "status", "--porcelain", "--untracked-files=no"],
        cwd=ROOT, capture_output=True, text=True, check=True,
    ).stdout
    if dirty:
        print("worktree: uncommitted changes in this checkout are NOT in the new worktree:")
        print("".join(f"  {line}\n" for line in dirty.splitlines()[:20]), end="")
    WORKTREES.mkdir(parents=True, exist_ok=True)
    run(["git", "worktree", "add", "-b", args.name, str(target), commit], ROOT)
    local = ROOT / ".envrc.local"
    if local.is_file():
        shutil.copy2(local, target / ".envrc.local")
        (target / ".envrc.local").chmod(0o600)
        print("worktree: copied .envrc.local")
    if shutil.which("direnv"):
        run(["direnv", "allow", str(target)], target)
    run(["just", "skills-sync"], target)
    if args.python or args.native:
        run(["uv", "sync", "--locked"], target)
        run(["just", "py-sync-native" if args.native else "py-sync"], target)
    # Informational: a fresh checkout without --python legitimately lacks a venv.
    print("+ just doctor", flush=True)
    subprocess.run(["just", "doctor"], cwd=target, check=False)
    print(f"worktree: created. cd {target}; remove with: git worktree remove {target}")
    return 0


if __name__ == "__main__":
    sys.exit(main())

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Run REUSE over Git-eligible source files, including untracked implementation files.

REUSE's directory-only Git ignore inventory misses ignored children of a wholly
untracked directory. A source snapshot prevents acquisition environments in a new
nested project from entering the check, while retaining every eligible source file.
"""

from __future__ import annotations

import shutil
import subprocess
from pathlib import Path
from tempfile import TemporaryDirectory


def main() -> int:
    """Run the installed CLI against the complete current source snapshot."""
    root = Path(__file__).resolve().parents[1]
    result = subprocess.run(
        ["git", "ls-files", "-z", "--cached", "--others", "--exclude-standard"],
        cwd=root,
        check=True,
        capture_output=True,
    )
    paths = sorted(set(result.stdout.decode().split("\0")) - {""})
    with TemporaryDirectory(prefix="pse-reuse-") as directory:
        snapshot = Path(directory)
        for name in paths:
            source = root / name
            if source.is_file() and not source.is_symlink():
                destination = snapshot / name
                destination.parent.mkdir(parents=True, exist_ok=True)
                shutil.copyfile(source, destination)
        return subprocess.run(
            [str(root / ".venv/bin/reuse"), "--root", str(snapshot), "lint"],
            cwd=root,
            check=False,
        ).returncode


if __name__ == "__main__":
    raise SystemExit(main())

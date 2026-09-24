# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Inventory build storage without cleaning active artifacts or shared caches."""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tomllib
from pathlib import Path

from scripts import build_environment, native_cache


def main() -> None:
    root = build_environment.ROOT
    settings = tomllib.loads((root / ".config/build.toml").read_text())
    env = build_environment.configure(root, dict(os.environ))
    paths = [root / "target", root / "build", native_cache.cache_root(env)]
    if env.get("SCCACHE_DIR"):
        paths.append(Path(env["SCCACHE_DIR"]))
    print(
        json.dumps(
            {
                "free_bytes": shutil.disk_usage(root).free,
                "free_space_floor_gib": settings["free_space_gib"],
                "retention": "stable + selected nightly + at most one temporary candidate; no automatic deletion",
                "paths": [str(path) for path in paths],
            },
            indent=2,
        )
    )
    subprocess.run(["du", "-h", "--max-depth=1", *map(str, paths)], check=False)


if __name__ == "__main__":
    main()

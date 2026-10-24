# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact protocol-sensitive Cargo tool selections, shared by setup and consumers."""
from __future__ import annotations

import argparse
import hashlib
import re
import shutil
import subprocess
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]


def versions() -> dict[str, str]:
    with (ROOT / ".config/diagnostic-tools.toml").open("rb") as stream:
        return tomllib.load(stream)["cargo-tools"]


def specifications(names: list[str]) -> list[str]:
    pins = versions()
    result = []
    for item in names or list(pins):
        name, separator, requested = item.partition("@")
        if not re.fullmatch(r"[a-z0-9-]+", name):
            raise ValueError(f"invalid Cargo tool name: {name}")
        if name in pins:
            if separator and requested != pins[name]:
                raise ValueError(f"{name} must use declared version {pins[name]}")
            result.append(f"{name}@{pins[name]}")
        else:
            result.append(item)
    return result


def selected_identity(name: str) -> dict[str, str]:
    expected = versions()[name]
    executable = shutil.which(name)
    if executable is None:
        raise ValueError(f"{name} unavailable; run just bootstrap-rust-tools")
    path = Path(executable).resolve()
    output = subprocess.check_output([str(path), name.removeprefix("cargo-"), "--version"], text=True).strip()
    if not re.search(rf"(?<![\d.]){re.escape(expected)}(?![\d.])", output):
        raise ValueError(f"{name} version differs from {expected}: {output}")
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"name": name, "version": expected, "path": str(path), "sha256": digest}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("names", nargs="*")
    args = parser.parse_args()
    names = [part for item in args.names for part in re.split(r"[\s,]+", item) if part]
    print("\n".join(specifications(names)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

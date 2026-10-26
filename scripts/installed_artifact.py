# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact ordinary Maturin installation association, without producer orchestration."""

from __future__ import annotations

import hashlib
import shutil
import subprocess
import tempfile
from collections import Counter
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping, Sequence


def associate(
    original: Path,
    installed: Path,
    original_sha256: str,
    installed_sha256: str,
    messages: Sequence[Mapping[str, object]],
    *,
    environment: Mapping[str, str],
) -> dict[str, object]:
    """Replay only Cargo-declared editable RPATH changes and compare ALL bytes."""
    if original_sha256 == installed_sha256:
        return {"kind": "identical-bytes"}
    linked: list[str] = []
    for message in messages:
        if message.get("reason") != "build-script-executed":
            continue
        libraries, paths = (
            message.get("linked_libs", []),
            message.get("linked_paths", []),
        )
        if (
            not isinstance(libraries, list)
            or not isinstance(paths, list)
            or any(not isinstance(value, str) for value in [*libraries, *paths])
        ):
            raise ValueError("invalid actual Cargo dynamic-link paths")
        if not any(
            "=" not in value or value.split("=", 1)[0].startswith("dylib")
            for value in libraries
        ):
            continue
        for value in paths:
            if "=" not in value:
                linked.append(value)
            elif value.split("=", 1)[0] in {"native", "all"}:
                linked.append(value.split("=", 1)[1])
    tool = shutil.which("patchelf", path=environment.get("PATH"))
    if tool is None:
        raise ValueError("Maturin RPATH association requires its selected patchelf")

    def rpath(path: Path) -> list[str]:
        value = subprocess.check_output(
            [tool, "--print-rpath", str(path)], text=True, env=environment
        )
        return [part for part in value.strip().split(":") if part]

    old, actual = rpath(original), rpath(installed)
    appended = [value for value in linked if value not in old]
    if actual[: len(old)] != old or Counter(actual[len(old) :]) != Counter(appended):
        raise ValueError(
            "Imported extension is not the admitted Maturin RPATH transformation"
        )
    with tempfile.TemporaryDirectory(prefix="pse-maturin-rpath-") as scratch:
        replay = Path(scratch) / original.name
        shutil.copyfile(original, replay)
        subprocess.run(
            [tool, "--remove-rpath", str(replay)], env=environment, check=True
        )
        subprocess.run(
            [tool, "--force-rpath", "--set-rpath", ":".join(actual), str(replay)],
            env=environment,
            check=True,
        )
        with replay.open("rb") as stream:
            replayed = hashlib.file_digest(stream, "sha256").hexdigest()
        if replayed != installed_sha256:
            raise ValueError(
                "Imported extension differs beyond the admitted RPATH-only transformation"
            )
    return {
        "kind": "maturin-rpath-only",
        "original_rpath": old,
        "installed_rpath": actual,
        "appended_rpaths": appended,
        "replayed_sha256": replayed,
        "tool": tool,
    }

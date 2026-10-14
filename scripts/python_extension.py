# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Inspect the actual imported Python extension before correctness execution."""

from __future__ import annotations

import hashlib
import json
import os
import subprocess
import sys
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[1]


def capture(
    environment: Mapping[str, str], *, validated: bool = True, native: bool = False
) -> dict[str, object]:
    """Bind Rust cfg flags to the imported checkout path and observed binary bytes."""
    program = (
        "import json, sys, pse\n"
        "from pse._build import compiled_features\n"
        "print(json.dumps({'package': pse.__file__, "
        "'native': sys.modules['pse._native'].__file__, "
        "'features': compiled_features()}))\n"
    )
    try:
        imported = json.loads(
            subprocess.check_output(
                [sys.executable, "-c", program], cwd=ROOT, env=environment, text=True
            )
        )
    except (subprocess.CalledProcessError, json.JSONDecodeError) as error:
        raise ValueError(
            "cannot inspect compiled extension features; rebuild with just py-sync-correctness"
        ) from error
    if not isinstance(imported, dict):
        raise TypeError("invalid compiled extension report")
    package, binary, features = (
        imported.get("package"),
        imported.get("native"),
        imported.get("features"),
    )
    if not isinstance(package, str) or not isinstance(binary, str):
        raise TypeError("invalid compiled extension import paths")
    package_path, binary_path = Path(package).resolve(), Path(binary).resolve()
    expected = (ROOT / "python/pse").resolve()
    if package_path != expected / "__init__.py" or binary_path.parent != expected:
        raise ValueError("native Python import belongs to a different checkout")
    if (
        not isinstance(features, list)
        or any(not isinstance(feature, str) for feature in features)
        or len(features) != len(set(features))
    ):
        raise ValueError("invalid compiled extension feature flags")
    if validated and "force-validate" not in features:
        raise ValueError(
            "Python correctness requires a force-validated extension; run just py-sync-correctness"
        )
    if native and "native-solvers" not in features:
        raise ValueError(
            "native Python correctness requires the linked extension; run just py-sync-correctness native"
        )
    with binary_path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {"binary": str(binary_path), "sha256": digest, "features": features}


def execution_environment(
    environment: Mapping[str, str], artifact: Mapping[str, object]
) -> dict[str, str]:
    """Carry observations to pytest, which independently verifies its own import."""
    return {
        **environment,
        "PSE_NATIVE_EXPECTED_BINARY": str(artifact["binary"]),
        "PSE_NATIVE_EXPECTED_SHA256": str(artifact["sha256"]),
        "PSE_NATIVE_EXPECTED_FEATURES": json.dumps(artifact["features"]),
    }


def develop_command(root: Path, mode: str, profile: str) -> list[str]:
    """The actual Maturin correctness installer selection shared by strict capture."""
    if mode not in {"dev", "native"}:
        raise ValueError("correctness extension mode must be dev or native")
    return [
        str(root / ".venv/bin/maturin"), "develop", "--uv", "--profile", profile,
        "--locked", "--features",
        "force-validate,native-solvers" if mode == "native" else "force-validate",
    ]


def sync(mode: str, profile: str = "dev") -> int:
    """Build the explicitly validated correctness extension before test collection."""
    if mode == "auto":
        # This selects a build, never certifies its features. The resulting imported
        # binary is checked independently before correctness execution.
        from scripts import doctor  # noqa: PLC0415

        mode = doctor.extension_kind()
        if mode == "absent":
            mode = "dev"
    if mode not in {"dev", "native"}:
        raise ValueError("correctness extension mode must be dev, native or auto")
    wrapper = [str(ROOT / "scripts/pse-env"), "--recipe-default-class", "compile"]
    if mode == "native":
        wrapper.append("--native=solver,klu,isolation,uno,petsc")
    wrapper.append("--")
    environment = {**os.environ, "VIRTUAL_ENV": str(ROOT / ".venv")}
    commands = [
        ["uv", "sync", "--locked", "--no-install-project"],
        develop_command(ROOT, mode, profile),
        [
            "cargo",
            "run",
            "-p",
            "xtask",
            "--no-default-features",
            "--locked",
            "--",
            "python-stubs",
        ],
    ]
    for command in commands:
        result = subprocess.call([*wrapper, *command], cwd=ROOT, env=environment)
        if result:
            return result
    return 0


if __name__ == "__main__":
    if len(sys.argv) not in {3, 4} or sys.argv[1] != "sync":
        raise SystemExit(
            "usage: python -m scripts.python_extension sync dev|native|auto [profile]"
        )
    raise SystemExit(sync(sys.argv[2], sys.argv[3] if len(sys.argv) == 4 else "dev"))

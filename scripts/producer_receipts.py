# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Observe existing producer artifacts without importing capture/test orchestration."""

from __future__ import annotations

import hashlib
import json
import re
import subprocess
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from collections.abc import Mapping


def read_object(path: Path) -> dict[str, object]:
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise TypeError(f"Expected JSON object: {path}")
    return value


def hash_value(value: object) -> bool:
    return (
        isinstance(value, str)
        and re.fullmatch(r"(?:blake3:)?[0-9a-f]{64}", value) is not None
    )


def file_observation(path: Path) -> dict[str, object]:
    path = path.absolute()
    with path.open("rb") as stream:
        digest = hashlib.file_digest(stream, "sha256").hexdigest()
    return {
        "path": str(path),
        "sha256": digest,
        "canonical": str(path.resolve(strict=True)),
        "mode": path.stat().st_mode,
    }


def current_receipt(receipt: dict[str, object]) -> bool:
    version = receipt.get("receipt_version")
    return isinstance(version, int) and not isinstance(version, bool) and version == 2


def receipt_artifact(receipt: dict[str, object], role: str) -> dict[str, object]:
    if not current_receipt(receipt):
        raise ValueError(f"{role} has unsupported deployment receipt interpretation")
    association = receipt.get("deployment")
    if not isinstance(association, dict) or not isinstance(
        association.get("artifact"), dict
    ):
        raise TypeError(f"{role} lacks actual artifact association")
    artifact = association["artifact"]
    if not isinstance(artifact.get("path"), str) or artifact != file_observation(
        Path(artifact["path"])
    ):
        raise ValueError(f"{role} actual artifact changed or is incomplete")
    return artifact


def verify_native_provider(
    receipt_path: Path, package_id: str, out_dir: Path, environment: Mapping[str, str]
) -> bool:
    """Pure verification through the single Rust capture/framing owner; no Cargo."""
    try:
        value = read_object(receipt_path)
        if (
            not current_receipt(value)
            or value.get("persistent_reuse_eligible") is not True
            or value.get("reasons") != []
        ):
            return False
        association = value.get("deployment")
        if not isinstance(association, dict) or not isinstance(
            association.get("workspace_root"), str
        ):
            return False
        tool = Path(association["workspace_root"]) / "target/debug/xtask"
        result = subprocess.run(
            [
                str(tool),
                "verify-native-provider",
                "--receipt",
                str(receipt_path),
                "--package-id",
                package_id,
                "--out-dir",
                str(out_dir),
            ],
            env=dict(environment),
            capture_output=True,
            check=False,
        )
    except (OSError, ValueError, TypeError):
        return False
    else:
        return result.returncode == 0

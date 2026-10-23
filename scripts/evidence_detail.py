# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit detail selection, independent of index path strings or query execution."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from scripts import active_diagnostics, run_inspection


def diagnostic_detail(
    directory: Path, member: str, *, tail: int = 20, expected_digest: str | None = None
) -> dict:
    """Read one explicitly supplied owner action, verifying its current recorded digest."""
    record = active_diagnostics.read_action(directory, summary_only=True)
    if expected_digest is not None and hashlib.sha256(
        (directory / "action.json").read_bytes()
    ).hexdigest() != expected_digest.removeprefix("sha256:"):
        return {
            "member": member,
            "status": "unavailable",
            "reason": "current action differs from selected frozen action digest",
        }
    selected = [
        artifact for artifact in record["artifacts"] if artifact["member"] == member
    ]
    if len(selected) != 1 or not isinstance(selected[0].get("sha256"), str):
        return {
            "member": member,
            "status": "unavailable",
            "reason": "no exact owner-declared artifact digest",
        }
    return run_inspection.artifact_observation(
        directory.absolute() / member, selected[0]["sha256"], tail, None
    )


def invocation_detail(
    handle: str, *, registry: Path | None, subject: str | None, tail: int = 20
) -> dict:
    """The original owner resolves current custody and protects its selected report read."""
    return run_inspection.product_result(
        handle, registry=registry, subject=subject, tail=tail
    )


def copied_detail(
    directory: Path, member: str, *, tail: int = 20, expected_digest: str | None = None
) -> dict:
    """A supplied copied closure grants only its manifest's exact relative member.

    Old source paths are never followed. The descriptor remains open through digest
    verification and the bounded read, using the original artifact-file protection.
    """
    if (
        tail < 0
        or ".." in directory.parts
        or any(p.is_symlink() for p in (directory, *directory.parents))
    ):
        raise ValueError("invalid explicit copied closure")
    path = directory / "evidence-copy.json"
    if path.is_symlink() or not path.is_file() or path.stat().st_size > 262144:
        raise ValueError("bounded ordinary supplied closure manifest required")
    manifest = json.loads(path.read_text())
    if manifest.get("schema") != "selected-evidence-copy-v1" or not isinstance(
        manifest.get("members"), dict
    ):
        raise ValueError("unsupported supplied detail copy codec")
    if Path(member).name != member:
        raise ValueError("select one direct copied member")
    declared = manifest["members"].get(member)
    if not isinstance(declared, dict) or not isinstance(declared.get("sha256"), str):
        return {
            "member": member,
            "status": "unavailable",
            "reason": "not in supplied closure",
        }
    if expected_digest is not None and declared[
        "sha256"
    ] != expected_digest.removeprefix("sha256:"):
        return {
            "member": member,
            "status": "unavailable",
            "reason": "supplied member differs from selected frozen artifact digest",
        }
    return run_inspection.artifact_observation(
        directory.absolute() / member, declared["sha256"], tail, None
    )

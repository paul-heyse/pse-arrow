# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit detail selection, independent of index path strings or query execution."""
from __future__ import annotations

import json
from pathlib import Path

from scripts import active_diagnostics, run_inspection


def diagnostic_detail(directory: Path, member: str, *, tail: int = 20) -> dict:
    """Read one explicitly supplied owner action, verifying its current recorded digest."""
    record = active_diagnostics.read_action(directory, summary_only=True)
    selected = [artifact for artifact in record["artifacts"] if artifact["member"] == member]
    if len(selected) != 1 or not isinstance(selected[0].get("sha256"), str):
        return {"member": member, "status": "unavailable", "reason": "no exact owner-declared artifact digest"}
    return run_inspection.artifact_observation(directory.absolute() / member, selected[0]["sha256"], tail, None)


def invocation_detail(handle: str, *, registry: Path | None, subject: str | None, tail: int = 20) -> dict:
    """The original owner resolves current custody and protects its selected report read."""
    return run_inspection.product_result(handle, registry=registry, subject=subject, tail=tail)


def copied_detail(directory: Path, member: str, *, tail: int = 20) -> dict:
    """A supplied copied closure grants only its manifest's exact relative member.

    Old source paths are never followed. The descriptor remains open through digest
    verification and the bounded read, using the original artifact-file protection.
    """
    if tail < 0 or ".." in directory.parts or any(p.is_symlink() for p in (directory, *directory.parents)):
        raise ValueError("invalid explicit copied closure")
    manifest = json.loads((directory / "evidence-copy.json").read_text())
    if manifest.get("schema") != "selected-evidence-copy-v1" or not isinstance(manifest.get("members"), dict):
        raise ValueError("unsupported supplied detail copy codec")
    if Path(member).name != member:
        raise ValueError("select one direct copied member")
    declared = manifest["members"].get(member)
    if not isinstance(declared, dict) or not isinstance(declared.get("sha256"), str):
        return {"member": member, "status": "unavailable", "reason": "not in supplied closure"}
    return run_inspection.artifact_observation(directory.absolute() / member, declared["sha256"], tail, None)

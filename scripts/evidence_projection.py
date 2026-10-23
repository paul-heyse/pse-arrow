# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Detached, supplied-only projections of original FileStore invocation authority.

No registry initialization, terminal composition, payload reads or resource borrows
occur here. An exported reference describes evidence; it grants no live custody.
"""

from __future__ import annotations

import hashlib
import json
from pathlib import Path
from typing import NotRequired, TypedDict

from scripts import resource_store, run_inspection


class EvidenceFact(TypedDict):
    member: str
    kind: str
    status: str
    crate_name: NotRequired[str]
    test_id: NotRequired[str]
    detail_ref: NotRequired[str]


class EvidenceSnapshot(TypedDict):
    version: int
    authority: str
    codec: str
    source_id: str
    snapshot_digest: str
    coverage: str
    expected_members: list[str]
    facts: list[EvidenceFact]


def snapshot_digest(value: object) -> str:
    return hashlib.sha256(
        json.dumps(
            value, sort_keys=True, separators=(",", ":"), allow_nan=False
        ).encode()
    ).hexdigest()


def invocation_snapshot(
    handle: str, *, registry: Path | None = None
) -> EvidenceSnapshot:
    """Export exactly one authenticated invocation, including explicit missing reports."""
    root, declaration, state = run_inspection.resolve(handle, registry)
    activation = resource_store.read(root / "activation.json")
    if (
        activation.get("version") != 2
        or activation.get("format") != "exact-resource-files"
        or not isinstance(activation.get("migration"), str)
        or len(activation["migration"]) != 64
        or any(
            character not in "0123456789abcdef" for character in activation["migration"]
        )
    ):
        raise ValueError("Unsupported original FileStore authority")
    kind = declaration.get("kind")
    if kind != "rust" and kind != "python":
        raise ValueError("Unsupported original invocation kind")
    selected = declaration["selected"]
    if kind == "rust" and any(
        not isinstance(item.get("binary_id"), str) for item in selected
    ):
        raise ValueError("Rust subject lacks its original selected binary identity")
    catalog = [run_inspection.subject_identity(kind, item) for item in selected]
    if len(set(catalog)) != len(catalog):
        raise ValueError("Duplicate original selected subject")
    frozen = state.get("terminal") if state["phase"] == "frozen" else None
    if frozen is not None and not isinstance(frozen.get("complete"), bool):
        raise ValueError("Invalid original terminal completeness")
    outcomes = frozen.get("outcomes", {}) if frozen else {}
    evidence = frozen.get("evidence", {}) if frozen else {}
    if (
        not isinstance(outcomes, dict)
        or set(outcomes) - set(catalog)
        or any(
            value not in {"pass", "failure", "incomplete"}
            for value in outcomes.values()
        )
        or not isinstance(evidence, dict)
    ):
        raise ValueError("Original terminal differs from selected subject catalog")
    rows = evidence.get("results", [])
    if not isinstance(rows, list) or any(not isinstance(row, dict) for row in rows):
        raise ValueError("Invalid original report rows")
    reported: dict[str, str] = {}
    for row in rows:
        identity = row.get("nodeid")
        if (
            identity is None
            and isinstance(row.get("class"), str)
            and isinstance(row.get("name"), str)
        ):
            identity = row["class"] + "::" + row["name"]
        if not isinstance(identity, str) or identity not in catalog:
            # Original reconciliation may preserve unexpected rows as diagnostics.
            # They are not added to the selected population by this projection.
            continue
        status = row.get("status")
        if not isinstance(status, str) or identity in reported:
            raise ValueError("Invalid or duplicate original subject report")
        reported[identity] = status
    complete = bool(frozen and frozen.get("complete") is True)
    code = evidence.get("workload_exit")
    command_status = (
        "complete_pass"
        if complete and code == 0
        else "complete_failure"
        if complete
        else "command_failure"
        if type(code) is int and code not in {0, 1, 100}
        else "incomplete"
        if frozen
        else state["phase"]
    )
    detail = str(root / "invocations" / (declaration["nonce"] + ".json"))
    facts: list[EvidenceFact] = [
        {
            "member": "invocation",
            "kind": "invocation",
            "status": command_status,
            "detail_ref": detail,
        }
    ]
    for identity, selection in zip(catalog, selected, strict=True):
        raw = reported.get(identity)
        status = (
            raw
            if isinstance(raw, str)
            and raw in {"skipped", "reused", "transferred", "not_run", "error"}
            else "unreported"
            if identity not in outcomes
            else outcomes[identity]
        )
        fact: EvidenceFact = {
            "member": "subject:" + identity,
            "kind": kind + "_test",
            "status": status,
            "test_id": identity,
            "detail_ref": detail + "#subject=" + identity,
        }
        # binary-id is the owner's selected executable identity, not a crate name.
        # Only project a crate mapping when the owner explicitly declares one.
        if isinstance(selection.get("crate_name"), str):
            fact["crate_name"] = selection["crate_name"]
        facts.append(fact)
    return {
        "version": 1,
        "authority": "filestore:"
        + snapshot_digest({"registry": str(root), "activation": activation}),
        "codec": "exact-resource-files-invocation-v2",
        "source_id": declaration["nonce"],
        "snapshot_digest": snapshot_digest(
            {"declaration": declaration, "state": state}
        ),
        "coverage": "complete" if complete else "partial",
        "expected_members": [fact["member"] for fact in facts],
        "facts": facts,
    }

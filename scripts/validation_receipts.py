# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Authentication and classification of the assessment runner's existing receipts."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def changed(before: dict, after: dict) -> list[str]:
    return sorted(
        key for key in before.keys() | after.keys() if before.get(key) != after.get(key)
    )


def qualified(check: dict) -> bool:
    """A nonblocking finding differs from a failed invocation or missing evidence."""
    return (
        check["status"] == "passed"
        or (check.get("role") == "advisory" and check["status"] == "findings")
        or (check.get("role") == "deferred" and check["status"] == "unsupported")
    )


def verify_native(native: dict) -> None:
    """Revalidate the actual binary and linked library bytes, including retained evidence."""
    if native.get("schema") != "native-profile-v1" or not native.get("files"):
        raise ValueError("missing native identity")
    for name, expected in native["files"].items():
        with Path(name).open("rb") as stream:
            if hashlib.file_digest(stream, "sha256").hexdigest() != expected:
                raise ValueError("native binary or linked library changed")
    if native.get("threads") != dict.fromkeys(
        ("OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"), "1"
    ):
        raise ValueError("native thread budget differs")


def classify(check: dict, output: Path) -> None:
    if check["status"] == "interrupted":
        return
    if (
        check["gate"] == "doc-lint"
        and check["exit_code"] == 2
        and "doc-lint: not implemented in phase 0 (see docs/adr/register.md)"
        in (output / check["log"]).read_text()
    ):
        check["status"] = "unsupported"
        check["authority"] = "R-20"
    if check["gate"] in {"native-test", "native-python"}:
        path = output / f"{check['gate']}-native.json"
        try:
            native = json.loads(path.read_text())
            verify_native_capture(native, check)
            verify_native(native)
            check["native"] = native
            check["artifacts"][path.name] = digest(path)
        except (OSError, ValueError, KeyError, TypeError) as error:
            check["status"] = "failed"
            check["report_errors"].append(str(error))
    tool = output / f"{check['gate']}-tool.json"
    if check.get("role") == "advisory" or check["gate"] == "unsafe-surface":
        if not tool.is_file():
            check["status"] = "failed"
            check["report_errors"].append("missing tool execution receipt")
            return
        try:
            record = json.loads(tool.read_text())
            validate_tool(record, check)
            check["tool"] = record
            check["status"] = record["status"]
            check["artifacts"][tool.name] = digest(tool)
        except (OSError, ValueError, KeyError) as error:
            check["status"] = "failed"
            check["report_errors"].append(str(error))


def verify_native_capture(native: dict, check: dict) -> None:
    if native["captured"] < check["started"]:
        raise ValueError("stale native identity")


def validate_tool(record: dict, check: dict) -> None:
    """Require tool receipt identity, current invocation and consistent status."""
    if record["gate"] != check["gate"] or record["started"] < check["started"]:
        raise ValueError("wrong or stale tool receipt")
    expected_exit = {"passed": 0, "findings": 1, "failed": 2}.get(record["status"])
    if (
        expected_exit is None
        or check["exit_code"] != expected_exit
        or not record.get("invocations")
    ):
        raise ValueError(
            "tool status contradicts its wrapper exit or lacks invocations"
        )
    if check["report_errors"]:
        raise ValueError("tool report does not override collection failures")


def reuse_checks(
    parent: Path,
    snapshot: dict,
    environment: dict,
    scope: list[dict],
    reuse: set[str],
    transfer: set[str],
    reason: str | None,
) -> list[dict]:
    """Carry only explicitly selected observations, keeping their original origin.

    File identity is an input to the decision, never an inferred impact analysis.
    A reviewed transfer records a rationale; it is never reported as a new test.
    """
    if reuse & transfer:
        raise ValueError("a gate cannot be both reused and transferred")
    if transfer and not (reason and reason.strip()):
        raise ValueError("reviewed transfer requires a rationale")
    prior = json.loads((parent / "checks.json").read_text())
    if prior.get("version") != 4:
        raise ValueError("reuse requires an ordinary version 4 report")
    declarations = {g["name"]: g for g in json.loads(json.dumps(scope))}
    previous = {g["name"]: g for g in prior["scope"]}
    observations = {c["gate"]: c for c in prior["checks"]}
    retained = []
    for name in sorted(reuse | transfer):
        if name not in declarations or previous.get(name) != declarations[name]:
            raise ValueError("reused command scope differs")
        check = observations[name]
        if not qualified(check):
            raise ValueError("only successful observations can be retained")
        origin = Path(check.get("origin", str(parent)))
        origin_report = origin / "checks.json"
        origin_digest = check.get("origin_digest", digest(origin_report))
        if digest(origin_report) != origin_digest:
            raise ValueError("changed original report")
        original = json.loads(origin_report.read_text())
        changes = changed(original["source_files"], snapshot)
        for artifact, expected in check.get("artifacts", {}).items():
            path = (origin / artifact).resolve()
            path.relative_to(origin.resolve())
            if digest(path) != expected:
                raise ValueError("changed retained artifact")
        if name in reuse:
            if changes or original["environment"] != environment:
                raise ValueError(
                    "unchanged-input reuse requires identical inputs and environment"
                )
            if check.get("native"):
                verify_native(check["native"])
        retained.append(
            {
                **check,
                "origin": str(origin),
                "origin_digest": origin_digest,
                "evidence_kind": "unchanged-input-reuse"
                if name in reuse
                else "reviewed-transfer",
                "changed_inputs": changes,
                "transfer_reason": reason if name in transfer else None,
            }
        )
    return retained


def native_selection(log: str) -> list[dict]:
    """Read nextest's actual selected identities, not a source-code name guess."""
    inventories = []
    for line in log.splitlines():
        try:
            value = json.loads(line)
        except ValueError:
            continue
        if isinstance(value, dict) and "rust-suites" in value:
            inventories.append(value)
    if len(inventories) != 1:
        raise ValueError("missing or ambiguous nextest enumeration")
    selected = [
        {"class": suite["binary-id"], "name": name}
        for suite in inventories[0]["rust-suites"].values()
        for name, test in suite["testcases"].items()
        if test["filter-match"]["status"] == "matches"
    ]
    if not selected:
        raise ValueError("no tests selected")
    return selected


def reconcile(check: dict) -> None:
    """Preserve unexecuted selected identities as explicit not-run results."""

    def identity(item: dict) -> tuple:
        return (
            (item["nodeid"],) if item.get("nodeid") else (item["class"], item["name"])
        )

    if "selected" in check:
        selected = {identity(item): item for item in check["selected"]}
        actual = {identity(item) for item in check["results"]}
        if len(selected) != len(check["selected"]):
            check["report_errors"].append("duplicate selected identities")
        for key in actual - selected.keys():
            check["report_errors"].append(f"unexpected executed identity: {key}")
        for key in selected.keys() - actual:
            check["results"].append(
                {
                    **selected[key],
                    "status": "not_run",
                    "details": "selected but no terminal test receipt",
                }
            )
        if actual != selected.keys():
            check["report_errors"].append("selected and executed identities differ")
    if check["report_errors"] and check["status"] != "interrupted":
        check["status"] = "failed"


def python_selection(path: Path) -> list[dict]:
    selected = path.read_text().splitlines()
    if not selected or any(not item for item in selected):
        raise ValueError("invalid Python collection")
    return [{"nodeid": item} for item in selected]


def enumeration_selection(output: Path, listing: dict) -> list[dict]:
    if listing["status"] != "passed":
        raise ValueError("native enumeration failed")
    return native_selection((output / listing["log"]).read_text())

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Authentication and classification of the assessment runner's existing receipts."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

from scripts import test_resources
from scripts.validation_scope import input_identity


def digest(path: Path) -> str:
    with path.open("rb") as stream:
        return hashlib.file_digest(stream, "sha256").hexdigest()


def require_fresh(path: Path, started: float, kind: str) -> None:
    """Refuse selection evidence older than the invocation that produced it."""
    if path.stat().st_mtime < started:
        raise ValueError(f"stale {kind} selection")


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
    if check.get("native_required") or check["gate"] in {
        "native-test",
        "native-python",
    }:
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
    if native["captured"] < check.get("invocation_started", check["started"]):
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
    consumer: Path | None = None,
) -> list[dict]:
    """Carry only explicitly selected observations, keeping their original origin.

    File identity is an input to the decision, never an inferred impact analysis.
    A reviewed transfer records a rationale; it is never reported as a new test.
    """
    if reuse & transfer:
        raise ValueError("a gate cannot be both reused and transferred")
    if transfer and not (reason and reason.strip()):
        raise ValueError("reviewed transfer requires a rationale")

    with test_resources.reference_report(parent, consumer):
        return _reuse_guarded(
            parent, snapshot, environment, scope, reuse, transfer, reason, consumer
        )


def _reuse_guarded(
    parent: Path,
    snapshot: dict,
    environment: dict,
    scope: list[dict],
    reuse: set[str],
    transfer: set[str],
    reason: str | None,
    consumer: Path | None,
) -> list[dict]:
    prior = json.loads((parent / "checks.json").read_text())
    if prior.get("version") != 5:
        raise ValueError("reuse requires an current version 5 report")
    if not prior.get("input_coverage"):
        raise ValueError("retained report lacks trustworthy input coverage")
    declarations = {g["name"]: g for g in json.loads(json.dumps(scope))}
    previous = {g["name"]: g for g in prior["scope"]}
    observations = {c["gate"]: c for c in prior["checks"]}
    retained = []
    for name in sorted(reuse | transfer):
        if name in {"features-combinations", "features-no-default"}:
            raise ValueError(
                "feature gates resume combinations through fresh enumeration; whole-gate reuse/transfer is unsupported"
            )
        if name not in declarations or previous.get(name) != declarations[name]:
            raise ValueError("reused command scope differs")
        check = observations[name]
        if check.get("invocation") != declarations[name]:
            raise ValueError("recorded invocation identity differs")
        if not qualified(check):
            raise ValueError("only successful observations can be retained")
        origin = Path(check.get("origin", str(parent)))

        with test_resources.reference_report(origin, consumer):
            origin_report = origin / "checks.json"
            origin_digest = (
                check["origin_digest"]
                if "origin_digest" in check
                else digest(origin_report)
            )
            if digest(origin_report) != origin_digest:
                raise ValueError("changed original report")
            # Preserve syntax validation of the authenticated original report.
            json.loads(origin_report.read_text())
            current_inputs = input_identity(
                declarations[name]["input_scope"], snapshot, environment
            )
            if not check.get("inputs"):
                raise ValueError("missing scoped input identity")
            changes = changed(check["inputs"]["files"], current_inputs["files"])
            for artifact, expected in check.get("artifacts", {}).items():
                path = (origin / artifact).resolve()
                path.relative_to(origin.resolve())
                if digest(path) != expected:
                    raise ValueError("changed retained artifact")
            if name in reuse:
                if check["inputs"] != current_inputs:
                    raise ValueError(
                        "unchanged-input reuse requires identical inputs and environment"
                    )
                if check.get("native"):
                    verify_native(check["native"])
            transfers = list(check.get("applicability_transfers", []))
            if name in transfer:
                transfers.append(
                    {
                        "from_inputs": check["inputs"],
                        "to_inputs": current_inputs,
                        "changed_inputs": changes,
                        "reason": reason,
                    }
                )
        retained.append(
            {
                **check,
                "inputs": current_inputs,
                "observed_inputs": check.get("observed_inputs", check["inputs"]),
                "origin": str(origin),
                "origin_digest": origin_digest,
                "evidence_kind": "unchanged-input-reuse"
                if name in reuse
                else "reviewed-transfer",
                "changed_inputs": changes
                if name in transfer
                else check.get("changed_inputs", []),
                "transfer_reason": reason
                if name in transfer
                else check.get("transfer_reason"),
                "applicability_transfers": transfers,
            }
        )
    return retained


def native_inventory(log: str) -> dict:
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
    inventory = inventories[0]
    suites = inventory["rust-suites"]
    if not isinstance(suites, dict):
        raise ValueError("unsupported nextest suite catalogue")
    for suite in suites.values():
        if (
            not isinstance(suite, dict)
            or not isinstance(suite.get("binary-id"), str)
            or not isinstance(suite.get("testcases"), dict)
        ):
            raise ValueError("unsupported nextest binary catalogue")
        for name, test in suite["testcases"].items():
            if (
                not isinstance(name, str)
                or not isinstance(test, dict)
                or not isinstance(test.get("filter-match"), dict)
                or test["filter-match"].get("status") not in {"matches", "mismatch"}
            ):
                raise ValueError("unsupported nextest filter-match protocol")
    return inventory


def native_selection(log: str) -> list[dict]:
    inventory = native_inventory(log)
    selected = [
        {"class": suite["binary-id"], "name": name}
        for suite in inventory["rust-suites"].values()
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
        if len(actual) != len(check["results"]):
            check["report_errors"].append("duplicate terminal identities")
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


def feature_execution(record: dict, literal: list[str]) -> None:
    """Admit only actual scoped correctness composition, never a display string."""
    if (
        record.get("schema") != "cargo-feature-execution-v1"
        or record.get("literal_argv") != literal
        or record.get("correctness") != "arrow-unit-graph-force-validation-v1"
        or not record.get("effective_environment")
        or not record.get("validation_graphs")
        or not isinstance(record.get("actual_argv"), list)
        or record["actual_argv"][1:2] != ["check"]
    ):
        raise ValueError(
            "feature execution lacks literal scope or actual force-validation context"
        )
    for graph in record["validation_graphs"]:
        if not graph.get("argv") or len(graph.get("sha256", "")) != 64:
            raise ValueError("missing actual validation graph identity")


def feature_campaign(directory: Path) -> dict:
    """Validate completeness against the frozen enumeration, including missing rows."""
    report = json.loads((directory / "feature-campaign.json").read_text())
    if report.get("schema") != "validation-feature-campaign-v1":
        raise ValueError("feature reuse requires a current complete-identity campaign")
    commands = report["frozen"]["ordered_literal_argv"]
    combinations = report["combinations"]
    if not commands or len(commands) != len(combinations):
        raise ValueError("missing feature combination from frozen enumeration")
    for index, (command, combination) in enumerate(
        zip(commands, combinations, strict=True)
    ):
        if combination["index"] != index or combination["literal_argv"] != command:
            raise ValueError(
                "feature combination ordering differs from frozen enumeration"
            )
        if (
            combination["status"] == "passed"
            and combination.get("evidence_kind") != "unchanged-input-reuse"
        ):
            attempts = combination["attempts"]
            if (
                not attempts
                or attempts[-1]["status"] != "passed"
                or attempts[-1]["exit_code"] != 0
            ):
                raise ValueError("feature pass lacks a completed successful attempt")
            feature_execution(attempts[-1]["execution"], command)
    if report.get("required_checks_covered") and (
        not report.get("source_unchanged")
        or not report.get("complete")
        or any(combination["status"] != "passed" for combination in combinations)
    ):
        raise ValueError("incomplete or failed feature combinations cannot qualify")
    return report


def feature_combination_origin(origin: dict, frozen: dict, index: int) -> None:
    directory = Path(origin["path"]).resolve()
    if (
        origin["index"] != index
        or digest(directory / "feature-campaign.json") != origin["sha256"]
    ):
        raise ValueError("changed original feature campaign")
    report = feature_campaign(directory)
    if report["frozen"] != frozen or not report.get("source_unchanged"):
        raise ValueError(
            "feature reuse requires the identical full Rust/compiler/native closure"
        )
    combination = report["combinations"][index]
    if (
        combination["status"] != "passed"
        or combination.get("evidence_kind") != "executed"
    ):
        raise ValueError("feature reuse requires an original executed pass")
    attempt = combination["attempts"][-1]
    for name, expected in attempt["artifacts"].items():
        path = (directory / name).resolve()
        path.relative_to(directory)
        if digest(path) != expected:
            raise ValueError("changed original feature artifact")

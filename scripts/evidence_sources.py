# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Owner-decoded explicit assessment, diagnostic, host and measurement sources.

This module never enumerates storage to discover sources. Paths are explicit inputs;
references in compact snapshots are descriptive and never opened by query execution.
"""

from __future__ import annotations

import csv
import hashlib
import json
import math
import time
from typing import TYPE_CHECKING

from scripts import (
    active_diagnostics,
    case_measure,
    evidence_projection,
    host_admission,
    storage_health,
    test_run,
    validation_receipts,
    validation_result,
)

if TYPE_CHECKING:
    from pathlib import Path

    from scripts.evidence_projection import EvidenceFact, EvidenceSnapshot

# These are declarations, not a free-form container for raw reports or logs.
CONTEXT_FIELDS = (
    "workload",
    "input",
    "revision",
    "build",
    "profile",
    "features",
    "provider",
    "isa",
    "capture",
    "host",
    "environment",
    "source",
    "mode",
    "process",
)


def context_fact(value: dict[str, object]) -> EvidenceFact:
    if set(value) - set(CONTEXT_FIELDS):
        raise ValueError("undeclared evidence context field")
    encoded = json.dumps(
        {key: value.get(key) for key in CONTEXT_FIELDS},
        sort_keys=True,
        allow_nan=False,
        separators=(",", ":"),
    )
    if len(encoded) > 4096:
        raise ValueError("context exceeds compact metadata bound")
    identity = evidence_projection.snapshot_digest(json.loads(encoded))
    return {
        "member": "context:" + identity,
        "kind": "context",
        "status": "declared",
        "context_id": identity,
        "context_json": encoded,
    }


def compact(value: object) -> str | None:
    """Retain an exact content identity for bulky owner context, never raw logs."""
    return None if value is None else evidence_projection.snapshot_digest(value)


def source_snapshot(
    authority: str,
    codec: str,
    source_id: str,
    original: object,
    facts: list[EvidenceFact],
    *,
    complete: bool,
) -> EvidenceSnapshot:
    if len({fact["member"] for fact in facts}) != len(facts):
        raise ValueError("duplicate original evidence membership")
    return {
        "version": 2,
        "authority": authority,
        "codec": codec,
        "source_id": source_id,
        "snapshot_digest": evidence_projection.snapshot_digest(original),
        "coverage": "complete" if complete else "partial",
        "expected_members": [fact["member"] for fact in facts],
        "facts": facts,
    }


def link_fact(
    member: str, relation: str, target: str, *, status: str = "declared"
) -> EvidenceFact:
    return {
        "member": member,
        "kind": "declared_link",
        "status": status,
        "link_relation": relation,
        "link_target": target,
    }


def inventory_mapping(
    path: Path, *, selected: list[dict[str, str]] | None = None
) -> dict[str, str]:
    """Use the owner's actual nextest decoder and explicit suite package-name only."""
    inventory = validation_receipts.native_inventory(path.read_text())
    if selected is not None:
        original = {
            (item["binary_id"], item["name"], item.get("executable"))
            for item in selected
        }
        supplied = {
            (item["binary_id"], item["name"], item.get("executable"))
            for item in test_run.rust_selected(inventory)
        }
        # An original declaration can omit executable paths, but not subject identity.
        if {(b, n) for b, n, _ in original} != {(b, n) for b, n, _ in supplied} or any(
            e is not None and (b, n, e) not in supplied for b, n, e in original
        ):
            raise ValueError(
                "supplied inventory differs from original invocation selection"
            )
    result = {}
    for suite in inventory["rust-suites"].values():
        binary = suite["binary-id"]
        package = suite.get("package-name")
        if not isinstance(binary, str) or (
            package is not None and not isinstance(package, str)
        ):
            raise ValueError("invalid nextest declared package mapping")
        if package is not None:
            if binary in result and result[binary] != package:
                raise ValueError("ambiguous nextest package mapping")
            result[binary] = package
    return result


def assessment_snapshot(directory: Path) -> EvidenceSnapshot:
    receipt = validation_result.read_receipt(directory)
    recorded = {check["gate"]: check for check in receipt["checks"]}
    ctx = context_fact(
        {
            "source": compact(receipt.get("source_files")),
            "environment": compact(receipt.get("environment")),
            "mode": receipt.get("mode"),
        }
    )
    facts: list[EvidenceFact] = [
        {
            "member": "assessment",
            "kind": "assessment",
            "status": "complete" if receipt["complete"] else "incomplete",
            "context_id": ctx["context_id"],
        },
        ctx,
    ]
    for gate in receipt["scope"]:
        name = gate["name"]
        check = recorded.get(name)
        facts.append(
            {
                "member": "gate:" + name,
                "kind": "assessment_gate",
                "status": check["status"] if check else "unreported",
                "context_id": ctx["context_id"],
            }
        )
        if check is None:
            continue
        code = check.get("exit_code")
        if type(code) is int and code not in {0, 1, 100}:
            facts.append(
                {
                    "member": "command:" + name,
                    "kind": "assessment_command",
                    "status": "command_failure",
                    "context_id": ctx["context_id"],
                }
            )
        selected = check.get("selected", [])
        if not isinstance(selected, list):
            raise TypeError("invalid assessment selection")

        def identity(row: dict) -> str:
            if isinstance(row.get("nodeid"), str):
                return row["nodeid"]
            if isinstance(row.get("class"), str) and isinstance(row.get("name"), str):
                return row["class"] + "::" + row["name"]
            raise ValueError("assessment lacks exact test identity")

        catalog = {identity(item): item for item in selected}
        if len(catalog) != len(selected):
            raise ValueError("duplicate assessment selection")
        reports = check.get("results", [])
        if not isinstance(reports, list):
            raise TypeError("invalid assessment results")
        reports_by_id = {identity(item): item for item in reports}
        if len(reports_by_id) != len(reports):
            raise ValueError("duplicate assessment report")
        gate_ctx = context_fact(
            {
                "source": compact(check.get("inputs")),
                "profile": check.get("profile"),
                "mode": check.get("mode"),
                "build": compact(check.get("native")),
            }
        )
        if gate_ctx["member"] not in {f["member"] for f in facts}:
            facts.append(gate_ctx)
        for test_id, item in catalog.items():
            report = reports_by_id.get(test_id)
            status = report.get("status", "unreported") if report else "unreported"
            fact: EvidenceFact = {
                "member": "test:" + name + ":" + test_id,
                "kind": "python_test" if "nodeid" in item else "rust_test",
                "status": "pass" if status == "passed" else status,
                "test_id": test_id,
                "context_id": gate_ctx["context_id"],
            }
            if "class" in item:
                fact["binary_id"] = item["class"]
            if isinstance(item.get("crate_name"), str):
                fact["crate_name"] = item["crate_name"]
            facts.append(fact)
        # Preserve actual reuse rather than interpreting copied passing rows as new execution.
        if isinstance(check.get("origin"), str):
            facts.append(
                link_fact(
                    "origin:" + name,
                    "assessment_origin",
                    check["origin"],
                    status="reused",
                )
            )
    return source_snapshot(
        "assessment:" + str(directory.absolute()),
        "assessment-v5",
        str(directory.absolute()),
        receipt,
        facts,
        complete=receipt["complete"],
    )


def diagnostic_snapshot(directory: Path) -> EvidenceSnapshot:
    # read_action's summary path checks source codec but never opens artifact bytes.
    record = active_diagnostics.read_action(directory, summary_only=True)
    ctx = context_fact(
        {
            "process": compact(record.get("subject")),
            "capture": {"action": record.get("action"), "bounds": record.get("bounds")},
        }
    )
    facts: list[EvidenceFact] = [
        {
            "member": "action",
            "kind": "diagnostic_tool",
            "status": record.get("status", "incomplete"),
            "context_id": ctx["context_id"],
            "detail_ref": "sha256:"
            + hashlib.sha256((directory / "action.json").read_bytes()).hexdigest(),
        },
        {
            "member": "target",
            "kind": "diagnostic_target",
            "status": str(record.get("target_outcome", "unknown")),
        },
        {
            "member": "science",
            "kind": "diagnostic_science",
            "status": str(record.get("scientific_outcome", "unknown")),
        },
        ctx,
    ]
    association = record.get("invocation")
    if isinstance(association, dict):
        if not isinstance(association.get("registry"), str) or not isinstance(
            association.get("identity"), str
        ):
            raise TypeError("invalid diagnostic invocation association")
        facts.append(
            link_fact(
                "link:invocation",
                "invocation",
                association["registry"] + "#" + association["identity"],
            )
        )
    if isinstance(record.get("operation"), str):
        facts.append(link_fact("link:operation", "operation", record["operation"]))
    # Links remain original owner declarations, never proximity or causal inference.
    for key in ("storage_health", "run", "attempt", "provider"):
        if isinstance(record.get(key), str):
            facts.append(link_fact("link:" + key, key, record[key]))
    for artifact in record["artifacts"]:
        name = artifact["member"]
        digest = artifact.get("sha256")
        facts.append(
            {
                "member": "artifact:" + name,
                "kind": "diagnostic_artifact",
                "status": "declared" if isinstance(digest, str) else "unavailable",
                "detail_ref": "sha256:" + digest
                if isinstance(digest, str)
                else "unavailable",
            }
        )
    for fact in facts:
        fact["context_id"] = ctx["context_id"]
    return source_snapshot(
        "diagnostic-action:" + str(directory.absolute()),
        active_diagnostics.SCHEMA,
        str(directory.absolute()),
        record,
        facts,
        complete=record.get("status") not in {None, "incomplete"},
    )


def storage_health_snapshot(path: Path) -> EvidenceSnapshot:
    """Project one original health snapshot; never inspect live service or engine files."""
    import msgspec  # noqa: PLC0415 -- host codec remains independent of product/native API

    record = storage_health.read(path)
    original = msgspec.to_builtins(record)
    reference = storage_health.reference(record)
    identity = record.service.identity
    context = context_fact(
        {
            "process": msgspec.to_builtins(record.observer),
            "host": compact(msgspec.to_builtins(record.host)),
            "capture": {
                "started_unix_ns": record.started_unix_ns,
                "finished_unix_ns": record.finished_unix_ns,
                "service": msgspec.to_builtins(identity),
                "correspondence": record.correspondence,
            },
        }
    )
    facts: list[EvidenceFact] = [
        context,
        {"member": "health", "kind": "diagnostic_tool", "status": record.tool_outcome, "detail_ref": reference},
        {"member": "service", "kind": "diagnostic_target", "status": record.service_outcome},
        {"member": "science", "kind": "diagnostic_science", "status": record.scientific_outcome},
        link_fact("link:storage_health", "storage_health", reference),
    ]
    if identity is not None:
        facts.append(link_fact("link:service", "service", "selected-service:sha256:" + evidence_projection.snapshot_digest(msgspec.to_builtins(identity))))
    for process in record.cgroup.processes:
        owner = process.identity
        target = f"process:{owner.boot}:{owner.pid}:{owner.start}"
        facts.append(link_fact("link:" + target, "process", target))
    for name, field in record.metrics.items():
        facts.append({"member": "metric:" + name, "kind": "storage_health_field", "status": field.status, "detail_ref": reference})
    for scope, fields in (("host", record.host), ("configuration", record.configuration), ("cgroup", record.cgroup.fields)):
        for name, field in fields.items():
            facts.append({"member": scope + ":" + name, "kind": "storage_health_field", "status": field.status, "detail_ref": reference})
    for fact in facts:
        fact["context_id"] = context["context_id"]
    return source_snapshot(
        "storage-health:" + record.requested_state,
        storage_health.SCHEMA,
        str(path.absolute()),
        original,
        facts,
        complete=record.tool_outcome == "complete",
    )


def host_snapshot(
    directory: Path, *, deadline: float | None = None
) -> EvidenceSnapshot:
    value = host_admission.readonly_snapshot(
        directory, deadline=deadline or time.monotonic() + 10
    )
    facts: list[EvidenceFact] = []
    for identity, owner in value["owners"].items():
        ctx = context_fact(
            {
                "process": {k: owner.get(k) for k in ("pid", "start", "boot")},
                "host": compact(owner.get("units")),
            }
        )
        facts.extend(
            [
                ctx,
                {
                    "member": "allocation:" + identity,
                    "kind": "host_allocation",
                    "status": "observed",
                    "context_id": ctx["context_id"],
                },
            ]
        )
    # Equal process contexts can occur on several allocations.
    facts = list({fact["member"]: fact for fact in facts}.values())
    return source_snapshot(
        "host-admission:" + str(directory.absolute()),
        "host-admission-ledger-v1",
        str(directory.absolute()),
        value,
        facts,
        complete=True,
    )


def measurement_snapshot(
    path: Path,
    *,
    criterion: dict[str, Path] | None = None,
    invocations: dict[str, Path] | None = None,
) -> EvidenceSnapshot:
    report = json.loads(path.read_text())
    if isinstance(report, dict) and report.get("schema") == "case-smoke-v1":
        selected = report.get("selected_cases")
        if (
            report.get("measured") is not False
            or not isinstance(selected, list)
            or any(not isinstance(value, str) for value in selected)
            or len(set(selected)) != len(selected)
        ):
            raise ValueError("invalid original unmeasured case-smoke population")
        if criterion or invocations:
            raise ValueError(
                "smoke report declares no Criterion or per-invocation measurement membership"
            )
        ctx = context_fact(
            {
                "profile": report.get("profile"),
                "capture": {
                    key: report.get(key)
                    for key in (
                        "recording_mode",
                        "review_artifacts",
                        "capture_from_start",
                        "compiled_evaluator",
                    )
                },
            }
        )
        facts: list[EvidenceFact] = [ctx]
        for identity in selected:
            facts.append(
                {
                    "member": "case:" + identity,
                    "kind": "measurement_population",
                    "status": "not_measured",
                    "population_id": "case:" + identity,
                    "context_id": ctx["context_id"],
                }
            )
        return source_snapshot(
            "case-measure:" + str(path.absolute()),
            "case-smoke-v1",
            str(path.absolute()),
            report,
            facts,
            complete=True,
        )
    if (
        not isinstance(report, dict)
        or report.get("schema") != "process-cost-v5"
        or not isinstance(report.get("cases"), list)
        or not isinstance(report.get("selected_cases"), list)
    ):
        raise ValueError("unsupported case_measure report codec")
    cases = {}
    for case in report["cases"]:
        identity = case.get("case", case.get("id"))
        if not isinstance(identity, str) or identity in cases:
            raise ValueError("invalid or duplicate original case identity")
        cases[identity] = case
    selected = report["selected_cases"]
    if len(set(selected)) != len(selected) or set(cases) - set(selected):
        raise ValueError("case report differs from selected population")
    criterion = criterion or {}
    invocations = invocations or {}
    if set(invocations) - set(selected):
        raise ValueError("invocation source outside selected cases")
    if set(criterion) - set(selected):
        raise ValueError("Criterion source outside selected cases")
    facts: list[EvidenceFact] = []
    originals: dict[str, object] = {"report": report}
    for identity in selected:
        case = cases.get(identity)
        status = (
            "unreported"
            if case is None
            else str(
                case.get(
                    "status",
                    "measured" if report.get("measured") is True else "unknown",
                )
            )
        )
        case = case or {}
        ctx = context_fact(
            {
                "workload": {
                    "id": identity,
                    "declaration": compact(case.get("workload")),
                },
                "input": compact(
                    case.get("input", case.get("spec", case.get("workload")))
                ),
                "revision": report.get("revision"),
                "source": compact(report.get("source_observations_start")),
                "build": report.get("binary_sha256"),
                "profile": report.get("profile", {}).get("cargo_profile"),
                "features": report.get("profile", {}).get("features"),
                "provider": case.get("provider", report.get("provider")),
                "isa": case.get("isa"),
                "capture": case.get("capture"),
                "host": compact(report.get("host_conditions_start")),
                "environment": compact(report.get("environment_start")),
            }
        )
        facts.append(ctx)
        population = "case:" + identity
        facts.append(
            {
                "member": population,
                "kind": "measurement_population",
                "status": status,
                "population_id": population,
                "context_id": ctx["context_id"],
            }
        )
        statistics = case.get("criterion_statistics", {})
        if not isinstance(statistics, dict):
            raise TypeError("invalid original Criterion statistics")
        for name in (
            "mean_nanoseconds",
            "sample_mean_nanoseconds",
            "sample_standard_error_nanoseconds",
            "sample_min_nanoseconds",
            "sample_max_nanoseconds",
        ):
            value = statistics.get(name)
            if value is None:
                continue
            if (
                isinstance(value, bool)
                or not isinstance(value, (int, float))
                or not math.isfinite(value)
                or value < 0
            ):
                raise ValueError("invalid original reported measurement statistic")
            facts.append(
                {
                    "member": population + ":summary:" + name,
                    "kind": "measurement_summary",
                    "status": "reported",
                    "population_id": population,
                    "context_id": ctx["context_id"],
                    "value_ns": float(value),
                }
            )
        association = (
            case.get("invocation_observations", {})
            .get("criterion_membership", {})
            .get("status", "unavailable")
        )
        facts.append(
            {
                "member": population + ":association",
                "kind": "measurement_association",
                "status": association,
                "population_id": population,
                "context_id": ctx["context_id"],
            }
        )
        overhead = case.get("capture_overhead_nanoseconds")
        if (
            isinstance(overhead, (int, float))
            and not isinstance(overhead, bool)
            and math.isfinite(overhead)
            and overhead >= 0
        ):
            facts.append(
                {
                    "member": population + ":capture",
                    "kind": "capture_cost",
                    "status": "measured",
                    "population_id": population,
                    "context_id": ctx["context_id"],
                    "value_ns": float(overhead),
                }
            )
        if identity in invocations:
            supplied = invocations[identity]
            digest = hashlib.sha256(supplied.read_bytes()).hexdigest()
            if digest not in case.get("artifacts", {}).values():
                raise ValueError(
                    "invocation records lack an authentic report artifact digest"
                )
            with supplied.open() as stream:
                records = [json.loads(line) for line in stream if line.strip()]
            # Validate population through its owner without following old recorded paths.
            memory = {"records": records}
            if "iterations" in case:
                memory["iterations"] = case["iterations"]
            case_measure.invocation_population(supplied.parent, memory)
            originals[identity + ":invocations"] = records
            seen = set()
            for offset, record in enumerate(records):
                sequence = record.get("invocation_sequence")
                if sequence is not None and (
                    type(sequence) is not int or sequence < 0 or sequence in seen
                ):
                    raise ValueError("invalid or duplicate actual invocation sequence")
                seen.add(sequence)
                member = (
                    "sequence:" + str(sequence)
                    if sequence is not None
                    else "jsonl-row:" + str(offset)
                )
                status = record.get("outcome", "unknown")
                if not isinstance(status, str):
                    raise TypeError("invalid invocation outcome")
                fact: EvidenceFact = {
                    "member": population + ":invocation:" + member,
                    "kind": "harness_invocation",
                    "status": status,
                    "population_id": population,
                    "sample_id": member,
                    "context_id": ctx["context_id"],
                }
                seconds = record.get("seconds")
                if seconds is not None:
                    if (
                        isinstance(seconds, bool)
                        or not isinstance(seconds, (int, float))
                        or not math.isfinite(seconds)
                        or seconds < 0
                    ):
                        raise ValueError("invalid invocation duration")
                    fact["value_ns"] = seconds * 1e9
                facts.append(fact)
        if identity in criterion:
            supplied = criterion[identity]
            # Reuse owner validation; keep actual CSV row membership, never invocation IDs.
            digest = hashlib.sha256(supplied.read_bytes()).hexdigest()
            if digest not in case.get("artifacts", {}).values():
                raise ValueError(
                    "Criterion samples lack an authentic report artifact digest"
                )
            case_measure.samples(supplied)
            with supplied.open(newline="") as stream:
                rows = list(csv.DictReader(stream))
            originals[identity] = rows
            for offset, row in enumerate(rows):
                facts.append(
                    {
                        "member": population + ":sample:" + str(offset),
                        "kind": "criterion_sample",
                        "status": "measured",
                        "population_id": population,
                        "sample_id": "csv-row:" + str(offset),
                        "context_id": ctx["context_id"],
                        "value_ns": float(row["sample_measured_value"])
                        / float(row["iteration_count"]),
                    }
                )
    facts = list({fact["member"]: fact for fact in facts}.values())
    return source_snapshot(
        "case-measure:" + str(path.absolute()),
        "process-cost-v5",
        str(path.absolute()),
        originals,
        facts,
        complete=all(identity in cases for identity in selected),
    )

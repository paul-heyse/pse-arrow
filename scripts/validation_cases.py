# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact witnesses for the current native simulator, separate from gate success."""

from __future__ import annotations

import math
import re

from scripts.validation_receipts import qualified
from scripts.validation_scope import comprehensive, expand


def validate_manifest(value: dict) -> None:
    gates = value.get("review_gates", [])
    if gates != [f"G{i}" for i in range(1, 9)] + [f"PS-G{i}" for i in range(1, 4)]:
        raise ValueError("explicit core and process review gate inventory required")
    profiles = {p["id"]: p for p in value.get("profiles", [])}
    if not profiles or len(profiles) != len(value["profiles"]):
        raise ValueError("missing or duplicate execution profiles")
    cases = {case["id"]: case for case in value["cases"]}
    if len(cases) != len(value["cases"]):
        raise ValueError("duplicate acceptance case")
    for case in cases.values():
        profile = profiles.get(case["profile"])
        if (
            profile is None
            or case["mode"] != profile["mode"]
            or case["runner"] != profile["runner"]
        ):
            raise ValueError(f"unknown or inconsistent profile: {case['id']}")
        if case.get("phase") not in {"functional", "performance"}:
            raise ValueError("case requires an execution phase")
        if case.get("class") not in {"unit", "component", "integration", "performance"}:
            raise ValueError("case requires a test class")
        tests = case.get("tests", [])
        if case.get("artifact") == "architecture-review-v1" and tests != gates:
            raise ValueError("review witnesses differ from declared gates")
        if (
            not tests
            or len(tests) != len(set(tests))
            or any(not t or t.endswith("::") for t in tests)
        ):
            raise ValueError("case requires distinct exact test identities")
        if case.get("status") != "implemented-unqualified":
            raise ValueError("declarations cannot assert qualification")
        if not case.get("oracle") or not case.get("rationale"):
            raise ValueError("case requires an oracle and target reuse rationale")
    obligations = value.get("acceptance", [])
    if len(obligations) != 18 or {r["id"] for r in obligations} != {
        f"Q{i:02}" for i in range(1, 19)
    }:
        raise ValueError("Q01-Q18 must each be declared exactly once")
    for requirement in obligations:
        phase = "performance" if requirement["id"] == "Q18" else "functional"
        ids = requirement.get("cases", [])
        if requirement.get("phase") != phase or not ids or len(ids) != len(set(ids)):
            raise ValueError("requirement has missing/duplicate cases or wrong phase")
        if any(i not in cases or cases[i]["phase"] != phase for i in ids):
            raise ValueError("requirement references missing or wrong-phase cases")
    referenced = {i for r in obligations for i in r["cases"]}
    if referenced != cases.keys():
        raise ValueError("every case must prove a current requirement")


def identity(result: dict) -> tuple[str, str]:
    if result.get("nodeid"):
        return result["nodeid"].partition("::")[0], result["nodeid"]
    return result.get("class", ""), result.get("name", "")


def case_witnesses(case: dict, checks: list[dict]) -> list[dict]:
    required = set(case["tests"])
    owner = case["source"] if case["runner"] == "pytest" else case["binary"]
    witnesses = []
    for check in checks:
        if not qualified(check) or check.get("report_errors"):
            continue
        if check.get("mode") != case["mode"] or check.get("profile") != case["profile"]:
            continue
        if case.get("gate") and check["gate"] != case["gate"]:
            continue
        if case.get("artifact"):
            report = check.get("measurement", {})
            try:
                validate_artifact(report, case)
            except (ValueError, KeyError, TypeError):
                continue
            witnesses.append({"gate": check["gate"], "artifact": case["artifact"]})
            continue
        relevant = [
            r
            for r in check.get("results", [])
            if identity(r)[0] == owner and identity(r)[1] in required
        ]
        names = [identity(r)[1] for r in relevant]
        if (
            set(names) == required
            and len(names) == len(required)
            and all(r["status"] == "passed" for r in relevant)
        ):
            witnesses.extend(
                {"gate": check["gate"], "test": name} for name in sorted(names)
            )
    return witnesses


def coverage(declaration: dict, checks: list[dict], phase: str) -> dict:
    observed = {}
    for case in declaration.get("cases", []):
        if case.get("phase", "functional") != phase:
            continue
        witnesses = case_witnesses(case, checks)
        observed[case["id"]] = {
            "status": "passed" if witnesses else "not_observed",
            "witnesses": witnesses,
        }
    obligations = {}
    completed = {
        c["gate"] for c in checks if qualified(c) and not c.get("report_errors")
    }
    for requirement in declaration.get("acceptance", []):
        if requirement.get("phase", "functional") != phase:
            continue
        gates = {
            gate.name
            for name in requirement.get("gates", [])
            for gate in (
                comprehensive(name.removeprefix("all-"))
                if name in {"all-functional", "all-performance"}
                else expand((name,))
            )
        }
        cases = requirement.get("cases", [])
        obligations[requirement["id"]] = (
            bool(cases)
            and gates <= completed
            and all(observed.get(case, {}).get("status") == "passed" for case in cases)
        )
    required = {f"Q{i:02}" for i in range(1, 18)} if phase == "functional" else {"Q18"}
    missing = sorted(required - obligations.keys())
    return {
        "cases": observed,
        "obligations": obligations,
        "missing_obligations": missing,
        "complete": bool(observed)
        and not missing
        and all(obligations.values())
        and all(row["status"] == "passed" for row in observed.values()),
    }


def validate_artifact(report: dict, case: dict) -> None:
    """Artifact cases require complete measurements or independent gate decisions."""
    rows = report.get("cases", [])
    if (
        report.get("schema") != case["artifact"]
        or report.get("profile") != case["profile"]
    ):
        raise ValueError("wrong artifact schema or execution profile")
    if not re.fullmatch(r"[0-9a-f]{64}", report.get("source_digest", "")):
        raise ValueError("artifact requires current source digest")
    names = [r.get("id") for r in rows]
    if len(names) != len(set(names)) or set(names) != set(case["tests"]):
        raise ValueError("artifact requires every exact witness once")
    if case["artifact"] == "process-cost-v2":
        workloads = {row["id"]: row for row in case["workloads"]}
        if set(workloads) != set(case["tests"]) or len(workloads) != len(
            case["workloads"]
        ):
            raise ValueError("workload metadata must cover each exact case once")
        if report.get("compilation_timed") is not False:
            raise ValueError("Rust compilation is outside case measurements")
        if not report.get("toolchain") or not re.fullmatch(
            r"[0-9a-f]{64}", report.get("binary_digest", "")
        ):
            raise ValueError(
                "measurement requires compiled binary and toolchain identity"
            )
        threads = dict.fromkeys(
            ["OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"], "1"
        )
        if report.get("thread_environment") != threads:
            raise ValueError("nested native thread budget differs")
        native = report.get("native", {})
        if (
            native.get("schema") != "plan14-native-profile-v1"
            or native.get("profile", {}).get("id") != case["profile"]
            or native.get("threads") != threads
            or native.get("toolchain") != report["toolchain"]
            or native.get("files", {}).get(report.get("binary_path"))
            != report["binary_digest"]
            or report.get("binary_path") not in native.get("links", {})
        ):
            raise ValueError("missing compiled binary or linked library provenance")
        for row in rows:
            for key in [
                "pool_peak_bytes",
                "process_peak_rss_bytes",
                "mean_nanoseconds",
                "iterations",
            ]:
                v = row.get(key)
                if type(v) not in {int, float} or not math.isfinite(v) or v <= 0:
                    raise ValueError(f"missing or invalid measured {key}")
            if (
                row.get("sample_count", 0) < 10
                or not row.get("artifacts")
                or row.get("native_threads") != 1
            ):
                raise ValueError(
                    "missing samples, artifact provenance or native thread budget"
                )
            if any(
                not re.fullmatch(r"[0-9a-f]{64}", digest)
                for digest in row["artifacts"].values()
            ):
                raise ValueError("invalid measurement artifact digest")
            workload = workloads[row["id"]]
            if (
                row.get("workload") != workload
                or row.get("threads") != workload["threads"]
                or sorted(row.get("variables_observed", []))
                != sorted(workload["variables"])
            ):
                raise ValueError("wrong workload shape or thread profile")
            phases = row.get("phase_seconds", {})
            if not phases or any(
                type(v) not in {int, float} or not math.isfinite(v) or v < 0
                for v in phases.values()
            ):
                raise ValueError("missing or invalid measured phase costs")
            interval = row["confidence_interval"]
            if not (
                0 < interval["confidence_level"] < 1
                and 0
                < interval["lower_bound"]
                <= row["mean_nanoseconds"]
                <= interval["upper_bound"]
                < math.inf
            ):
                raise ValueError("invalid Criterion interval")
    elif case["artifact"] == "architecture-review-v1":
        for row in rows:
            if (
                row.get("gate") != row["id"]
                or row.get("source_digest") != report["source_digest"]
                or row.get("verdict") not in {"Accept", "Accept-scoped"}
            ):
                raise ValueError("wrong or stale independent gate decision")
            if (
                not row.get("reviewer")
                or not row.get("implementation_authors")
                or row["reviewer"] in row["implementation_authors"]
            ):
                raise ValueError("review is not independent of implementation authors")
            if (
                row.get("open_must_findings") != 0
                or not row.get("scope")
                or not row.get("evidence")
            ):
                raise ValueError("incomplete independent review")
            if any(
                not e.get("path")
                or not re.fullmatch(r"[0-9a-f]{64}", e.get("digest", ""))
                for e in row["evidence"]
            ):
                raise ValueError("unbound independent review evidence")
            if (
                not re.fullmatch(r"[0-9a-f]{64}", row.get("review_digest", ""))
                or row.get("artifacts", {}).get(row.get("review_path"))
                != row["review_digest"]
                or any(
                    row["artifacts"].get(e["path"]) != e["digest"]
                    for e in row["evidence"]
                )
            ):
                raise ValueError(
                    "review and evidence must be retained authenticated artifacts"
                )
    else:
        raise ValueError("unknown acceptance artifact schema")

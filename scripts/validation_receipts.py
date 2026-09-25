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


def documentation_only(names: list[str]) -> bool:
    """The sole source-change exception for retained executable measurements."""
    return all(
        name in {"README.md", "AGENTS.md", "CLAUDE.md"}
        or (name.startswith("docs/") and name.endswith(".md"))
        for name in names
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
    if native.get("schema") != "plan14-native-profile-v1" or not native.get("files"):
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
    if check["gate"] in {"plan14-native", "plan14-python"}:
        path = output / f"{check['gate']}-native.json"
        try:
            from scripts import (  # noqa: PLC0415 -- avoids validation runner cycle
                implementation_phase,
            )

            profile = next(
                p
                for p in implementation_phase.manifest(
                    Path(__file__).resolve().parents[1]
                )["profiles"]
                if p["id"] == check["profile"]
            )
            native = json.loads(path.read_text())
            if (
                native["schema"] != "plan14-native-profile-v1"
                or native["profile"] != profile
                or native["captured"] < check["started"]
                or not native["files"]
            ):
                raise ValueError("wrong or stale linked native profile")  # noqa: TRY301 -- converted to a failed evidence record
            verify_native(native)
            check["native"] = native
            check["artifacts"][path.name] = digest(path)
        except (OSError, ValueError, KeyError, TypeError, StopIteration) as error:
            check["status"] = "failed"
            check["report_errors"].append(str(error))
    if check["gate"] in {"plan14-measure", "plan14-reviews"}:
        from scripts import (  # noqa: PLC0415 -- avoids validation runner cycle
            implementation_phase,
            validation_cases,
        )
        from scripts.plan14_measure import (  # noqa: PLC0415 -- avoids validation runner cycle
            source_digest,
        )

        root = Path(__file__).resolve().parents[1]
        path = output / f"{check['gate']}.json"
        try:
            report = json.loads(path.read_text())
            case = next(
                c
                for c in implementation_phase.manifest(root)["cases"]
                if c.get("gate") == check["gate"]
            )
            validation_cases.validate_artifact(report, case)
            if report["started"] < check["started"] or report[
                "source_digest"
            ] != source_digest(root):
                raise ValueError("stale measurement or review artifact")  # noqa: TRY301 -- converted to a failed evidence record
            if check["gate"] == "plan14-measure":
                native = report["native"]
                profile = next(
                    p
                    for p in implementation_phase.manifest(root)["profiles"]
                    if p["id"] == case["profile"]
                )
                if (
                    native["profile"] != profile
                    or native["captured"] < check["started"]
                ):
                    raise ValueError("wrong or stale measurement native profile")  # noqa: TRY301 -- converted to a failed evidence record
                verify_native(native)
            for row in report["cases"]:
                for name, expected in row.get("artifacts", {}).items():
                    artifact = (output / name).resolve()
                    artifact.relative_to(output.resolve())
                    if digest(artifact) != expected:
                        raise ValueError("changed measurement artifact")  # noqa: TRY301 -- converted to a failed evidence record
                    check["artifacts"][name] = expected
            check["measurement"] = report
            check["artifacts"][path.name] = digest(path)
        except (OSError, ValueError, KeyError, TypeError, StopIteration) as error:
            check["status"] = "failed"
            check["report_errors"].append(str(error))
        return
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


def continuation(
    parent: Path,
    snapshot: dict,
    rerun: set[str],
    reason: str | None,
    *,
    plan: int,
    mode: str | None = None,
    environment: dict | None = None,
    scope: list[dict] | None = None,
) -> tuple[dict, list[dict]]:
    """Validate a parent and carry authenticated successful observations only."""
    receipt_path = parent / "checks.json"
    chain = set()
    current = parent
    while True:
        current = current.resolve()
        if current in chain:
            raise ValueError("cyclic continuation chain")
        chain.add(current)
        ancestor = json.loads((current / "checks.json").read_text())
        if ancestor.get("version") != 3 or ancestor.get("plan") != plan:
            raise ValueError("continuation ancestor belongs to another plan or format")
        if mode is not None and ancestor.get("mode") != mode:
            raise ValueError("continuation ancestor belongs to another phase")
        link = ancestor.get("parent")
        if not link:
            break
        current = Path(link["path"])
        if digest(current / "checks.json") != link["digest"]:
            raise ValueError("changed continuation ancestor")
    prior = json.loads(receipt_path.read_text())
    if prior.get("version") != 3 or prior.get("plan") != plan:
        raise ValueError("continuation requires the selected plan's version 3 receipt")
    if mode is not None and prior.get("mode") != mode:
        raise ValueError("continuation phase differs from its parent")
    if environment is not None and prior.get("environment") != environment:
        raise ValueError("execution environment differs; start a fresh assessment")
    prior_scope = {gate["name"]: gate for gate in prior.get("scope", [])}
    if scope is not None:
        mismatched = {
            gate["name"]
            for gate in scope
            if gate["name"] in prior_scope
            and prior_scope[gate["name"]] != json.loads(json.dumps(gate))
        }
        if not mismatched <= rerun:
            raise ValueError(
                "changed gate declarations require explicit --rerun selections"
            )
    changes = changed(prior["source_files"], snapshot)
    observed_changes = any(
        check.get("changed_source") and not check.get("retained")
        for check in prior["checks"]
    )
    if (changes or observed_changes) and (not reason or not rerun):
        raise ValueError(
            "changed source requires --change-reason and affected --rerun gates"
        )
    retained = []
    for check in prior["checks"]:
        if (
            check["gate"] in rerun
            or not qualified(check)
            or (
                check.get("changed_source")
                and check["gate"] in {"plan14-measure", "plan14-reviews"}
            )
        ):
            continue
        origin = Path(check.get("origin", str(parent)))
        if origin.resolve() not in chain:
            raise ValueError("check origin is outside authenticated continuation chain")
        origin_sources = json.loads((origin / "checks.json").read_text())[
            "source_files"
        ]
        origin_changes = changed(origin_sources, snapshot)
        if check["gate"] == "plan14-measure" and not documentation_only(origin_changes):
            raise ValueError(
                "executable inputs changed; rerun plan14-measure after functional qualification"
            )
        if check["gate"] == "plan14-reviews" and origin_changes:
            raise ValueError("changed source requires fresh independent plan14-reviews")
        artifacts = check.get("artifacts", {})
        if not artifacts:
            raise ValueError(f"missing authenticated artifacts: {check['gate']}")
        for name, expected in artifacts.items():
            path = origin / name
            if (
                Path(name).is_absolute()
                or ".." in Path(name).parts
                or digest(path) != expected
            ):
                raise ValueError(f"changed parent artifact: {name}")
        if check.get("native"):
            verify_native(check["native"])
        if check.get("measurement", {}).get("native"):
            verify_native(check["measurement"]["native"])
        retained.append({**check, "origin": str(origin), "retained": True})
    return {
        "path": str(parent),
        "digest": digest(receipt_path),
        "changed_source": changes,
        "change_reason": reason,
    }, retained


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

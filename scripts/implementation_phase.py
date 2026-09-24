# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Current-plan evidence validation and functional-before-performance ordering."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import tomllib
from dataclasses import asdict
from pathlib import Path

from scripts import validation as validation_tools
from scripts import validation_cases, validation_receipts
from scripts.validation_scope import comprehensive, development, expand


def active_plan(root: Path) -> int:
    value = tomllib.loads((root / "Cargo.toml").read_text())["workspace"]["metadata"][
        "pse"
    ]["execution"]["active-plan"]
    if type(value) is not int or value != 14:
        raise ValueError("unsupported active execution plan")
    return value


def validate_scope(value: dict) -> None:
    """Plan 14 requirements are authored here, never inherited from an old campaign."""
    if value.get("plan") != 14:
        raise ValueError("historical plan cannot authorize Plan 14")
    if value.get("carried") or value.get("carried_deletions"):
        raise ValueError("inherited cases are not Plan 14 scope")
    for kind in ("cases", "acceptance"):
        for row in value.get(kind, []):
            if not re.fullmatch(
                r"m(?:0[0-9]|1[0-9]|2[0-2])\.[a-z0-9.-]+"
                if kind == "cases"
                else r"Q(?:0[1-9]|1[0-8])",
                row["id"],
            ) or any(key.startswith("current_") for key in row):
                raise ValueError("legacy mapping requires a new target-specific case")
            if not re.fullmatch(r"M(?:0[0-9]|1[0-9]|2[0-2])", row.get("owner", "")):
                raise ValueError("target case requires an owner")


def manifest(root: Path, plan: int | None = None) -> dict:
    plan = execution_plan(root, plan)
    matches = list((root / "docs/plans").glob(f"{plan:02}-acceptance-cases.toml"))
    if len(matches) != 1:
        raise ValueError(f"Plan {plan}: expected one acceptance manifest")
    value = tomllib.loads(matches[0].read_text())
    if value.get("version") != 3 or value.get("plan") != plan:
        raise ValueError("wrong plan or unsupported coverage format")
    ids = [case["id"] for case in value.get("cases", [])]
    if not ids or len(set(ids)) != len(ids):
        raise ValueError("missing or duplicate case identities")
    for case in value["cases"]:
        for field in (
            "package",
            "binary",
            "tests",
            "profile",
            "class",
            "mode",
            "positive",
            "negative",
        ):
            if not case.get(field):
                raise ValueError(f"missing {field} for {case['id']}")
    validate_scope(value)
    validation_cases.validate_manifest(value)
    return value


def digest(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def validate_sources(root: Path, declaration: dict) -> None:
    """Check current evidence routes; actual executed identities come from runner reports."""
    for case in declaration.get("cases", []):
        names = case.get("sources", [case["source"]] if "source" in case else [])
        if not names:
            raise ValueError(f"missing source route: {case['id']}")
        for name in names:
            if not relative(root, name).is_file():
                raise ValueError(f"missing current source: {case['id']}: {name}")
        tests = case.get("tests", [])
        if not tests or len(tests) != len(set(tests)):
            raise ValueError(
                f"missing or duplicate exact test identities: {case['id']}"
            )
        # Exact compiled/collected identities are checked separately. Textual function
        # guesses cannot establish module paths, cfg admission or parametrization.
        for fixture in case.get("fixtures", []):
            if not relative(root, fixture).is_file():
                raise ValueError(f"missing fixture: {fixture}")


def relative(root: Path, name: str) -> Path:
    path = Path(name)
    if path.is_absolute() or ".." in path.parts:
        raise ValueError("evidence path must be repository relative")
    result = (root / path).resolve()
    result.relative_to(root.resolve())
    return result


def validate_development(root: Path, declaration: dict, directory: Path) -> None:
    """Authenticate actual runner receipt and unit coverage, using the shared collector."""
    receipt = json.loads((directory / "checks.json").read_text())
    expected_scope = json.loads(json.dumps([asdict(g) for g in development()]))
    if (
        receipt.get("version") != 3
        or receipt.get("plan") != declaration["plan"]
        or receipt.get("mode") != "development"
        or receipt.get("baseline_failures") != 0
        or receipt.get("complete") is not True
        or receipt.get("source_unchanged") is not True
        or receipt.get("required_checks_covered") is not True
        or receipt.get("provenance_errors")
        or receipt.get("scope") != expected_scope
        or receipt.get("source_files") != validation_tools.sources(root)
    ):
        raise ValueError(
            "incomplete, stale or wrong-profile development runner receipt"
        )
    _, checks = validation_receipts.continuation(
        directory,
        receipt["source_files"],
        set(),
        None,
        plan=declaration["plan"],
        mode="development",
        scope=expected_scope,
    )
    if len(checks) != len(expected_scope) or {c["gate"] for c in checks} != {
        g["name"] for g in expected_scope
    }:
        raise ValueError("missing or duplicate development runner gates")
    for check in checks:
        if (
            not validation_receipts.qualified(check)
            or check.get("exit_code") != 0
            or check.get("report_errors")
        ):
            raise ValueError("unqualified development runner")
        origin = Path(check["origin"])
        results, errors = validation_tools.collect_report(
            origin / f"{check['gate']}.xml", origin, check["gate"], check["started"]
        )
        if (
            errors
            or results != check["results"]
            or any(r["status"] != "passed" for r in results)
        ):
            raise ValueError(
                "development JUnit differs from actual successful observations"
            )
    required = [c for c in declaration["cases"] if c["class"] == "unit"]
    missing = [
        c["id"] for c in required if not validation_cases.case_witnesses(c, checks)
    ]
    if missing:
        raise ValueError(f"missing development cases: {missing}")


def execution_plan(root: Path, selected: int | None) -> int:
    active = active_plan(root)
    if selected is not None and selected != active:
        raise ValueError(
            f"historical plan {selected} cannot authorize active Plan {active}"
        )
    return active


def require_functional(root: Path, directory: Path | None) -> dict:
    """Authenticate current-plan functional evidence before measurements."""
    if directory is None:
        value = os.environ.get("PSE_FUNCTIONAL_RECEIPT")
        if not value:
            raise ValueError(
                "Plan 14 performance requires --functional-from current functional evidence"
            )
        directory = Path(value)
    directory = (root / directory).resolve()
    directory.relative_to(root.resolve())
    path = directory / "checks.json"
    receipt = json.loads(path.read_text())
    if (
        receipt.get("version") != 3
        or receipt.get("plan") != active_plan(root)
        or receipt.get("mode") != "functional"
        or receipt.get("baseline_failures") != 0
        or receipt.get("complete") is not True
        or receipt.get("source_unchanged") is not True
        or receipt.get("required_checks_covered") is not True
        or receipt.get("provenance_errors")
        or receipt.get("case_coverage", {}).get("complete") is not True
    ):
        raise ValueError(
            "performance requires complete successful current-plan functional evidence"
        )
    expected = json.loads(
        json.dumps([asdict(gate) for gate in comprehensive("functional")])
    )
    if receipt.get("scope") != expected:
        raise ValueError(
            "functional campaign does not cover the current complete scope"
        )
    checks = receipt.get("checks", [])
    if len(checks) != len(expected) or {item["gate"] for item in checks} != {
        item["name"] for item in expected
    }:
        raise ValueError("missing or duplicate functional gates")
    if any(not validation_receipts.qualified(check) for check in checks):
        raise ValueError("functional campaign has unqualified gates")
    coverage = validation_cases.coverage(manifest(root), checks, "functional")
    if not coverage["complete"] or coverage != receipt["case_coverage"]:
        raise ValueError(
            "functional case coverage is missing or differs from current evidence"
        )
    # Continuation verifies every retained origin and artifact against its complete
    # authenticated chain; the source comparison below also refuses a stale tip.
    validation_receipts.continuation(
        directory,
        receipt["source_files"],
        set(),
        None,
        plan=receipt["plan"],
        mode="functional",
        scope=expected,
    )
    if receipt["source_files"] != validation_tools.sources(root):
        raise ValueError("source differs from functional campaign")
    return {"path": str(directory), "digest": digest(path), "plan": receipt["plan"]}


def guard(root: Path, names: list[str], functional_from: Path | None = None) -> None:
    # This checkout's active plan is explicit in its manifest, not an environment override.
    manifest(root)
    gates = expand(tuple(names))
    if any(gate.phase == "performance" for gate in gates):
        require_functional(root, functional_from)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("action", choices=("guard", "acceptance", "check-manifest"))
    parser.add_argument("names", nargs="*")
    parser.add_argument("--plan", type=int, choices=(14,))
    args = parser.parse_args()
    root = Path(__file__).resolve().parents[1]
    if args.action == "acceptance" and len(args.names) != 1:
        parser.error("acceptance requires one new output directory")
    try:
        if args.action == "guard":
            guard(root, args.names)
        elif args.action == "check-manifest":
            validate_sources(root, manifest(root, execution_plan(root, args.plan)))
        elif args.action == "acceptance":
            execution_plan(root, args.plan)
            return subprocess.call(["just", "assessment", args.names[0]], cwd=root)
    except (OSError, ValueError, KeyError) as error:
        print(f"Plan {args.plan}: {error}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

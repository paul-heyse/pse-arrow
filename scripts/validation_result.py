# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read one assessment checkpoint without running or maintaining assessments."""

from __future__ import annotations

import argparse
import contextlib
import json
import os
import sys
from pathlib import Path

from scripts import assessment_store, run_inspection, test_resources
from scripts.validation_receipts import qualified


def read_receipt(run: Path) -> dict:
    """Explicit complete export through the versioned assessment owner."""
    with test_resources.borrow_report(run):
        receipt = assessment_store.export(run)
    if not isinstance(receipt, dict) or receipt.get("version") != 6:
        raise ValueError("expected a current assessment record")
    for field, kind in (
        ("scope", list),
        ("checks", list),
        ("complete", bool),
        ("source_unchanged", bool),
        ("baseline_failures", int),
        ("provenance_errors", list),
    ):
        if type(receipt.get(field)) is not kind:
            raise ValueError(f"invalid or missing {field}")
    if (
        "required_checks_covered" in receipt
        and type(receipt["required_checks_covered"]) is not bool
    ):
        raise ValueError("invalid required_checks_covered")
    names = set()
    for declaration in receipt["scope"]:
        if not isinstance(declaration, dict) or not isinstance(
            declaration.get("name"), str
        ):
            raise TypeError("invalid scope declaration")
        if declaration["name"] in names:
            raise ValueError("duplicate scope gate")
        names.add(declaration["name"])
    recorded = set()
    for check in receipt["checks"]:
        if not isinstance(check, dict):
            raise TypeError("invalid check record")
        for field in ("gate", "status", "log"):
            if not isinstance(check.get(field), str):
                raise TypeError(f"invalid or missing check {field}")
        if check["gate"] not in names or check["gate"] in recorded:
            raise ValueError("undeclared or duplicate recorded gate")
        recorded.add(check["gate"])
        if not isinstance(check.get("artifacts", {}), dict):
            raise TypeError("invalid check artifacts")
        if not all(isinstance(name, str) for name in check.get("artifacts", {})):
            raise ValueError("invalid artifact path")
        if check.get("origin") is not None and not isinstance(check["origin"], str):
            raise ValueError("invalid reuse origin")
    return receipt


def file_observation(path: Path, lines: int = 0) -> dict:
    """Report missing files explicitly and read at most the final 64 KiB of a log."""
    observation = {"path": str(path), "status": "available", "tail": []}
    try:
        with path.open("rb") as stream:
            if lines:
                stream.seek(0, os.SEEK_END)
                stream.seek(max(0, stream.tell() - 64 * 1024))
                observation["tail"] = (
                    stream.read().decode(errors="replace").splitlines()[-lines:]
                )
    except OSError as error:
        observation["status"] = (
            "missing" if isinstance(error, FileNotFoundError) else "unavailable"
        )
        observation["error"] = str(error)
    return observation


def selected_result(
    run: Path,
    receipt: dict,
    *,
    gate: str | None,
    failures: bool,
    tail: int,
    origins: dict[str, Path],
) -> dict:
    recorded = set(
        receipt.get("recorded_gates", [check["gate"] for check in receipt["checks"]])
    )
    checks = []
    for check in receipt["checks"]:
        if gate is not None and gate != check["gate"]:
            continue
        if failures and qualified(check):
            continue
        origin = origins[check["gate"]]
        checks.append(
            {
                "record": check,
                "qualified": qualified(check),
                "origin_path": str(origin),
                "log": file_observation(
                    origin / check["log"],
                    tail if gate is not None or not qualified(check) else 0,
                ),
                "artifacts": [
                    {**file_observation(origin / name), "recorded_digest": digest}
                    for name, digest in check.get("artifacts", {}).items()
                ],
            }
        )
    return {
        "run_path": str(run),
        "record_path": str(run / "checks.json"),
        "assessment": {
            key: value
            for key, value in receipt.items()
            if key not in {"checks", "scope", "recorded_gates"}
        },
        "scope": receipt["scope"],
        "checks": checks,
        "unattempted_scope": [
            declaration
            for declaration in receipt["scope"]
            if declaration["name"] not in recorded
            and (gate is None or declaration["name"] == gate)
        ],
    }


def read_assessment(
    run: Path,
    *,
    gate: str | None = None,
    failures: bool = False,
    tail: int = 0,
    source_detail: bool = False,
    full_export: bool = False,
    historical: bool = False,
) -> dict:
    """Read authentic selected assessment detail while its owners remain borrowed."""
    if tail < 0:
        raise ValueError("tail must be nonnegative")
    run = run.resolve(strict=True)
    with contextlib.ExitStack() as borrows:
        borrows.enter_context(test_resources.borrow_report(run))
        if historical:
            if not full_export:
                raise ValueError("Historical inspection requires explicit full export")
            receipt = assessment_store.original_export(run)
            receipt = {**receipt, "original_inspection": True, "automatic_reuse": False}
        else:
            image = assessment_store.root(run)
            names = {
                check["gate"]
                for check in image["checks"]
                if full_export
                or gate == check["gate"]
                or (failures and not qualified(check))
            }
            receipt = assessment_store.selected(
                run, image, gates=names, source=source_detail or full_export
            )
        if gate is not None and gate not in {item["name"] for item in receipt["scope"]}:
            raise UnknownGateError(f"gate {gate!r} is not in this run's scope")
        origins: dict[str, Path] = {}
        borrowed = {run}
        for check in receipt["checks"]:
            if gate is not None and check["gate"] != gate:
                continue
            if failures and qualified(check):
                continue
            origin = Path(check["origin"]) if check.get("origin") else run
            origin = (origin if origin.is_absolute() else run / origin).resolve()
            origins[check["gate"]] = origin
            if origin not in borrowed:
                borrows.enter_context(test_resources.borrow_report(origin))
                borrowed.add(origin)
        return selected_result(
            run, receipt, gate=gate, failures=failures, tail=tail, origins=origins
        )


class UnknownGateError(ValueError):
    """A requested gate is not part of the authentic frozen selection."""


def display(result: dict) -> None:
    print(f"Assessment run: {result['run_path']}")
    print(f"Checkpoint: {result['record_path']}")
    assessment = result["assessment"]
    for field in (
        "mode",
        "evidence",
        "complete",
        "required_checks_covered",
        "baseline_failures",
        "source_unchanged",
        "input_coverage",
        "parent",
    ):
        print(
            f"{field}: {json.dumps(assessment[field]) if field in assessment else 'not recorded'}"
        )
    for error in assessment["provenance_errors"]:
        print(f"provenance error: {error}")
    print("Recorded checks:")
    for item in result["checks"]:
        check = item["record"]
        print(
            f"  {check['gate']}: {check['status']}; role={check.get('role', 'not recorded')}; "
            f"qualified={item['qualified']}; evidence_kind={check.get('evidence_kind', 'not recorded')}"
        )
        print(f"    evidence origin: {item['origin_path']}")
        for field in (
            "exit_code",
            "spawn_error",
            "report_errors",
            "origin_digest",
            "transfer_reason",
            "applicability_transfers",
            "changed_source",
            "changed_environment",
        ):
            if field in check:
                print(f"    {field}: {json.dumps(check[field])}")
        log = item["log"]
        print(f"    log: {log['path']} ({log['status']})")
        if "error" in log:
            print(f"      {log['error']}")
        for line in log["tail"]:
            print(f"      {line}")
        for artifact in item["artifacts"]:
            print(
                f"    artifact: {artifact['path']} ({artifact['status']}); recorded_digest={artifact['recorded_digest']}"
            )
            if "error" in artifact:
                print(f"      {artifact['error']}")
    print("Unattempted scope (no recorded check):")
    for declaration in result["unattempted_scope"]:
        print(
            f"  {declaration['name']}; role={declaration.get('role', 'not recorded')}"
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("run_path", type=Path)
    parser.add_argument(
        "--kind", choices=("assessment", "product-test"), default="assessment"
    )
    parser.add_argument(
        "--registry", type=Path, help="exact resource registry for a product invocation"
    )
    parser.add_argument("--subject", help="exact selected product subject identity")
    parser.add_argument("--offset", type=int, default=0)
    parser.add_argument("--limit", type=int, default=100)
    parser.add_argument(
        "--failures", action="store_true", help="show unqualified recorded checks"
    )
    parser.add_argument(
        "--gate", help="select a declared gate, including an unattempted gate"
    )
    parser.add_argument(
        "--tail", type=int, default=20, help="log tail lines (default 20; 0 disables)"
    )
    parser.add_argument("--json", action="store_true")
    parser.add_argument("--source-detail", action="store_true")
    parser.add_argument("--full-export", action="store_true")
    parser.add_argument("--historical", action="store_true")
    args = parser.parse_args(argv)
    if args.tail < 0:
        parser.error("--tail must be nonnegative")
    if args.offset < 0 or not 1 <= args.limit <= 1000:
        parser.error("--offset must be nonnegative and --limit must be in 1..1000")
    if args.kind == "product-test":
        if args.gate is not None:
            parser.error("--gate belongs to assessment results; use --subject")
        try:
            result = run_inspection.product_result(
                str(args.run_path),
                registry=args.registry,
                subject=args.subject,
                failures=args.failures,
                offset=args.offset,
                limit=args.limit,
                tail=args.tail if args.subject is not None or args.failures else 0,
            )
        except (OSError, TypeError, KeyError, ValueError, RuntimeError) as error:
            print(
                json.dumps({"error": str(error), "run_path": str(args.run_path)})
                if args.json
                else f"result: unreadable product invocation: {error}",
                file=sys.stdout if args.json else sys.stderr,
            )
            return 1
        if args.json:
            print(json.dumps(result, indent=2))
        else:
            print(
                f"Product run: {result['invocation']} ({result['run_kind']}); registry={result['registry']}"
            )
            print(
                f"Terminal: {result['terminal_status']}; phase={result['phase']}; workload_exit={result['workload_exit']}; owned_drain={result['owned_drain']}"
            )
            if result["terminal"] is not None:
                print(f"Complete: {result['terminal']['complete']}")
            for item in result["subjects"]:
                print(f"  {item['identity']}: {item['outcome']}")
            print(f"Selected but unreported: {len(result['selected_unreported'])}")
            for error in result["report_errors"]:
                print(f"Report error: {error}")
            for item in result["resources"]:
                print(
                    f"Resource: {item['resource']}; {item.get('retention_reason') or item['status']}"
                )
            for item in result["payloads"]:
                print(f"Artifact: {item['path']} ({item['status']})")
                for line in item["tail"]:
                    print(f"  {line}")
        return 0
    if (
        args.registry is not None
        or args.subject is not None
        or args.offset
        or args.limit != 100
    ):
        parser.error(
            "--registry, --subject, --offset and --limit belong to product-test results"
        )
    try:
        result = read_assessment(
            args.run_path,
            gate=args.gate,
            failures=args.failures,
            tail=args.tail,
            source_detail=args.source_detail,
            full_export=args.full_export,
            historical=args.historical,
        )
    except UnknownGateError as error:
        parser.error(str(error))
    except (OSError, TypeError, ValueError, RuntimeError) as error:
        if args.json:
            print(json.dumps({"error": str(error), "run_path": str(args.run_path)}))
        else:
            print(f"result: unreadable assessment: {error}", file=sys.stderr)
        return 1
    if args.json:
        print(json.dumps(result, indent=2))
    else:
        display(result)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

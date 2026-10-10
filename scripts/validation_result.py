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

from scripts import test_resources
from scripts.validation_receipts import qualified


def read_receipt(run: Path) -> dict:
    """Read the runner's atomic version 5 checkpoint once, checking readable shape."""
    with (run / "checks.json").open() as stream:
        receipt = json.load(stream)
    if not isinstance(receipt, dict) or receipt.get("version") != 5:
        raise ValueError("expected an assessment version 5 record")
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
    recorded = {check["gate"] for check in receipt["checks"]}
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
            if key not in {"checks", "scope"}
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
        "--failures", action="store_true", help="show unqualified recorded checks"
    )
    parser.add_argument(
        "--gate", help="select a declared gate, including an unattempted gate"
    )
    parser.add_argument(
        "--tail", type=int, default=20, help="log tail lines (default 20; 0 disables)"
    )
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    if args.tail < 0:
        parser.error("--tail must be nonnegative")
    try:
        run = args.run_path.resolve()
        with contextlib.ExitStack() as borrows:
            borrows.enter_context(test_resources.borrow_report(run))
            receipt = read_receipt(run)
            if args.gate is not None and args.gate not in {
                declaration["name"] for declaration in receipt["scope"]
            }:
                parser.error(f"gate {args.gate!r} is not in this run's scope")
            origins: dict[str, Path] = {}
            borrowed = {run}
            for check in receipt["checks"]:
                if args.gate is not None and check["gate"] != args.gate:
                    continue
                if args.failures and qualified(check):
                    continue
                origin = Path(check["origin"]) if check.get("origin") else run
                # Resolve each pointer once, before borrowing and payload access.
                origin = (origin if origin.is_absolute() else run / origin).resolve()
                origins[check["gate"]] = origin
                if origin not in borrowed:
                    borrows.enter_context(test_resources.borrow_report(origin))
                    borrowed.add(origin)
            result = selected_result(
                run,
                receipt,
                gate=args.gate,
                failures=args.failures,
                tail=args.tail,
                origins=origins,
            )
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

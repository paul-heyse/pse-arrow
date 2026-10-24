# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded cargo-mutants consumer through existing correctness and native owners."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path
from typing import Never

from scripts import arrow_validation, cargo_tools, native_tests, validation

ROOT = Path(__file__).resolve().parents[1]


def _invalid_protocol(message: str) -> Never:
    """Keep this owner's document/observation refusals in its ValueError contract."""
    raise ValueError(message)


def classification(outcome: str) -> str:
    return {
        "CaughtMutant": "caught",
        "MissedMutant": "missed",
        "Unviable": "unviable",
        "UnviableMutant": "unviable",
        "Timeout": "timeout",
        "Success": "baseline-passed",
        "Failure": "setup-failed",
    }.get(outcome, "unknown")


def split_arguments(arguments: list[str]) -> tuple[list[str], list[str]]:
    """Mutants 27.1 trailing args are test-only; -C reaches Build and Test.

    Both phases use nextest, so preserve nextest's Cargo-profile spelling. Exact
    test packages come from the already validated root graph, not mutated-package
    defaults, which could silently add an unvalidated test root.
    """
    common: list[str] = []
    tests: list[str] = []
    index = 0
    while index < len(arguments):
        argument = arguments[index]
        key = argument.split("=", 1)[0]
        if argument == "--":
            tests.extend(arguments[index:])
            break
        package = key in {"-p", "--package", "--exclude", "--workspace", "--all"} or (
            argument.startswith("-p")
            and not argument.startswith("--")
            and len(argument) > 2
        )
        compile_option = key in arrow_validation.VALUE | arrow_validation.FLAG | {
            "--cargo-profile",
            "--cargo-message-format",
            "--cargo-quiet",
            "--cargo-verbose",
            "--profile",
            "-P",
            "--config-file",
            "--tool-config-file",
        }
        if argument.startswith("-F") and len(argument) > 2:
            compile_option = True
        # Nextest -j/--jobs are runner width, not Cargo compilation width.
        if key in {"-j", "--jobs"}:
            compile_option = False
        destination = common if compile_option else tests
        needs_value = key in arrow_validation.VALUE | arrow_validation.NEXTEST_VALUE | {
            "--cargo-profile",
            "--cargo-message-format",
        }
        if key in {"--workspace", "--all"}:
            needs_value = False
        consumed = [argument]
        if (
            needs_value
            and "=" not in argument
            and not (
                argument.startswith(("-p", "-j", "-F"))
                and not argument.startswith("--")
                and len(argument) > 2
            )
        ):
            index += 1
            if index == len(arguments):
                raise ValueError(f"missing value for {argument}")
            consumed.append(arguments[index])
        if key in {
            "--target-dir",
            "--archive-file",
            "--binaries-metadata",
            "--cargo-metadata",
            "--workspace-remap",
        }:
            raise ValueError(f"{key} cannot bypass the isolated mutation copy")
        if key == "--manifest-path":
            value = argument.split("=", 1)[1] if "=" in argument else consumed[1]
            if Path(value).is_absolute() or ".." in Path(value).parts:
                raise ValueError("mutation manifest must remain inside its owned copy")
        if not package:
            destination.extend(consumed)
        index += 1
    return common, tests


def decode_outcomes(value: object) -> tuple[list[dict], str]:
    if not isinstance(value, dict) or not isinstance(value.get("outcomes"), list):
        _invalid_protocol("unsupported cargo-mutants outcome protocol")
    rows = value["outcomes"]
    if any(
        not isinstance(row, dict)
        or classification(row.get("summary", "")) == "unknown"
        or not isinstance(row.get("phase_results"), list)
        for row in rows
    ):
        raise ValueError("unknown cargo-mutants scenario outcome")
    baseline = [row for row in rows if row.get("scenario") == "Baseline"]
    mutants = [
        row
        for row in rows
        if isinstance(row.get("scenario"), dict)
        and isinstance(row["scenario"].get("Mutant"), dict)
    ]
    if len(baseline) != 1 or len(baseline) + len(mutants) != len(rows):
        raise ValueError("missing or ambiguous unchanged baseline / mutant scenarios")
    if any(
        row["summary"] == "Success"
        or not isinstance(row["scenario"]["Mutant"].get("name"), str)
        for row in mutants
    ):
        raise ValueError("mutant lacks a classified assertion challenge")
    if any(
        not isinstance(phase, dict) for row in rows for phase in row["phase_results"]
    ):
        raise ValueError("unknown cargo-mutants phase protocol")
    status = classification(baseline[0]["summary"])
    if status == "baseline-passed" and not any(
        phase.get("phase") == "Test" and phase.get("process_status") == "Success"
        for phase in baseline[0]["phase_results"]
    ):
        raise ValueError("unchanged baseline lacks a successful actual test phase")
    if status == "baseline-passed" and not mutants:
        raise ValueError("no selected mutants were challenged")
    return [
        {"classification": classification(row["summary"]), "original": row}
        for row in rows
    ], "passed" if status == "baseline-passed" else status


def run(
    path: str,
    output: Path | None,
    arguments: list[str],
    pattern: str | None,
    base: str | None,
    native: bool,
) -> int:
    target = (ROOT / path).resolve()
    target.relative_to(ROOT)
    if (
        not target.is_file()
        or target.suffix != ".rs"
        or any(
            part in {"generated", "tests", "benches"}
            for part in target.relative_to(ROOT).parts
        )
    ):
        raise ValueError(
            "select an explicit existing production Rust file; test-only diffs need their production target"
        )
    output = validation.fresh_output(ROOT, output, "sensitivity")
    tool = cargo_tools.selected_identity("cargo-mutants")
    selection = (
        native_tests.rust_command("run", arguments)
        if native
        else ["cargo", "nextest", "run", "--locked", *arguments]
    )
    resolution = arrow_validation.resolve(selection)
    common, test_arguments = split_arguments(resolution.command[3:])
    command = [
        "cargo",
        "mutants",
        "--test-tool",
        "nextest",
        "--baseline",
        "run",
        "--copy-vcs=true",
        "--file",
        path,
        "--output",
        str(output),
        "--no-shuffle",
        "--timeout",
        "60",
        "--build-timeout",
        "900",
    ]
    for package in resolution.packages:
        command += ["--test-package", package]
    command += ["--cargo-arg=" + argument for argument in common]
    if pattern:
        command += ["--re", pattern]
    if base:
        patch = subprocess.check_output(
            ["git", "diff", "--no-ext-diff", "--no-renames", base, "--", path], cwd=ROOT
        )
        if not patch:
            raise ValueError(
                "selected production target has no changes; omit --base for an explicit whole-file challenge"
            )
        (output / "selected.diff").write_bytes(patch)
        command += ["--in-diff", str(output / "selected.diff")]
    command += ["--", *test_arguments]
    record: dict[str, object] = {
        "schema": "assertion-sensitivity-v1",
        "tool": tool,
        "command": command,
        "target": path,
        "correctness_command": resolution.command,
        "inputs": validation.sources(ROOT),
        "environment": validation.relevant_environment(),
        "native_requested": native,
        "limits": [
            "a survivor is not automatically equivalent",
            "bounded selected predicates only; no repository score",
        ],
    }
    validation.write_json(output / "selection.json", record)
    code = subprocess.run(
        command, cwd=ROOT, env=dict(os.environ), check=False
    ).returncode
    report = output / "mutants.out/outcomes.json"
    record.update(
        exit_code=code,
        raw_outcomes=str(report),
        equivalent="not-inferred",
        unchanged_during_run=record["inputs"] == validation.sources(ROOT),
    )
    try:
        rows, baseline = decode_outcomes(json.loads(report.read_text()))
        record.update(
            outcomes=rows, baseline_status=baseline, protocol_status="observed"
        )
    except (OSError, ValueError, TypeError, KeyError) as error:
        record.update(
            outcomes=[],
            baseline_status="unavailable",
            protocol_status="unavailable",
            protocol_error=str(error),
        )
    validation.write_json(output / "sensitivity.json", record)
    print(output / "sensitivity.json")
    return code or int(
        record["protocol_status"] != "observed"
        or record["baseline_status"] != "passed"
        or not record["unchanged_during_run"]
    )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("path")
    parser.add_argument("--output", type=Path)
    parser.add_argument("--pattern", help="bounded cargo-mutants regex selection")
    parser.add_argument("--base", help="optional production diff selection")
    parser.add_argument(
        "--native",
        action="store_true",
        help="compose selected native test root features; launch through pse-env --native",
    )
    import sys  # noqa: PLC0415 -- selected operation owner; source-only discovery must not prepare this lifecycle

    supplied = list(sys.argv[1:] if argv is None else argv)
    boundary = supplied.index("--") if "--" in supplied else len(supplied)
    args = parser.parse_args(supplied[:boundary])
    arguments = supplied[boundary + 1 :]
    return run(args.path, args.output, arguments, args.pattern, args.base, args.native)


if __name__ == "__main__":
    raise SystemExit(main())

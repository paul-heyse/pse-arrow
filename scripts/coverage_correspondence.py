# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source-fresh changed-region observations over a selected LLVM coverage capture."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import tomllib
import xml.etree.ElementTree as ET
from pathlib import Path
from typing import Never

from defusedxml import ElementTree as DefusedET
from defusedxml.common import DefusedXmlException

from scripts import arrow_validation, cargo_tools, validation, validation_scope

ROOT = Path(__file__).resolve().parents[1]
EXCLUSIONS = r"(generated/|xtask/|benches/|tests/)"


def _invalid_protocol(message: str) -> Never:
    """Keep this owner's document/observation refusals in its ValueError contract."""
    raise ValueError(message)


def context(arguments: list[str], branch: bool) -> dict:
    return {
        "inputs": validation_scope.input_identity(
            "rust-product",
            validation.sources(ROOT, scope="rust-product"),
            validation.relevant_environment(),
        ),
        "compiler": subprocess.check_output(["rustc", "-Vv"], text=True),
        "tool": cargo_tools.selected_identity("cargo-llvm-cov"),
        "arguments": arguments,
        "branch": branch,
        "exclusions": EXCLUSIONS,
        "instrumentation_environment": {
            key: value
            for key, value in os.environ.items()
            if key
            in {
                "LLVM_COV",
                "LLVM_PROFDATA",
                "LLVM_COV_FLAGS",
                "LLVM_PROFDATA_FLAGS",
                "CARGO_LLVM_COV_FLAGS",
                "CARGO_LLVM_PROFDATA_FLAGS",
                "RUSTDOCFLAGS",
            }
            or key.startswith(("CARGO_PROFILE_", "NEXTEST_"))
        },
        "nextest_configuration": configuration_identity(arguments),
    }


def option(arguments: list[str], names: set[str], default: str) -> str:
    selected = default
    for index, argument in enumerate(arguments):
        key, separator, value = argument.partition("=")
        if key in names:
            if not separator:
                if index + 1 == len(arguments):
                    raise ValueError(f"missing value for {argument}")
                value = arguments[index + 1]
            selected = value
    return selected


def configuration_identity(arguments: list[str]) -> dict:
    path = Path(
        option(arguments, {"--config-file"}, str(ROOT / ".config/nextest.toml"))
    )
    if not path.is_absolute():
        path = ROOT / path
    return {"path": str(path), "sha256": hashlib.sha256(path.read_bytes()).hexdigest()}


def junit_path(arguments: list[str]) -> Path:
    """Root nextest config owns report placement, independently of LLVM's target.

    Tool-specific config layering is not reconstructed; unsupported layouts retain
    explicit unavailable population, never an unrelated or old report.
    """
    if any(argument.split("=", 1)[0] == "--tool-config-file" for argument in arguments):
        raise ValueError(
            "tool-specific nextest configuration requires explicit population inspection"
        )
    config = configuration_identity(arguments)
    with Path(config["path"]).open("rb") as stream:
        selected = tomllib.load(stream)
    profile = option(arguments, {"--profile", "-P"}, "ci")
    profiles = selected.get("profile", {})
    default = profiles.get("default", {}).get("junit", {})
    path = profiles.get(profile, {}).get("junit", {}).get("path", default.get("path"))
    if not isinstance(path, str):
        _invalid_protocol("selected nextest profile has no known JUnit path")
    store = selected.get("store", {}).get("dir", "target/nextest")
    return ROOT / store / profile / path


def population(raw: bytes) -> list[dict]:
    try:
        root = DefusedET.fromstring(raw, forbid_dtd=True)
    except (ET.ParseError, DefusedXmlException) as error:
        raise ValueError("malformed nextest JUnit") from error
    if root.tag != "testsuites" or not root.get("uuid"):
        raise ValueError("unsupported nextest JUnit identity")
    return [
        {
            "binary": case.get("classname"),
            "test": case.get("name"),
            "outcome": "failed"
            if case.find("failure") is not None or case.find("error") is not None
            else "not-run"
            if case.find("skipped") is not None
            else "passed",
        }
        for case in root.iter("testcase")
    ]


def junit_image(path: Path) -> tuple[bytes, tuple[int, ...]]:
    with path.open("rb") as stream:
        before = os.fstat(stream.fileno())
        raw = stream.read()
        after = os.fstat(stream.fileno())

    def identity(stat: os.stat_result) -> tuple[int, ...]:
        return (
            stat.st_dev,
            stat.st_ino,
            stat.st_size,
            stat.st_mtime_ns,
            stat.st_ctime_ns,
        )

    if identity(before) != identity(after):
        raise ValueError("nextest JUnit changed while being read")
    return raw, identity(after)


def changed_regions(base: str, path: str) -> list[dict[str, int]]:
    diff = subprocess.check_output(
        [
            "git",
            "diff",
            "--no-ext-diff",
            "--no-renames",
            "--unified=0",
            base,
            "--",
            path,
        ],
        cwd=ROOT,
        text=True,
    )
    regions = [
        {"start": int(start), "lines": int(count or "1")}
        for start, count in re.findall(
            r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", diff, re.MULTILINE
        )
    ]
    if (
        not regions
        and subprocess.run(
            ["git", "ls-files", "--error-unmatch", "--", path],
            cwd=ROOT,
            capture_output=True,
            check=False,
        ).returncode
    ):
        source = ROOT / path
        if source.is_file():
            regions = [{"start": 1, "lines": len(source.read_text().splitlines())}]
    return regions


def line_counts(segments: list[list]) -> dict[int, int]:
    counts: dict[int, int] = {}
    if not isinstance(segments, list) or any(
        not isinstance(segment, list)
        or len(segment) < 6
        or any(type(value) is not int or value < 0 for value in segment[:3])
        or type(segment[3]) is not bool
        for segment in segments
    ):
        raise ValueError("unsupported LLVM segment record")
    for index, segment in enumerate(segments):
        line, _column, count, has_count, _entry, *rest = segment
        if not has_count or (len(rest) and rest[0]):
            continue
        end = segments[index + 1] if index + 1 < len(segments) else [line + 1, 1]
        last = end[0] if end[1] > 1 else end[0] - 1
        for number in range(line, last + 1):
            counts[number] = max(counts.get(number, 0), count)
    return counts


def correspondence(
    report: dict, changes: dict[str, list[dict[str, int]]]
) -> list[dict]:
    # Qualified with cargo-llvm-cov 0.9.1 / pinned nightly's LLVM export 3.1.
    # Unknown protocols must not silently claim changed-path evidence.
    if (
        not isinstance(report, dict)
        or report.get("type") != "llvm.coverage.json.export"
        or report.get("version") != "3.1.0"
    ):
        raise ValueError("unsupported LLVM coverage JSON protocol")
    if not isinstance(report.get("data"), list) or len(report["data"]) != 1:
        raise ValueError("expected one merged LLVM coverage export unit")
    files = {
        str(Path(f["filename"]).resolve()): f
        for unit in report["data"]
        for f in unit["files"]
    }
    rows: list[dict[str, object]] = []
    for path, regions in sorted(changes.items()):
        if not path.endswith(".rs") or re.search(EXCLUSIONS, path):
            continue
        file = files.get(str((ROOT / path).resolve()))
        if file is None:
            rows.append({"path": path, "regions": regions, "status": "not-in-capture"})
            continue
        lines = line_counts(file["segments"])
        selected = {
            line
            for region in regions
            for line in range(region["start"], region["start"] + region["lines"])
        }
        branches = [
            {"line": b[0], "column": b[1], "true_count": b[4], "false_count": b[5]}
            for b in file.get("branches", [])
            if b[0] in selected
        ]
        rows.append(
            {
                "path": path,
                "regions": regions,
                "status": "observed",
                "covered_lines": sorted(
                    line for line in selected if lines.get(line, 0) > 0
                ),
                "missing_lines": sorted(
                    line for line in selected if line in lines and lines[line] == 0
                ),
                "non_executable_or_unknown_lines": sorted(selected - lines.keys()),
                "branches": branches,
                "branch_status": "observed" if branches else "no-branch-detail",
            }
        )
    return rows


def capture(output: Path | None, arguments: list[str], branch: bool) -> int:
    output = validation.fresh_output(ROOT, output, "coverage")
    branch = branch or "--branch" in arguments
    arguments = [arg for arg in arguments if arg != "--branch"] or ["--workspace"]
    selected = context(arguments, branch)
    profile = (
        []
        if any(arg.split("=", 1)[0] in {"--profile", "-P"} for arg in arguments)
        else ["--profile", "ci"]
    )
    command = arrow_validation.compose(
        [
            "cargo",
            "llvm-cov",
            "nextest",
            "--locked",
            *profile,
            "--json",
            "--output-path",
            str(output / "llvm.json"),
            "--ignore-filename-regex",
            EXCLUSIONS,
            *(["--branch"] if branch else []),
            *arguments,
        ]
    )
    env = dict(os.environ, CARGO_LLVM_COV_TARGET_DIR=str(output / "target"))
    record: dict[str, object] = {
        "schema": "coverage-correspondence-v1",
        "context": selected,
        "command": command,
        "individual_test_attribution": "unknown; selected capture population only",
        "native_coverage": "opaque",
    }
    before = None
    junit = None
    try:
        junit = junit_path(arguments)
        before = junit_image(junit) if junit.is_file() else None
    except (OSError, ValueError) as error:
        record["population_error"] = str(error)
    code = subprocess.run(command, cwd=ROOT, env=env, check=False).returncode
    executed = None
    try:
        if junit is not None:
            raw, original_identity = junit_image(junit)
            if before is not None and (
                raw == before[0] or original_identity == before[1]
            ):
                _invalid_protocol("nextest JUnit was not replaced by this capture")
            executed = population(raw)
            (output / "population.xml").write_bytes(raw)
            record["population_sha256"] = hashlib.sha256(raw).hexdigest()
            record["population_source"] = str(junit)
            record["population_file_identity"] = original_identity
    except (OSError, ValueError, ET.ParseError) as error:
        record["population_error"] = str(error)
    report = output / "llvm.json"
    record.update(
        exit_code=code,
        population=executed,
        population_status="observed" if executed is not None else "unavailable",
        unchanged_during_capture=selected == context(arguments, branch),
        llvm_sha256=hashlib.sha256(report.read_bytes()).hexdigest()
        if report.is_file()
        else None,
    )
    validation.write_json(output / "capture.json", record)
    print(output / "capture.json")
    return code


def inspect(directory: Path, base: str) -> dict:
    # selected operation owner; source-only discovery must not prepare this lifecycle
    from scripts import affected  # noqa: PLC0415

    record = json.loads((directory / "capture.json").read_text())
    if (
        not isinstance(record, dict)
        or record.get("schema") != "coverage-correspondence-v1"
    ):
        raise ValueError("unsupported coverage capture")
    current = context(record["context"]["arguments"], record["context"]["branch"])
    fresh = record["unchanged_during_capture"] and record["context"] == current
    raw = (directory / "llvm.json").read_bytes()
    if hashlib.sha256(raw).hexdigest() != record["llvm_sha256"]:
        raise ValueError("coverage report bytes differ from capture")
    if record.get("population_status") == "observed":
        original = (directory / "population.xml").read_bytes()
        if (
            hashlib.sha256(original).hexdigest() != record.get("population_sha256")
            or population(original) != record["population"]
        ):
            raise ValueError("executed population differs from captured nextest report")
    changes = {
        path: changed_regions(base, path) for _, path in affected.changed_files(base)
    }
    return {
        "schema": "changed-source-coverage-v1",
        "capture": str(directory),
        "fresh": fresh,
        "status": "observed"
        if fresh and record["exit_code"] == 0
        else "stale-or-unsuccessful",
        "population": record["population"],
        "population_status": record["population_status"],
        "exclusions": EXCLUSIONS,
        "regions": correspondence(json.loads(raw), changes) if fresh else [],
        "selection": "conservative dependency reach remains authoritative; coverage never omits tests",
        "limits": [
            "coverage does not establish assertion adequacy",
            "merged capture has no individual-test attribution",
            "opaque native code is unobserved",
        ],
    }


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="action", required=True)
    run = commands.add_parser("capture")
    run.add_argument("--output", type=Path)
    run.add_argument("--branch", action="store_true")
    run.add_argument("arguments", nargs=argparse.REMAINDER)
    view = commands.add_parser("report")
    view.add_argument("--capture", type=Path, required=True)
    view.add_argument("--base", default="HEAD")
    view.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    if args.action == "capture":
        arguments = (
            args.arguments[1:] if args.arguments[:1] == ["--"] else args.arguments
        )
        return capture(args.output, arguments, args.branch)
    result = inspect(args.capture, args.base)
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

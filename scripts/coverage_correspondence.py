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
import xml.etree.ElementTree as ET
from pathlib import Path
from scripts import arrow_validation, cargo_tools, validation, validation_scope

ROOT = Path(__file__).resolve().parents[1]
EXCLUSIONS = r"(generated/|xtask/|benches/|tests/)"


def context(arguments: list[str], branch: bool) -> dict:
    return {"inputs": validation_scope.input_identity("rust-product", validation.sources(ROOT), validation.relevant_environment()),
            "compiler": subprocess.check_output(["rustc", "-Vv"], text=True),
            "tool": cargo_tools.selected_identity("cargo-llvm-cov"),
            "arguments": arguments, "branch": branch, "exclusions": EXCLUSIONS}


def changed_regions(base: str, path: str) -> list[dict[str, int]]:
    diff = subprocess.check_output(["git", "diff", "--no-ext-diff", "--no-renames", "--unified=0", base, "--", path], cwd=ROOT, text=True)
    regions = [{"start": int(start), "lines": int(count or "1")} for start, count in
               re.findall(r"^@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", diff, re.MULTILINE)]
    if not regions and subprocess.run(["git", "ls-files", "--error-unmatch", "--", path], cwd=ROOT, capture_output=True).returncode:
        source = ROOT / path
        if source.is_file():
            regions = [{"start": 1, "lines": len(source.read_text().splitlines())}]
    return regions


def line_counts(segments: list[list]) -> dict[int, int]:
    counts: dict[int, int] = {}
    for index, segment in enumerate(segments):
        line, column, count, has_count, _entry, *rest = segment
        if not has_count or (len(rest) and rest[0]):
            continue
        end = segments[index+1] if index+1 < len(segments) else [line+1, 1]
        last = end[0] if end[1] > 1 else end[0]-1
        for number in range(line, last+1):
            counts[number] = max(counts.get(number, 0), count)
    return counts


def correspondence(report: dict, changes: dict[str, list[dict[str, int]]]) -> list[dict]:
    files = {str(Path(f["filename"]).resolve()): f for unit in report["data"] for f in unit["files"]}
    rows = []
    for path, regions in sorted(changes.items()):
        if not path.endswith(".rs") or re.search(EXCLUSIONS, path):
            continue
        file = files.get(str((ROOT/path).resolve()))
        if file is None:
            rows.append({"path": path, "regions": regions, "status": "not-in-capture"})
            continue
        lines = line_counts(file["segments"])
        selected = {line for region in regions for line in range(region["start"], region["start"]+region["lines"])}
        branches = [{"line": b[0], "column": b[1], "true_count": b[4], "false_count": b[5]}
                    for b in file.get("branches", []) if b[0] in selected]
        rows.append({"path": path, "regions": regions, "status": "observed",
                     "covered_lines": sorted(line for line in selected if lines.get(line, 0)>0),
                     "missing_lines": sorted(line for line in selected if line in lines and lines[line]==0),
                     "non_executable_or_unknown_lines": sorted(selected-lines.keys()),
                     "branches": branches, "branch_status": "observed" if branches else "no-branch-detail"})
    return rows


def capture(output: Path, arguments: list[str], branch: bool) -> int:
    output = validation.fresh_output(ROOT, output, "coverage")
    branch = branch or "--branch" in arguments
    arguments = [arg for arg in arguments if arg != "--branch"] or ["--workspace"]
    selected = context(arguments, branch)
    command = arrow_validation.compose(["cargo", "llvm-cov", "nextest", "--locked", "--profile", "ci",
        "--json", "--output-path", str(output/"llvm.json"), "--ignore-filename-regex", EXCLUSIONS,
        *(["--branch"] if branch else []), *arguments])
    env = dict(os.environ, CARGO_LLVM_COV_TARGET_DIR=str(output/"target"))
    record = {"schema": "coverage-correspondence-v1", "context": selected, "command": command,
              "individual_test_attribution": "unknown; selected capture population only", "native_coverage": "opaque"}
    code = subprocess.run(command, cwd=ROOT, env=env, check=False).returncode
    junit = output/"target/nextest/ci/junit.xml"
    population = None
    if junit.is_file():
        population = [{"binary": case.get("classname"), "test": case.get("name"),
                       "outcome": "failed" if case.find("failure") is not None or case.find("error") is not None
                       else "not-run" if case.find("skipped") is not None else "passed"}
                      for case in ET.parse(junit).iter("testcase")]
    report = output/"llvm.json"
    record.update(exit_code=code, population=population, population_status="observed" if population is not None else "unavailable",
                  unchanged_during_capture=selected == context(arguments, branch),
                  llvm_sha256=hashlib.sha256(report.read_bytes()).hexdigest() if report.is_file() else None)
    validation.write_json(output/"capture.json", record)
    print(output/"capture.json")
    return code


def inspect(directory: Path, base: str) -> dict:
    from scripts import affected
    record = json.loads((directory/"capture.json").read_text())
    if record.get("schema") != "coverage-correspondence-v1":
        raise ValueError("unsupported coverage capture")
    current = context(record["context"]["arguments"], record["context"]["branch"])
    fresh = record["unchanged_during_capture"] and record["context"] == current
    raw = (directory/"llvm.json").read_bytes()
    if hashlib.sha256(raw).hexdigest() != record["llvm_sha256"]:
        raise ValueError("coverage report bytes differ from capture")
    changes = {path: changed_regions(base, path) for _, path in affected.changed_files(base)}
    return {"schema": "changed-source-coverage-v1", "capture": str(directory), "fresh": fresh,
            "status": "observed" if fresh and record["exit_code"] == 0 else "stale-or-unsuccessful",
            "population": record["population"], "exclusions": EXCLUSIONS,
            "regions": correspondence(json.loads(raw), changes) if fresh else [],
            "selection": "conservative dependency reach remains authoritative; coverage never omits tests",
            "limits": ["coverage does not establish assertion adequacy", "merged capture has no individual-test attribution", "opaque native code is unobserved"]}


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    commands = parser.add_subparsers(dest="action", required=True)
    run = commands.add_parser("capture")
    run.add_argument("--output", type=Path, required=True)
    run.add_argument("--branch", action="store_true")
    run.add_argument("arguments", nargs=argparse.REMAINDER)
    view = commands.add_parser("report")
    view.add_argument("--capture", type=Path, required=True)
    view.add_argument("--base", default="HEAD")
    view.add_argument("--json", action="store_true")
    args = parser.parse_args(argv)
    if args.action == "capture":
        arguments = args.arguments[1:] if args.arguments[:1] == ["--"] else args.arguments
        return capture(args.output, arguments, args.branch)
    result = inspect(args.capture, args.base)
    print(json.dumps(result, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

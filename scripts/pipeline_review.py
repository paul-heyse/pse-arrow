# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Execute explicit catalog cases with prebuilt binaries and publish retained observations."""

from __future__ import annotations

import argparse
import csv
import html
import json
import math
import os
from pathlib import Path
from typing import TYPE_CHECKING
from urllib.parse import quote

from scripts import case_audit, case_measure

if TYPE_CHECKING:
    from collections.abc import Iterator

ROOT = Path(__file__).resolve().parents[1]


def catalog(selection: list[str]) -> list[tuple[str, dict]]:
    """Return existing declarations, in the caller's explicit order."""
    if not selection or len(set(selection)) != len(selection):
        raise ValueError("select distinct explicit catalog case IDs")
    declarations = [
        (target, json.loads((ROOT / ".config" / filename).read_text()))
        for target, filename in (
            ("native_process", "process-cases.json"),
            ("modeling_preparation", "preparation-cases.json"),
        )
    ]
    case_measure.selected_workloads(
        tuple(document for _, document in declarations), selection
    )
    owners = {
        workload["id"]: (target, workload)
        for target, document in declarations
        for workload in document["workloads"]
    }
    return [owners[identity] for identity in selection]


def binaries_from(source: Path | None, overrides: list[str]) -> dict[str, Path]:
    """Read a supplied target/path map; never invoke Cargo or infer a build path."""
    supplied = {} if source is None else json.loads(source.read_text())
    if isinstance(supplied, dict) and "binaries" in supplied:
        supplied = supplied["binaries"]
    if not isinstance(supplied, dict):
        raise TypeError("prebuilt binaries must be an object keyed by benchmark target")
    for override in overrides:
        name, separator, value = override.partition("=")
        if not separator or not name or not value:
            raise ValueError("--binary requires target=path")
        supplied[name] = value
    result = {}
    for name, value in supplied.items():
        if not isinstance(name, str) or not isinstance(value, str):
            raise TypeError("binary target names and paths must be strings")
        path = Path(value).resolve()
        if not path.is_file() or not os.access(path, os.X_OK):
            raise ValueError(f"prebuilt binary is not executable: {path}")
        result[name] = path
    return result


def _write_json(path: Path, value: object) -> None:
    path.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")


def execute_case(
    binary: str | Path,
    workload: dict,
    directory: Path,
    *,
    recording_mode: str = "selected",
    review_artifacts: bool = False,
    capture_from_start: bool = False,
    deadline_seconds: float = 60.0,
    label: str = "",
    env: dict[str, str] | None = None,
) -> dict:
    """Execute one declared case through the existing scientific command/audit owners."""
    if workload.get("audit_supported") is not True:
        raise ValueError(
            f"case does not declare audit stop/cleanup support: {workload['id']}"
        )
    if not math.isfinite(deadline_seconds) or deadline_seconds <= 0:
        raise ValueError("deadline must be finite and positive")
    if recording_mode not in {"ordinary", "selected"} or (
        review_artifacts and recording_mode == "ordinary"
    ):
        raise ValueError("review artifacts require selected recording mode")
    binary = Path(binary).resolve()
    directory = directory.resolve()
    directory.mkdir(parents=True)
    identity = workload["id"]
    environment = {
        **(os.environ if env is None else env),
        "PSE_PROCESS_COST_CASE": identity,
        "PSE_PROCESS_COST_SPEC": json.dumps(workload),
        "PSE_PROCESS_COST_OUTPUT": str(directory),
        "PSE_PREPARATION_CASE": identity,
        "PSE_PREPARATION_OUTPUT": str(directory),
    }
    record = {
        "schema": "pipeline-review-case-v1",
        "case": identity,
        "directory": str(directory),
        "binary": str(binary),
        "label": label,
        "recording_mode": recording_mode,
        "review_artifacts": review_artifacts,
        "capture_from_start": capture_from_start,
        "deadline_seconds": deadline_seconds,
        "workload": workload,
        "conditions": {
            name: environment.get(name)
            for name in (
                "OMP_NUM_THREADS",
                "OPENBLAS_NUM_THREADS",
                "MKL_NUM_THREADS",
                "OMP_PROC_BIND",
                "OMP_PLACES",
                "MKL_CBWR",
                "MKL_DYNAMIC",
                "PSE_DIAGNOSTICS",
                "PSE_SURREAL_STATE",
            )
        },
    }
    try:
        environment.update(case_measure.managed_fixture_environment(workload))
        command = case_measure.process_command(str(binary), "--test", workload)
        record["command"] = command
        record["result"] = case_audit.run_case(
            command,
            cwd=ROOT,
            env=environment,
            directory=directory,
            deadline_seconds=deadline_seconds,
            review_artifacts=review_artifacts,
            capture_from_start=capture_from_start,
            recording_mode=recording_mode,
        )
    except KeyboardInterrupt:
        record["operator_interrupted"] = True
        observation = directory / "diagnostics/observation.json"
        if observation.is_file():
            record["result"] = json.loads(observation.read_text())
        _write_json(directory / "case-run.json", record)
        raise
    except Exception as error:
        record["execution_error"] = f"{type(error).__name__}: {error}"
        observation = directory / "diagnostics/observation.json"
        if observation.is_file():
            record["result"] = json.loads(observation.read_text())
    _write_json(directory / "case-run.json", record)
    return record


def _read_json(path: Path) -> object:
    with path.open("rb") as source:
        data = source.read((64 << 20) + 1)
    if len(data) > 64 << 20:
        raise ValueError("publication input exceeds 64 MiB; original retained")
    return json.loads(data)


def _status(record: dict) -> str:
    result = record.get("result", {})
    if (
        record.get("execution_error")
        or result.get("fixture_cleanup_failures", 0)
        or result.get("managed_receiver_drain_failures", 0)
    ):
        return "failed"
    return result.get("status", "unavailable")


def _scalars(
    value: object,
    pointer: str = "",
) -> Iterator[tuple[str, str | bool | int | float | None]]:
    """Preserve source values and missingness; never sum inclusive observations."""
    if isinstance(value, dict):
        for name, child in value.items():
            key = str(name).replace("~", "~0").replace("/", "~1")
            yield from _scalars(child, pointer + "/" + key)
    elif isinstance(value, list):
        for index, child in enumerate(value):
            yield from _scalars(child, pointer + f"/{index}")
    elif value is None or isinstance(value, (str, bool, int, float)):
        yield pointer, value


def _case_directories(inputs: list[Path]) -> list[Path]:
    directories = []
    for source in inputs:
        supplied = source.resolve()
        if supplied.is_file():
            supplied = supplied.parent
        if (supplied / "case-run.json").is_file() or (
            supplied / "diagnostics/observation.json"
        ).is_file():
            directories.append(supplied)
        elif supplied.is_dir():
            directories.extend(
                child
                for child in sorted(supplied.iterdir())
                if child.is_dir()
                and (
                    (child / "case-run.json").is_file()
                    or (child / "diagnostics/observation.json").is_file()
                )
            )
        else:
            raise ValueError(f"case input is unavailable: {supplied}")
    return list(dict.fromkeys(directories))


def _read_json_lines(path: Path) -> Iterator[tuple[int, object]]:
    with path.open("rb") as source:
        line = 0
        while encoded := source.readline((64 << 20) + 1):
            line += 1
            if len(encoded) > 64 << 20 or not encoded.endswith(b"\n"):
                raise ValueError(
                    f"line {line} exceeds allowance or has an incomplete tail; original retained"
                )
            if encoded.strip():
                yield line, json.loads(encoded)


def _read_case_record(directory: Path) -> tuple[dict, str, dict]:
    receipt = directory / "case-run.json"
    record = (
        _read_json(receipt)
        if receipt.is_file()
        else {
            "case": directory.name,
            "directory": str(directory),
            "result": _read_json(directory / "diagnostics/observation.json"),
        }
    )
    if not isinstance(record, dict):
        raise TypeError("case receipt must be an object")
    identity = record["case"]
    result = record.get("result", {})
    if not isinstance(identity, str):
        raise TypeError("case identity must be a string")
    if not isinstance(result, dict):
        raise TypeError("case result must be an object")
    return record, identity, result


def publish(
    output: Path,
    *,
    runs: list[Path] | None = None,
    supplements: list[Path] | None = None,
    label: str = "",
) -> dict:
    """Publish references and exact source scalar values, preserving original samples."""
    output = output.resolve()
    output.mkdir(parents=True, exist_ok=True)
    rows = []
    index: dict = {
        "schema": "pipeline-review-v1",
        "label": label,
        "cases": [],
        "supplements": [],
        "errors": [],
        "scope": "Once-only diagnostic executions; no statistical performance qualification. Inclusive owners are not summed. Missing facts remain unavailable.",
    }

    def relative(path: Path) -> str:
        return os.path.relpath(path, output)

    def source_values(identity: str, path: Path) -> None:
        def append(document: object, line: int | None = None) -> None:
            for pointer, value in _scalars(document):
                rows.append(
                    {
                        "case": identity,
                        "source": relative(path),
                        "record_line": line,
                        "pointer": pointer,
                        "value_json": json.dumps(value, allow_nan=False),
                    }
                )

        try:
            if path.suffix == ".jsonl":
                for line, document in _read_json_lines(path):
                    append(document, line)
            else:
                append(_read_json(path))
        except (OSError, ValueError) as error:
            index["errors"].append(f"{relative(path)}: {error}")

    for directory in _case_directories([output] if runs is None else runs):
        try:
            record, identity, result = _read_case_record(directory)
            artifacts = [
                relative(path)
                for path in sorted(directory.rglob("*"))
                if path.is_file()
            ]
            index["cases"].append(
                {
                    **record,
                    "directory": relative(directory),
                    "status": _status(record),
                    "artifacts": artifacts,
                    "coverage": {
                        "diagnostic_failures": result.get("diagnostic_failures", []),
                        "diagnostic_errors": result.get("diagnostic_errors", []),
                        "review": result.get("review_artifacts"),
                        "inspection_windows": result.get("inspection_windows"),
                        "cleanup_scope": result.get("cleanup_scope", "unavailable"),
                    },
                }
            )
            sources = [
                path
                for path in sorted(directory.glob("*.json"))
                if path.name != "case-run.json"
            ]
            sources.extend(sorted(directory.glob("*-observations.jsonl")))
            sources.extend(
                directory / name
                for name in (
                    "diagnostics/audit-outcome.json",
                    "diagnostics/detail.json",
                )
                if (directory / name).is_file()
            )
            for source in sources:
                source_values(identity, source)
            for source in sorted(
                (directory / "diagnostics/math-originals").glob("*.json")
            ):
                source_values(identity, source)
        except (OSError, ValueError, KeyError, TypeError) as error:
            index["errors"].append(f"{relative(directory)}: {error}")

    for supplement_source in supplements or []:
        supplied = supplement_source.resolve()
        if not supplied.exists():
            index["errors"].append(f"supplement unavailable: {supplied}")
            continue
        artifacts = (
            sorted(path for path in supplied.rglob("*") if path.is_file())
            if supplied.is_dir()
            else [supplied]
        )
        index["supplements"].append(
            {
                "path": relative(supplied),
                "artifacts": [relative(path) for path in artifacts],
            }
        )
        for source in artifacts:
            if source.suffix in {".json", ".jsonl"}:
                source_values(f"supplement:{supplied.name}", source)

    with (output / "observations.csv").open("w", newline="") as destination:
        writer = csv.DictWriter(
            destination,
            fieldnames=("case", "source", "record_line", "pointer", "value_json"),
        )
        writer.writeheader()
        writer.writerows(rows)
    index["scalar_rows"] = len(rows)
    _write_json(output / "index.json", index)
    markdown = [
        f"# {label or 'Numerical pipeline review'}",
        "",
        index["scope"],
        "",
        "| Case | Original outcome | Cleanup failures | Raw artifacts |",
        "| --- | --- | --- | --- |",
    ]
    for case in index["cases"]:
        result = case.get("result", {})
        raw = " ".join(
            f"[{Path(path).name}]({quote(path, safe='/')})"
            for path in case["artifacts"]
        )
        markdown.append(
            f"| {case['case']} | {case['status']} | {result.get('fixture_cleanup_failures', 'unavailable')} | {raw} |"
        )
    markdown.extend(
        [
            "",
            "[Exact observed scalar values](observations.csv)",
            "",
            "## Supplements",
            "",
        ]
    )
    for supplement in index["supplements"]:
        markdown.extend(
            f"- [{path}]({quote(path, safe='/')})" for path in supplement["artifacts"]
        )
    markdown.extend(
        ["", "## Publication gaps", "", *[f"- {error}" for error in index["errors"]]]
    )
    (output / "index.md").write_text("\n".join(markdown) + "\n")
    body = [
        f"<h1>{html.escape(label or 'Numerical pipeline review')}</h1>",
        f"<p>{html.escape(index['scope'])}</p>",
        '<p><a href="observations.csv">Exact observed scalar values</a> · <a href="index.json">Full outcomes and coverage</a></p>',
    ]
    for case in index["cases"]:
        body.append(
            f"<h2>{html.escape(case['case'])}: {html.escape(case['status'])}</h2>"
        )
        body.append(
            "<details><summary>Original outcome and coverage</summary><pre>"
            + html.escape(
                json.dumps(
                    {"result": case.get("result"), "coverage": case["coverage"]},
                    indent=2,
                )
            )
            + "</pre></details><ul>"
        )
        body.extend(
            f'<li><a href="{quote(path, safe="/")}">{html.escape(path)}</a></li>'
            for path in case["artifacts"]
        )
        body.append("</ul>")
    body.append("<h2>Supplements</h2><ul>")
    body.extend(
        f'<li><a href="{quote(path, safe="/")}">{html.escape(path)}</a></li>'
        for supplement in index["supplements"]
        for path in supplement["artifacts"]
    )
    body.append(
        "</ul><h2>Publication gaps</h2><pre>"
        + html.escape("\n".join(index["errors"]))
        + "</pre>"
    )
    (output / "index.html").write_text(
        '<!doctype html><html><head><meta charset="utf-8"><title>Numerical pipeline review</title><style>body{font:16px system-ui;max-width:1000px;margin:3rem auto;padding:0 1rem}pre{white-space:pre-wrap;overflow-wrap:anywhere}li{overflow-wrap:anywhere}</style></head><body>'
        + "\n".join(body)
        + "</body></html>\n"
    )
    return index


def _execution_inputs(
    args: argparse.Namespace,
) -> tuple[list[tuple[str, dict]], dict[str, Path]]:
    if not math.isfinite(args.deadline) or args.deadline <= 0:
        raise ValueError("deadline must be finite and positive")
    if args.review_artifacts and args.recording_mode == "ordinary":
        raise ValueError("review artifacts require selected recording mode")
    selected = catalog(args.case)
    binaries = binaries_from(args.binaries, args.binary)
    for target, workload in selected:
        if target not in binaries:
            raise ValueError(f"missing prebuilt binary: {target}")
        if workload.get("audit_supported") is not True:
            raise ValueError(f"case lacks audit stop/cleanup support: {workload['id']}")
    return selected, binaries


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--binary", action="append", default=[], metavar="TARGET=PATH")
    parser.add_argument("--binaries", type=Path)
    parser.add_argument("--case", action="append", default=[])
    parser.add_argument(
        "--recording-mode", choices=("ordinary", "selected"), default="selected"
    )
    parser.add_argument("--review-artifacts", action="store_true")
    parser.add_argument("--capture-from-start", action="store_true")
    parser.add_argument(
        "--deadline", "--deadline-seconds", dest="deadline", type=float, default=60
    )
    parser.add_argument("--label", default="")
    parser.add_argument("--publish-only", action="store_true")
    parser.add_argument("--input", action="append", default=[], type=Path)
    parser.add_argument("--supplement", action="append", default=[], type=Path)
    args = parser.parse_args(argv)
    if args.publish_only:
        if args.case or args.binary or args.binaries:
            parser.error(
                "publish-only accepts existing --input and --supplement artifacts"
            )
        result = publish(
            args.output,
            runs=args.input or None,
            supplements=args.supplement,
            label=args.label,
        )
    else:
        try:
            selected, binaries = _execution_inputs(args)
            args.output.mkdir(parents=True)
        except (OSError, ValueError, TypeError) as error:
            parser.error(str(error))
        try:
            for target, workload in selected:
                execute_case(
                    binaries[target],
                    workload,
                    args.output / workload["id"],
                    recording_mode=args.recording_mode,
                    review_artifacts=args.review_artifacts,
                    capture_from_start=args.capture_from_start,
                    deadline_seconds=args.deadline,
                    label=args.label,
                )
        finally:
            result = publish(args.output, supplements=args.supplement, label=args.label)
    return int(
        bool(result["errors"])
        or any(case["status"] != "completed" for case in result["cases"])
    )


if __name__ == "__main__":
    raise SystemExit(main())

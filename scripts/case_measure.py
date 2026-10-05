# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Criterion CSV collector: untimed build, fresh case process, separate pool and RSS."""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import math
import os
import statistics
import subprocess
import time
from pathlib import Path

from scripts import validation, validation_receipts
from scripts.native_tests import native_provenance
from scripts.validation_scope import (
    FUNCTIONAL_SCOPES,
    RUST_INPUTS,
    input_identity,
    native_gate,
)

ROOT = Path(__file__).resolve().parents[1]


def measurement_inputs(snapshot: dict) -> dict:
    paths = (
        *RUST_INPUTS,
        "benches",
        ".config/process-cases.json",
        ".config/preparation-cases.json",
        "scripts/case_measure.py",
    )
    return {
        name: value
        for name, value in snapshot.items()
        if any(
            name == prefix
            or name.startswith(prefix + "/")
            or (prefix.startswith("scripts/") and name.startswith(prefix))
            for prefix in paths
        )
    }


def snapshot_digest(snapshot: dict) -> str:
    return hashlib.sha256(json.dumps(snapshot, sort_keys=True).encode()).hexdigest()


def build_benchmark(
    command: list[str], output: Path, phase: str
) -> subprocess.CompletedProcess[str]:
    """Retain Cargo JSON and surface rendered diagnostics before failing the collector."""
    build = subprocess.run(
        command,
        cwd=ROOT,
        check=False,
        text=True,
        capture_output=True,
    )
    stdout = build.stdout or ""
    stderr = build.stderr or ""
    json_log = output / f"{phase}-cargo.jsonl"
    stderr_log = output / f"{phase}-cargo.stderr.log"
    json_log.write_text(stdout)
    stderr_log.write_text(stderr)
    if build.returncode:
        diagnostics = []
        for line in stdout.splitlines():
            try:
                row = json.loads(line)
            except json.JSONDecodeError:
                diagnostics.append(line)
                continue
            if isinstance(row, dict) and row.get("reason") == "compiler-message":
                rendered = row.get("message", {}).get("rendered")
                if rendered:
                    diagnostics.append(rendered)
        error = subprocess.CalledProcessError(
            build.returncode, command, output=stdout, stderr=stderr
        )
        raise RuntimeError(
            f"Cargo benchmark build failed with exit {build.returncode}; "
            f"full output: {json_log}, {stderr_log}\n"
            + "\n".join(diagnostics)
            + "\n"
            + stderr
        ) from error
    return build


def samples(path: Path) -> dict:
    """Read Criterion's public CSV format; retain the original Criterion report."""
    values = []
    total_time = total_iterations = 0.0
    with path.open(newline="") as source:
        for row in csv.DictReader(source):
            elapsed, iterations = (
                float(row["sample_measured_value"]),
                float(row["iteration_count"]),
            )
            if row["unit"] != "ns" or not all(
                math.isfinite(v) and v > 0 for v in (elapsed, iterations)
            ):
                raise ValueError(
                    "Criterion samples require finite positive nanoseconds and iteration counts"
                )
            values.append(elapsed / iterations)
            total_time += elapsed
            total_iterations += iterations
    if len(values) < 2:
        raise ValueError("at least two Criterion samples are required")
    return {
        "mean_nanoseconds": total_time / total_iterations,
        "sample_count": len(values),
        "sample_mean_nanoseconds": statistics.mean(values),
        "sample_standard_error_nanoseconds": statistics.stdev(values)
        / math.sqrt(len(values)),
        "sample_min_nanoseconds": min(values),
        "sample_max_nanoseconds": max(values),
        "uncertainty": "sample standard error, not a bootstrap confidence interval",
    }


def require_functional(
    root: Path,
    path: Path,
    workloads: list[dict],
    *,
    snapshot: dict | None = None,
    profile: str = "dev",
) -> dict:
    """Consume only exact declared selections or their explicit linked covering gate."""
    report = path / "checks.json" if path.is_dir() else path
    evidence = json.loads(report.read_text())
    if (
        evidence.get("version") != 5
        or evidence.get("baseline_failures") != 0
        or not evidence.get("input_coverage")
    ):
        raise ValueError(
            "measurement requires a current zero-baseline functional report"
        )
    if snapshot is None:
        snapshot = validation.sources(root)
    required = {
        name for workload in workloads for name in workload.get("functional_scopes", [])
    }
    if not required or any(
        not workload.get("functional_scopes") for workload in workloads
    ):
        raise ValueError("measurement workload lacks functional prerequisite scopes")
    declarations = {item["name"]: item for item in evidence["scope"]}
    observations = {check["gate"]: check for check in evidence["checks"]}
    consumed = {}
    for name in sorted(required):
        if name not in FUNCTIONAL_SCOPES:
            raise ValueError("unknown functional prerequisite scope")
        exact = FUNCTIONAL_SCOPES[name]
        covering = native_gate()
        gate = next(
            (
                candidate
                for candidate in (exact, covering)
                if declarations.get(candidate.name)
                == json.loads(json.dumps(validation.asdict(candidate)))
            ),
            None,
        )
        if gate is None:
            raise ValueError(
                f"missing exact functional invocation or explicit native covering identity: {name}"
            )
        check = observations.get(gate.name, {})
        if (
            not validation_receipts.qualified(
                {"status": check.get("status", "missing")}
            )
            or check.get("exit_code") != 0
            or check.get("report_errors")
            or not check.get("selected")
            or not check.get("results")
            or any(result["status"] != "passed" for result in check["results"])
            or check.get("changed_source")
            or check.get("invocation") != declarations[gate.name]
            or check.get("mode") != gate.mode
            or check.get("profile") != gate.profile
        ):
            raise ValueError(f"incomplete functional prerequisite: {name}")
        terminal = {**check, "results": list(check["results"]), "report_errors": []}
        validation.compose_selection(terminal)
        if terminal["report_errors"]:
            raise ValueError("incomplete or contradictory selected terminal evidence")
        inputs = input_identity(gate.input_scope, snapshot, evidence["environment"])
        if check.get("inputs") != inputs:
            raise ValueError(
                "functional inputs changed; explicitly reassess affected observations"
            )
        # Read current relevant environment through the same identity operation.
        native = check.get("native", {})
        current_environment = validation.relevant_environment()
        actual_environment = native.get("environment")
        if (
            actual_environment is None
            or input_identity(gate.input_scope, snapshot, current_environment)[
                "environment"
            ]
            != input_identity(gate.input_scope, snapshot, actual_environment)[
                "environment"
            ]
        ):
            raise ValueError("functional relevant environment changed")
        validation_receipts.verify_native(native)
        if native.get("profile", {}).get("cargo_profile") != profile:
            raise ValueError("functional Cargo profile differs from measurement")
        expected_features = {
            "pse-runtime/native-solvers",
            "pse-tests-conformance/native-acceptance",
            "pse-relations/force-validate",
        }
        if set(native.get("profile", {}).get("features", [])) != expected_features:
            raise ValueError("functional native feature graph differs")
        origin = Path(check.get("origin", report.parent))
        for artifact, expected in check.get("artifacts", {}).items():
            target = (origin / artifact).resolve()
            target.relative_to(origin.resolve())
            if validation_receipts.digest(target) != expected:
                raise ValueError("changed functional artifact")
        consumed[name] = {
            "gate": gate.name,
            "native": native,
            "evidence_kind": check["evidence_kind"],
            "origin": str(origin),
        }
    return {
        "path": str(report.resolve()),
        "digest": validation_receipts.digest(report),
        "prerequisites": consumed,
    }


def compatible_native(functional: dict, native: dict) -> None:
    """Test and benchmark executables differ; their toolchain/providers must agree."""
    for claim in functional["prerequisites"].values():
        prior = claim["native"]
        if prior["toolchain"] != native["toolchain"]:
            raise ValueError("measurement toolchain differs from functional execution")
        prior_libraries = {
            name: value
            for name, value in prior["files"].items()
            if name not in prior["links"]
        }
        libraries = {
            name: value
            for name, value in native["files"].items()
            if name not in native["links"]
        }
        if not libraries or any(
            prior_libraries.get(name) != value for name, value in libraries.items()
        ):
            raise ValueError(
                "measurement native providers differ from functional execution"
            )


def selected_workloads(
    declarations: tuple[dict, ...], selection: list[str]
) -> list[dict]:
    """Select declared identities exactly; no selector means the complete campaign."""
    workloads = [w for declaration in declarations for w in declaration["workloads"]]
    identities = [w["id"] for w in workloads]
    if len(set(identities)) != len(identities):
        raise ValueError("measurement workload identities must be unique")
    unknown = sorted(set(selection) - set(identities))
    if unknown:
        raise ValueError(f"unknown measurement cases: {', '.join(unknown)}")
    return [w for w in workloads if not selection or w["id"] in selection]


def smoke_campaign(
    output: Path, profile: str, process: dict, preparation: dict, selected: set[str]
) -> None:
    """Execute benchmark controls once; this produces no performance qualification."""
    targets = {
        target
        for declaration, target in (
            (process, "native_process"),
            (preparation, "modeling_preparation"),
            ({"workloads": [{"id": "document-admission"}]}, "document_admission"),
        )
        if any(workload["id"] in selected for workload in declaration["workloads"])
    }
    command = [
        "cargo",
        "bench",
        "-p",
        "pse-benches",
        "--locked",
        "--profile",
        profile,
        "--features",
        "native-process,pse-relations/force-validate",
        "--no-run",
        "--message-format=json",
    ]
    for target in sorted(targets):
        command.extend(("--bench", target))
    build = build_benchmark(command, output, "smoke")
    binaries = {
        row["target"]["name"]: row["executable"]
        for line in build.stdout.splitlines()
        if line.startswith("{")
        for row in [json.loads(line)]
        if row.get("reason") == "compiler-artifact"
        and row.get("executable")
        and row.get("target", {}).get("name") in targets
    }
    if set(binaries) != targets:
        raise ValueError("all selected benchmark control executables must be built")
    for declaration, target in (
        (process, "native_process"),
        (preparation, "modeling_preparation"),
        ({"workloads": [{"id": "document-admission"}]}, "document_admission"),
    ):
        for workload in declaration["workloads"]:
            name = workload["id"]
            if name not in selected:
                continue
            directory = output / name
            directory.mkdir()
            env = {
                **os.environ,
                "PSE_PROCESS_COST_CASE": name,
                "PSE_PROCESS_COST_SPEC": json.dumps(workload),
                "PSE_PROCESS_COST_OUTPUT": str(directory),
                "PSE_PREPARATION_CASE": name,
                "PSE_PREPARATION_OUTPUT": str(directory),
                "PSE_ADMISSION_OUTPUT": str(directory),
            }
            with (directory / "process.log").open("w") as log:
                subprocess.run(
                    [binaries[target], "--test"],
                    cwd=ROOT,
                    env=env,
                    check=True,
                    stdout=log,
                    stderr=subprocess.STDOUT,
                )
    validation.write_json(
        output / "case-smoke.json",
        {
            "schema": "case-smoke-v1",
            "measured": False,
            "selected_cases": sorted(selected),
            "native": native_provenance(
                {
                    "cargo_profile": profile,
                    "features": ["native-process", "pse-relations/force-validate"],
                },
                list(binaries.values()),
            ),
        },
    )


def preparation_campaign(
    output: Path, profile: str, workloads: list[dict], admission_selected: bool
) -> dict:
    """Build untimed, then isolate the selected preparation/admission workloads."""
    command = [
        "cargo",
        "bench",
        "-p",
        "pse-benches",
        "--bench",
        "modeling_preparation",
        "--bench",
        "document_admission",
        "--locked",
        "--profile",
        profile,
        "--features",
        "native-process,pse-relations/force-validate",
        "--no-run",
        "--message-format=json",
    ]
    build = build_benchmark(command, output, "preparation")
    binaries = {
        row["target"]["name"]: row["executable"]
        for line in build.stdout.splitlines()
        if line.startswith("{")
        for row in [json.loads(line)]
        if row.get("reason") == "compiler-artifact"
        and row.get("executable")
        and row.get("target", {}).get("name")
        in {"modeling_preparation", "document_admission"}
    }
    if set(binaries) != {"modeling_preparation", "document_admission"}:
        raise ValueError(
            "both preparation and admission benchmark artifacts are required"
        )
    cases = []
    for workload in workloads:
        name = workload["id"]
        directory = output / "preparation" / name
        directory.mkdir(parents=True, exist_ok=False)
        env = {
            **os.environ,
            "PSE_PREPARATION_CASE": name,
            "PSE_PREPARATION_OUTPUT": str(directory),
        }
        with (directory / "process.log").open("w") as log:
            subprocess.run(
                [binaries["modeling_preparation"], "--bench"],
                cwd=ROOT,
                env=env,
                check=True,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
        cases.append(
            {
                **json.loads((directory / f"{name}.json").read_text()),
                **samples(
                    directory / "criterion/modeling_preparation" / name / "new/raw.csv"
                ),
            }
        )
    admission = None
    if admission_selected:
        directory = output / "admission"
        directory.mkdir(parents=True, exist_ok=False)
        with (directory / "process.log").open("w") as log:
            subprocess.run(
                [binaries["document_admission"], "--bench"],
                cwd=ROOT,
                env={**os.environ, "PSE_ADMISSION_OUTPUT": str(directory)},
                check=True,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
        admission = json.loads((directory / "admission.json").read_text())
        admission["samples"] = {
            phase: samples(
                directory
                / "criterion/document_admission"
                / phase
                / "100000/new/raw.csv"
            )
            for phase in ("load_and_decode", "admit")
        }
    return {
        "native": native_provenance(
            {
                "cargo_profile": profile,
                "features": ["native-process", "pse-relations/force-validate"],
            },
            list(binaries.values()),
        ),
        "cases": cases,
        "admission": admission,
    }


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("output", type=Path)
    parser.add_argument("--functional-from", type=Path)
    parser.add_argument(
        "--smoke",
        action="store_true",
        help="Run each selected control once without timing samples or a measurement receipt.",
    )
    parser.add_argument(
        "--case",
        action="append",
        default=[],
        help="Declared process, preparation or document-admission case; repeat to select several. Defaults to all.",
    )
    args = parser.parse_args()
    declaration = json.loads((ROOT / ".config/process-cases.json").read_text())
    preparation = json.loads((ROOT / ".config/preparation-cases.json").read_text())
    admission = {
        "workloads": [{"id": "document-admission", "functional_scopes": ["admission"]}]
    }
    selection = selected_workloads((declaration, preparation, admission), args.case)
    selected = {w["id"] for w in selection}
    if args.smoke:
        if args.functional_from:
            parser.error("smoke controls do not consume qualification receipts")
        output = validation.fresh_output(ROOT, args.output)
        smoke_campaign(
            output, declaration["cargo_profile"], declaration, preparation, selected
        )
        return 0
    if args.functional_from is None:
        parser.error("measurement requires --functional-from completed qualification")
    snapshot = validation.sources(ROOT)
    functional = require_functional(
        ROOT,
        args.functional_from,
        selection,
        snapshot=snapshot,
        profile=declaration["cargo_profile"],
    )
    output = validation.fresh_output(ROOT, args.output)
    started = time.time()
    inputs = measurement_inputs(snapshot)
    before = snapshot_digest(inputs)
    profile = {
        "cargo_profile": declaration["cargo_profile"],
        "features": ["native-process", "pse-relations/force-validate"],
    }
    command = [
        "cargo",
        "bench",
        "-p",
        "pse-benches",
        "--bench",
        "native_process",
        "--locked",
        "--profile",
        profile["cargo_profile"],
        "--features",
        "native-process,pse-relations/force-validate",
        "--no-run",
        "--message-format=json",
    ]
    build = build_benchmark(command, output, "process")
    artifacts = [
        json.loads(line) for line in build.stdout.splitlines() if line.startswith("{")
    ]
    binaries = {
        r["executable"]
        for r in artifacts
        if r.get("reason") == "compiler-artifact"
        and r.get("target", {}).get("name") == "native_process"
        and r.get("executable")
    }
    if len(binaries) != 1:
        raise ValueError("expected one compiled benchmark executable")
    binary = Path(binaries.pop())
    native = native_provenance(profile, [str(binary)])
    compatible_native(functional, native)
    cases = []
    for workload in declaration["workloads"]:
        name = workload["id"]
        if name not in selected:
            continue
        directory = output / "process-cost" / name
        directory.mkdir(parents=True, exist_ok=False)
        env = {
            **os.environ,
            "PSE_PROCESS_COST_CASE": name,
            "PSE_PROCESS_COST_OUTPUT": str(directory),
            "PSE_PROCESS_COST_SPEC": json.dumps(workload),
        }
        with (directory / "process.log").open("w") as log:
            subprocess.run(
                [str(binary), "--bench"],
                cwd=ROOT,
                env=env,
                check=True,
                stdout=log,
                stderr=subprocess.STDOUT,
            )
        memory = json.loads((directory / f"{name}-memory.json").read_text())
        sample_summary = samples(directory / "criterion/process" / name / "new/raw.csv")
        files = {
            str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in directory.rglob("*")
            if p.is_file()
        }
        cases.append(
            {
                **memory,
                **sample_summary,
                "artifacts": files,
            }
        )
    report = {
        "schema": "process-cost-v3",
        "profile": profile,
        "functional": functional,
        "started": started,
        "source_digest": before,
        "measurement_inputs": inputs,
        "compilation_timed": False,
        "selected_cases": [w["id"] for w in selection],
        "binary_path": str(binary.resolve()),
        "binary_digest": native["files"][str(binary.resolve())],
        "toolchain": native["toolchain"],
        "native": native,
        "thread_environment": {
            k: os.environ.get(k)
            for k in ["OMP_NUM_THREADS", "OPENBLAS_NUM_THREADS", "MKL_NUM_THREADS"]
        },
        "cases": cases,
    }
    preparation_report: dict = (
        preparation_campaign(
            output,
            profile["cargo_profile"],
            [w for w in preparation["workloads"] if w["id"] in selected],
            "document-admission" in selected,
        )
        if any(w["id"] in selected for w in preparation["workloads"])
        or "document-admission" in selected
        else {"cases": [], "admission": None, "not_selected": True}
    )
    report["thermodynamic_preparation"] = preparation_report
    if preparation_report.get("native"):
        compatible_native(functional, preparation_report["native"])
    if snapshot_digest(measurement_inputs(validation.sources(ROOT))) != before:
        raise ValueError("sources changed during preparation measurement")
    validation.write_json(output / "case-measure.json", report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

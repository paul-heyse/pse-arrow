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
from scripts.validation_scope import comprehensive

ROOT = Path(__file__).resolve().parents[1]


def source_digest(root: Path) -> str:
    return hashlib.sha256(
        json.dumps(validation.sources(root), sort_keys=True).encode()
    ).hexdigest()


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


def require_functional(root: Path, path: Path) -> dict:
    report = path / "checks.json" if path.is_dir() else path
    evidence = json.loads(report.read_text())
    if (
        evidence.get("version") != 4
        or not evidence.get("required_checks_covered")
        or evidence.get("scope")
        != json.loads(json.dumps([validation.asdict(g) for g in comprehensive()]))
    ):
        raise ValueError(
            "case measurement requires completed ordinary functional qualification"
        )
    if evidence["source_files"] != validation.sources(root):
        raise ValueError(
            "functional inputs changed; explicitly reassess or transfer affected observations"
        )
    for check in evidence["checks"]:
        if check.get("native"):
            validation_receipts.verify_native(check["native"])
    return {"path": str(report.resolve()), "digest": validation_receipts.digest(report)}


def preparation_campaign(output: Path, profile: str) -> dict:
    """Build untimed, then isolate every declared M1 workload and the 100k-row admission."""
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
    build = subprocess.run(
        command, cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE
    )
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
    declaration = json.loads((ROOT / ".config/preparation-cases.json").read_text())
    cases = []
    for workload in declaration["workloads"]:
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
            directory / "criterion/document_admission" / phase / "100000/new/raw.csv"
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
    parser.add_argument("--functional-from", type=Path, required=True)
    args = parser.parse_args()
    functional = require_functional(ROOT, args.functional_from)
    output = validation.fresh_output(ROOT, args.output)
    declaration = json.loads((ROOT / ".config/process-cases.json").read_text())
    started = time.time()
    before = source_digest(ROOT)
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
    build = subprocess.run(
        command, cwd=ROOT, check=True, text=True, stdout=subprocess.PIPE
    )
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
    cases = []
    for workload in declaration["workloads"]:
        name = workload["id"]
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
    if source_digest(ROOT) != before:
        raise ValueError("sources changed during measurement")
    report = {
        "schema": "process-cost-v3",
        "profile": profile,
        "functional": functional,
        "started": started,
        "source_digest": before,
        "compilation_timed": False,
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
    report["thermodynamic_preparation"] = preparation_campaign(
        output, profile["cargo_profile"]
    )
    if source_digest(ROOT) != before:
        raise ValueError("sources changed during preparation measurement")
    validation.write_json(output / "case-measure.json", report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

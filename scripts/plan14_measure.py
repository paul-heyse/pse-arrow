# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Guarded Criterion collector: fresh process per cost case, separate pool and RSS metrics."""

from __future__ import annotations
import hashlib
import json
import os
import subprocess
import sys
import time
from pathlib import Path
from scripts import implementation_phase as phase, validation
from scripts.plan14_acceptance import native_provenance

ROOT = Path(__file__).resolve().parents[1]


def source_digest(root: Path) -> str:
    return hashlib.sha256(
        json.dumps(validation.sources(root), sort_keys=True).encode()
    ).hexdigest()


def main() -> int:
    output = Path(sys.argv[1]).resolve()
    output.mkdir(parents=True, exist_ok=True)
    phase.guard(ROOT, ["plan14-measure"])
    declaration = phase.manifest(ROOT)
    case = next(
        c for c in declaration["cases"] if c.get("artifact") == "process-cost-v1"
    )
    started = time.time()
    before = source_digest(ROOT)
    command = [
        "cargo",
        "bench",
        "-p",
        "pse-benches",
        "--bench",
        "native_process",
        "--locked",
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
    profile = next(p for p in declaration["profiles"] if p["id"] == case["profile"])
    native = native_provenance(profile, [str(binary)])
    cases = []
    for name in case["tests"]:
        directory = output / "process-cost" / name
        directory.mkdir(parents=True, exist_ok=False)
        env = {
            **os.environ,
            "PSE_PLAN14_COST_CASE": name,
            "PSE_PLAN14_COST_OUTPUT": str(directory),
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
        estimate = json.loads(
            (directory / "criterion/process" / name / "new/estimates.json").read_text()
        )
        samples = json.loads(
            (directory / "criterion/process" / name / "new/sample.json").read_text()
        )
        mean = estimate["mean"]
        files = {
            str(p.relative_to(output)): hashlib.sha256(p.read_bytes()).hexdigest()
            for p in directory.rglob("*")
            if p.is_file()
        }
        cases.append(
            {
                **memory,
                "mean_nanoseconds": mean["point_estimate"],
                "confidence_interval": mean["confidence_interval"],
                "sample_count": len(samples["times"]),
                "artifacts": files,
            }
        )
    if source_digest(ROOT) != before:
        raise ValueError("sources changed during measurement")
    report = {
        "schema": "process-cost-v1",
        "profile": case["profile"],
        "started": started,
        "source_digest": before,
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
    validation.write_json(output / "plan14-measure.json", report)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded public Python/PyO3/Rust/KINSOL sample target using qualified fixtures."""

from __future__ import annotations

import argparse
import json
import math
import os
import threading
import time
from pathlib import Path

import pse
from pse.contracts.documents import ReviewContextRequest
from pse.contracts.enums import NativeBackend, NativeSolveIntent, PresolvePolicyKind
from pse.tests.test_local_runtime import physical_documents
from pse.tests.test_native_workflow import package_documents
from scripts import (
    active_diagnostics,
    operation_lifetime,
    resource_store,
)


def run(output: Path, seconds: float) -> dict:
    output.mkdir(mode=0o700, exist_ok=False)
    spill = output / "spill"
    spill.mkdir(mode=0o700)
    settings = pse.EngineSettings(
        memory_limit_bytes=64 << 30,
        threads=2,
        target_partitions=1,
        spill_dir=str(spill),
        max_spill_bytes=1 << 30,
        batch_size=7,
        cache=pse.CacheSettings(
            working_bytes=16 << 30,
            concurrent_loads=2,
            inflight_bytes=64 << 20,
            inspection_bytes=4 << 20,
        ),
    )
    source = "package diagnostic_root { def Root { var x:Scalar; eq square:x*x==2; annotation start x(1); } }"
    (output / "root.pse").write_text(source)
    runtime = pse.Runtime(settings, ephemeral=True)
    stop = threading.Event()
    progress = {"python_peer_iterations": 0}

    def python_cpu_peer() -> None:
        while not stop.is_set():
            progress["python_peer_iterations"] += sum(range(128))
            time.sleep(0.001)

    peer = threading.Thread(target=python_cpu_peer, name="diagnostic-python-peer")
    try:
        physical = runtime.physical_from_documents(physical_documents())
        package = runtime.modeling_from_documents([package_documents(source)], physical)
        case = next(
            item.declaration_id
            for item in package.declarations()
            if item.name == "Root"
        )
        prepared = package.prepare_solve(
            case,
            pse.SolveSettings(
                intent=NativeSolveIntent.ROOT,
                backend=NativeBackend.KINSOL,
                presolve=PresolvePolicyKind.OFF,
            ),
        )
        subject = {
            "pid": os.getpid(),
            "start": operation_lifetime.start_identity(os.getpid()),
            "boot": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
            "executable": active_diagnostics.regular(Path("/proc/self/exe").readlink()),
            "created_monotonic_ns": time.monotonic_ns(),
            "route": "public ephemeral Runtime nonlinear scalar KINSOL with presolve OFF",
        }
        resource_store.publish(output / "subject.json", subject)
        print(json.dumps({"ready": subject}), flush=True)
        peer.start()
        time.sleep(1.5)
        deadline = time.monotonic() + seconds
        runs = 0
        result = None
        while time.monotonic() < deadline and runs < 1000:
            result = prepared.start().wait()
            if not result.usable:
                raise RuntimeError(
                    "mixed diagnostic target's actual KINSOL result was unusable"
                )
            runs += 1
        if result is None:
            raise RuntimeError("mixed diagnostic target did not execute a solve")
        context = result.review_context(
            request=ReviewContextRequest(
                steps=(),
                include_native_records=True,
                include_sources=True,
                maximum_bytes=16 << 20,
            )
        )
        records = [
            json.loads(bytes(fact.payload))
            for fact in context.facts
            if fact.codec == "native-attempt-record.v1"
        ]
        if not any(record.get("backend") == "kinsol" for record in records):
            raise RuntimeError(
                "result did not retain an actual KINSOL native attempt record"
            )
        projection = {
            "subject": subject,
            "runs": runs,
            "last_run_id": str(result.run_id),
            "usable": result.usable,
            "native_records": records,
            **progress,
            "coverage": "known Python main/new thread, Rust-owned runtime workers and actual native KINSOL attempts; no timing or scientific qualification claim",
        }
        resource_store.publish(output / "result.json", projection)
        print(json.dumps(projection), flush=True)
        return projection
    finally:
        stop.set()
        if peer.ident is not None:
            peer.join(timeout=5)
        runtime.close()


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--seconds", type=float, default=8)
    args = parser.parse_args()
    if not math.isfinite(args.seconds) or not 0 < args.seconds <= 30:
        parser.error("duration must be finite in (0,30]")
    run(args.output.absolute(), args.seconds)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

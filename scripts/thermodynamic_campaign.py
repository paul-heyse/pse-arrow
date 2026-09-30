# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Plan 23 paired feeds and complete independent native outcomes; no failure filtering."""

from __future__ import annotations

import argparse
import hashlib
import json
import time
from pathlib import Path
from tempfile import TemporaryDirectory

import msgspec
import pyarrow as pa
import pyarrow.parquet as pq

import pse
from pse import conformance
from pse.contracts.documents import PointOverlay, SolveControls, SolveSettings
from pse.contracts.enums import NativeBackend, NativeSolveIntent, NativeTermination
from scripts.thermodynamic_feeds import COUNT, FEEDS, ROOT


def documents(path: Path) -> dict[str, str | bytes]:
    return {
        file.relative_to(path).as_posix(): file.read_bytes()
        for file in path.rglob("*")
        if file.is_file()
        and file.suffix in {".toml", ".pse", ".yaml", ".yml", ".parquet"}
    }


def measure(output: Path, selected: str) -> None:
    output.mkdir(parents=True, exist_ok=False)
    manifest = conformance.load_manifest(ROOT / "packages/reference/conformance.toml")
    spec = next(run for run in manifest.runs if run.name == "seed")
    base = ROOT / "packages/reference"
    feeds = pq.read_table(FEEDS).to_pylist()
    if len(feeds) != COUNT or [row["index"] for row in feeds] != list(range(COUNT)):
        raise ValueError("the complete ordered frozen 200-feed case set is required")
    with TemporaryDirectory(prefix="pse-thermodynamic-campaign-") as spill:
        engine = pse.EngineSettings(
            memory_limit_bytes=48 << 30,
            threads=1,
            spill_dir=spill,
            max_spill_bytes=1 << 30,
            batch_size=1024,
        )
        runtime = pse.Runtime(engine)
        package = runtime.modeling_from_documents(
            [
                documents(base / spec.package),
                *(documents(base / name) for name in spec.dependencies),
            ],
            runtime.physical_from_documents(documents(base / spec.physical)),
        ).with_limits(pse.ModelingLimits(items=1000000, body_occurrences=65536))
        cases = {
            row.name: row.declaration_id
            for row in package.declarations()
            if row.name.startswith("measurement_")
        }
        settings = SolveSettings(
            backend=NativeBackend.IPOPT,
            intent=NativeSolveIntent.FEASIBLE_POINT,
            controls=SolveControls(time_limit=600, threads=1, history=4096),
        )
        phases = (
            ("measurement_smooth", "measurement_nested")
            if selected == "flash"
            else ("measurement_value_sweep",)
        )
        summaries = []
        for name in phases:
            overlays = (
                tuple(
                    PointOverlay(
                        values={
                            "root.inlet.T": row["temperature"],
                            "root.inlet.pressure": row["pressure"],
                            "root.inlet.z[chem.benzene]": row["benzene"],
                            "root.inlet.z[chem.toluene]": 1.0 - row["benzene"],
                        }
                    )
                    for row in feeds
                )
                if selected == "flash"
                else tuple(
                    PointOverlay(values={"root.inlet.T": 360.0 + 10.0 * index / 999})
                    for index in range(1000)
                )
            )
            started = time.perf_counter()
            study = package.study(
                tuple(cases[name] for _ in overlays),
                settings,
                overlays=overlays,
                maximum_points=len(overlays),
            )
            elapsed = time.perf_counter() - started
            resources = runtime.resource_usage()
            successful = accepted = 0
            with (output / f"{name}.jsonl").open("w") as stream:
                for index in range(study.count):
                    result = study.result(index)
                    attempt = None if result is None else result.attempt()
                    failure = (
                        study.failure(index) if result is None else result.failure()
                    )
                    native_success = attempt is not None and attempt.termination in (
                        NativeTermination.SUCCESS,
                        NativeTermination.ACCEPTABLE,
                    )
                    successful += int(native_success)
                    accepted += int(result is not None and result.accepted)
                    events, dropped = ([], 0) if attempt is None else attempt.progress()
                    record = {
                        "index": index,
                        "feed": feeds[index]
                        if selected == "flash"
                        else {"temperature": 360.0 + 10.0 * index / 999},
                        "accepted": result is not None and result.accepted,
                        "native_success": native_success,
                        "termination": None if attempt is None else attempt.termination,
                        "native_status": None
                        if attempt is None
                        else attempt.native_status,
                        "metrics": None
                        if attempt is None
                        else attempt.metrics().values(),
                        "progress": [
                            {
                                "step": event.step,
                                "phase": event.phase,
                                "elapsed_seconds": event.elapsed_seconds,
                                "values": event.values(),
                            }
                            for event in events
                        ],
                        "dropped_events": dropped,
                        "failure": None
                        if failure is None
                        else {
                            "code": failure.code,
                            "message": failure.message,
                            "stage": failure.stage,
                        },
                    }
                    stream.write(json.dumps(record, allow_nan=False) + "\n")
            pq.write_table(pa.table(study.table()), output / f"{name}.parquet")
            findings = pa.table(study.findings())
            with (
                pa.OSFile(str(output / f"{name}.findings.arrow"), "wb") as sink,
                pa.ipc.new_file(sink, findings.schema) as writer,
            ):
                writer.write_table(findings)
            summaries.append(
                {
                    "case": name,
                    "count": study.count,
                    "unattempted": study.unattempted,
                    "native_success": successful,
                    "accepted": accepted,
                    "seconds": elapsed,
                    "preparations": msgspec.to_builtins(study.preparations),
                    "memory": {
                        "pool_peak_bytes": resources.pool_peak_bytes,
                        "process_peak_rss_bytes": resources.process_peak_rss_bytes,
                        "scope": "deployment pool and process lifetime high-water marks",
                    },
                }
            )
            print(json.dumps(summaries[-1]), flush=True)
        (output / "summary.json").write_text(
            json.dumps(
                {
                    "scope": selected,
                    "feed_sha256": hashlib.sha256(FEEDS.read_bytes()).hexdigest(),
                    "feed_generator": pq.read_schema(FEEDS)
                    .metadata[b"generator"]
                    .decode(),
                    "pool_limit_bytes": 48 << 30,
                    "solver_threads": 1,
                    "time_limit_seconds": 600,
                    "start": "no_prior_start",
                    "predecessors": "none",
                    "outcomes": summaries,
                },
                indent=2,
            )
            + "\n"
        )


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("phase", choices=("flash", "study"))
    parser.add_argument("output", type=Path, nargs="?")
    args = parser.parse_args()
    if args.output is None:
        parser.error("measurement requires a fresh output directory")
    else:
        measure(args.output, args.phase)


if __name__ == "__main__":
    main()

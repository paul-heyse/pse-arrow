# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Plan 23 paired feeds and complete independent native outcomes; no failure filtering."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import time
from pathlib import Path
from tempfile import TemporaryDirectory

import msgspec
import pyarrow as pa
import pyarrow.parquet as pq

import pse
from pse import conformance
from pse.contracts.documents import (
    AssemblyLimits,
    BindingAssignment,
    BindingQuantity,
    BindingTargetPath,
    CaseOperation,
    EvaluationLimits,
    Limits,
    OperationRequestDeclaredCase,
    Optimization,
    PointOverlay,
    PreparationSettings,
    SolveControls,
    SolveSettings,
    StartPolicyFresh,
    StudyCompilerProfile,
    StudyPoint,
    StudyPointPolicy,
    StudyRequest,
    StudyRunControls,
)
from pse.contracts.enums import (
    ModelingAnalysisRoute,
    NativeBackend,
    NativeSolveIntent,
    NativeTermination,
)
from scripts.thermodynamic_feeds import COUNT, FEEDS, ROOT


def documents(path: Path) -> dict[str, str | bytes]:
    return {
        file.relative_to(path).as_posix(): file.read_bytes()
        for file in path.rglob("*")
        if file.is_file()
        and file.suffix in {".toml", ".pse", ".yaml", ".yml", ".parquet"}
    }


def binary_hex(value: object) -> str:
    """Retain Arrow binary identity/hash cells as hexadecimal in JSON receipts."""
    if isinstance(value, bytes):
        return value.hex()
    message = f"unsupported campaign JSON value: {type(value).__name__}"
    raise TypeError(message)


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
        runtime = pse.Runtime(engine, substrate=os.environ["PSE_SURREAL_STATE"])
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
        # Campaign-owned bounds are explicit inputs to admission, not production defaults.
        preparation = PreparationSettings(
            compiler=StudyCompilerProfile(
                class_proof_work=1_000_000,
                assembly=AssemblyLimits(
                    contributions=1_000_000,
                    native_index=2_147_483_647,
                    worker_bytes=2 << 30,
                ),
                optimization=Optimization(
                    cores=1, horner_iterations=10, cpe_iterations=10
                ),
                evaluation=EvaluationLimits(
                    derivative_components=1_000_000,
                    operations=100_000_000,
                    scratch_bytes=1 << 30,
                    provider_calls=1_000_000,
                ),
            ),
            limits=Limits(
                depth=64, items=1_000_000, members=1_000_000, body_occurrences=65536
            ),
        )
        physical_text = (base / spec.physical / "materials/physical.yaml").read_text()
        physical_document = msgspec.json.decode(
            "\n".join(
                line for line in physical_text.splitlines() if not line.startswith("#")
            ),
            type=dict[str, object],
        )
        quantities = msgspec.convert(
            physical_document["quantity_types"], type=list[dict[str, object]]
        )
        units = msgspec.convert(
            physical_document["units"], type=list[dict[str, object]]
        )

        def assignment(
            target: str, magnitude: float, quantity: str, symbol: str
        ) -> BindingAssignment:
            quantity_id = next(
                row["quantity_type_id"]
                for row in quantities
                if row.get("name") == quantity
            )
            unit_id = next(row["unit_id"] for row in units if row["symbol"] == symbol)
            if not isinstance(quantity_id, str) or not isinstance(unit_id, str):
                message = "reference physical identities must be declared strings"
                raise TypeError(message)
            return BindingAssignment(
                target=BindingTargetPath(value=target),
                value=BindingQuantity(
                    magnitude=magnitude, quantity=quantity_id, unit=unit_id
                ),
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
                        assignments=(
                            assignment(
                                "root.inlet.T", row["temperature"], "Temperature", "K"
                            ),
                            assignment(
                                "root.inlet.pressure", row["pressure"], "Pressure", "Pa"
                            ),
                            assignment(
                                "root.inlet.z[chem.benzene]",
                                row["benzene"],
                                "MoleFraction",
                                "1",
                            ),
                            assignment(
                                "root.inlet.z[chem.toluene]",
                                1.0 - row["benzene"],
                                "MoleFraction",
                                "1",
                            ),
                        )
                    )
                    for row in feeds
                )
                if selected == "flash"
                else tuple(
                    PointOverlay(
                        assignments=(
                            assignment(
                                "root.inlet.T",
                                360.0 + 10.0 * index / 999,
                                "Temperature",
                                "K",
                            ),
                        )
                    )
                    for index in range(1000)
                )
            )
            started = time.perf_counter()
            definition = package.admit_study(
                StudyRequest(
                    points=tuple(
                        StudyPoint(
                            operation=OperationRequestDeclaredCase(
                                request=CaseOperation(
                                    case=cases[name].to_hex(),
                                    route=ModelingAnalysisRoute.STEADY,
                                    settings=settings,
                                )
                            ),
                            preparation=preparation,
                            overlay=overlay,
                            policy=StudyPointPolicy(
                                key=index,
                                dependencies=(),
                                start=StartPolicyFresh(),
                                attempt_limit=1,
                            ),
                        )
                        for index, overlay in enumerate(overlays)
                    )
                )
            )
            study = package.study(
                definition,
                controls=StudyRunControls(maximum_points=len(overlays)),
            )
            elapsed = time.perf_counter() - started
            resources = runtime.resource_usage()
            successful = usable = 0
            with (output / f"{name}.jsonl").open("w") as stream:
                for index in range(study.count):
                    result = study.result(index)
                    completion = None if result is None else result.completion
                    native_success = (
                        completion is not None
                        and bool(completion.solves)
                        and all(
                            solve.termination
                            in (NativeTermination.SUCCESS, NativeTermination.ACCEPTABLE)
                            for solve in completion.solves
                        )
                    )
                    successful += int(native_success)
                    usable += int(result is not None and result.usable)
                    failure = study.failure(index)
                    diagnostics = () if result is None else result.diagnostics()
                    # These final tables retain every solve and metric, including failed
                    # and unattempted steps. Progress is persisted as event.<seq>.<phase>
                    # namespaces, with drop counts in the progress namespace.
                    tables = (
                        {}
                        if result is None
                        else {
                            table_name: pa.table(result.table(table_name)).to_pylist()
                            for table_name in (
                                "runtime.solve_runs",
                                "runtime.solve_metrics",
                            )
                        }
                    )
                    record = {
                        "index": index,
                        "feed": feeds[index]
                        if selected == "flash"
                        else {"temperature": 360.0 + 10.0 * index / 999},
                        "usable": result is not None and result.usable,
                        "native_success": native_success,
                        "outcome": msgspec.to_builtins(study.outcome(index)),
                        "completion": msgspec.to_builtins(completion),
                        "tables": tables,
                        "diagnostics": [
                            {
                                "message": diagnostic.message,
                                "envelope": msgspec.to_builtins(diagnostic.envelope),
                            }
                            for diagnostic in diagnostics
                        ],
                        "failure": None
                        if failure is None
                        else {
                            "message": failure.message,
                            "envelope": msgspec.to_builtins(failure.envelope),
                        },
                    }
                    stream.write(
                        json.dumps(record, allow_nan=False, default=binary_hex) + "\n"
                    )
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
                    "usable": usable,
                    "conclusion": msgspec.to_builtins(study.conclusion),
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

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite native-owner -> checkpoint6 -> production SDK -> backend acceptance.

Requires the linked correctness extension and an already running telemetry service.
Run explicitly through scripts/pse-env --native at the functional boundary. Exactly
two tiny solves run; cold concurrent preparation, retained reuse, a parameter edit
and structural preparation are additional distinct populations. Optional canonical
coverage requires an explicitly selected, already provisioned disposable database.
No service is installed, restarted or stopped. A supplied pre-restart server metric
snapshot can witness an external owner's actual restart without subtracting epochs.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import math
import os
import subprocess
import sys
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import TYPE_CHECKING

from scripts import operation_lifetime, telemetry_service
from scripts.telemetry_investigation import BackendClient, QueryRequest
from scripts.tests.telemetry_operational_acceptance import environment, write
from scripts.tests.telemetry_pipeline_acceptance import mapping, records, require

if TYPE_CHECKING:
    from pse import PreparedOperation, Runtime
    from pse._runs import RunHandle
    from scripts.telemetry_investigation import QueryAnswer

ROOT = Path(__file__).resolve().parents[2]
TIMEOUT_SECONDS = 120.0


def checkpoint(runtime: Runtime, not_before_us: int) -> dict[str, object]:
    """Read the native versioned publication after its performing owners settled."""
    from pse import codec, diagnostics  # noqa: PLC0415

    path = runtime.observations().checkpoint
    deadline = time.monotonic() + 15
    while True:
        if path.is_file():
            image = diagnostics.read_checkpoint(path)
            if image.published_unix_micros >= not_before_us:
                value = mapping(json.loads(codec.encode_json(image)))
                require(
                    value.get("version") == 6,
                    "native_operational.checkpoint_version_changed",
                )
                require(
                    mapping(value["locator"]).get("pid") == os.getpid(),
                    "native_operational.checkpoint_process_changed",
                )
                require(
                    mapping(value["runtime_metrics"]).get("operational") is not None,
                    "native_operational.owner_sources_missing",
                )
                return value
        require(time.monotonic() < deadline, "native_operational.checkpoint_not_fresh")
        time.sleep(0.05)


def stage(snapshot: dict[str, object], name: str) -> dict[str, object]:
    matches = [
        row
        for row in records(mapping(snapshot["operational"])["preparation"])
        if row.get("stage") == name
    ]
    require(len(matches) == 1, "native_operational.preparation_stage_missing")
    return matches[0]


def validate_preparation_stages(snapshots: dict[str, object]) -> None:
    """Independent scripted request counts; these are not scientific solve totals."""
    images = {
        name: mapping(mapping(image)["runtime_metrics"])
        for name, image in snapshots.items()
    }
    for name in ("frontier", "basis"):
        require(
            int(str(stage(images["cold"], name)["producers"]))
            - int(str(stage(images["before"], name)["producers"]))
            == 1,
            "native_operational.cold_producer_population_changed",
        )
    require(
        int(str(stage(images["cold"], "basis")["acquisitions"]))
        - int(str(stage(images["before"], "basis")["acquisitions"]))
        == 2,
        "native_operational.concurrent_acquisition_population_changed",
    )
    require(
        int(str(stage(images["retained"], "basis")["retained_reuses"]))
        > int(str(stage(images["first_solve"], "basis")["retained_reuses"])),
        "native_operational.retained_reuse_not_observed",
    )
    require(
        stage(images["retained"], "basis")["producers"]
        == stage(images["first_solve"], "basis")["producers"],
        "native_operational.retained_read_produced_again",
    )
    require(
        int(str(mapping(images["structural_edit"]["preparations"])["views"]))
        > int(str(mapping(images["value_edit"]["preparations"])["views"])),
        "native_operational.structural_view_not_rebuilt",
    )


def source_expectations(
    snapshot: dict[str, object],
) -> dict[str, list[tuple[dict[str, str], int | float]]]:
    """Map only identical populations; never infer per-operation work from totals."""
    native, preparation, canonical = (
        mapping(snapshot[key])
        for key in ("operational", "preparations", "canonical_queries")
    )
    require(
        not native["saturated"] and not canonical["saturated"],
        "native_operational.saturated_source",
    )
    require(
        native["source_started_unix_micros"] is not None,
        "native_operational.native_source_absent",
    )
    answer: dict[str, list[tuple[dict[str, str], int | float]]] = {
        "pse_native_supervisor_events": [
            ({"event": event}, int(str(native[field])))
            for event, field in (
                ("attempt", "native_attempts"),
                ("admitted", "native_admitted"),
                ("dispatched", "native_dispatched"),
                ("joined", "native_joined"),
                ("cancel_requested", "native_cancel_requests"),
                ("deadline_requested", "native_deadline_requests"),
            )
        ],
        "pse_native_supervisors": [({}, int(str(native["native_supervisors"])))],
        "pse_preparation_products": [
            ({"product": product}, int(str(preparation[field])))
            for product, field in (
                ("structural_view", "views"),
                ("observation_program", "observations"),
                ("value_rebuilt", "rebuilt"),
                ("value_shared", "shared"),
            )
        ],
        "pse_preparation_flight_events": [],
        "pse_operational_source_start": [
            (
                {"owner": "native_preparation"},
                int(str(native["source_started_unix_micros"])) / 1_000_000,
            )
        ],
    }
    for row in records(native["preparation"]):
        answer["pse_preparation_flight_events"].extend(
            ({"stage": str(row["stage"]), "event": event}, int(str(row[field])))
            for event, field in (
                ("acquisition", "acquisitions"),
                ("retained_reuse", "retained_reuses"),
                ("producer", "producers"),
                ("flight_waiter", "flight_waiters"),
            )
        )
    if canonical["source_started_unix_micros"] is not None:
        answer["pse_canonical_query_events"] = [
            ({"event": event}, int(str(canonical[field])))
            for event, field in (
                ("builder", "builders"),
                ("request_failed", "request_failures"),
                ("statement", "statements"),
                ("statement_succeeded", "statements_succeeded"),
                ("statement_failed", "statements_failed"),
                ("statement_timed", "statements_timed"),
                ("statement_untimed", "statements_untimed"),
            )
        ]
        answer["pse_operational_source_start"].append(
            (
                {"owner": "canonical_query"},
                int(str(canonical["source_started_unix_micros"])) / 1_000_000,
            )
        )
        if int(str(canonical["statements_timed"])) > 0:
            answer["pse_canonical_server_duration"] = [
                ({}, int(str(canonical["server_duration_nanos"])) / 1_000_000_000)
            ]
    # Exact histogram count matches completed owners here: SDK activation preceded
    # every source event, all supervisors have returned, and producers have settled.
    require(
        native["native_supervisors"] == 0,
        "native_operational.supervisors_not_quiescent",
    )
    answer["pse_owner_interval_count"] = [
        (
            {"owner_family": "native", "stage": "supervisor"},
            int(str(native["native_attempts"])),
        )
    ]
    for row in records(native["preparation"]):
        if int(str(row["producers"])):
            answer["pse_owner_interval_count"].append(
                (
                    {"owner_family": "preparation", "stage": str(row["stage"])},
                    int(str(row["producers"])),
                )
            )
    return answer


def compare_points(
    answer: QueryAnswer,
    expected: list[tuple[dict[str, str], int | float]],
    instance: str,
    not_before_us: int,
) -> None:
    """Select latest cumulative observations per series, never sum repeated totals."""
    require(
        answer.status == "available" and answer.source is not None and answer.records,
        "native_operational.metric_unavailable",
    )
    require(
        answer.meaning == "observed_points" and not answer.complete,
        "native_operational.metric_claim_changed",
    )
    for attributes, value in expected:
        selected = []
        for row in answer.records:
            require(
                row.get("service_instance_id") == instance,
                "native_operational.metric_process_changed",
            )
            if int(str(row["_timestamp"])) >= not_before_us and all(
                row.get(key.replace(".", "_")) == label
                for key, label in attributes.items()
            ):
                selected.append(row)
        require(selected, "native_operational.metric_series_missing")
        latest = max(selected, key=lambda row: int(str(row["_timestamp"])))
        actual = latest.get("value")
        require(
            isinstance(actual, (int, float)) and not isinstance(actual, bool),
            "native_operational.metric_value_missing",
        )
        require(
            math.isclose(
                float(str(actual)),
                value,
                rel_tol=0,
                abs_tol=1e-6
                if answer.request.metric == "pse_operational_source_start"
                else 1e-12,
            ),
            "native_operational.native_backend_population_differs",
        )
        if answer.request.metric not in {
            "pse_native_supervisors",
            "pse_operational_source_start",
        }:
            require(
                latest.get("aggregation_temporality")
                == "AGGREGATION_TEMPORALITY_CUMULATIVE",
                "native_operational.temporality_changed",
            )
            require(
                latest.get("start_time") is not None,
                "native_operational.sdk_start_missing",
            )


def producer(
    state: Path,
    ready: Path,
    release: Path,
    database_state: Path | None,
    database: str | None,
) -> dict[str, object]:
    import tempfile  # noqa: PLC0415

    import msgspec  # noqa: PLC0415
    import pyarrow as pa  # noqa: PLC0415

    import pse  # noqa: PLC0415
    from pse import codec, diagnostics  # noqa: PLC0415
    from pse.contracts.documents import CheckpointIdentity  # noqa: PLC0415
    from pse.contracts.enums import (  # noqa: PLC0415
        NativeBackend,
        NativeSolveIntent,
        PresolvePolicyKind,
    )
    from pse.tests.test_local_runtime import physical_documents  # noqa: PLC0415
    from pse.tests.test_native_workflow import package_documents  # noqa: PLC0415
    from scripts import (  # noqa: PLC0415
        surreal_server,
        telemetry_process,
    )

    require(
        os.environ.get("PSE_TELEMETRY_STATE") == str(state),
        "native_operational.telemetry_state_changed",
    )
    with (
        tempfile.TemporaryDirectory(prefix="pse-native-operational-") as spill,
        telemetry_process.bootstrap(
            "pse-native-operational-acceptance", native=True
        ) as lifetime,
    ):
        native, resolved = lifetime.native, lifetime.settings
        if native is None or resolved is None:
            raise RuntimeError("native_operational.sdk_not_initialized")
        require(
            lifetime.status == "initialized", "native_operational.sdk_not_initialized"
        )
        require(
            native.activation.get("status") in {"initialized", "already_initialized"},
            "native_operational.native_sdk_not_initialized",
        )
        settings = pse.EngineSettings(
            memory_limit_bytes=64 << 30,
            threads=2,
            spill_dir=spill,
            max_spill_bytes=1 << 30,
            batch_size=32,
        )
        server_before = (
            surreal_server.metric_snapshot(database_state, time.monotonic() + 10)
            if database_state is not None
            else None
        )
        runtime = pse.Runtime(
            settings,
            ephemeral=database_state is None,
            substrate=str(database_state) if database_state is not None else None,
            database=database,
        )
        snapshots: dict[str, object] = {}
        try:
            physical = runtime.physical_from_documents(physical_documents())

            def source(target: int, *, structural: bool = False) -> dict[str, str]:
                return package_documents(
                    "package telemetry { def Root { param target:Scalar="
                    + str(target)
                    + "; var x:Scalar; eq balance:"
                    + ("2*x" if structural else "x")
                    + "==target; annotation start x(0); } }"
                )

            package = runtime.modeling_from_documents([source(3)], physical)
            root = next(
                row.declaration_id
                for row in package.declarations()
                if row.name == "Root"
            )
            solve = pse.SolveSettings(
                intent=NativeSolveIntent.ROOT,
                backend=NativeBackend.KINSOL,
                presolve=PresolvePolicyKind.OFF,
            )
            snapshots["before"] = checkpoint(runtime, time.time_ns() // 1000)
            barrier = threading.Barrier(2)

            def prepare() -> PreparedOperation:
                barrier.wait(timeout=15)
                return package.prepare_solve(root, solve)

            with ThreadPoolExecutor(max_workers=2) as workers:
                futures = [workers.submit(prepare), workers.submit(prepare)]
                prepared = [future.result(timeout=60) for future in futures]
            require(
                prepared[0].identity == prepared[1].identity,
                "native_operational.shared_request_identity_differs",
            )
            snapshots["cold"] = checkpoint(runtime, time.time_ns() // 1000)

            def verified_result(handle: RunHandle, target: int) -> None:
                result = handle.wait()
                require(result.usable, "native_operational.solve_not_usable")
                stream = result.table("runtime.solve_variables")
                try:
                    candidates = [
                        row
                        for row in pa.table(stream).to_pylist()
                        if not row["parameter"]
                    ]
                finally:
                    stream.close()
                require(
                    len(candidates) == 1
                    and math.isclose(
                        float(candidates[0]["value"]), target, rel_tol=0, abs_tol=1e-8
                    ),
                    "native_operational.solved_value_differs",
                )

            first_handle = prepared[0].start()
            verified_result(first_handle, 3)
            first_observations = first_handle.observations()
            if first_observations is None:
                raise RuntimeError("native_operational.original_operation_missing")
            operation = mapping(json.loads(codec.encode_json(first_observations.id)))
            snapshots["first_solve"] = checkpoint(runtime, time.time_ns() // 1000)
            require(
                package.prepare_solve(root, solve).identity == prepared[0].identity,
                "native_operational.retained_request_identity_differs",
            )
            require(
                package.reload_from_parent([source(3)]).revision_id
                == package.revision_id,
                "native_operational.unchanged_source_not_reused",
            )
            snapshots["retained"] = checkpoint(runtime, time.time_ns() // 1000)
            changed = package.reload_from_parent([source(4)])
            require(
                changed.revision_id != package.revision_id,
                "native_operational.value_edit_revision_unchanged",
            )
            value_prepared = changed.prepare_solve(root, solve)
            require(
                value_prepared.identity != prepared[0].identity,
                "native_operational.value_preparation_identity_unchanged",
            )
            verified_result(value_prepared.start(), 4)
            snapshots["value_edit"] = checkpoint(runtime, time.time_ns() // 1000)
            structural = changed.reload_from_parent([source(4, structural=True)])
            require(
                structural.revision_id != changed.revision_id,
                "native_operational.structural_edit_revision_unchanged",
            )
            structural_prepared = structural.prepare_solve(root, solve)
            require(
                structural_prepared.identity != value_prepared.identity,
                "native_operational.structural_preparation_identity_unchanged",
            )
            snapshots["structural_edit"] = final = checkpoint(
                runtime, time.time_ns() // 1000
            )
            validate_preparation_stages(snapshots)
            require(
                mapping(final["runtime_metrics"])["canonical_queries"] is not None,
                "native_operational.canonical_source_missing",
            )
            final_source = mapping(final["runtime_metrics"])
            if database_state is not None:
                require(
                    int(str(mapping(final_source["canonical_queries"])["statements"]))
                    > 0,
                    "native_operational.sdk_database_statements_absent",
                )
            retained_checkpoint = ready.with_name("retained-native-checkpoint.json")
            retained_bytes = json.dumps(
                final, separators=(",", ":"), allow_nan=False
            ).encode()
            require(
                len(retained_bytes) <= 1 << 20,
                "native_operational.retained_checkpoint_bound",
            )
            retained_checkpoint.write_bytes(retained_bytes)
            retained_checkpoint.chmod(0o600)
            require(
                mapping(
                    json.loads(
                        codec.encode_json(
                            diagnostics.read_checkpoint(retained_checkpoint)
                        )
                    )
                )
                == final,
                "native_operational.retained_checkpoint_changed",
            )
            completed = [
                row
                for row in records(final["operations"])
                if row.get("finished_at") is not None
            ]
            require(completed, "native_operational.finished_operation_not_retained")
            requested_solve_operation = operation
            selected_operation = next(
                (row for row in completed if mapping(row["id"]) == operation),
                completed[0],
            )
            operation = mapping(selected_operation["id"])
            expected_identity = msgspec.convert(
                {"process": final["process"], "locator": final["locator"]},
                type=CheckpointIdentity,
            )
            expected_wire = mapping(json.loads(codec.encode_json(expected_identity)))
            produced: dict[str, object] = {
                "pid": os.getpid(),
                "process_start": operation_lifetime.start_identity(os.getpid()),
                "service_instance_id": resolved.wire.resource["service.instance.id"],
                "configuration_id": resolved.configuration_id,
                "checkpoint": str(retained_checkpoint),
                "source_checkpoint": str(runtime.observations().checkpoint),
                "checkpoint_copy": "native-admitted same snapshot, JSON formatting normalized",
                "operation": operation,
                "requested_solve_operation": requested_solve_operation,
                "operation_selection": "original completed solve when retained, otherwise an actual terminal preparation owner from this same checkpoint",
                "expected": expected_wire,
                "snapshots": {
                    name: {
                        key: image[key]
                        for key in (
                            "version",
                            "sequence",
                            "process",
                            "locator",
                            "published_unix_micros",
                            "runtime_metrics",
                        )
                    }
                    for name, raw in snapshots.items()
                    for image in (mapping(raw),)
                },
                "snapshot_coverage": "selected identity/owner metrics for phases; complete final native checkpoint retained separately",
                "final_source": final_source,
                "final_checkpoint_us": final["published_unix_micros"],
                "scientific_solves": 2,
                "shared_preparation_demand": 2,
                "source_revisions": {
                    "cold": package.revision_id,
                    "value_edit": changed.revision_id,
                    "structural_edit": structural.revision_id,
                },
                "server_before": server_before,
                "server_after": surreal_server.metric_snapshot(
                    database_state, time.monotonic() + 10
                )
                if database_state is not None
                else None,
            }
            source_expectations(final_source)
            write(ready, produced)
            deadline = time.monotonic() + TIMEOUT_SECONDS
            while not release.is_file():
                require(
                    time.monotonic() < deadline,
                    "native_operational.observer_release_timeout",
                )
                time.sleep(0.1)
            return produced
        finally:
            runtime.close()


async def observe(
    state: Path,
    output: Path,
    sampler: str,
    database_state: Path | None,
    database: str | None,
) -> dict[str, object]:
    ready, release = output / "producer-ready.json", output / "producer-release"
    command = [
        sys.executable,
        "-m",
        "scripts.tests.telemetry_native_operational_acceptance",
        "--produce",
        "--state",
        str(state),
        "--ready",
        str(ready),
        "--release",
        str(release),
    ]
    if database_state is not None:
        command += [
            "--database-state",
            str(database_state),
            "--database",
            str(database),
        ]
    selected = environment(state, sampler)
    selected["PSE_DIAGNOSTICS_DIR"] = str(output / "native-checkpoints")
    selected["PSE_DIAGNOSTICS"] = "on"
    selected["OTEL_LOGS_EXPORTER"] = "none"
    backend = BackendClient()
    with (
        (output / "producer-stdout.json").open("w") as stdout,
        (output / "producer-stderr.txt").open("w") as stderr,
    ):
        process = subprocess.Popen(
            command, cwd=ROOT, env=selected, stdout=stdout, stderr=stderr
        )
        try:
            deadline = time.monotonic() + TIMEOUT_SECONDS
            while not ready.is_file():
                require(
                    process.poll() is None and time.monotonic() < deadline,
                    "native_operational.producer_not_ready",
                )
                await asyncio.sleep(0.1)
            produced = mapping(json.loads(ready.read_bytes()))
            expectations = source_expectations(mapping(produced["final_source"]))
            answers = {}
            for metric, values in expectations.items():
                while True:
                    answer = await backend.query(
                        state,
                        QueryRequest(
                            signal="metrics",
                            metric=metric,
                            service_instance_id=str(produced["service_instance_id"]),
                            start_time_us=int(str(produced["final_checkpoint_us"])),
                            end_time_us=time.time_ns() // 1000 + 1,
                            limit=1000,
                        ),
                    )
                    try:
                        compare_points(
                            answer,
                            values,
                            str(produced["service_instance_id"]),
                            int(str(produced["final_checkpoint_us"])),
                        )
                        answers[metric] = answer.model_dump(mode="json")
                        break
                    except RuntimeError as error:
                        require(time.monotonic() < deadline, str(error))
                        await asyncio.sleep(0.2)
            # A never-emitted family preserves the backend's empty/unsupported/unavailable state, never a fabricated zero.
            absent = await backend.query(
                state,
                QueryRequest(
                    signal="metrics",
                    metric="pse_acceptance_absent_native_family",
                    service_instance_id=str(produced["service_instance_id"]),
                    start_time_us=int(str(produced["final_checkpoint_us"])),
                    end_time_us=time.time_ns() // 1000 + 1,
                ),
            )
            require(
                absent.status in {"empty", "unsupported", "unavailable"}
                and not absent.records,
                "native_operational.absent_family_fabricated",
            )
            return {
                "producer": produced,
                "backend": answers,
                "absent_family": absent.model_dump(mode="json"),
            }
        finally:
            release.touch()
            await backend.close()
            try:
                await asyncio.to_thread(process.wait, timeout=15)
            except subprocess.TimeoutExpired:
                process.terminate()
                await asyncio.to_thread(process.wait, timeout=5)
            require(process.returncode == 0, "native_operational.producer_failed")


async def run(
    state: Path,
    output: Path,
    sampler: str,
    database_state: Path | None,
    database: str | None,
    previous_server: Path | None,
) -> int:
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    receipt: dict[str, object] = {
        "status": "running",
        "baseline_failures": 0,
        "scope": "two real native solves, edit/reuse preparation, exact native checkpoint source totals versus actual SDK/backend points",
        "not_established": [
            "performance or cache-effectiveness improvement",
            "duration histograms reconstructed from count/sum",
            "database causal transport continuity",
            "global server work attributed to one application operation",
            "Rust exemplar support",
        ],
    }
    try:
        inspected = telemetry_service.inspect(state)
        services = mapping(inspected.get("services"))
        require(
            all(
                mapping(services.get(name)).get("ownership") == "verified"
                and mapping(services.get(name)).get("active_state") == "active"
                for name in ("collector", "openobserve")
            ),
            "native_operational.telemetry_service_not_owned",
        )
        receipt["observed"] = result = await observe(
            state, output, sampler, database_state, database
        )
        if database_state is not None:
            from scripts import surreal_server  # noqa: PLC0415

            produced = mapping(result["producer"])
            before, after = (
                mapping(produced["server_before"]),
                mapping(produced["server_after"]),
            )
            receipt["shared_server_observation"] = (
                surreal_server.compare_metric_snapshots(before, after)
            )
            receipt["unavailable_server_families"] = [
                name
                for name, family in mapping(after["families"]).items()
                if mapping(family).get("availability") != "observed"
            ]
            if previous_server is not None:
                previous = mapping(json.loads(previous_server.read_bytes()))
                require(
                    previous.get("schema") == "selected-native-metrics-v1"
                    and previous.get("identity") != after.get("identity"),
                    "native_operational.restart_witness_not_distinct",
                )
                try:
                    surreal_server.compare_metric_snapshots(previous, after)
                except surreal_server.SupervisorError:
                    receipt["restart"] = {
                        "status": "distinct_source_generations_refused_comparison",
                        "before": previous,
                        "after": after,
                    }
                else:
                    raise RuntimeError(
                        "native_operational.restart_epoch_comparison_accepted"
                    )
        receipt.update(
            status="passed",
            sampler=sampler,
            database_coverage="selected canonical query adapter and shared server"
            if database_state is not None
            else "not_run: no disposable database selected",
        )
    except Exception as error:
        receipt.update(
            status="failed",
            error=str(error)
            if isinstance(error, RuntimeError)
            else type(error).__name__,
        )
    write(output / "acceptance.json", receipt)
    print(
        json.dumps(
            {"status": receipt["status"], "receipt": str(output / "acceptance.json")}
        )
    )
    return 0 if receipt["status"] == "passed" else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--output", type=Path)
    parser.add_argument(
        "--sampler", choices=("always_off", "always_on"), default="always_off"
    )
    parser.add_argument("--database-state", type=Path)
    parser.add_argument("--database")
    parser.add_argument("--previous-server-snapshot", type=Path)
    parser.add_argument("--produce", action="store_true")
    parser.add_argument("--ready", type=Path)
    parser.add_argument("--release", type=Path)
    args = parser.parse_args()
    if (args.database_state is None) != (args.database is None):
        parser.error(
            "--database-state and --database select one already-provisioned disposable database together"
        )
    if args.previous_server_snapshot is not None and args.database_state is None:
        parser.error("--previous-server-snapshot requires a selected database service")
    if args.produce:
        if args.ready is None or args.release is None:
            parser.error("--produce requires --ready and --release")
        print(
            json.dumps(
                producer(
                    args.state,
                    args.ready,
                    args.release,
                    args.database_state,
                    args.database,
                ),
                allow_nan=False,
            )
        )
        return 0
    if args.output is None:
        parser.error("--output is required")
    return asyncio.run(
        run(
            args.state.resolve(),
            args.output.absolute(),
            args.sampler,
            args.database_state,
            args.database,
            args.previous_server_snapshot,
        )
    )


if __name__ == "__main__":
    raise SystemExit(main())

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded disposable native database telemetry journey; ordinary servers stay untouched."""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import socket
import subprocess
import sys
import time
from pathlib import Path

from scripts import surreal_server as server
from scripts import surreal_telemetry, surrealism_module
from scripts.telemetry_investigation import BackendClient, QueryRequest
from scripts.tests.surreal_fixture_check import (
    initialize_canonical,
    owned_fixture,
    query,
    record_ownership,
)


def require(value: object, reason: str) -> None:
    if not value:
        raise RuntimeError(reason)


def points(
    telemetry_state: Path, metric: str, instance: str, started_us: int
) -> dict[str, object]:
    async def read() -> dict[str, object]:
        async with BackendClient() as client:
            answer = await client.query(
                telemetry_state,
                QueryRequest(
                    signal="metrics",
                    metric=metric,
                    service_instance_id=instance,
                    start_time_us=started_us,
                    end_time_us=time.time_ns() // 1000 + 1,
                    limit=100,
                ),
            )
            return answer.model_dump(mode="json")

    return asyncio.run(read())


def wait_points(
    telemetry_state: Path, metric: str, instance: str, started_us: int
) -> dict[str, object]:
    deadline = time.monotonic() + 45
    while True:
        answer = points(telemetry_state, metric, instance, started_us)
        if answer.get("records"):
            return answer
        server.remaining(deadline)
        time.sleep(1)


def capture_workload(state: Path, output: Path, telemetry_state: Path) -> None:
    selected = server.selected_service(state, time.monotonic() + 15)
    instance = selected.identity["telemetry_service_instance_id"]
    require(isinstance(instance, str), "database.capture_process_identity_absent")
    started = time.time_ns() // 1000
    query(
        state,
        "DEFINE TABLE IF NOT EXISTS telemetry_probe SCHEMALESS; UPSERT telemetry_probe:ack CONTENT {phase:'capture'}; SELECT phase FROM telemetry_probe:ack;",
    )
    observed = wait_points(
        telemetry_state, "surrealdb_storage_surrealmx_live_keys", str(instance), started
    )
    server.write_json(
        output, {"identity": selected.identity, "metrics_during_capture": observed}
    )


def run(telemetry_state: Path, output: Path, *, component_test: bool = False) -> None:
    output.mkdir(mode=0o700, parents=True, exist_ok=False)
    fixture_root = output / "fixtures"
    fixture_root.mkdir(mode=0o700)
    os.environ["PSE_TELEMETRY_STATE"] = str(telemetry_state)
    receipt: dict[str, object] = {
        "status": "running",
        "baseline_failures": 0,
        "scope": "disposable server; shared actual Collector/OpenObserve; no ordinary scientific lifecycle effects",
    }
    try:
        with owned_fixture(fixture_root, output / "retirement") as directory:
            state = directory / "live"
            with socket.socket() as probe:
                probe.bind(("127.0.0.1", 0))
                port = probe.getsockname()[1]
            minimum_server_mib = (
                8
                * server.integer(surrealism_module.LIMITS["max_memory_bytes"])
                * server.integer(surrealism_module.LIMITS["max_pool_size"])
                // server.MIB
            )
            args = server.parser().parse_args(
                [
                    "setup",
                    "--state",
                    str(state),
                    "--port",
                    str(port),
                    "--no-resident",
                    "--interpretation",
                    server.SUBSTRATE_INTERPRETATION,
                    "--memory-mib",
                    str(minimum_server_mib + 512),
                    "--server-memory-mib",
                    str(minimum_server_mib),
                    "--native-workers",
                    "1",
                    "--native-worker-memory-mib",
                    "512",
                ]
            )
            server.setup(args)
            config = server.config_for(state)
            try:
                initialize_canonical(state)
                config = server.config_for(state)
                server.start(state, config)
                record_ownership(directory, server.unit_name(state))
                if component_test:
                    test = "python/pse/tests/test_capability_adapters.py::test_capability_project_selection_metadata_and_retained_revision"
                    root = Path(__file__).resolve().parents[2]
                    command = [
                        str(root / ".venv/bin/python"),
                        "-m",
                        "pytest",
                        "-n0",
                        test,
                        "--basetemp",
                        str(directory / "pytest"),
                    ]
                    with (output / "component.log").open("w") as log:
                        result = subprocess.run(
                            command,
                            cwd=root,
                            env={**os.environ, "PSE_SURREAL_STATE": str(state)},
                            stdout=log,
                            stderr=subprocess.STDOUT,
                            timeout=240,
                            check=False,
                        )
                    receipt.update(
                        scope="one selected Python component against an owned ready database",
                        component={
                            "command": command,
                            "exit_code": result.returncode,
                            "state": str(state),
                        },
                        status="passed" if result.returncode == 0 else "failed",
                    )
                    require(
                        result.returncode == 0, "database.selected_component_failed"
                    )
                    return
                initial = server.metric_snapshot(state, time.monotonic() + 15)
                server.write_json(output / "before.json", initial)
                server.write_json(
                    output / "ready.json",
                    {
                        "state": str(state),
                        "database": config["database"],
                        "identity": initial["identity"],
                    },
                )
                print(
                    json.dumps(
                        {
                            "phase": "database_ready",
                            "state": str(state),
                            "database": config["database"],
                        }
                    ),
                    flush=True,
                )
                selected = server.selected_service(state, time.monotonic() + 15)
                instance = str(selected.identity["telemetry_service_instance_id"])
                receipt["initial"] = initial
                started = time.time_ns() // 1000
                native_output = output / "native"
                environment = {**os.environ, "PSE_SURREAL_STATE": str(state)}
                with (output / "native.log").open("w") as log:
                    result = subprocess.run(
                        [
                            sys.executable,
                            "-m",
                            "scripts.tests.telemetry_native_operational_acceptance",
                            "--state",
                            str(telemetry_state),
                            "--output",
                            str(native_output),
                            "--sampler",
                            "always_off",
                            "--database-state",
                            str(state),
                            "--database",
                            str(config["database"]),
                        ],
                        env=environment,
                        stdout=log,
                        stderr=subprocess.STDOUT,
                        timeout=240,
                        check=False,
                    )
                require(
                    result.returncode == 0, "database.native_canonical_journey_failed"
                )
                receipt["native_receipt"] = str(native_output / "acceptance.json")
                receipt["mx_otlp"] = wait_points(
                    telemetry_state,
                    "surrealdb_storage_surrealmx_live_keys",
                    instance,
                    started,
                )
                receipt["process_otlp"] = wait_points(
                    telemetry_state, "surrealdb_process_memory", instance, started
                )
                receipt["query_otlp"] = wait_points(
                    telemetry_state, "surrealdb_query", instance, started
                )
                after_queries = server.metric_snapshot(state, time.monotonic() + 15)
                server.write_json(output / "after-queries.json", after_queries)
                families = server.object_mapping(after_queries["families"])
                for prefix in (
                    "surrealdb_process_",
                    "surrealdb_query_",
                    "surrealdb_storage_surrealmx_",
                ):
                    require(
                        any(
                            name.startswith(prefix)
                            and server.object_mapping(family).get("availability")
                            == "observed"
                            for name, family in families.items()
                        ),
                        "database.native_metric_population_absent",
                    )
                receipt["server_query_observation"] = server.compare_metric_snapshots(
                    initial, after_queries
                )
                server.stop(state, server.config_for(state))
                server.start(state, server.config_for(state))
                record_ownership(directory, server.unit_name(state))
                restarted = server.metric_snapshot(state, time.monotonic() + 15)
                server.write_json(output / "after-restart.json", restarted)
                restarted_identity = server.object_mapping(restarted["identity"])
                require(
                    restarted_identity.get("telemetry_service_instance_id") != instance,
                    "database.restart_reused_telemetry_incarnation",
                )
                receipt["restart_identity"] = restarted_identity
                receipt["restart_otlp"] = wait_points(
                    telemetry_state,
                    "surrealdb_storage_surrealmx_live_keys",
                    str(restarted_identity["telemetry_service_instance_id"]),
                    time.time_ns() // 1000,
                )
                capture = surreal_telemetry.capture_operation(
                    state,
                    output / "trace-capture",
                    [
                        sys.executable,
                        "-m",
                        "scripts.tests.telemetry_database_acceptance",
                        "--capture-workload",
                        "--server-state",
                        str(state),
                        "--telemetry-state",
                        str(telemetry_state),
                        "--output",
                        str(output / "capture-metrics.json"),
                    ],
                )
                require(
                    capture.get("workload") == 0
                    and server.integer(capture.get("exported_bytes")) > 0,
                    "database.trace_capture_or_parallel_metric_delivery_failed",
                )
                receipt["trace_capture"] = capture
                record_ownership(directory, server.unit_name(state))
                absent = points(
                    telemetry_state,
                    "pse_tel10_deliberately_absent_family",
                    str(restarted_identity["telemetry_service_instance_id"]),
                    started,
                )
                require(
                    not absent.get("records"), "database.absent_family_invented_points"
                )
                receipt["absent_family"] = absent
            finally:
                server.stop(state, server.config_for(state))
            try:
                server.metric_snapshot(state, time.monotonic() + 10)
            except server.SupervisorError:
                receipt["stopped_read"] = {
                    "status": "unavailable",
                    "zero_samples_inferred": False,
                }
            else:
                raise RuntimeError("database.stopped_server_metrics_read_succeeded")
        receipt["status"] = "passed"
    except BaseException as error:
        receipt.update(status="failed", failure=type(error).__name__, reason=str(error))
        raise
    finally:
        server.write_json(output / "receipt.json", receipt)


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--telemetry-state", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--capture-workload", action="store_true")
    parser.add_argument("--server-state", type=Path)
    parser.add_argument(
        "--component-test",
        action="store_true",
        help="Run only the selected project-selection component against a fresh owned provider",
    )
    args = parser.parse_args()
    if args.capture_workload:
        if args.server_state is None:
            parser.error("capture workload requires --server-state")
        capture_workload(args.server_state, args.output, args.telemetry_state)
    else:
        run(args.telemetry_state, args.output, component_test=args.component_test)


if __name__ == "__main__":
    main()

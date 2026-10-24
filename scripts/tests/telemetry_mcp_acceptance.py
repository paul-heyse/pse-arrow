# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit joined STDIO, real native checkpoint and owned telemetry acceptance.

Requires the already prepared linked correctness extension. Reuses the original
native scientific fixture and disposable service owner; never builds per case or
enables login startup. Failed private evidence is retained after owned cleanup.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import subprocess
import sys
import time
from pathlib import Path

from fastmcp import Client
from fastmcp.client.transports import StdioTransport

from scripts.tests.telemetry_native_acceptance import ROOT, TRACE_A, TRACE_B
from scripts.tests.telemetry_pipeline_acceptance import (
    Backend,
    Fixture,
    mapping,
    records,
    require,
    wait_for,
)
from scripts.tests.test_capability_native import source_request


async def selected(
    client: Client, name: str, arguments: dict[str, object], *, action: bool = False
) -> dict[str, object]:
    answer = await client.call_tool(
        "capabilities_invoke_action" if action else "capabilities_invoke_read",
        {"capability": name, "arguments": arguments},
        meta={"traceparent": "00-" + TRACE_B + "-2468abcdef123456-01"},
    )
    content = mapping(answer.structured_content)
    if set(content) == {"result"}:
        content = mapping(content["result"])
    return mapping(content["payload"])


async def consume(
    fixture: Fixture, ready: dict[str, object], environment: dict[str, str]
) -> dict[str, object]:
    transport = StdioTransport(
        command=str(ROOT / ".venv/bin/python"),
        args=["-m", "scripts.capabilities"],
        cwd=str(ROOT),
        env=environment,
        keep_alive=False,
    )
    async with Client(transport, timeout=60) as client:
        tools = {tool.name for tool in await client.list_tools(cache_mode="refresh")}
        require("capabilities_invoke_read" in tools, "mcp.bootstrap_discovery_missing")
        image = await selected(
            client,
            "native_diagnostic_checkpoint",
            {"checkpoint": ready["checkpoint"], "expected": None},
        )
        checkpoint = mapping(image["document"])
        locator = mapping(checkpoint["locator"])
        require(locator["pid"] == ready["pid"], "mcp.native_target_substituted")
        measurements = mapping(checkpoint["runtime_metrics"])
        require(
            mapping(measurements["locator"]) == locator,
            "mcp.measurement_source_changed",
        )
        executors = records(measurements["executors"])
        require(
            any(
                item["role"] == "python" and item["workers"] == 2 for item in executors
            ),
            "mcp.actual_executor_not_observed",
        )
        classes = records(measurements["task_classes"])
        require(
            any(
                item["class"] == "workflow" and int(str(item["poll_count"])) > 0
                for item in classes
            ),
            "mcp.actual_future_polls_missing",
        )
        processes = await selected(
            client,
            "native_diagnostic_processes",
            {
                "directory": str(Path(str(ready["checkpoint"])).parent),
                "request": {"document_version": 1, "after": None, "limit": 16},
            },
        )
        health = await selected(
            client, "telemetry_health", {"state": str(fixture.state)}
        )
        require(Fixture.ready(health), "mcp.owned_pipeline_not_ready")
        config = await selected(
            client, "telemetry_configuration", {"state": str(fixture.state)}
        )
        require(config["disabled"] is False, "mcp.configuration_not_selected")
        admitted = await selected(
            client, "native_source_admit", source_request(), action=True
        )
        require(admitted["outcome"] == "admitted", "mcp.real_source_not_admitted")
        require(
            mapping(admitted["revision"])["reopenable"] is False,
            "mcp.child_lifetime_changed",
        )
        return {
            "checkpoint": str(ready["checkpoint"]),
            "locator": locator,
            "runtime_metrics": measurements,
            "process_discovery": processes,
            "native_source_outcome": admitted["outcome"],
            "service_configuration_id": fixture.configuration_id,
        }


def run(output: Path) -> int:
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    fixture = Fixture(output / "service-state", 90)
    ready, release = output / "ready.json", output / "release"
    receipt: dict[str, object] = {
        "baseline": 0,
        "status": "running",
        "not_established": [
            "database/Surrealism continuity",
            "exported cancellation parent chain",
            "arbitrary crash durability",
            "performance improvement",
        ],
    }
    producer = None
    passed = False
    try:
        fixture.configure()
        fixture.start()
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(("OTEL_", "PSE_TELEMETRY_"))
        }
        environment.update(
            {
                "PSE_TELEMETRY_STATE": str(fixture.state),
                "PSE_DIAGNOSTICS_DIR": str(output / "native-checkpoints"),
                "OTEL_EXPORTER_OTLP_TIMEOUT": "1",
                "OTEL_BSP_SCHEDULE_DELAY": "100",
                "OTEL_METRIC_EXPORT_INTERVAL": "250",
                "OTEL_BSP_EXPORT_TIMEOUT": "2000",
            }
        )
        started_us = time.time_ns() // 1000
        with (
            (output / "producer-stdout.json").open("w") as stdout,
            (output / "producer-stderr.txt").open("w") as stderr,
        ):
            producer = subprocess.Popen(
                [
                    sys.executable,
                    "-m",
                    "scripts.tests.telemetry_native_acceptance",
                    "--produce",
                    "--ready",
                    str(ready),
                    "--release",
                    str(release),
                ],
                cwd=ROOT,
                env=environment,
                stdout=stdout,
                stderr=stderr,
            )
            wait_for(
                lambda: ready.is_file() or producer.poll() is not None,
                bool,
                90,
                "mcp.native_producer_not_ready",
            )
            require(ready.is_file(), "mcp.native_producer_exited_early")
            native = mapping(json.loads(ready.read_bytes()))
            require(native["pid"] == producer.pid, "mcp.native_ready_identity_changed")
            receipt["mcp"] = asyncio.run(consume(fixture, native, environment))
            backend = Backend(fixture)
            points = wait_for(
                lambda: backend.rows(
                    "metrics",
                    "pse_runtime_executor_workers",
                    "SELECT * FROM pse_runtime_executor_workers",
                    started_us,
                    time.time_ns() // 1000,
                ),
                bool,
                30,
                "mcp.native_metric_not_query_visible",
            )
            require(
                any(
                    point.get("executor_role") == "python" and point.get("value") == 2
                    for point in points
                ),
                "mcp.native_metric_population_changed",
            )
            receipt["metric_visibility"] = points
            release.touch()
            require(producer.wait(timeout=30) == 0, "mcp.native_outcome_changed")
        produced = mapping(json.loads((output / "producer-stdout.json").read_bytes()))
        require(produced["native_result"] == "usable", "mcp.scientific_result_changed")
        receipt["producer"] = produced
        from scripts import telemetry_query  # noqa: PLC0415 -- selected bounded owner.

        for trace_id in (TRACE_A, TRACE_B):
            trace = wait_for(
                lambda trace_id=trace_id: telemetry_query.lookup_trace(
                    fixture.state,
                    trace_id,
                    start_time_us=started_us,
                    end_time_us=time.time_ns() // 1000,
                ),
                lambda answer: bool(answer.get("records")),
                30,
                "mcp.trace_not_visible",
            )
            spans = records(trace["records"])
            require(
                any(
                    row.get("operation_name") == "pse.native.operation" for row in spans
                ),
                "mcp.native_trace_edge_missing",
            )
            expected_parent = (
                "1234567890abcdef" if trace_id == TRACE_A else "2468abcdef123456"
            )
            require(
                any(row.get("parent_span_id") == expected_parent for row in spans),
                "mcp.independent_parent_missing",
            )
            receipt["trace_" + trace_id] = trace
        fixture.unit_action("collector", "stop")
        receipt["collector_down_fallback"] = asyncio.run(
            consume_fallback(fixture, environment)
        )
        passed = True
        receipt["status"] = "passed"
    except Exception as error:
        receipt.update(
            status="failed",
            error=str(error)
            if isinstance(error, RuntimeError)
            else type(error).__name__,
        )
    finally:
        release.touch()
        if producer is not None and producer.poll() is None:
            try:
                producer.wait(timeout=10)
            except subprocess.TimeoutExpired:
                producer.terminate()
                producer.wait(timeout=10)
        try:
            receipt["cleanup"] = fixture.cleanup(remove_state=passed)
        except Exception as error:
            receipt.update(status="failed", cleanup_error=type(error).__name__)
        (output / "acceptance.json").write_text(
            json.dumps(receipt, indent=2, allow_nan=False) + "\n"
        )
    print(
        json.dumps(
            {"status": receipt["status"], "receipt": str(output / "acceptance.json")}
        )
    )
    return 0 if receipt["status"] == "passed" else 1


async def consume_fallback(
    fixture: Fixture, environment: dict[str, str]
) -> dict[str, object]:
    async with Client(
        StdioTransport(
            command=str(ROOT / ".venv/bin/python"),
            args=["-m", "scripts.capabilities"],
            cwd=str(ROOT),
            env=environment,
            keep_alive=False,
        ),
        timeout=30,
    ) as client:
        health = await selected(
            client, "telemetry_health", {"state": str(fixture.state)}
        )
        require(
            mapping(mapping(health["pipeline"])["collector"])["status"]
            == "unavailable",
            "mcp.stopped_collector_hidden",
        )
        config = await selected(
            client, "telemetry_configuration", {"state": str(fixture.state)}
        )
        require(
            config["configuration_id"] == fixture.configuration_id,
            "mcp.local_configuration_lost",
        )
        return {"collector": "unavailable", "configuration": "readable"}


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    return run(parser.parse_args().output.absolute())


if __name__ == "__main__":
    raise SystemExit(main())

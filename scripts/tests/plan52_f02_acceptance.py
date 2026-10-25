# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit F02 canonical request -> native SDK -> server -> module acceptance.

Requires a current linked correctness extension, canonical-tools executable and
running selected telemetry pipeline. This driver provisions only its disposable
database, uses the existing diagnostic_action owned launch for the real producer,
and restores/drains the ordinary finite-capture owner. It never builds or installs.
Streaming error/cancellation and same-generation reconnect qualification are
separate Rust controls; a cancelled study is not a cancelled response stream.
"""

from __future__ import annotations

import argparse
import asyncio
import json
import os
import socket
import sys
import tempfile
import threading
import time
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import TYPE_CHECKING
from uuid import uuid4

from scripts import surreal_server as server
from scripts import surreal_telemetry, surrealism_module
from scripts.telemetry_investigation import BackendClient, QueryRequest
from scripts.tests.surreal_fixture_check import (
    initialize_canonical,
    owned_fixture,
    record_ownership,
)
from scripts.tests.telemetry_operational_acceptance import environment
from scripts.tests.telemetry_pipeline_acceptance import mapping, records, require

if TYPE_CHECKING:
    from collections.abc import Mapping

ROOT = Path(__file__).resolve().parents[2]
ANCESTOR = "d3c4b5a697887766"


def producer(state: Path, database: str, output: Path, expected_trace: str) -> int:
    """Two native sibling requests share one real SDK engine, with actual parents."""
    from opentelemetry import context, trace  # noqa: PLC0415

    import pse  # noqa: PLC0415
    from pse import codec  # noqa: PLC0415
    from pse.contracts.documents import (  # noqa: PLC0415
        ProjectMetadataRequest,
        ProjectReferenceBranch,
        ProjectSelectRequest,
    )
    from pse.contracts.enums import ProjectMetadataKind  # noqa: PLC0415
    from pse.tests.study_fixtures import point  # noqa: PLC0415
    from pse.tests.study_fixtures import request as study_request  # noqa: PLC0415
    from pse.tests.test_studies import _package  # noqa: PLC0415
    from scripts import telemetry_process  # noqa: PLC0415
    from scripts.capabilities.native_worker import native_report, scientific_error  # noqa: PLC0415

    image: dict[str, object] = {"status": "running", "baseline_failures": 0}
    try:
        require(
            telemetry_process.current_carrier().get("traceparent", "")[3:35]
            == expected_trace,
            "f02.literal_launch_lost_authentic_carrier",
        )
        require(
            mapping(telemetry_process.local_status().get("native")).get("status")
            in {"initialized", "already_initialized"},
            "f02.native_provider_unavailable",
        )
        with tempfile.TemporaryDirectory(prefix="pse-f02-native-") as spill:
            settings = pse.EngineSettings(
                memory_limit_bytes=16 << 30,
                threads=16,
                spill_dir=spill,
                max_spill_bytes=1 << 30,
                batch_size=32,
            )
            runtime = pse.Runtime(
                settings, substrate=str(state), database=database, ephemeral=False
            )
            try:
                package, case = _package(
                    runtime,
                    source='package f02 { def Root { param x:Scalar=2; annotation report x("constant"); } }',
                )
                inherited = context.get_current()
                barrier = threading.Barrier(2)

                def sibling(key: int) -> dict[str, object]:
                    token = context.attach(inherited)
                    try:
                        with trace.get_tracer(__name__).start_as_current_span(
                            "pse.f02.sibling", attributes={"fixture.member": key}
                        ) as span:
                            barrier.wait(timeout=30)
                            local = package.admit_study(
                                study_request(point(case, pse.SolveSettings(), key))
                            )
                            definition = runtime.record_study(local)
                            # Activation calls the actual installed study module.
                            # No numerical solve is needed to establish that edge.
                            handle = runtime.start_study(definition)
                            before = handle.status()
                            require(len(before.points) == 1, "f02.study_population_changed")
                            cancelled = handle.cancel()
                            after = handle.status()
                            require(after.cancelled, "f02.native_cancel_not_recorded")
                            return {
                                "span_id": f"{span.get_span_context().span_id:016x}",
                                "trace_id": f"{span.get_span_context().trace_id:032x}",
                                "study": handle.study_id.to_hex(),
                                "before": codec.document_builtins(before),
                                "cancel": codec.document_builtins(cancelled),
                                "after": codec.document_builtins(after),
                            }
                    finally:
                        context.detach(token)

                with ThreadPoolExecutor(max_workers=2) as workers:
                    futures = [workers.submit(sibling, key) for key in (7, 11)]
                    siblings = [future.result(timeout=150) for future in futures]
                require(
                    len({item["study"] for item in siblings}) == 2
                    and len({item["span_id"] for item in siblings}) == 2,
                    "f02.independent_siblings_merged",
                )
                image["siblings"] = siblings
                metadata = ProjectMetadataRequest(kind=ProjectMetadataKind.PROJECTS, owner="", cursor=None)
                token = context.attach(context.Context())
                try:
                    require(not telemetry_process.current_carrier(), "f02.stale_context_before_missing_call")
                    runtime.project_metadata(metadata)
                    image["missing_context"] = {
                        "native_outcome": "observed",
                        "external_parent": "absent",
                        "continuity": "explicit_gap; local instrumentation may create a new root",
                    }
                finally:
                    context.detach(token)
                try:
                    physical = runtime.physical_from_documents(
                        {
                            str(path.relative_to(ROOT / "tests/fixtures/packages/physical-primitives")): path.read_bytes()
                            for path in (ROOT / "tests/fixtures/packages/physical-primitives").rglob("*")
                            if path.is_file()
                        }
                    )
                    runtime.project_select(
                        ProjectSelectRequest(reference=ProjectReferenceBranch(key="f02-deliberately-absent")),
                        physical,
                    )
                except Exception as error:
                    require(native_report(error) is not None, "f02.refusal_has_no_native_diagnostic")
                    image["database_refusal"] = scientific_error(error, operation="project_select")
                else:
                    raise RuntimeError("f02.expected_native_refusal_missing")
                runtime.project_metadata(metadata)
                require(telemetry_process.current_carrier().get("traceparent", "")[3:35] == expected_trace, "f02.missing_context_leaked")
                image["after_refusal"] = "native metadata read succeeded"
            finally:
                runtime.close()
        image["status"] = "passed"
        return 0
    except BaseException as error:
        image.update(status="failed", failure=type(error).__name__, reason=str(error))
        raise
    finally:
        server.write_json(output, image)


async def canonical_request(
    state: Path,
    database: str,
    output: Path,
    trace_id: str,
    selected_environment: Mapping[str, str],
) -> dict[str, object]:
    from fastmcp import Client  # noqa: PLC0415
    from fastmcp.client.transports import StdioTransport  # noqa: PLC0415

    transport = StdioTransport(
        command=str(ROOT / ".venv/bin/python"),
        args=["-m", "scripts.capabilities"],
        cwd=str(ROOT),
        env=dict(selected_environment),
        keep_alive=False,
    )
    async with Client(transport, timeout=240) as client:
        await client.call_tool(
            "capabilities_describe",
            {"capability": "diagnostic_action", "include_schema": True},
        )
        await client.call_tool("capabilities_select", {"capabilities": ["diagnostic_action"]})
        answer = await client.call_tool(
            "capabilities_invoke_action",
            {
                "capability": "diagnostic_action",
                "arguments": {
                    "request": {
                        "action": "gdb-launch",
                        "output": str(output / "owned-action"),
                        "seconds": 180.0,
                        "output_bytes": 1 << 20,
                        "command": [
                            str(ROOT / ".venv/bin/python"),
                            "-m", "scripts.tests.plan52_f02_acceptance",
                            "--producer", "--server-state", str(state),
                            "--database", database, "--output", str(output / "producer.json"),
                            "--expected-trace", trace_id,
                        ],
                    }
                },
            },
            meta={"traceparent": f"00-{trace_id}-{ANCESTOR}-01", "tracestate": "f02=independent"},
        )
        content = mapping(answer.structured_content)
        if set(content) == {"result"}:
            content = mapping(content["result"])
        return content


def compact(row: dict[str, object]) -> dict[str, object]:
    """Retain only span identity/parentage; no SQL, payload or arbitrary attributes."""
    return {
        "trace_id": row.get("trace_id"),
        "span_id": row.get("span_id"),
        "parent_span_id": row.get("parent_span_id"),
        "name": row.get("operation_name", row.get("name")),
        "service_instance_id": row.get("service_instance_id"),
    }


async def trace_rows(telemetry_state: Path, trace_id: str, started: int) -> list[dict[str, object]]:
    # A real canonical operation emits many small SDK/server spans. Select
    # narrower original time windows before hydration when a row bound is hit;
    # never reconstruct a missing parent or claim a frozen/complete population.
    intervals = [(started, time.time_ns() // 1000 + 1)]
    indexed: dict[str, dict[str, object]] = {}
    reads = 0
    async with BackendClient() as client:
        while intervals:
            reads += 1
            require(reads <= 64, "f02.bounded_trace_population_exceeded")
            lower, upper = intervals.pop()
            answer = await client.query(
                telemetry_state,
                QueryRequest(
                    signal="traces", trace_id=trace_id, start_time_us=lower,
                    end_time_us=upper, limit=200,
                ),
            )
            require(answer.status in {"available", "empty", "partial"}, "f02.continuous_trace_unavailable")
            for row in answer.records:
                selected = compact(dict(row))
                indexed[str(selected["span_id"])] = selected
            if len(answer.records) == 200:
                require(upper - lower > 1, "f02.same_instant_population_exceeded")
                middle = lower + (upper - lower) // 2
                intervals.extend(((lower, middle), (middle, upper)))
        return list(indexed.values())


def parent_chain(rows: list[dict[str, object]], leaf: str) -> list[dict[str, object]]:
    by_id = {str(row["span_id"]): row for row in rows}
    result = []
    seen: set[str] = set()
    while leaf in by_id:
        require(leaf not in seen, "f02.parent_cycle")
        seen.add(leaf)
        row = by_id[leaf]
        result.append(row)
        leaf = str(row["parent_span_id"])
    return result


def chains(rows: list[dict[str, object]], siblings: list[dict[str, object]]) -> list[list[dict[str, object]]]:
    accepted = []
    for sibling in siblings:
        sibling_id = str(sibling["span_id"])
        candidates = []
        for module in rows:
            if module["name"] not in {"invoke", "invoke_with_timeout"}:
                continue
            chain = parent_chain(rows, str(module["span_id"]))
            names = {row["name"] for row in chain}
            if sibling_id not in {row["span_id"] for row in chain}:
                continue
            if not {"executor", "surrealdb.sdk.request", "pse.native.operation", "pse.helper"} <= names:
                continue
            if chain[-1]["parent_span_id"] != ANCESTOR:
                continue
            # The receiving service has an actual direct SDK parent, not a
            # matching request label or reconstructed time-window association.
            boundaries = [
                (child, parent)
                for child, parent in zip(chain, chain[1:], strict=False)
                if parent["name"] == "surrealdb.sdk.request"
                and child["name"] != "surrealdb.sdk.request"
                and child["service_instance_id"] != parent["service_instance_id"]
            ]
            if boundaries:
                candidates.append(chain)
        require(candidates, "f02.complete_module_parent_chain_not_observed")
        selected = min(candidates, key=len)
        require(
            len({row["service_instance_id"] for row in selected if row["service_instance_id"]}) >= 3,
            "f02.process_incarnations_not_observed",
        )
        require(
            sum(row["span_id"] == item["span_id"] for row in selected for item in siblings) == 1,
            "f02.sibling_parent_leak",
        )
        accepted.append(selected)
    return accepted


def observe(telemetry_state: Path, trace_id: str, started: int, siblings: list[dict[str, object]], extra: list[dict[str, object]] | None = None) -> dict[str, object]:
    deadline = time.monotonic() + 60
    while True:
        rows = asyncio.run(trace_rows(telemetry_state, trace_id, started))
        if extra:
            indexed = {str(row["span_id"]): row for row in rows}
            for row in extra:
                key = str(row["span_id"])
                # Finite records omit resources: retain the continuous record's
                # process identity when the exact same source span is present.
                if key not in indexed:
                    indexed[key] = row
            rows = list(indexed.values())
        try:
            selected = chains(rows, siblings)
            return {"trace_id": trace_id, "complete": False, "rows": rows, "parent_chains": selected}
        except RuntimeError:
            require(time.monotonic() < deadline, "f02.complete_module_parent_chain_not_observed")
            time.sleep(0.5)


def journey(telemetry_state: Path, state: Path, database: str, output: Path) -> None:
    selected_environment = environment(telemetry_state)
    selected_environment["PSE_SURREAL_STATE"] = str(state)
    receipt: dict[str, object] = {"status": "running", "baseline_failures": 0}
    try:
        continuous = output / "continuous"
        continuous.mkdir(mode=0o700)
        trace_id = uuid4().hex
        started = time.time_ns() // 1000
        receipt["canonical"] = asyncio.run(canonical_request(state, database, continuous, trace_id, selected_environment))
        produced = server.read_json(continuous / "producer.json")
        require(produced.get("status") == "passed", "f02.native_producer_failed")
        receipt["continuous"] = observe(telemetry_state, trace_id, started, records(produced["siblings"]))
        finite = output / "finite"
        finite.mkdir(mode=0o700)
        finite_id = uuid4().hex
        finite_started = time.time_ns() // 1000
        # The original finite owner provides data bounds, independently reported
        # workload/restoration and physical drain. It exports only server spans.
        original = os.environ.copy()
        os.environ.clear()
        os.environ.update(selected_environment)
        try:
            capture = surreal_telemetry.capture_operation(
                state, output / "capture",
                [sys.executable, "-m", "scripts.tests.plan52_f02_acceptance",
                 "--canonical", "--telemetry-state", str(telemetry_state),
                 "--server-state", str(state), "--database", database,
                 "--output", str(finite), "--expected-trace", finite_id],
            )
        finally:
            os.environ.clear()
            os.environ.update(original)
        receipt["capture"] = capture
        require(capture.get("workload") == 0, "f02.finite_workload_failed")
        require(mapping(capture.get("restoration")).get("phase") == "restored", "f02.finite_restoration_missing")
        require(not capture.get("capture_errors") and capture.get("omitted_spans") == 0, "f02.finite_capture_incomplete")
        captured = [compact(row) for row in records(capture["spans"]) if row.get("trace_id") == finite_id]
        instance = mapping(mapping(capture["activation"])["after"])["telemetry_service_instance_id"]
        require(isinstance(instance, str), "f02.finite_server_incarnation_missing")
        for row in captured:
            row["service_instance_id"] = instance
        produced = server.read_json(finite / "producer.json")
        require(produced.get("status") == "passed", "f02.finite_native_producer_failed")
        receipt["finite"] = observe(telemetry_state, finite_id, finite_started, records(produced["siblings"]), captured)
        receipt.update(
            status="passed",
            controls="Actual HTTP/2 stream error/cancellation and dispatch uncertainty are verified separately by grpc_lifecycle::tests; same-generation reconnect requires its native integration controls.",
            scientific_outcomes="Original native study cancellation and missing-branch refusal are retained in each producer.json; tracing does not settle them.",
        )
    except BaseException as error:
        receipt.update(status="failed", failure=type(error).__name__, reason=str(error))
        raise
    finally:
        capture_path = output / "capture/capture.json"
        if "capture" not in receipt and capture_path.is_file():
            receipt["capture"] = server.read_json(capture_path)
        server.write_json(output / "acceptance.json", receipt)


def run(telemetry_state: Path, output: Path) -> None:
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    fixtures = output / "fixtures"
    fixtures.mkdir(mode=0o700)
    with owned_fixture(fixtures, output / "retirement") as directory:
        state = directory / "live"
        with socket.socket() as listener:
            listener.bind(("127.0.0.1", 0))
            port = listener.getsockname()[1]
        minimum = 8 * server.integer(surrealism_module.LIMITS["max_memory_bytes"]) * server.integer(surrealism_module.LIMITS["max_pool_size"]) // server.MIB
        args = server.parser().parse_args([
            "setup", "--state", str(state), "--port", str(port), "--no-resident",
            "--interpretation", server.SUBSTRATE_INTERPRETATION,
            "--memory-mib", str(minimum + 512), "--server-memory-mib", str(minimum),
            "--native-workers", "1", "--native-worker-memory-mib", "512",
        ])
        server.setup(args)
        try:
            initialize_canonical(state)
            server.start(state, server.config_for(state))
            record_ownership(directory, server.unit_name(state))
            journey(telemetry_state, state, str(server.config_for(state)["database"]), output)
            record_ownership(directory, server.unit_name(state))
        finally:
            server.stop(state, server.config_for(state))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--telemetry-state", type=Path)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--server-state", type=Path)
    parser.add_argument("--database")
    parser.add_argument("--expected-trace")
    parser.add_argument("--producer", action="store_true")
    parser.add_argument("--canonical", action="store_true")
    args = parser.parse_args()
    if args.producer or args.canonical:
        if args.server_state is None or args.database is None or args.expected_trace is None:
            parser.error("fixed child requires server-state, database and expected-trace")
        if args.producer:
            from scripts.telemetry_process import run_helper  # noqa: PLC0415

            return run_helper("pse-f02-producer", lambda: producer(args.server_state, args.database, args.output, args.expected_trace), native=True)
        if args.telemetry_state is None:
            parser.error("canonical request requires telemetry-state")
        answer = asyncio.run(canonical_request(args.server_state, args.database, args.output, args.expected_trace, environment(args.telemetry_state)))
        server.write_json(args.output / "canonical.json", answer)
        return 0
    if args.telemetry_state is None:
        parser.error("acceptance requires an already running telemetry-state")
    if args.server_state is not None:
        if args.database is None:
            parser.error("selected disposable server requires database")
        args.output.mkdir(parents=True, mode=0o700, exist_ok=False)
        journey(args.telemetry_state, args.server_state, args.database, args.output)
    else:
        original = os.environ.copy()
        selected_environment = environment(args.telemetry_state)
        os.environ.clear()
        os.environ.update(selected_environment)
        try:
            run(args.telemetry_state, args.output)
        finally:
            os.environ.clear()
            os.environ.update(original)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

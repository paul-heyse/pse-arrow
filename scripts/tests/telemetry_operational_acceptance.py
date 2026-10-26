# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Explicit finite SDK -> Collector -> OpenObserve -> canonical STDIO acceptance.

Requires an already running selected service; never installs, starts, interrupts or
stops it. Fixture stimuli exercise the production host instruments and Views, not
build performance. Run only at the coordinator's qualification boundary. Original
SDK point clocks, process incarnations and bounded typed answers stay in the private
output directory. This establishes neither native metric fidelity nor exact
population inference from sampled traces, rate estimates or exemplars.
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
from pathlib import Path
from typing import TYPE_CHECKING, cast
from unittest.mock import patch

from opentelemetry.exporter.otlp.proto.http.metric_exporter import OTLPMetricExporter
from opentelemetry.exporter.otlp.proto.http.trace_exporter import OTLPSpanExporter

from scripts import operation_lifetime
from scripts import telemetry_service
from scripts.telemetry_investigation import BackendClient, QueryAnswer, QueryRequest
from scripts.tests.telemetry_pipeline_acceptance import mapping, records, require

if TYPE_CHECKING:
    from collections.abc import Sequence

    from opentelemetry.sdk.metrics.export import MetricExportResult, MetricsData
    from opentelemetry.sdk.trace import ReadableSpan
    from opentelemetry.sdk.trace.export import SpanExportResult

ROOT = Path(__file__).resolve().parents[2]
STIMULI = (0.001, 0.01, 0.0101)
# Literal independent oracle: never derive these from the production declarations
# or the backend adapter's normalization and aggregation code.
EXPECTED_BOUNDS = (0.001, 0.01, 0.1, 1.0, 10.0, 60.0, 300.0, 1800.0)
EXPECTED_SDK_BUCKETS = (1, 1, 1, 0, 0, 0, 0, 0, 0)
EXPECTED_BACKEND_BUCKETS = {
    "0.001": 1,
    "0.01": 2,
    "0.1": 3,
    "1": 3,
    "10": 3,
    "60": 3,
    "300": 3,
    "1800": 3,
    "inf": 3,
}
STREAMS = (
    "pse_build_records",
    "pse_build_duration_count",
    "pse_build_duration_sum",
    "pse_build_duration_bucket",
)
TIMEOUT_SECONDS = 30.0


def sequence(value: object) -> list[object]:
    require(isinstance(value, list), "operational.array_expected")
    return cast("list[object]", value)


def write(path: Path, value: dict[str, object]) -> None:
    temporary = path.with_suffix(".pending")
    temporary.write_text(json.dumps(value, indent=2, allow_nan=False) + "\n")
    temporary.chmod(0o600)
    temporary.replace(path)


def producer(
    state: Path, sampler: str, ready: Path, release: Path
) -> dict[str, object]:
    """Keep the SDK alive through observation; all network work is outside science."""
    from opentelemetry import trace  # noqa: PLC0415
    from opentelemetry.sdk.metrics import MeterProvider  # noqa: PLC0415
    from opentelemetry.sdk.trace import TracerProvider  # noqa: PLC0415

    from scripts import (  # noqa: PLC0415
        native_operation,
        telemetry_metrics,
        telemetry_process,
    )

    require(sampler in {"always_on", "always_off"}, "operational.sampler_invalid")
    require(
        os.environ.get("PSE_TELEMETRY_STATE") == str(state), "operational.state_changed"
    )
    exports: list[dict[str, object]] = []
    exported_spans: list[str] = []
    lock = threading.Lock()
    metric_export, span_export = OTLPMetricExporter.export, OTLPSpanExporter.export

    def observed_metrics(
        exporter: OTLPMetricExporter,
        data: MetricsData,
        timeout_millis: float = 10000,
        **kwargs: object,
    ) -> MetricExportResult:
        selected = []
        for resource in data.resource_metrics:
            for scope in resource.scope_metrics:
                for metric in scope.metrics:
                    if metric.name in {"pse.build.records", "pse.build.duration"}:
                        selected.append(mapping(json.loads(metric.to_json())))
        with lock:
            exports.extend(selected)
        return metric_export(exporter, data, timeout_millis=timeout_millis, **kwargs)

    def observed_spans(
        exporter: OTLPSpanExporter, spans: Sequence[ReadableSpan]
    ) -> SpanExportResult:
        with lock:
            exported_spans.extend(
                f"{span.context.span_id:016x}"
                for span in spans
                if span.context is not None
            )
        return span_export(exporter, spans)

    with (
        patch.object(OTLPMetricExporter, "export", observed_metrics),
        patch.object(OTLPSpanExporter, "export", observed_spans),
        telemetry_process.bootstrap("pse-operational-acceptance") as lifetime,
    ):
        require(lifetime.status == "initialized", "operational.sdk_not_initialized")
        settings = lifetime.settings
        if settings is None:
            raise RuntimeError("operational.settings_absent")
        host = telemetry_metrics.host_metrics()
        spans = []
        for index, value in enumerate(STIMULI):
            with trace.get_tracer("pse.operational.acceptance").start_as_current_span(
                "fixture.operational"
            ) as span:
                context = span.get_span_context()
                spans.append(
                    {
                        "trace_id": f"{context.trace_id:032x}",
                        "span_id": f"{context.span_id:016x}",
                        "sampled": context.trace_flags.sampled,
                    }
                )
                attributes = {"fixture.invocation_id": f"private-{index}"}
                host.build_records.add(1, {"record": "compiler-message", **attributes})
                host.build_duration.record(
                    value, {"status": "passed", "failure_class": "none", **attributes}
                )
        for provider in lifetime.providers:
            if isinstance(provider, (MeterProvider, TracerProvider)):
                require(
                    provider.force_flush(timeout_millis=5000),
                    "operational.sdk_flush_failed",
                )

        def image() -> dict[str, object]:
            with lock:
                return {
                    "sampler": sampler,
                    "pid": os.getpid(),
                    "process_start": operation_lifetime.start_identity(os.getpid()),
                    "service_instance_id": settings.wire.resource[
                        "service.instance.id"
                    ],
                    "configuration_id": settings.configuration_id,
                    "spans": list(spans),
                    "exported_span_ids": list(exported_spans),
                    "metric_exports": list(exports),
                }

        initial = image()
        validate_sdk(initial)
        write(ready, initial)
        deadline = time.monotonic() + TIMEOUT_SECONDS
        while not release.is_file():
            require(time.monotonic() < deadline, "operational.producer_release_timeout")
            time.sleep(0.2)
        lifetime.shutdown()
        final = image()
        validate_sdk(final)
        return final


def validate_sdk(produced: dict[str, object]) -> dict[str, int]:
    """Validate actual exporter input independently of the query adapter."""
    spans = records(produced["spans"])
    expected_spans = 3 if produced["sampler"] == "always_on" else 0
    require(len(spans) == 3, "operational.sdk_span_stimuli_changed")
    require(
        sum(bool(span["sampled"]) for span in spans) == expected_spans,
        "operational.sdk_sampling_changed",
    )
    require(
        len(sequence(produced["exported_span_ids"])) == expected_spans,
        "operational.sdk_exported_span_count_changed",
    )
    starts: dict[str, int] = {}
    for metric in records(produced["metric_exports"]):
        data = mapping(metric["data"])
        require(
            data["aggregation_temporality"] == 2, "operational.sdk_temporality_changed"
        )
        points = records(data["data_points"])
        require(len(points) == 1, "operational.sdk_unbounded_series")
        point = points[0]
        name = str(metric["name"])
        start = int(str(point["start_time_unix_nano"]))
        require(
            start > 0 and int(str(point["time_unix_nano"])) >= start,
            "operational.sdk_clock_invalid",
        )
        require(
            name not in starts or starts[name] == start,
            "operational.sdk_start_changed_without_reset",
        )
        starts[name] = start
        if name == "pse.build.records":
            require(
                point["value"] == 3
                and point["attributes"] == {"record": "compiler-message"},
                "operational.sdk_counter_population_changed",
            )
        elif name == "pse.build.duration":
            require(
                point["count"] == 3
                and math.isclose(
                    float(str(point["sum"])), 0.0211, rel_tol=0, abs_tol=1e-12
                ),
                "operational.sdk_histogram_totals_changed",
            )
            require(
                tuple(sequence(point["explicit_bounds"])) == EXPECTED_BOUNDS
                and tuple(sequence(point["bucket_counts"])) == EXPECTED_SDK_BUCKETS,
                "operational.sdk_histogram_buckets_changed",
            )
            require(
                point["attributes"] == {"status": "passed", "failure_class": "none"},
                "operational.sdk_histogram_dimensions_changed",
            )
    require(
        set(starts) == {"pse.build.records", "pse.build.duration"},
        "operational.sdk_metrics_absent",
    )
    return starts


def validate_points(
    answer: QueryAnswer,
    produced: dict[str, object],
    stream: str,
    *,
    exact_timestamps: bool = False,
) -> None:
    starts = validate_sdk(produced)
    require(
        answer.status == "available" and answer.records and answer.source is not None,
        "operational.metric_not_available",
    )
    source = answer.source
    require(
        source is not None
        and source.stream == stream
        and source.configuration_id == produced["configuration_id"],
        "operational.metric_source_changed",
    )
    require(
        answer.meaning == "observed_points" and not answer.complete,
        "operational.metric_claim_changed",
    )
    instrument = (
        "pse.build.records" if stream == "pse_build_records" else "pse.build.duration"
    )
    export_times = {
        int(str(point["time_unix_nano"])) // 1000
        for metric in records(produced["metric_exports"])
        if metric["name"] == instrument
        for point in records(mapping(metric["data"])["data_points"])
    }
    for row in answer.records:
        require(
            row.get("service_instance_id") == produced["service_instance_id"],
            "operational.metric_incarnation_changed",
        )
        require(
            int(str(row.get("start_time"))) == starts[instrument],
            "operational.original_start_time_changed",
        )
        require(
            row.get("aggregation_temporality") == "AGGREGATION_TEMPORALITY_CUMULATIVE",
            "operational.backend_temporality_changed",
        )
        require(
            "fixture_invocation_id" not in row and "fixture.invocation_id" not in row,
            "operational.filtered_id_became_dimension",
        )
        timestamp = int(str(row.get("_timestamp")))
        require(
            timestamp >= min(export_times)
            and source is not None
            and timestamp <= source.retrieved_at_ns // 1000,
            "operational.backend_source_clock_changed",
        )
        if exact_timestamps:
            require(
                timestamp in export_times,
                "operational.original_point_timestamp_changed",
            )
        if stream == "pse_build_records":
            require(
                row.get("value") == 3 and row.get("record") == "compiler-message",
                "operational.backend_counter_changed",
            )
        elif stream == "pse_build_duration_count":
            require(
                row.get("value") == 3, "operational.backend_histogram_count_changed"
            )
        elif stream == "pse_build_duration_sum":
            require(
                math.isclose(
                    float(str(row.get("value"))), 0.0211, rel_tol=0, abs_tol=1e-12
                ),
                "operational.backend_histogram_sum_changed",
            )
        elif stream == "pse_build_duration_bucket":
            require(
                str(row.get("le")) in EXPECTED_BACKEND_BUCKETS
                and row.get("value") == EXPECTED_BACKEND_BUCKETS[str(row.get("le"))],
                "operational.backend_bucket_population_changed",
            )
    if stream == "pse_build_duration_bucket":
        require(
            {str(row.get("le")) for row in answer.records}
            == set(EXPECTED_BACKEND_BUCKETS),
            "operational.backend_buckets_missing",
        )


def validate_traces(answer: QueryAnswer, produced: dict[str, object]) -> None:
    require(
        answer.source is not None
        and answer.meaning == "observed_points"
        and not answer.complete,
        "operational.trace_source_missing",
    )
    expected = (
        {str(span["span_id"]) for span in records(produced["spans"])}
        if produced["sampler"] == "always_on"
        else set()
    )
    require(
        answer.status == ("available" if expected else "empty"),
        "operational.trace_sampling_visibility_changed",
    )
    require(
        {row.get("span_id") for row in answer.records} == expected,
        "operational.trace_sampled_span_count_changed",
    )
    require(
        all(
            row.get("service_instance_id") == produced["service_instance_id"]
            for row in answer.records
        ),
        "operational.trace_incarnation_changed",
    )


def environment(state: Path, sampler: str = "always_on") -> dict[str, str]:
    selected = {
        key: value
        for key, value in os.environ.items()
        if not key.startswith(("OTEL_", "PSE_TELEMETRY_"))
    }
    selected.update(
        PSE_TELEMETRY_STATE=str(state),
        OTEL_TRACES_SAMPLER=sampler,
        OTEL_LOGS_EXPORTER="none",
        OTEL_METRIC_EXPORT_INTERVAL="1000",
        OTEL_EXPORTER_OTLP_TIMEOUT="2",
        OTEL_BSP_SCHEDULE_DELAY="100",
        OTEL_BSP_EXPORT_TIMEOUT="2000",
    )
    return selected


async def mcp_points(
    state: Path, requests: dict[str, QueryRequest], selected_environment: dict[str, str]
) -> dict[str, QueryAnswer]:
    from fastmcp import Client  # noqa: PLC0415
    from fastmcp.client.transports import StdioTransport  # noqa: PLC0415

    transport = StdioTransport(
        command=sys.executable,
        args=["-m", "scripts.capabilities"],
        cwd=str(ROOT),
        env=selected_environment,
        keep_alive=False,
    )
    async with Client(transport, timeout=10) as client:
        await client.call_tool(
            "capabilities_describe",
            {"capability": "telemetry_query", "include_schema": True},
        )
        selection = await client.call_tool(
            "capabilities_select", {"capabilities": ["telemetry_query"]}
        )
        require(
            "telemetry_query"
            in sequence(mapping(selection.structured_content)["callable_tools"]),
            "operational.mcp_capability_not_selected",
        )
        answers = {}
        for stream, request in requests.items():
            result = await client.call_tool(
                "capabilities_invoke_read",
                {
                    "capability": "telemetry_query",
                    "arguments": {
                        "state": str(state),
                        "request": request.model_dump(mode="json"),
                    },
                },
            )
            content = mapping(result.structured_content)
            while set(content) == {"result"}:
                content = mapping(content["result"])
            require(
                content.get("operation") == "telemetry_query",
                "operational.mcp_wrong_operation",
            )
            answers[stream] = QueryAnswer.model_validate(content["payload"])
        return answers


async def observe_phase(state: Path, output: Path, sampler: str) -> dict[str, object]:
    ready, release = output / f"{sampler}-ready.json", output / f"{sampler}-release"
    command = [
        sys.executable,
        "-m",
        "scripts.tests.telemetry_operational_acceptance",
        "--produce",
        "--state",
        str(state),
        "--sampler",
        sampler,
        "--ready",
        str(ready),
        "--release",
        str(release),
    ]
    selected_environment = environment(state, sampler)
    with (
        (output / f"{sampler}-stdout.json").open("w") as stdout,
        (output / f"{sampler}-stderr.txt").open("w") as stderr,
    ):
        process = subprocess.Popen(
            command, cwd=ROOT, env=selected_environment, stdout=stdout, stderr=stderr
        )
        backend = BackendClient()
        answers: dict[str, QueryAnswer] = {}
        mcp_answers: dict[str, QueryAnswer] = {}
        trace_answer: QueryAnswer | None = None
        try:
            deadline = time.monotonic() + TIMEOUT_SECONDS
            while not ready.is_file():
                require(
                    process.poll() is None and time.monotonic() < deadline,
                    "operational.producer_not_ready",
                )
                await asyncio.sleep(0.2)
            produced = mapping(json.loads(ready.read_bytes()))
            validate_sdk(produced)
            starts = validate_sdk(produced)
            requests = {}
            for stream in STREAMS:
                while True:
                    request = QueryRequest(
                        signal="metrics",
                        metric=stream,
                        service_instance_id=str(produced["service_instance_id"]),
                        start_time_us=min(starts.values()) // 1000 - 1_000_000,
                        end_time_us=time.time_ns() // 1000 + 1,
                        limit=1000,
                    )
                    answer = await backend.query(state, request)
                    if answer.status == "available" and answer.records:
                        validate_points(answer, produced, stream)
                        answers[stream], requests[stream] = answer, request
                        break
                    require(
                        time.monotonic() < deadline,
                        "operational.backend_metric_not_visible",
                    )
                    await asyncio.sleep(0.2)
            mcp_answers = await mcp_points(state, requests, environment(state))
            for stream, answer in mcp_answers.items():
                validate_points(answer, produced, stream)
            while True:
                trace_answer = await backend.query(
                    state,
                    QueryRequest(
                        signal="traces",
                        service_instance_id=str(produced["service_instance_id"]),
                        start_time_us=min(starts.values()) // 1000 - 1_000_000,
                        end_time_us=time.time_ns() // 1000 + 1,
                        limit=1000,
                    ),
                )
                expected = 3 if sampler == "always_on" else 0
                if (
                    trace_answer.status in {"available", "empty"}
                    and len(trace_answer.records) == expected
                ):
                    validate_traces(trace_answer, produced)
                    break
                require(
                    time.monotonic() < deadline, "operational.sampled_spans_not_visible"
                )
                await asyncio.sleep(0.2)
        finally:
            write(
                output / f"{sampler}-observations.json",
                {
                    "backend": {
                        name: answer.model_dump(mode="json")
                        for name, answer in answers.items()
                    },
                    "canonical_mcp": {
                        name: answer.model_dump(mode="json")
                        for name, answer in mcp_answers.items()
                    },
                    "trace": trace_answer.model_dump(mode="json")
                    if trace_answer is not None
                    else None,
                },
            )
            release.touch()
            await backend.close()
            try:
                await asyncio.to_thread(process.wait, timeout=10)
            except subprocess.TimeoutExpired:
                process.terminate()
                await asyncio.to_thread(process.wait, timeout=5)
        require(process.returncode == 0, "operational.producer_failed")
    final = mapping(json.loads((output / f"{sampler}-stdout.json").read_bytes()))
    for stream, answer in {**answers, **mcp_answers}.items():
        validate_points(answer, final, stream, exact_timestamps=True)
    return {
        "producer": final,
        "backend": {
            name: answer.model_dump(mode="json") for name, answer in answers.items()
        },
        "canonical_mcp": {
            name: answer.model_dump(mode="json") for name, answer in mcp_answers.items()
        },
        "sampled_trace_observation": trace_answer.model_dump(mode="json")
        if trace_answer is not None
        else None,
    }


async def run(state: Path, output: Path) -> int:
    output.mkdir(parents=True, mode=0o700, exist_ok=False)
    receipt: dict[str, object] = {
        "baseline_failures": 0,
        "status": "running",
        "state": str(state),
        "scope": "Python production SDK policy, actual existing Collector/OpenObserve and canonical STDIO typed reads",
        "not_established": [
            "native metric populations or Rust exemplars",
            "rates, increases or quantile estimates",
            "exemplar population membership",
            "performance improvement",
            "crash durability",
        ],
    }
    try:
        inspected = telemetry_service.inspect(state)
        services = mapping(inspected.get("services"))
        require(
            all(
                mapping(services.get(name)).get("ownership") == "verified"
                and isinstance(mapping(services.get(name)).get("process"), dict)
                and mapping(services.get(name)).get("active_state") == "active"
                for name in ("collector", "openobserve")
            ),
            "operational.selected_service_not_running_or_owned",
        )
        receipt["service_before"] = inspected
        on = await observe_phase(state, output, "always_on")
        # Publish real sampled stimuli first so absence checks never assume that
        # an empty installation already has a trace stream/attribute schema.
        off = await observe_phase(state, output, "always_off")
        first, second = mapping(on["producer"]), mapping(off["producer"])
        require(
            first["service_instance_id"] != second["service_instance_id"],
            "operational.process_incarnation_reused",
        )
        first_starts, second_starts = validate_sdk(first), validate_sdk(second)
        require(
            all(second_starts[name] > start for name, start in first_starts.items()),
            "operational.new_process_start_not_reset",
        )
        after = telemetry_service.inspect(state)
        require(
            after.get("configuration") == inspected.get("configuration")
            and after.get("services") == inspected.get("services"),
            "operational.selected_service_changed",
        )
        receipt.update(
            status="passed",
            always_off=off,
            always_on=on,
            service_after=after,
            checks=[
                "3 counter events and histogram count/sum/buckets under each sampler",
                "original SDK cumulative start and point timestamps retained",
                "independent process reset starts and resource incarnations",
                "filtered invocation IDs absent from SDK/backend dimensions",
                "canonical STDIO typed reads retain source clocks and backend incarnation",
            ],
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
    parser.add_argument("--produce", action="store_true")
    parser.add_argument("--sampler", choices=("always_off", "always_on"))
    parser.add_argument("--ready", type=Path)
    parser.add_argument("--release", type=Path)
    args = parser.parse_args()
    if args.produce:
        if args.sampler is None or args.ready is None or args.release is None:
            parser.error("--produce requires --sampler, --ready and --release")
        print(
            json.dumps(
                producer(args.state, args.sampler, args.ready, args.release),
                allow_nan=False,
            )
        )
        return 0
    if args.output is None:
        parser.error("--output is required")
    return asyncio.run(run(args.state.resolve(), args.output.absolute()))


if __name__ == "__main__":
    raise SystemExit(main())

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Finite actual estimator/exemplar qualification against an existing telemetry owner."""

from __future__ import annotations

import argparse
import asyncio
import json
import math
import os
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from opentelemetry.exporter.otlp.proto.http.metric_exporter import OTLPMetricExporter

from scripts.telemetry_investigation import BackendClient, QueryAnswer, QueryRequest
from scripts.tests.telemetry_operational_acceptance import (
    ROOT,
    environment,
    mcp_points,
    sequence,
    write,
)
from scripts.tests.telemetry_pipeline_acceptance import mapping, records, require

if TYPE_CHECKING:
    from collections.abc import Mapping

    from opentelemetry.sdk.metrics.export import MetricExportResult, MetricsData


def produce(state: Path, output: Path) -> None:
    from opentelemetry import trace  # noqa: PLC0415
    from opentelemetry.sdk.metrics import MeterProvider  # noqa: PLC0415

    from scripts import telemetry_metrics, telemetry_process  # noqa: PLC0415

    exports: list[dict[str, object]] = []
    spans: list[dict[str, object]] = []
    original = OTLPMetricExporter.export

    def export(
        exporter: OTLPMetricExporter,
        data: MetricsData,
        timeout_millis: float = 10000,
        **kwargs: object,
    ) -> MetricExportResult:
        for resource in data.resource_metrics:
            for scope in resource.scope_metrics:
                for metric in scope.metrics:
                    if metric.name in {"pse.build.records", "pse.build.duration"}:
                        exports.append(mapping(json.loads(metric.to_json())))
        return original(exporter, data, timeout_millis=timeout_millis, **kwargs)

    require(os.environ.get("PSE_TELEMETRY_STATE") == str(state), "semantics.state")
    with (
        patch.object(OTLPMetricExporter, "export", export),
        telemetry_process.bootstrap("pse-semantics-acceptance") as lifetime,
    ):
        require(lifetime.status == "initialized", "semantics.sdk")
        settings = lifetime.settings
        if settings is None:
            raise RuntimeError("semantics.settings")
        host = telemetry_metrics.host_metrics()
        for batch in range(3):
            if batch:
                time.sleep(2)
            for index, value in enumerate((0.001, 0.01, 0.0101)):
                with trace.get_tracer("pse.semantics.fixture").start_as_current_span(
                    "fixture.semantics"
                ) as span:
                    context = span.get_span_context()
                    marker = f"batch-{batch}-event-{index}"
                    spans.append(
                        {
                            "trace_id": f"{context.trace_id:032x}",
                            "span_id": f"{context.span_id:016x}",
                            "marker": marker,
                            "value": value,
                        }
                    )
                    host.build_records.add(1, {"record": "compiler-message"})
                    host.build_duration.record(
                        value,
                        {
                            "status": "passed",
                            "failure_class": "none",
                            "fixture.invocation_id": marker,
                        },
                    )
            for provider in lifetime.providers:
                if isinstance(provider, MeterProvider):
                    require(
                        provider.force_flush(timeout_millis=2000), "semantics.flush"
                    )
        write(
            output,
            {
                "configuration_id": settings.configuration_id,
                "service_instance_id": settings.wire.resource["service.instance.id"],
                "metric_exports": exports,
                "spans": spans,
            },
        )


class RecordingBackend(BackendClient):
    """Keep actual backend refusal bodies without authentication headers."""

    def __init__(self) -> None:
        super().__init__()
        self.raw: list[dict[str, object]] = []

    async def request(
        self,
        url: str,
        *,
        headers: Mapping[str, str] | None = None,
        body: dict[str, object] | None = None,
        generation: tuple[object, ...] | None = None,
        method: str | None = None,
    ) -> tuple[int, bytes]:
        response = await super().request(
            url, headers=headers, body=body, generation=generation, method=method
        )
        self.raw.append(
            {
                "url": url,
                "status": response[0],
                "body": response[1].decode("utf-8", errors="replace"),
            }
        )
        return response


def values(answer: QueryAnswer) -> list[tuple[float, float]]:
    require(
        answer.status == "available", f"semantics.{answer.request.kind}.{answer.status}"
    )
    result: list[tuple[float, float]] = []
    for row in answer.records:
        for item in sequence(row.get("values")):
            pair = sequence(item)
            result.append((float(str(pair[0])), float(str(pair[1]))))
    return result


async def run(state: Path, output: Path) -> int:
    output.mkdir(parents=True, exist_ok=False)
    receipt: dict[str, object] = {
        "status": "failed",
        "rust_exemplars": "unsupported",
        "exemplar_meaning": "related examples only",
        "oracle": {
            "collections": [3, 6, 9],
            "each_batch": [0.001, 0.01, 0.0101],
            "fraction_at_0_01": 2 / 3,
            "histogram_interpolated_p95": 0.0865,
        },
    }
    selected = environment(state)
    selected["OTEL_METRIC_EXPORT_INTERVAL"] = "60000"
    backend = RecordingBackend()
    answers: dict[str, QueryAnswer] = {}
    try:
        with (
            (output / "producer-stdout.txt").open("w") as stdout,
            (output / "producer-stderr.txt").open("w") as stderr,
        ):
            process = await asyncio.create_subprocess_exec(
                sys.executable,
                "-m",
                "scripts.tests.telemetry_semantics_acceptance",
                "--produce",
                "--state",
                str(state),
                "--output",
                str(output / "producer.json"),
                cwd=ROOT,
                env=selected,
                stdout=stdout,
                stderr=stderr,
            )
            try:
                require(
                    await asyncio.wait_for(process.wait(), timeout=15) == 0,
                    "semantics.producer",
                )
            finally:
                if process.returncode is None:
                    process.terminate()
                    try:
                        await asyncio.wait_for(process.wait(), timeout=3)
                    except TimeoutError:
                        process.kill()
                        await process.wait()
        produced = mapping(json.loads((output / "producer.json").read_bytes()))
        receipt["producer"] = produced
        metrics = records(produced["metric_exports"])
        counter_points = [
            records(mapping(metric["data"])["data_points"])[0]
            for metric in metrics
            if metric["name"] == "pse.build.records"
        ][:3]
        require(
            [point["value"] for point in counter_points] == [3, 6, 9],
            "semantics.sdk_counts",
        )
        times = [int(str(point["time_unix_nano"])) / 1e9 for point in counter_points]
        slope = 6 / (times[2] - times[0])
        start, end = int((times[0] + 0.25) * 1e6), int((times[2] + 0.25) * 1e6)
        window = (end - start) // 1_000_000
        receipt["independent_counter_slope"] = slope
        receipt["lookback_seconds"] = window
        receipt["comparison_evaluation_start_seconds"] = times[1] + 0.25
        receipt["comparison_evaluation_end_seconds"] = times[2]
        receipt["edge_exclusion"] = "Compare evaluations after the second original source point and no later than the final original source point; rounded range edges may extrapolate beyond retained stimuli."
        requests: dict[str, QueryRequest] = {}
        for kind, metric in (
            ("rate", "pse_build_records"),
            ("increase", "pse_build_records"),
            ("p95", "pse_build_duration"),
            ("fraction_below", "pse_build_duration"),
            ("exemplars", "pse_build_duration_bucket"),
        ):
            requests[kind] = QueryRequest.model_validate(
                {
                    "signal": "metrics",
                    "kind": kind,
                    "metric": metric,
                    "service_instance_id": produced["service_instance_id"],
                    "start_time_us": int((times[0] - 1) * 1e6)
                    if kind == "exemplars"
                    else start,
                    "end_time_us": end,
                    "step_seconds": 1,
                    "limit": 100,
                    **({"threshold": 0.01} if kind == "fraction_below" else {}),
                }
            )
        for key, stream in (
            ("points", "pse_build_records"),
            ("histogram_count", "pse_build_duration_count"),
            ("histogram_sum", "pse_build_duration_sum"),
            ("histogram_buckets", "pse_build_duration_bucket"),
        ):
            requests[key] = QueryRequest(
                signal="metrics",
                metric=stream,
                service_instance_id=str(produced["service_instance_id"]),
                start_time_us=int((times[0] - 1) * 1e6),
                end_time_us=end,
                limit=100,
            )

        deadline = time.monotonic() + 20
        while True:
            for key in (
                "points",
                "histogram_count",
                "histogram_sum",
                "histogram_buckets",
            ):
                answers[key] = await backend.query(state, requests[key])
            if (
                all(
                    any(row.get("value") == 9 for row in answers[key].records)
                    for key in ("points", "histogram_count")
                )
                and len(answers["histogram_buckets"].records) >= 27
                and len(answers["histogram_sum"].records) >= 3
            ):
                break
            require(time.monotonic() < deadline, "semantics.points_not_visible")
            await asyncio.sleep(0.2)
        for kind, request in requests.items():
            if kind in {"rate", "increase", "p95", "fraction_below", "exemplars"}:
                answers[kind] = await backend.query(state, request)
        receipt["backend"] = {
            kind: answer.model_dump(mode="json") for kind, answer in answers.items()
        }
        mcp_answers = await mcp_points(
            state, {kind: requests[kind] for kind in ("rate", "exemplars")}, selected
        )
        receipt["canonical_mcp"] = {
            kind: answer.model_dump(mode="json") for kind, answer in mcp_answers.items()
        }
        # Compare original OTLP points, independent of PromQL and production normalization.
        for point in counter_points:
            rows = [
                row
                for row in answers["points"].records
                if row.get("_timestamp") == int(str(point["time_unix_nano"])) // 1000
            ]
            require(
                bool(rows)
                and all(
                    row.get("value") == point["value"]
                    and row.get("start_time") == str(point["start_time_unix_nano"])
                    for row in rows
                ),
                "semantics.backend_original_counter",
            )
        histograms = [
            records(mapping(metric["data"])["data_points"])[0]
            for metric in metrics
            if metric["name"] == "pse.build.duration"
        ][:3]
        for index, point in enumerate(histograms, 1):
            require(
                point["count"] == index * 3
                and math.isclose(float(str(point["sum"])), index * 0.0211),
                "semantics.sdk_histogram",
            )
            require(
                point["bucket_counts"] == [index, index, index, 0, 0, 0, 0, 0, 0],
                "semantics.sdk_buckets",
            )
            stamp = int(str(point["time_unix_nano"])) // 1000
            for key, expected in (
                ("histogram_count", index * 3),
                ("histogram_sum", index * 0.0211),
            ):
                rows = [
                    row
                    for row in answers[key].records
                    if row.get("_timestamp") == stamp
                ]
                require(
                    bool(rows)
                    and all(
                        math.isclose(float(str(row["value"])), expected) for row in rows
                    ),
                    "semantics.backend_histogram_point",
                )
            for boundary, expected in (
                ("0.001", index),
                ("0.01", index * 2),
                ("0.1", index * 3),
                ("inf", index * 3),
            ):
                rows = [
                    row
                    for row in answers["histogram_buckets"].records
                    if row.get("_timestamp") == stamp and row.get("le") == boundary
                ]
                require(
                    bool(rows) and all(row.get("value") == expected for row in rows),
                    "semantics.backend_bucket_point",
                )
        for answer in answers.values():
            require(
                answer.source is not None
                and answer.source.configuration_id == produced["configuration_id"],
                "semantics.source_generation",
            )
        for kind, expected in (
            ("rate", slope),
            ("increase", slope * window),
            ("p95", 0.0865),
            ("fraction_below", 2 / 3),
        ):
            measured = [
                (stamp, value)
                for stamp, value in values(answers[kind])
                if times[1] + 0.25 <= stamp <= times[2]
            ]
            require(bool(measured), f"semantics.{kind}.interior_missing")
            tolerance = 0.05 if kind in {"rate", "increase"} else 1e-6
            require(
                all(
                    math.isclose(value, expected, rel_tol=tolerance, abs_tol=1e-7)
                    for _, value in measured
                ),
                f"semantics.{kind}.estimate_mismatch",
            )
        mcp_rates = [
            value
            for stamp, value in values(mcp_answers["rate"])
            if times[1] + 0.25 <= stamp <= times[2]
        ]
        require(
            bool(mcp_rates)
            and all(math.isclose(value, slope, rel_tol=0.05) for value in mcp_rates),
            "semantics.mcp_rate",
        )
        expected_examples = {}
        span_ids = {str(span["span_id"]): span for span in records(produced["spans"])}
        for point in histograms:
            for example in records(point["exemplars"]):
                span_id = f"{int(str(example['span_id'])):016x}"
                trace_id = f"{int(str(example['trace_id'])):032x}"
                span = span_ids[span_id]
                require(span["trace_id"] == trace_id, "semantics.sdk_exemplar_context")
                require(
                    mapping(example["filtered_attributes"])["fixture.invocation_id"]
                    == span["marker"],
                    "semantics.sdk_filtered_attribute",
                )
                expected_examples[(trace_id, span_id)] = example
        for answer in (answers["exemplars"], mcp_answers["exemplars"]):
            require(
                answer.status == "available" and answer.meaning == "related_examples",
                "semantics.example_meaning",
            )
            observed = 0
            for row in answer.records:
                require(
                    "fixture.invocation_id" not in mapping(row["seriesLabels"]),
                    "semantics.example_id_dimension",
                )
                for example in records(row["exemplars"]):
                    labels = mapping(example["labels"])
                    original_example = expected_examples[
                        (str(labels["trace_id"]), str(labels["span_id"]))
                    ]
                    require(
                        labels["fixture.invocation_id"]
                        == mapping(original_example["filtered_attributes"])[
                            "fixture.invocation_id"
                        ],
                        "semantics.example_attribute",
                    )
                    require(
                        float(str(example["value"])) == original_example["value"],
                        "semantics.example_value",
                    )
                    require(
                        int(str(example["timestamp"]))
                        == int(str(original_example["time_unix_nano"]))
                        // 1_000_000_000,
                        "semantics.example_source_clock",
                    )
                    observed += 1
            require(observed > 0, "semantics.examples_missing")
        receipt["python_exemplars"] = (
            "Original sampled SDK trace/span IDs, values, filtered attributes and source seconds preserved; related examples only. No attached native target was supplied."
        )
        receipt["status"] = "passed"
    except (AssertionError, OSError, RuntimeError, ValueError, TimeoutError) as error:
        receipt["error"] = str(error)
    finally:
        receipt["raw_backend"] = backend.raw
        receipt["backend"] = {
            kind: answer.model_dump(mode="json") for kind, answer in answers.items()
        }
        await backend.close()
        write(output / "acceptance.json", receipt)
    return 0 if receipt["status"] == "passed" else 1


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--state", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    parser.add_argument("--produce", action="store_true")
    args = parser.parse_args()
    if args.produce:
        produce(args.state, args.output)
        return 0
    return asyncio.run(run(args.state, args.output))


if __name__ == "__main__":
    raise SystemExit(main())

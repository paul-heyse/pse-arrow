# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Manual isolated OpenTelemetry SDK inquiry; no product imports or network."""

from __future__ import annotations

import argparse
import json
import os
from importlib.metadata import version
from pathlib import Path
from typing import TYPE_CHECKING

from opentelemetry.sdk.metrics import MeterProvider, TraceBasedExemplarFilter
from opentelemetry.sdk.metrics.export import (
    HistogramDataPoint,
    InMemoryMetricReader,
    Metric,
    NumberDataPoint,
)
from opentelemetry.sdk.metrics.view import ExplicitBucketHistogramAggregation, View
from opentelemetry.sdk.trace import ReadableSpan, SpanProcessor, TracerProvider
from opentelemetry.sdk.trace.export import (
    SimpleSpanProcessor,
    SpanExporter,
    SpanExportResult,
)
from opentelemetry.sdk.trace.export.in_memory_span_exporter import InMemorySpanExporter
from opentelemetry.sdk.trace.sampling import ALWAYS_OFF, ALWAYS_ON, AlwaysRecordSampler

if TYPE_CHECKING:
    from collections.abc import Sequence


def metric_named(reader: InMemoryMetricReader, name: str) -> Metric:
    data = reader.get_metrics_data()
    assert data is not None
    found = [
        metric
        for resource in data.resource_metrics
        for scope in resource.scope_metrics
        for metric in scope.metrics
        if metric.name == name
    ]
    assert len(found) == 1, (name, found)
    return found[0]


class LocalProcessor(SpanProcessor):
    def __init__(self) -> None:
        self.finished: list[ReadableSpan] = []

    def on_end(self, span: ReadableSpan) -> None:
        self.finished.append(span)


class RejectingExporter(SpanExporter):
    def __init__(self) -> None:
        self.rejected = 0
        self.calls = 0

    def export(self, spans: Sequence[ReadableSpan]) -> SpanExportResult:
        self.rejected += len(spans)
        self.calls += 1
        return SpanExportResult.FAILURE

    def shutdown(self) -> None:
        return None


def run() -> dict[str, object]:
    reader = InMemoryMetricReader()
    meter = MeterProvider(
        metric_readers=[reader],
        views=[
            View(
                instrument_name="stage.duration",
                attribute_keys={"stage"},
                aggregation=ExplicitBucketHistogramAggregation(boundaries=(0.1, 1.0)),
            )
        ],
    )
    duration = meter.get_meter("isolated-view-probe").create_histogram(
        "stage.duration", unit="s"
    )
    for value, stage, detail in (
        (0.05, "prepare", "request-a"),
        (0.2, "prepare", "request-b"),
        (2.0, "execute", "request-c"),
    ):
        duration.record(value, {"stage": stage, "detail": detail})
    metric = metric_named(reader, "stage.duration")
    points = list(metric.data.data_points)
    assert len(points) == 2
    histograms = []
    for point in points:
        assert isinstance(point, HistogramDataPoint)
        assert point.attributes is not None
        assert set(point.attributes) == {"stage"}
        histograms.append(json.loads(point.to_json()))
        if point.attributes["stage"] == "prepare":
            assert point.count == 2
            assert point.sum == 0.25
            assert tuple(point.bucket_counts) == (1, 1, 0)
            assert point.min == 0.05
            assert point.max == 0.2
        else:
            assert point.count == 1
            assert point.sum == 2.0
            assert tuple(point.bucket_counts) == (0, 0, 1)
    meter.shutdown()

    sampling_results: dict[str, object] = {}
    for label, sampler in (
        ("record_only", AlwaysRecordSampler(ALWAYS_OFF)),
        ("sampled", ALWAYS_ON),
    ):
        metric_reader = InMemoryMetricReader()
        metric_provider = MeterProvider(
            metric_readers=[metric_reader], exemplar_filter=TraceBasedExemplarFilter()
        )
        histogram = metric_provider.get_meter(
            "isolated-sampling-probe"
        ).create_histogram("sample.duration", unit="s")
        local = LocalProcessor()
        memory = InMemorySpanExporter()
        tracing = TracerProvider(sampler=sampler, meter_provider=metric_provider)
        tracing.add_span_processor(local)
        tracing.add_span_processor(
            SimpleSpanProcessor(memory, meter_provider=metric_provider)
        )
        with tracing.get_tracer("isolated-sampling-probe").start_as_current_span(
            label
        ) as span:
            assert span.is_recording()
            context = span.get_span_context()
            histogram.record(0.2, {"stage": "execute"})
        measured = metric_named(metric_reader, "sample.duration")
        exemplars = [
            exemplar
            for point in measured.data.data_points
            for exemplar in point.exemplars
        ]
        assert len(local.finished) == 1
        assert len(memory.get_finished_spans()) == (1 if label == "sampled" else 0)
        assert len(exemplars) == (1 if label == "sampled" else 0)
        if exemplars:
            assert exemplars[0].trace_id == context.trace_id
            assert exemplars[0].span_id == context.span_id
        sampling_results[label] = {
            "recording": True,
            "sampled_flag": context.trace_flags.sampled,
            "local_processor_finished": len(local.finished),
            "ordinary_exported_spans": len(memory.get_finished_spans()),
            "trace_based_exemplars": len(exemplars),
            "exemplar_trace_matches": bool(exemplars)
            and exemplars[0].trace_id == context.trace_id,
        }
        tracing.shutdown()
        metric_provider.shutdown()

    # Enable only in this disposable process; inject explicit metrics provider.
    os.environ["OTEL_PYTHON_SDK_INTERNAL_METRICS_ENABLED"] = "true"
    health_reader = InMemoryMetricReader()
    health_meter = MeterProvider(metric_readers=[health_reader])
    rejected = RejectingExporter()
    tracing = TracerProvider(sampler=ALWAYS_ON, meter_provider=health_meter)
    tracing.add_span_processor(
        SimpleSpanProcessor(rejected, meter_provider=health_meter)
    )
    with tracing.get_tracer("isolated-rejection-probe").start_as_current_span(
        "rejected-export"
    ):
        pass
    processed = metric_named(health_reader, "otel.sdk.processor.span.processed")
    processed_points = [
        json.loads(point.to_json()) for point in processed.data.data_points
    ]
    assert rejected.calls == 1
    assert rejected.rejected == 1
    processed_total = 0.0
    for point in processed.data.data_points:
        assert isinstance(point, NumberDataPoint)
        processed_total += point.value
    assert processed_total == 1
    assert all(
        point.attributes is not None and "error.type" not in point.attributes
        for point in processed.data.data_points
    )
    flush_result = tracing.force_flush()
    tracing.shutdown()
    health_meter.shutdown()
    return {
        "evidence": "Tested: isolated Python SDK APIs only; no product, MCP, network or backend",
        "versions": {
            name: version(name)
            for name in (
                "opentelemetry-api",
                "opentelemetry-sdk",
                "fastmcp",
                "mcp",
                "pydantic",
            )
        },
        "view": {
            "input_measurements": 3,
            "distinct_detail_labels": 3,
            "stage_series": 2,
            "unit": metric.unit,
            "points": histograms,
            "limit": "Attribute projection only; no general cardinality cap demonstrated",
        },
        "sampling": sampling_results,
        "rejecting_exporter": {
            "export_calls": rejected.calls,
            "spans_rejected": rejected.rejected,
            "successful_exports": 0,
            "processor_metric_points": processed_points,
            "provider_force_flush": flush_result,
            "interpretation": "Processed and force_flush success do not establish exporter acceptance",
        },
        "checks": {"assertion_failures": 0, "baseline": 0},
    }


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, required=True)
    output = parser.parse_args().output
    result = run()
    output.parent.mkdir(parents=True, exist_ok=True)
    with output.open("x") as destination:
        destination.write(json.dumps(result, indent=2, sort_keys=True) + "\n")
    print(output)

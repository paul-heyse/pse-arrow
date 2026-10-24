# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Bounded native FastMCP OTel inquiry using the canonical source-only interface.

The SDK/exporter is probe-local. No Collector, services, production configuration,
Rust bridge, or result authority is introduced. The child propagation case is a
Python fixture, not evidence of Rust or database trace continuity.
"""

from __future__ import annotations

import argparse
import asyncio
import contextlib
import importlib
import importlib.metadata
import io
import json
import logging
import os
import sys
import time
from pathlib import Path
from typing import TYPE_CHECKING, cast

from opentelemetry import propagate, trace
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import (
    SimpleSpanProcessor,
    SpanExporter,
    SpanExportResult,
)
from opentelemetry.sdk.trace.export.in_memory_span_exporter import InMemorySpanExporter

if TYPE_CHECKING:
    from collections.abc import Callable, Sequence

    from fastmcp import Client
    from fastmcp.client.client import CallToolResult
    from opentelemetry.sdk.trace import ReadableSpan
    from opentelemetry.trace import SpanContext

    from scripts.capabilities.server import Runtime

ROOT = Path(__file__).resolve().parents[2]


def build_endpoint() -> tuple[type[Client], Runtime]:
    """Load framework/owner only after the caller's SDK initialization decision."""
    client = cast("type[Client]", importlib.import_module("fastmcp").Client)
    build = cast(
        "Callable[[Path], Runtime]",
        importlib.import_module("scripts.capabilities.server").build,
    )
    return client, build(ROOT)


def span_context(span: ReadableSpan) -> SpanContext:
    context = span.context
    if context is None:
        raise TypeError("Expected a recorded span context")
    return context


def object_value(value: object) -> dict:
    if not isinstance(value, dict):
        raise TypeError("Expected an observed object")
    return value


async def canonical_call(client: Client, *, unknown: bool = False) -> CallToolResult:
    return await client.call_tool(
        "capabilities_describe",
        {
            "capability": "not-a-declared-capability"
            if unknown
            else "product_run_read",
            "include_schema": False,
        },
        raise_on_error=False,
    )


async def noop() -> dict:
    # Executed in its own fresh interpreter. Importing FastMCP alone does not
    # install an SDK, sampler, processor, exporter, or collector.
    client_type, runtime = build_endpoint()
    async with client_type(runtime.server) as client:
        result = await canonical_call(client)
    metadata = object_value(object_value(result.meta)["pse.observation"])
    observed = object_value(metadata["trace"])
    assert observed["recording"] is False
    assert observed["context"] == "unavailable"
    assert "trace_id" not in observed
    return {
        "canonical_trace": observed,
        "native_references": metadata["native_references"],
    }


def configure_sdk() -> tuple[TracerProvider, InMemorySpanExporter]:
    # This deliberately happens before FastMCP import in SDK-enabled processes.
    memory = InMemorySpanExporter()
    provider = TracerProvider()
    provider.add_span_processor(SimpleSpanProcessor(memory))
    trace.set_tracer_provider(provider)
    return provider, memory


def propagated_child() -> dict:
    provider, memory = configure_sdk()

    context = propagate.extract({"traceparent": os.environ["PSE_PROBE_TRACEPARENT"]})
    with trace.get_tracer("plan52o-python-child-fixture").start_as_current_span(
        "python-child-fixture", context=context
    ) as span:
        current = span.get_span_context()
    provider.force_flush()
    finished = memory.get_finished_spans()
    assert len(finished) == 1
    assert finished[0].parent is not None
    assert finished[0].parent.is_remote
    result = {
        "trace_id": f"{current.trace_id:032x}",
        "span_id": f"{current.span_id:016x}",
        "parent_span_id": f"{finished[0].parent.span_id:016x}",
        "parent_is_remote": True,
        "language": "Python fixture, not Rust",
    }
    provider.shutdown()
    return result


async def subprocess_json(*arguments: str, env: dict[str, str] | None = None) -> dict:
    process = await asyncio.create_subprocess_exec(
        sys.executable,
        "-m",
        "scripts.tests.capability_telemetry_inquiry",
        *arguments,
        cwd=ROOT,
        env=env,
        stdout=asyncio.subprocess.PIPE,
        stderr=asyncio.subprocess.PIPE,
    )
    async with asyncio.timeout(10):
        stdout, stderr = await process.communicate()
    if process.returncode != 0:
        raise RuntimeError(stderr.decode())
    return object_value(json.loads(stdout))


async def main(output: Path) -> None:
    output.mkdir(parents=True, exist_ok=False)
    started = time.monotonic()
    api_only = await subprocess_json("--noop")
    provider, memory = configure_sdk()

    # A second processor explicitly returns exporter failure. Its failure must
    # neither alter the tool result nor be misreported as native execution loss.
    class RejectingExporter(SpanExporter):
        rejected = 0

        def export(self, spans: Sequence[ReadableSpan]) -> SpanExportResult:
            self.rejected += len(spans)
            return SpanExportResult.FAILURE

        def shutdown(self) -> None:
            return None

    rejected = RejectingExporter()
    provider.add_span_processor(SimpleSpanProcessor(rejected))
    client_type, runtime = build_endpoint()
    tracer = trace.get_tracer("plan52o-telemetry-inquiry")
    log = io.StringIO()
    async with client_type(runtime.server) as client:
        with tracer.start_as_current_span("canonical-read-journey") as root_span:
            root_context = root_span.get_span_context()
            result = await canonical_call(client)
            assert not result.is_error
            observation = object_value(object_value(result.meta)["pse.observation"])
            observed_trace = object_value(observation["trace"])
            assert observed_trace["recording"] is True
            assert observed_trace["trace_id"] == f"{root_context.trace_id:032x}"
            assert observed_trace["export"] == "unverified"
            assert observation["native_references"] == []
            carrier: dict[str, str] = {}
            propagate.inject(carrier)
            child = await subprocess_json(
                "--propagated-child",
                env={**os.environ, "PSE_PROBE_TRACEPARENT": carrier["traceparent"]},
            )
            assert child["trace_id"] == f"{root_context.trace_id:032x}"
            assert child["parent_span_id"] == f"{root_context.span_id:016x}"

        with tracer.start_as_current_span("canonical-refusal-journey") as refusal_span:
            refusal_trace = refusal_span.get_span_context().trace_id
            with contextlib.redirect_stderr(log):
                refusal = await canonical_call(client, unknown=True)
            assert refusal.is_error

    assert provider.force_flush()
    finished = memory.get_finished_spans()
    success_spans = [
        span
        for span in finished
        if span_context(span).trace_id == root_context.trace_id
    ]
    refusal_spans = [
        span for span in finished if span_context(span).trace_id == refusal_trace
    ]
    observed_span = [
        span
        for span in success_spans
        if f"{span_context(span).span_id:016x}" == observed_trace["span_id"]
    ]
    assert len(observed_span) == 1
    assert any(
        span.instrumentation_scope is not None
        and span.instrumentation_scope.name == "fastmcp"
        for span in success_spans
    )
    span_ids = {span_context(span).span_id for span in success_spans}
    assert all(
        span.parent is None or span.parent.span_id in span_ids for span in success_spans
    )
    assert sum(span.parent is None for span in success_spans) == 1
    assert any(span.status.status_code.name == "ERROR" for span in refusal_spans)
    assert rejected.rejected > 0
    diagnostics = [
        object_value(json.loads(line))
        for line in log.getvalue().splitlines()
        if line.startswith('{"code":"pse.capabilities.interface_')
    ]
    assert diagnostics
    assert diagnostics[-1]["native_reference_count"] == 0

    def describe(span: ReadableSpan) -> dict:
        assert span.instrumentation_scope is not None
        return {
            "name": span.name,
            "scope": span.instrumentation_scope.name,
            "trace_id": f"{span_context(span).trace_id:032x}",
            "span_id": f"{span_context(span).span_id:016x}",
            "parent_span_id": f"{span.parent.span_id:016x}" if span.parent else None,
            "kind": span.kind.name,
            "status": span.status.status_code.name,
        }

    report = {
        "evidence": "Tested: canonical read/refusal with real native FastMCP spans, probe-local SDK and memory exporter",
        "versions": {
            name: importlib.metadata.version(name)
            for name in ("fastmcp", "mcp", "opentelemetry-api", "opentelemetry-sdk")
        },
        "api_only": api_only,
        "sdk_enabled_canonical_observation": observation,
        "success_trace": [describe(span) for span in success_spans],
        "refusal_trace": [describe(span) for span in refusal_spans],
        "refusal_diagnostic": diagnostics[-1],
        "standard_w3c_python_child": child,
        "exporter_failure": {
            "rejected_spans": rejected.rejected,
            "parallel_memory_exported_spans": len(finished),
            "tool_succeeded_despite_rejection": not result.is_error,
            "force_flush_returned_true": True,
            "observation_export_still": observed_trace["export"],
        },
        "limits": [
            "No Collector, persistent export, sampling study, or production exporter installation.",
            "Python child is a context-propagation fixture, not Rust or database instrumentation.",
            "Canonical observation retains end_to_end_continuity=unestablished and export=unverified.",
            "Trace completion and ERROR status do not establish scientific result, native terminal, or physical drain.",
        ],
        "elapsed_seconds": time.monotonic() - started,
        "failure_baseline": 0,
        "failures": 0,
    }
    (output / "results.json").write_text(json.dumps(report, indent=2) + "\n")
    print(json.dumps(report, indent=2))
    provider.shutdown()


if __name__ == "__main__":
    logging.disable(logging.CRITICAL)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path)
    parser.add_argument("--noop", action="store_true")
    parser.add_argument("--propagated-child", action="store_true")
    args = parser.parse_args()
    if args.noop:
        print(json.dumps(asyncio.run(noop())))
    elif args.propagated_child:
        print(json.dumps(propagated_child()))
    elif args.output is not None:
        asyncio.run(main(args.output))
    else:
        parser.error("select --output or a probe child mode")

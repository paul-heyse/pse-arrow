# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Process-owned telemetry through real bounded OTLP HTTP transports."""

# ruff: noqa: PT009 -- stdlib unittest controls

from __future__ import annotations

import json
import os
import socket
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest.mock import patch

from opentelemetry import context, trace
from opentelemetry.proto.collector.logs.v1.logs_service_pb2 import (
    ExportLogsServiceRequest,
)
from opentelemetry.proto.collector.metrics.v1.metrics_service_pb2 import (
    ExportMetricsServiceRequest,
)
from opentelemetry.proto.collector.trace.v1.trace_service_pb2 import (
    ExportTraceServiceRequest,
)
from opentelemetry.sdk.trace.sampling import Decision
from opentelemetry.trace.propagation.tracecontext import TraceContextTextMapPropagator

from scripts import telemetry_process
from scripts.generated.telemetry_settings import TelemetrySampler
from scripts.telemetry_config import resolve_process
from scripts.telemetry_process import (
    _sdk_sampler,
    current_carrier,
    helper_environment,
    inherited_context,
    validated_carrier,
)

ROOT = Path(__file__).resolve().parents[2]
PARENT = {
    "traceparent": "00-1234567890abcdef1234567890abcdef-1234567890abcdef-01",
    "tracestate": "fixture=selected",
}


class SamplerTests(unittest.TestCase):
    def test_enum_adapters_preserve_root_and_local_remote_parent_sampling(self) -> None:
        for selection, root_sampled, parent_based in (
            (TelemetrySampler.ALWAYS_ON, True, False),
            (TelemetrySampler.ALWAYS_OFF, False, False),
            (TelemetrySampler.TRACEIDRATIO, False, False),
            (TelemetrySampler.PARENTBASED_ALWAYS_ON, True, True),
            (TelemetrySampler.PARENTBASED_ALWAYS_OFF, False, True),
            (TelemetrySampler.PARENTBASED_TRACEIDRATIO, False, True),
        ):
            sampler = _sdk_sampler(selection, 0.0)
            with self.subTest(selection=selection):
                self.assertEqual(
                    sampler.should_sample(context.Context(), 1, "fixture").decision,
                    Decision.RECORD_AND_SAMPLE if root_sampled else Decision.DROP,
                )
                for remote in (False, True):
                    for sampled in (False, True):
                        parent = trace.SpanContext(
                            trace_id=1,
                            span_id=1,
                            is_remote=remote,
                            trace_flags=trace.TraceFlags(1 if sampled else 0),
                        )
                        parent_context = trace.set_span_in_context(
                            trace.NonRecordingSpan(parent), context.Context()
                        )
                        expected = sampled if parent_based else root_sampled
                        self.assertEqual(
                            sampler.should_sample(
                                parent_context, 1, "fixture"
                            ).decision,
                            Decision.RECORD_AND_SAMPLE if expected else Decision.DROP,
                        )


class ProcessTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.records: list[tuple[str, bytes, str | None]] = []
        self.authorizations: list[str | None] = []
        self.ambient_headers: list[str | None] = []
        self.content_types: list[str | None] = []
        self.response_status = 200
        self.response_body = b""
        self.delay = 0.0
        case = self

        class Handler(BaseHTTPRequestHandler):
            def do_POST(self) -> None:
                content = self.rfile.read(int(self.headers["Content-Length"]))
                case.records.append((self.path, content, self.headers.get("x-fixture")))
                case.authorizations.append(self.headers.get("Authorization"))
                case.ambient_headers.append(self.headers.get("x-ambient"))
                case.content_types.append(self.headers.get("Content-Type"))
                if case.delay:
                    time.sleep(case.delay)
                self.send_response(case.response_status)
                self.send_header("Content-Length", str(len(case.response_body)))
                self.end_headers()
                if case.response_body:
                    self.wfile.write(case.response_body)

            def log_message(self, format: str, *args: object) -> None:
                pass

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.server.daemon_threads = True
        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()
        self.addCleanup(self.close_server)
        self.environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith(("OTEL_", "PSE_TELEMETRY_"))
        }
        self.environment.update(
            {
                "PSE_TELEMETRY_STATE": self.temporary.name,
                "OTEL_EXPORTER_OTLP_ENDPOINT": f"http://127.0.0.1:{self.server.server_port}",
                "OTEL_EXPORTER_OTLP_HEADERS": "x-fixture=selected",
                "OTEL_EXPORTER_OTLP_TIMEOUT": "0.15",
                "OTEL_BSP_MAX_QUEUE_SIZE": "16",
                "OTEL_BSP_MAX_EXPORT_BATCH_SIZE": "4",
                "OTEL_BSP_SCHEDULE_DELAY": "60000",
                "OTEL_METRIC_EXPORT_INTERVAL": "3600000",
                "PSE_TELEMETRY_CARRIER": json.dumps(PARENT),
            }
        )

    def close_server(self) -> None:
        self.server.shutdown()
        self.server.server_close()
        self.thread.join()

    def child(
        self, source: str, *, environment: dict[str, str] | None = None
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            [sys.executable, "-c", source],
            env=self.environment if environment is None else environment,
            cwd=ROOT,
            check=True,
            text=True,
            capture_output=True,
            timeout=15,
        )

    def test_disabled_host_imports_no_sdk_or_scientific_extension(self) -> None:
        result = self.child(
            """import sys
from scripts.telemetry_process import bootstrap
with bootstrap("disabled") as owner:
    assert owner.status == "disabled"
    assert "opentelemetry.sdk.trace" not in sys.modules
    assert "pse" not in sys.modules
print("disabled")
""",
            environment={**self.environment, "OTEL_SDK_DISABLED": "true"},
        )
        self.assertEqual(result.stdout, "disabled\n")
        self.assertEqual(self.records, [])

    def test_real_export_parentage_resources_logs_metrics_and_shutdown(self) -> None:
        result = self.child("""import json,logging,os
from opentelemetry import trace,metrics
from scripts.telemetry_process import bootstrap,inherited_context,current_carrier
with bootstrap("fixture") as owner:
    assert owner.status == "initialized", owner.status
    with inherited_context(), trace.get_tracer("fixture").start_as_current_span("child"):
        print(json.dumps({"carrier":current_carrier(),"pid":os.getpid()}))
        logging.getLogger("fixture.app").warning("owned diagnostic", extra={"diagnostic.code":"fixture.selected","owner.stage":"dispatch","native.operation":"fixture-operation"})
        logging.getLogger("opentelemetry.fixture").warning("exporter internal")
        metrics.get_meter("fixture").create_counter("fixture.calls").add(1)
    borrowed = bootstrap("second-runtime")
    assert borrowed.status == "already_initialized"
    borrowed.shutdown()
    assert not owner.closed
""")
        produced = json.loads(result.stdout)
        carrier = produced["carrier"]
        self.assertEqual(
            carrier["traceparent"].split("-")[1], PARENT["traceparent"].split("-")[1]
        )
        self.assertNotEqual(
            carrier["traceparent"].split("-")[2], PARENT["traceparent"].split("-")[2]
        )

        spans = []
        logs = []
        incarnations = []
        process_pids = []
        metric_names = []
        for path, content, header in self.records:
            self.assertEqual(header, "selected")
            if path == "/v1/traces":
                request = ExportTraceServiceRequest.FromString(content)
                for resource in request.resource_spans:
                    attributes = {
                        attribute.key: attribute.value
                        for attribute in resource.resource.attributes
                    }
                    self.assertEqual(
                        attributes["process.pid"].WhichOneof("value"), "int_value"
                    )
                    process_pids.append(attributes["process.pid"].int_value)
                    incarnations.append(attributes["service.instance.id"].string_value)
                    spans.extend(
                        span for scope in resource.scope_spans for span in scope.spans
                    )
            elif path == "/v1/logs":
                request = ExportLogsServiceRequest.FromString(content)
                for resource in request.resource_logs:
                    attributes = {
                        attribute.key: attribute.value
                        for attribute in resource.resource.attributes
                    }
                    self.assertEqual(
                        attributes["process.pid"].WhichOneof("value"), "int_value"
                    )
                    process_pids.append(attributes["process.pid"].int_value)
                    incarnations.append(attributes["service.instance.id"].string_value)
                    logs.extend(
                        record
                        for scope in resource.scope_logs
                        for record in scope.log_records
                    )
            elif path == "/v1/metrics":
                request = ExportMetricsServiceRequest.FromString(content)
                for resource in request.resource_metrics:
                    attributes = {
                        attribute.key: attribute.value
                        for attribute in resource.resource.attributes
                    }
                    self.assertEqual(
                        attributes["process.pid"].WhichOneof("value"), "int_value"
                    )
                    process_pids.append(attributes["process.pid"].int_value)
                    incarnations.append(attributes["service.instance.id"].string_value)
                    metric_names.extend(
                        metric.name
                        for scope in resource.scope_metrics
                        for metric in scope.metrics
                    )
        child = next(span for span in spans if span.name == "child")
        self.assertEqual(child.parent_span_id.hex(), "1234567890abcdef")
        self.assertEqual(child.trace_id.hex(), "1234567890abcdef1234567890abcdef")
        self.assertEqual(len(logs), 1)
        self.assertEqual(logs[0].body.string_value, "owned diagnostic")
        self.assertEqual(logs[0].span_id, child.span_id)
        attributes = {
            attribute.key: attribute.value.string_value
            for attribute in logs[0].attributes
        }
        self.assertEqual(attributes["diagnostic.code"], "fixture.selected")
        self.assertIn("fixture.calls", metric_names)
        self.assertGreaterEqual(len(incarnations), 3)
        self.assertEqual(len(set(incarnations)), 1)
        self.assertEqual(set(process_pids), {produced["pid"]})
        self.assertIn("owned diagnostic", result.stderr)
        self.assertIn("SDK diagnostic", result.stderr)
        self.assertNotIn("exporter internal", result.stderr)

    def test_actual_refused_export_redacts_before_existing_handlers_and_late_logs(
        self,
    ) -> None:
        with socket.socket() as selected:
            selected.bind(("127.0.0.1", 0))
            refused_port = selected.getsockname()[1]
        result = self.child(
            """import io,json,logging,sys
from opentelemetry import trace
from scripts.telemetry_process import bootstrap
application_capture=io.StringIO()
logging.getLogger().setLevel(logging.DEBUG)
root_handler=logging.StreamHandler(application_capture)
logging.getLogger().addHandler(root_handler)
exporter=logging.getLogger("opentelemetry.exporter.otlp.proto.http.trace_exporter")
exporter_capture=io.StringIO()
exporter_handler=logging.StreamHandler(exporter_capture)
exporter.addHandler(exporter_handler)
application=logging.getLogger("fixture.application")
with bootstrap("redaction") as owner:
    assert owner.status == "initialized",owner.status
    application.warning("application stderr remains readable")
    with trace.get_tracer("fixture").start_as_current_span("refused-export"):
        pass
    try:
        raise RuntimeError("private-traceback-payload")
    except RuntimeError:
        exporter.exception("private-header-token=%s private-body-token=%s", "private-header-value", "private-body-value")
assert root_handler in logging.getLogger().handlers
assert exporter_handler in exporter.handlers
# Late batch diagnostics remain guarded after the provider shutdown call.
try:
    raise RuntimeError("private-late-traceback")
except RuntimeError:
    exporter.exception("private-late-payload")
# The same original record is safe before preexisting handlers render it,
# including Python versions whose Filter contract accepts only bool.
record=logging.LogRecord(exporter.name,logging.WARNING,"private-source-path",1,"private-record-body",(),None,"private-stack-payload")
record.private_extra="private-extra-payload"
exporter.handle(record)
assert record.name == "pse.telemetry.diagnostic"
assert record.args == () and record.exc_info is None and record.stack_info is None
assert not hasattr(record,"private_extra")
application.warning("application after shutdown")
print(json.dumps({"application":application_capture.getvalue(),"existing_exporter":exporter_capture.getvalue()}))
""",
            environment={
                **self.environment,
                "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT": f"http://127.0.0.1:{refused_port}/fixture-private-token/v1/traces",
                "OTEL_EXPORTER_OTLP_TIMEOUT": "2",
                "OTEL_LOGS_EXPORTER": "none",
                "OTEL_METRICS_EXPORTER": "none",
                "OTEL_EXPORTER_OTLP_HEADERS": "x-private-header=private-header-value",
            },
        )
        answer = json.loads(result.stdout)
        self.assertIn("application stderr remains readable", answer["application"])
        self.assertIn("application after shutdown", answer["application"])
        self.assertIn("Failed to export", answer["existing_exporter"])
        self.assertIn("Connection refused", result.stderr)
        self.assertIn("details=redacted", result.stderr)
        for sentinel in (
            "fixture-private-token",
            "private-traceback-payload",
            "private-header-token",
            "private-header-value",
            "private-body-token",
            "private-body-value",
            "private-late-traceback",
            "private-late-payload",
            "private-source-path",
            "private-record-body",
            "private-stack-payload",
            "private-extra-payload",
        ):
            with self.subTest(sentinel=sentinel):
                self.assertNotIn(sentinel, result.stdout + result.stderr)
        self.assertNotIn("Traceback", result.stdout + result.stderr)

    def test_actual_http_refusal_redacts_backend_response_payload(self) -> None:
        self.response_status = 401
        self.response_body = b"private-backend-body private-header-value"
        result = self.child(
            """import io,json,logging
from opentelemetry import trace
from scripts.telemetry_process import bootstrap
application_capture=io.StringIO()
handler=logging.StreamHandler(application_capture)
logging.getLogger().addHandler(handler)
logging.getLogger().setLevel(logging.DEBUG)
with bootstrap("redaction-http") as owner:
    assert owner.status == "initialized",owner.status
    with trace.get_tracer("fixture").start_as_current_span("refused-http"):
        pass
assert handler in logging.getLogger().handlers
print(json.dumps({"existing_application_handler":application_capture.getvalue()}))
""",
            environment={
                **self.environment,
                "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT": f"http://127.0.0.1:{self.server.server_port}/fixture-private-token/v1/traces",
                "OTEL_LOGS_EXPORTER": "none",
                "OTEL_METRICS_EXPORTER": "none",
                "OTEL_EXPORTER_OTLP_HEADERS": "x-private-header=private-header-value",
            },
        )
        self.assertIn("Failed to export", result.stderr)
        self.assertEqual(self.records[0][0], "/fixture-private-token/v1/traces")
        for sentinel in (
            "fixture-private-token",
            "private-backend-body",
            "private-header-value",
        ):
            with self.subTest(sentinel=sentinel):
                self.assertNotIn(sentinel, result.stdout + result.stderr)
        self.assertNotIn("Traceback", result.stdout + result.stderr)

    def test_explicit_empty_headers_cannot_inherit_ambient_auth_for_any_signal(
        self,
    ) -> None:
        selection = {**self.environment, "OTEL_EXPORTER_OTLP_HEADERS": ""}
        for signal in ("TRACES", "LOGS", "METRICS"):
            selection[f"OTEL_EXPORTER_OTLP_{signal}_HEADERS"] = ""
        netrc = Path(self.temporary.name) / "ambient.netrc"
        netrc.write_text(
            "machine 127.0.0.1 login fixture-user password fixture-netrc-value\n"
        )
        netrc.chmod(0o600)
        ambient = {
            **self.environment,
            "OTEL_EXPORTER_OTLP_HEADERS": "Authorization=fixture-ambient-base",
            "OTEL_EXPORTER_OTLP_COMPRESSION": "gzip",
            "NETRC": str(netrc),
            "HTTP_PROXY": "http://127.0.0.1:1",
            "HTTPS_PROXY": "http://127.0.0.1:1",
            "NO_PROXY": "",
        }
        for signal in ("TRACES", "LOGS", "METRICS"):
            ambient[f"OTEL_EXPORTER_OTLP_{signal}_HEADERS"] = (
                "Authorization=fixture-ambient-signal,x-ambient=fixture-ambient-value"
            )
        source = """import json,logging,os
from unittest.mock import patch
from requests import Session
from opentelemetry import trace,metrics
from scripts.telemetry_process import bootstrap
selection=json.loads(SELECTION)
closed=[]
original_close=Session.close
def close(session):
    closed.append(id(session))
    original_close(session)
with patch.object(Session,"close",close),bootstrap("explicit-headers",environment=selection) as owner:
    assert owner.status == "initialized",owner.status
    assert os.environ["OTEL_EXPORTER_OTLP_HEADERS"] == "Authorization=fixture-ambient-base"
    for signal in ("TRACES","LOGS","METRICS"):
        assert os.environ[f"OTEL_EXPORTER_OTLP_{signal}_HEADERS"] == "Authorization=fixture-ambient-signal,x-ambient=fixture-ambient-value"
    with trace.get_tracer("fixture").start_as_current_span("explicit-selection"):
        logging.getLogger("fixture.application").warning("selected empty headers")
        metrics.get_meter("fixture").create_counter("fixture.explicit_headers").add(1)
assert len(closed)==3 and len(set(closed))==3,closed
""".replace("SELECTION", repr(json.dumps(selection)))
        result = self.child(source, environment=ambient)
        self.assertEqual(result.stdout, "")
        self.assertEqual(
            {record[0] for record in self.records},
            {"/v1/traces", "/v1/logs", "/v1/metrics"},
        )
        self.assertTrue(all(value is None for value in self.authorizations))
        self.assertTrue(all(value is None for value in self.ambient_headers))
        self.assertEqual(set(self.content_types), {"application/x-protobuf"})

        for path, content, _header in self.records:
            if path == "/v1/traces":
                self.assertTrue(
                    ExportTraceServiceRequest.FromString(content).resource_spans
                )
            elif path == "/v1/logs":
                self.assertTrue(
                    ExportLogsServiceRequest.FromString(content).resource_logs
                )
            else:
                self.assertTrue(
                    ExportMetricsServiceRequest.FromString(content).resource_metrics
                )
        self.assertNotIn("fixture-ambient", result.stderr)

    def test_context_restoration_and_refusal(self) -> None:

        before = current_carrier()
        with inherited_context({"PSE_TELEMETRY_CARRIER": json.dumps(PARENT)}):
            self.assertEqual(current_carrier(), PARENT)
        self.assertEqual(current_carrier(), before)
        token = context.attach(TraceContextTextMapPropagator().extract(PARENT))
        try:
            env = helper_environment(
                {"unrelated": "kept", "PSE_TELEMETRY_CARRIER": "stale"}
            )
            self.assertEqual(json.loads(env["PSE_TELEMETRY_CARRIER"]), PARENT)
            self.assertEqual(env["unrelated"], "kept")
        finally:
            context.detach(token)
        for value in (
            {**PARENT, "baggage": "secret"},
            {"traceparent": "00-" + "0" * 32 + "-1234567890abcdef-01"},
            {**PARENT, "tracestate": "fixture=value,fixture=duplicate"},
            {**PARENT, "tracestate": "fixture=bad\nvalue"},
        ):
            self.assertEqual(validated_carrier(value), {})
        self.assertFalse(trace.get_current_span().get_span_context().is_valid)

    def test_refused_and_slow_transport_settles_with_real_deadline(self) -> None:
        for response_status, delay in ((503, 0.0), (200, 0.6)):
            self.response_status, self.delay = response_status, delay
            with self.subTest(status=response_status, delay=delay):
                started = time.monotonic()
                result = self.child(
                    """from opentelemetry import trace
from scripts.telemetry_process import bootstrap
with bootstrap("outage") as owner:
    assert owner.status == "initialized",owner.status
    with trace.get_tracer("fixture").start_as_current_span("outage"):
        pass
print("native-result-retained")
""",
                    environment={
                        **self.environment,
                        "OTEL_LOGS_EXPORTER": "none",
                        "OTEL_METRICS_EXPORTER": "none",
                    },
                )
                self.assertLess(time.monotonic() - started, 5)
                self.assertEqual(result.stdout, "native-result-retained\n")
                self.assertTrue(result.stderr)

    def test_fixed_helper_entry_attaches_parent_before_work(self) -> None:

        source = Path(self.temporary.name) / "request.json"
        destination = Path(self.temporary.name) / "response.json"
        # An invalid host record exercises the real selected worker and refuses
        # before archive admission/effects. It needs no scientific extension.
        source.write_text("{}")
        result = subprocess.run(
            [
                sys.executable,
                "-m",
                "scripts.capabilities.host_worker",
                "archive",
                str(source),
                str(destination),
            ],
            env=self.environment,
            cwd=ROOT,
            check=True,
            text=True,
            capture_output=True,
            timeout=15,
        )
        self.assertEqual(result.stdout, "")
        self.assertEqual(json.loads(destination.read_text())["error"]["stage"], "input")
        spans = []
        resources = []
        for path, content, _ in self.records:
            if path == "/v1/traces":
                request = ExportTraceServiceRequest.FromString(content)
                for resource in request.resource_spans:
                    resources.append(
                        {
                            attribute.key: attribute.value.string_value
                            for attribute in resource.resource.attributes
                        }
                    )
                    spans.extend(
                        span for scope in resource.scope_spans for span in scope.spans
                    )
        helper = next(span for span in spans if span.name == "pse.helper")
        self.assertEqual(helper.parent_span_id.hex(), "1234567890abcdef")
        self.assertEqual(helper.trace_id.hex(), "1234567890abcdef1234567890abcdef")
        self.assertTrue(
            all(resource["service.name"] == "pse-host-worker" for resource in resources)
        )

    def test_fresh_stdio_discovery_parentage_and_outage_shutdown(self) -> None:

        for response_status in (200, 503):
            self.response_status = response_status
            self.records.clear()
            with self.subTest(response_status=response_status):
                started = time.monotonic()
                result = self.child(
                    """import asyncio,json,os,sys
from pathlib import Path
from fastmcp import Client
from fastmcp.client.transports import StdioTransport
async def main():
    selected={key:value for key,value in os.environ.items() if key.startswith(("OTEL_","PSE_TELEMETRY_"))}
    selected.pop("PSE_TELEMETRY_CARRIER",None)
    stderr=Path(os.environ["PSE_TELEMETRY_STATE"])/"stdio-stderr.log"
    transport=StdioTransport(command=sys.executable,args=["-m","scripts.capabilities"],env=selected,cwd=str(Path.cwd()),log_file=stderr,keep_alive=False)
    async with Client(transport,timeout=5) as client:
        tools=await client.list_tools()
        assert "capabilities_domains" in {tool.name for tool in tools}
        result=await client.call_tool("capabilities_domains",{},meta={"traceparent":"00-1234567890abcdef1234567890abcdef-1234567890abcdef-01","tracestate":"fixture=selected"})
        assert not result.is_error
        assert result.data
    print(json.dumps({"protocol":"completed"}))
asyncio.run(main())
""",
                    environment={
                        **self.environment,
                        "OTEL_LOGS_EXPORTER": "none",
                        "OTEL_METRICS_EXPORTER": "none",
                    },
                )
                self.assertLess(time.monotonic() - started, 10)
                self.assertEqual(json.loads(result.stdout), {"protocol": "completed"})
                self.assertNotIn("Failed to parse", result.stderr)
                spans = []
                resources = []
                for path, content, _ in self.records:
                    if path == "/v1/traces":
                        request = ExportTraceServiceRequest.FromString(content)
                        for resource in request.resource_spans:
                            resources.append(
                                {
                                    attribute.key: attribute.value.string_value
                                    for attribute in resource.resource.attributes
                                }
                            )
                            spans.extend(
                                span
                                for scope in resource.scope_spans
                                for span in scope.spans
                            )
                parented = [
                    span
                    for span in spans
                    if span.parent_span_id.hex() == "1234567890abcdef"
                ]
                self.assertTrue(
                    parented, [(span.name, span.parent_span_id.hex()) for span in spans]
                )
                self.assertTrue(
                    all(
                        span.trace_id.hex() == "1234567890abcdef1234567890abcdef"
                        for span in parented
                    )
                )
                self.assertTrue(
                    all(
                        resource["service.name"] == "pse-capabilities"
                        for resource in resources
                    )
                )
                stderr = (Path(self.temporary.name) / "stdio-stderr.log").read_text()
                self.assertNotIn("Failed to parse", stderr)
                if response_status == 503:
                    self.assertTrue(stderr)

    def test_local_status_is_passive_and_unknown_is_not_zero(self) -> None:
        with (
            patch("scripts.telemetry_process._STATE.active", None),
            patch.object(
                telemetry_process.importlib,
                "import_module",
                side_effect=AssertionError("cold read cannot import native"),
            ),
            patch.object(
                telemetry_process,
                "resolve_process",
                side_effect=AssertionError("passive read must not resolve/bootstrap"),
            ),
        ):
            value = telemetry_process.local_status()
        self.assertEqual(value["status"], "not_initialized")
        self.assertEqual(value["queue_occupancy"], "unavailable")
        self.assertEqual(value["dropped_records"], "unavailable")
        settings = resolve_process(
            "fixture", environment={"PSE_TELEMETRY_STATE": self.temporary.name}
        )
        owner = telemetry_process.ProcessLifetime(settings, "disabled")
        with patch("scripts.telemetry_process._STATE.active", owner):
            self.assertEqual(telemetry_process.local_status()["status"], "disabled")
            self.assertEqual(
                telemetry_process.local_status()["delivery"], "not_observed"
            )
            owner.shutdown()
            self.assertTrue(telemetry_process.local_status()["closed"])

    def test_native_loss_snapshot_survives_shutdown_without_claiming_python_or_delivery(
        self,
    ) -> None:
        class Native:
            def __init__(self) -> None:
                self.activation: dict[str, str | None] = {"status": "initialized"}
                self.closed = False

            def shutdown(self) -> None:
                self.closed = True

            def loss_health(self) -> dict[str, object]:
                return {
                    "process_incarnation": "fixture-native-incarnation",
                    "signal": "traces",
                    "processor": "BatchSpanProcessor",
                    "first_drop_observed_at_unix_ns": 123,
                    "sdk_reported_dropped_span_count": 7 if self.closed else None,
                    "shutdown": "failed" if self.closed else "not_attempted",
                    "delivery": "not_observed",
                    "abrupt_exit_loss": "unknown",
                }

        native = Native()
        owner = telemetry_process.ProcessLifetime(None, "initialized", native=native)
        with patch("scripts.telemetry_process._STATE.active", owner):
            live = telemetry_process.local_status()
            owner.shutdown()
            final = telemetry_process.local_status()
        live_loss = live["dropped_records"]
        final_loss = final["dropped_records"]
        assert isinstance(live_loss, dict)
        assert isinstance(final_loss, dict)
        live_native = live_loss["native_rust"]
        final_native = final_loss["native_rust"]
        assert isinstance(live_native, dict)
        assert isinstance(final_native, dict)
        self.assertIsNone(live_native["sdk_reported_dropped_span_count"])
        self.assertEqual(final_native["sdk_reported_dropped_span_count"], 7)
        self.assertEqual(final_native["shutdown"], "failed")
        self.assertEqual(final_native["abrupt_exit_loss"], "unknown")
        self.assertEqual(final["delivery"], "not_observed")
        self.assertEqual(final["queue_occupancy"], "unavailable")
        python = final_loss["python"]
        assert isinstance(python, dict)
        self.assertEqual(python["status"], "unavailable")

    def test_native_loss_read_failure_is_passive_unavailable(self) -> None:
        class Native:
            def __init__(self) -> None:
                self.activation: dict[str, str | None] = {"status": "initialized"}

            def shutdown(self) -> None:
                pass

            def loss_health(self) -> dict[str, object] | None:
                raise RuntimeError("private-failure-must-not-render")

        owner = telemetry_process.ProcessLifetime(None, "initialized", native=Native())
        with (
            patch("scripts.telemetry_process._STATE.active", owner),
            patch.object(
                telemetry_process.importlib,
                "import_module",
                side_effect=AssertionError("passive read cannot import native"),
            ),
        ):
            value = telemetry_process.local_status()
        self.assertNotIn("private-failure", repr(value))
        loss = value["dropped_records"]
        assert isinstance(loss, dict)
        self.assertEqual(
            loss["native_rust"],
            {"status": "unavailable", "reason": "native_trace_observer_read_failed"},
        )

    def test_actual_disabled_native_bridge_retains_typed_passive_health(self) -> None:
        result = self.child(
            """import json
from scripts.generated.telemetry_settings import TelemetrySampler
from scripts.telemetry_config import resolve_process
from pse._telemetry import initialize
from pse._build import _telemetry_status
assert _telemetry_status() == "null"
settings = resolve_process("disabled-native-bridge")
owner = initialize(settings.native_json(), lambda: {})
assert owner.activation["status"] == "disabled"
live = owner.loss_health()
assert live is not None
assert live["process_incarnation"] == settings.wire.process_incarnation
assert live["scope"] == "pse-runtime"
assert live["signal"] == "traces"
assert live["processor"] == "BatchSpanProcessor"
assert live["shutdown"] == "unselected"
assert live["first_drop_observed"] is False
assert live["sdk_reported_dropped_span_count"] is None
owner.shutdown()
retained = owner.loss_health()
assert retained is not None
assert retained["process_incarnation"] == live["process_incarnation"]
assert retained["sdk_reported_dropped_span_count"] is None
assert retained["delivery"] == "not_observed"
assert retained["abrupt_exit_loss"] == "unknown"
assert json.loads(_telemetry_status())["process_incarnation"] == live["process_incarnation"]
owner.shutdown()
print("native-health-retained")
""",
            environment={**self.environment, "OTEL_SDK_DISABLED": "true"},
        )
        self.assertEqual(result.stdout, "native-health-retained\n")
        self.assertEqual(self.records, [])

    def test_preexisting_provider_is_preserved(self) -> None:
        result = self.child("""from opentelemetry import trace
from opentelemetry.sdk.trace import TracerProvider
from scripts.telemetry_process import bootstrap
existing=TracerProvider(shutdown_on_exit=False)
trace.set_tracer_provider(existing)
with bootstrap("borrower") as owner:
    assert owner.status == "initialization_refused"
    assert trace.get_tracer_provider() is existing
existing.shutdown()
""")
        self.assertEqual(result.stdout, "")
        self.assertIn("existing_provider_preserved", result.stderr)
        self.assertEqual(self.records, [])


if __name__ == "__main__":
    unittest.main()

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Actual canonical telemetry reads retain host limits and source identities."""
# ruff: noqa: PT009, PT027 -- stdlib host controls

from __future__ import annotations

import json
import sys
import tempfile
import threading
import time
import unittest
from contextlib import suppress
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest.mock import AsyncMock, patch

from fastmcp import Client
from fastmcp.client.transports import StdioTransport
from fastmcp.exceptions import ToolError
from jsonschema import Draft202012Validator

from scripts import (
    telemetry_investigation,
    telemetry_process,
    telemetry_service,
)
from scripts.capabilities.inventory import _families
from scripts.capabilities.server import build
from scripts.tests.test_capabilities import BOOTSTRAP

ROOT = Path(__file__).resolve().parents[2]
TRACE_ID = "1234567890abcdef1234567890abcdef"
PRIVATE = "private-telemetry-credential-or-response"
OPERATIONS = {
    "telemetry_health",
    "telemetry_configuration",
    "telemetry_trace_lookup",
    "telemetry_query",
    "telemetry_schema",
    "telemetry_selection_read",
    "telemetry_investigation_export",
    "telemetry_watch_read",
    "telemetry_watch_configure",
}


class TelemetryClientCase(unittest.IsolatedAsyncioTestCase):
    async def asyncSetUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.state = Path(temporary.name) / "missing-state"
        self.runtime = build(ROOT)
        self.client = Client(self.runtime.server)
        await self.client.__aenter__()
        self.addAsyncCleanup(self.client.__aexit__, None, None, None)

    async def call(self, operation: str, arguments: dict) -> dict:
        await self.client.call_tool(
            "capabilities_select", {"capabilities": [operation]}
        )
        result = await self.client.call_tool(
            "capabilities_invoke_read",
            {"capability": operation, "arguments": arguments},
        )
        structured = result.structured_content
        assert structured is not None
        schema = self.runtime.catalogue.tools[operation].output_schema
        assert schema is not None
        Draft202012Validator(schema).validate(structured)
        return structured["result"]


class TelemetryTests(TelemetryClientCase):
    async def test_fixed_inventory_progressive_reveal_and_original_native_reader(
        self,
    ) -> None:
        self.assertEqual(
            {item.name for item in await self.client.list_tools()}, BOOTSTRAP
        )
        family = self.runtime.catalogue.describe("telemetry-observation")
        operations = family["operations"]
        if not isinstance(operations, list):
            self.fail("Expected the declared operation list")
        self.assertEqual(set(operations), OPERATIONS)
        declaration = next(
            item
            for item in _families()
            if item.capability.id == "telemetry-observation"
        )
        self.assertEqual(declaration.recipes, ("telemetry",))
        for path in (
            "scripts/telemetry.py",
            "scripts/telemetry_config.py",
            "scripts/telemetry_process.py",
            "scripts/telemetry_query.py",
            "scripts/telemetry_service.py",
            "scripts/telemetry_launch.py",
        ):
            self.assertIn(path, declaration.sources)
        self.assertIn(
            "scripts/telemetry_query.py",
            self.runtime.interface.operations["telemetry_trace_lookup"],
        )
        for operation in OPERATIONS:
            spec = self.runtime.catalogue.capabilities[operation]
            self.assertEqual(spec.family, "telemetry-observation")
            self.assertEqual(spec.exposure, "tool")
            self.assertIn("diagnose-run", spec.workflows)
            annotations = self.runtime.catalogue.tools[operation].annotations
            if annotations is None:
                self.fail("Expected explicit read-only tool annotations")
            self.assertEqual(
                annotations.read_only_hint,
                operation
                not in {"telemetry_investigation_export", "telemetry_watch_configure"},
            )
            description = await self.client.call_tool(
                "capabilities_describe", {"capability": operation}
            )
            self.assertIn(operation, json.dumps(description.structured_content))
        step = next(
            step
            for step in self.runtime.catalogue.workflows["diagnose-run"].steps
            if step.id == "telemetry"
        )
        self.assertFalse(step.default)
        self.assertIn("native_diagnostic_inspect", self.runtime.catalogue.tools)
        native = self.runtime.catalogue.tools["native_diagnostic_inspect"]
        description = native.description
        if description is None:
            self.fail("Expected the original native reader description")
        self.assertIn("target locator", description)
        self.assertIn("registration coverage", description)
        self.assertEqual(
            {item.name for item in await self.client.list_tools()}, BOOTSTRAP
        )

    async def test_passive_backend_down_reads_preserve_calling_process_and_target_distinction(
        self,
    ) -> None:
        with (
            patch.object(telemetry_process._STATE, "active", None),  # noqa: SLF001 -- isolate calling-process state without initializing an SDK.
            patch.object(
                telemetry_service, "start", side_effect=AssertionError("started")
            ),
            patch.object(
                telemetry_process,
                "bootstrap",
                side_effect=AssertionError("SDK initialized"),
            ),
            patch("subprocess.run", side_effect=AssertionError("unexpected command")),
            patch.dict("os.environ", {"OTEL_SDK_DISABLED": "true"}),
        ):
            health = await self.call("telemetry_health", {"state": str(self.state)})
            trace = await self.call(
                "telemetry_trace_lookup",
                {
                    "state": str(self.state),
                    "trace_id": TRACE_ID,
                    "start_time_us": 100,
                    "end_time_us": 200,
                },
            )
        self.assertEqual(health["payload"]["pipeline"]["status"], "unavailable")
        self.assertEqual(health["payload"]["producer"]["status"], "not_initialized")
        self.assertIn("calling MCP process", " ".join(health["answer"]["reasons"]))
        self.assertEqual(trace["payload"]["reason"], "telemetry_not_configured")
        self.assertEqual(trace["answer"]["population"], "unavailable")
        self.assertFalse(trace["complete"])
        self.assertFalse(self.state.exists())
        self.assertNotIn("pse._native", sys.modules)

    async def test_effective_configuration_redacts_values_and_endpoint_paths(
        self,
    ) -> None:
        with patch.dict(
            "os.environ",
            {
                "OTEL_SDK_DISABLED": "false",
                "OTEL_EXPORTER_OTLP_ENDPOINT": "http://localhost:4318/" + PRIVATE,
                "OTEL_EXPORTER_OTLP_HEADERS": "Authorization=" + PRIVATE,
                "OTEL_RESOURCE_ATTRIBUTES": "private=" + PRIVATE,
            },
        ):
            result = await self.call(
                "telemetry_configuration", {"state": str(self.state)}
            )
        self.assertNotIn(PRIVATE, json.dumps(result))
        payload = result["payload"]
        self.assertEqual(payload["selection"], "environment")
        self.assertEqual(
            payload["signals"]["traces"]["header_names"], ["Authorization"]
        )
        self.assertIn("<redacted-path>", payload["signals"]["traces"]["endpoint"])
        self.assertEqual(payload["delivery"], "not_observed")
        self.assertEqual(payload["resource"]["service.name"], "pse-capabilities")
        self.assertFalse(self.state.exists())

    async def test_exact_inputs_refuse_unsafe_selections_before_backend_access(
        self,
    ) -> None:
        arguments = {"trace_id": TRACE_ID, "start_time_us": 100, "end_time_us": 200}
        await self.client.call_tool(
            "capabilities_select", {"capabilities": sorted(OPERATIONS)}
        )
        with patch.object(
            telemetry_service,
            "inspect",
            side_effect=AssertionError("invalid input reached backend"),
        ):
            for override in (
                {"trace_id": "0" * 32},
                {"trace_id": "X" * 32},
                {"trace_id": TRACE_ID + "' OR true"},
                {"start_time_us": 200},
                {"start_time_us": -1},
                {"end_time_us": "200"},
                {"start_time_us": True},
                {"limit": 1001},
                {"limit": 0},
                {"limit": True},
                {"state": ""},
                {"state": "x\x00y"},
                {"state": "x" * 4097},
                {"sql": PRIVATE},
                {"headers": {"Authorization": PRIVATE}},
            ):
                with (
                    self.subTest(override=override),
                    self.assertRaises(ToolError) as error,
                ):
                    await self.client.call_tool(
                        "telemetry_trace_lookup", {**arguments, **override}
                    )
                self.assertNotIn(PRIVATE, str(error.exception))
        for operation in ("telemetry_health", "telemetry_configuration"):
            with self.assertRaises(ToolError):
                await self.client.call_tool(operation, {"state": ""})

    async def test_owner_errors_are_safe_and_never_render_supplied_private_data(
        self,
    ) -> None:
        with (
            patch.object(
                telemetry_investigation.BackendClient,
                "query",
                new=AsyncMock(
                    side_effect=telemetry_service.TelemetryServiceError(PRIVATE)
                ),
            ),
            self.assertRaises(ToolError) as error,
        ):
            await self.call(
                "telemetry_trace_lookup",
                {
                    "trace_id": TRACE_ID,
                    "start_time_us": 100,
                    "end_time_us": 200,
                },
            )
        self.assertNotIn(PRIVATE, str(error.exception))
        self.assertIn("source_unavailable", str(error.exception))


class BackendTests(TelemetryClientCase):
    async def asyncSetUp(self) -> None:
        await super().asyncSetUp()
        self.body = b'{"hits":[]}'
        self.code = 200
        self.requests: list[dict] = []
        self.delay = 0.0
        case = self

        class Handler(BaseHTTPRequestHandler):
            def do_POST(self) -> None:
                request = self.rfile.read(int(self.headers.get("Content-Length", "0")))
                case.requests.append(json.loads(request))
                if case.delay:
                    time.sleep(case.delay)
                self.send_response(case.code)
                self.send_header("Content-Length", str(len(case.body)))
                self.end_headers()
                with suppress(BrokenPipeError):
                    self.wfile.write(case.body)

            def do_GET(self) -> None:
                body = (
                    b'otelcol_exporter_sent_spans{exporter="otlp"} 17\n'
                    if self.path == "/metrics"
                    else b"healthy"
                )
                self.send_response(200)
                self.send_header("Content-Length", str(len(body)))
                self.end_headers()
                self.wfile.write(body)

            def log_message(self, format: str, *args: object) -> None:
                del format, args

        backend = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        backend.daemon_threads = True
        thread = threading.Thread(
            target=lambda: backend.serve_forever(poll_interval=0.01), daemon=True
        )
        thread.start()
        self.addCleanup(backend.server_close)
        self.addCleanup(thread.join, 3)
        self.addCleanup(backend.shutdown)
        self.process = {
            "pid": 2345,
            "start_ticks": 5432,
            "invocation_id": "fixture-incarnation",
        }
        self.inspected = {
            "status": "configured",
            "configuration": {
                "configuration_id": "fixture-config",
                "streams": {"traces": "pse_traces"},
                "organization": "default",
            },
            "services": {
                "openobserve": {"ownership": "verified", "process": self.process}
            },
            "versions": {"openobserve": "1.1.2"},
            "retention_days": 7,
        }
        inspection = patch.object(
            telemetry_service, "inspect", return_value=self.inspected
        )
        self.inspect = inspection.start()
        self.addCleanup(inspection.stop)
        access = patch.object(
            telemetry_service,
            "backend_access",
            return_value=(
                f"http://127.0.0.1:{backend.server_port}",
                {"Authorization": PRIVATE},
            ),
        )
        access.start()
        self.addCleanup(access.stop)
        self.base = f"http://127.0.0.1:{backend.server_port}"

    async def lookup(self, limit: int = 3) -> dict:
        return await self.call(
            "telemetry_trace_lookup",
            {
                "state": str(self.state),
                "trace_id": TRACE_ID,
                "start_time_us": 100,
                "end_time_us": 200,
                "limit": limit,
            },
        )

    async def test_typed_metric_query_preserves_original_points_and_declared_schema(
        self,
    ) -> None:
        self.body = json.dumps(
            {
                "hits": [
                    {
                        "_timestamp": 101,
                        "start_time": 90,
                        "value": 4,
                        "owner_epoch": "a",
                    },
                    {
                        "_timestamp": 150,
                        "start_time": 140,
                        "value": 1,
                        "owner_epoch": "b",
                    },
                ]
            }
        ).encode()
        answer = await self.call(
            "telemetry_query",
            {
                "request": {
                    "signal": "metrics",
                    "metric": "native_work",
                    "start_time_us": 100,
                    "end_time_us": 200,
                }
            },
        )
        self.assertEqual(
            [item["start_time"] for item in answer["payload"]["records"]], [90, 140]
        )
        self.assertFalse(answer["complete"])
        self.assertEqual(answer["payload"]["meaning"], "observed_points")
        self.assertEqual(self.requests[-1]["query"]["size"], 100)
        result = await self.client.read_resource("pse://telemetry/schema")
        content = " ".join(item.text for item in result if hasattr(item, "text"))
        self.assertIn("microseconds", content)
        self.assertIn("pse_native_supervisor_events", content)

    async def test_watch_control_is_an_action_and_refuses_read_invoker(self) -> None:
        await self.client.call_tool(
            "capabilities_select", {"capabilities": ["telemetry_watch_configure"]}
        )
        with self.assertRaises(ToolError):
            await self.client.call_tool(
                "capabilities_invoke_read",
                {"capability": "telemetry_watch_configure", "arguments": {}},
            )

    async def test_actual_backend_partial_and_empty_results_preserve_attribution(
        self,
    ) -> None:
        self.body = json.dumps(
            {
                "hits": [
                    {
                        "trace_id": TRACE_ID,
                        "span_id": "234567890abcdef1",
                        "reference_parent_span_id": "34567890abcdef12",
                        "reference_parent_trace_id": "abcdef1234567890abcdef1234567890",
                        "service_service_instance_id": "native-target-incarnation",
                        "operation_name": "native.solve",
                        "_timestamp": 123,
                    }
                ],
                "is_partial": True,
            }
        ).encode()
        result = await self.lookup(limit=2)
        payload = result["payload"]
        self.assertEqual(payload["status"], "partial")
        self.assertEqual(payload["coverage"], "backend_partial")
        self.assertEqual(payload["source"]["backend_process"], self.process)
        self.assertEqual(payload["source"]["configuration_id"], "fixture-config")
        self.assertEqual(payload["source"]["backend_version"], "1.1.2")
        self.assertEqual(payload["source"]["start_time_us"], 100)
        self.assertEqual(
            payload["records"][0]["service_instance_id"], "native-target-incarnation"
        )
        self.assertEqual(payload["records"][0]["parent_span_id"], "34567890abcdef12")
        self.assertIsNone(payload["population_total"])
        self.assertFalse(payload["complete"])
        self.assertNotIn(PRIVATE, json.dumps(result))
        self.assertEqual(self.requests[0]["query"]["size"], 2)
        self.body = b'{"hits":[]}'
        empty = await self.lookup()
        self.assertEqual(empty["payload"]["status"], "empty")
        self.assertEqual(empty["answer"]["population"], "partial")

    async def test_health_counters_keep_pipeline_process_and_calling_producer_separate(
        self,
    ) -> None:
        self.inspected["configuration"]["backend_http_endpoint"] = self.base
        self.inspected["services"]["collector"] = {
            "ownership": "verified",
            "process": {"pid": 999, "start_ticks": 987},
        }
        self.inspected["endpoints"] = {
            "collector_health": self.base + "/",
            "collector_metrics": self.base + "/metrics",
        }
        with patch.object(telemetry_process._STATE, "active", None):  # noqa: SLF001 -- isolate calling-process state without initializing an SDK.
            result = await self.call("telemetry_health", {})
        payload = result["payload"]
        self.assertEqual(payload["producer"]["status"], "not_initialized")
        self.assertEqual(payload["pipeline"]["backend"]["status"], "responding")
        counter = payload["pipeline"]["collector_delivery"]
        self.assertEqual(counter["samples"][0]["value"], 17)
        self.assertEqual(counter["process"], {"pid": 999, "start_ticks": 987})
        self.assertEqual(counter["configuration_id"], "fixture-config")
        self.assertEqual(
            counter["query_visibility"], "not_established_by_exporter_counters"
        )

    async def test_backend_total_deadline_is_retained_by_binding(self) -> None:
        self.delay = 0.15
        with patch.object(telemetry_investigation, "TIMEOUT_SECONDS", 0.03):
            result = await self.lookup()
        self.assertEqual(result["payload"]["reason"], "backend_read_failed")
        self.assertEqual(result["answer"]["population"], "unavailable")

    async def test_stale_generation_does_not_serve_records(self) -> None:
        self.inspect.side_effect = [
            self.inspected,
            {
                **self.inspected,
                "services": {
                    "openobserve": {
                        "ownership": "verified",
                        "process": {**self.process, "start_ticks": 9876},
                    }
                },
            },
        ]
        result = await self.lookup()
        self.assertEqual(result["payload"]["status"], "stale")
        self.assertEqual(result["payload"]["records"], [])
        self.assertEqual(result["answer"]["population"], "unavailable")

    async def test_refused_malformed_and_oversize_backend_answers_are_unavailable(
        self,
    ) -> None:
        for code, body in (
            (503, PRIVATE.encode()),
            (200, b'{"hits":{}}'),
            (200, b"x" * (telemetry_investigation.MAX_RESPONSE_BYTES + 1)),
        ):
            self.code, self.body = code, body
            with self.subTest(code=code, bytes=len(body)):
                result = await self.lookup()
                self.assertEqual(result["payload"]["status"], "unavailable")
                self.assertEqual(result["payload"]["records"], [])
                self.assertNotIn(PRIVATE, json.dumps(result))

    async def test_answer_bound_refuses_detail_without_rewriting_target_records(
        self,
    ) -> None:
        self.body = json.dumps(
            {
                "hits": [
                    {
                        "trace_id": TRACE_ID,
                        "span_id": "234567890abcdef1",
                        "operation_name": "n" * 140000,
                    }
                ]
            }
        ).encode()
        result = await self.lookup()
        self.assertEqual(result["outcome"], "detail-refused")
        self.assertNotIn("payload", result)
        self.assertGreater(result["encoded_bytes"], result["maximum_bytes"])


class StdioTelemetryTests(unittest.IsolatedAsyncioTestCase):
    async def test_real_source_only_stdio_reveal_read_and_redaction_without_backend_or_native(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            scratch = Path(directory)
            state = scratch / "absent-state"
            guard = scratch / "sitecustomize.py"
            guard.write_text(
                "import sys\n"
                "class NoNative:\n"
                " def find_spec(self, fullname, path=None, target=None):\n"
                "  if fullname == 'pse' or fullname.startswith('pse.'):\n"
                "   raise AssertionError('source-only server imported scientific package')\n"
                "sys.meta_path.insert(0, NoNative())\n"
            )
            transport = StdioTransport(
                command=sys.executable,
                args=["-m", "scripts.capabilities"],
                cwd=str(ROOT),
                keep_alive=False,
                log_file=scratch / "server-stderr.log",
                env={
                    "PYTHONPATH": str(scratch) + ":" + str(ROOT),
                    "PSE_TELEMETRY_STATE": str(state),
                    "PSE_RESOURCE_CLASS": "invalid-workload-class",
                    "OTEL_SDK_DISABLED": "true",
                    "OTEL_EXPORTER_OTLP_HEADERS": "Authorization=" + PRIVATE,
                    "OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:1/" + PRIVATE,
                },
            )
            try:
                async with Client(transport, timeout=20) as client:
                    self.assertEqual(
                        {item.name for item in await client.list_tools()}, BOOTSTRAP
                    )
                    described = await client.call_tool(
                        "capabilities_describe",
                        {
                            "capability": "telemetry_trace_lookup",
                            "include_schema": True,
                        },
                    )
                    self.assertIn(
                        "start_time_us", json.dumps(described.structured_content)
                    )
                    await client.call_tool(
                        "capabilities_select", {"capabilities": sorted(OPERATIONS)}
                    )
                    self.assertEqual(
                        {item.name for item in await client.list_tools()},
                        BOOTSTRAP | OPERATIONS,
                    )
                    health = await client.call_tool(
                        "capabilities_invoke_read",
                        {"capability": "telemetry_health", "arguments": {}},
                    )
                    configuration = await client.call_tool(
                        "telemetry_configuration", {}
                    )
                    trace = await client.call_tool(
                        "telemetry_trace_lookup",
                        {
                            "trace_id": TRACE_ID,
                            "start_time_us": 100,
                            "end_time_us": 200,
                        },
                    )
                    assert health.structured_content is not None
                    assert trace.structured_content is not None
                    self.assertEqual(
                        health.structured_content["result"]["payload"]["pipeline"][
                            "status"
                        ],
                        "unavailable",
                    )
                    self.assertEqual(
                        trace.structured_content["result"]["payload"]["status"],
                        "unavailable",
                    )
                    self.assertNotIn(
                        PRIVATE, json.dumps(configuration.structured_content)
                    )
                    await client.call_tool("capabilities_reset", {})
                    self.assertEqual(
                        {item.name for item in await client.list_tools()}, BOOTSTRAP
                    )
                    self.assertFalse(state.exists())
            finally:
                await transport.close()


if __name__ == "__main__":
    unittest.main()

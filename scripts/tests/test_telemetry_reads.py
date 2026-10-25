# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Independent bounded HTTP-query and passive-health consumer contracts."""

# ruff: noqa: PT009, PT027 -- stdlib unittest controls
from __future__ import annotations

import io
import json
import tempfile
import threading
import time
import unittest
from contextlib import ExitStack, redirect_stdout
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
from pathlib import Path
from unittest.mock import patch

from scripts import telemetry, telemetry_investigation, telemetry_process, telemetry_query, telemetry_service

TRACE_ID = "1234567890abcdef1234567890abcdef"
PARENT_TRACE_ID = "abcdef1234567890abcdef1234567890"
SPAN_ID = "234567890abcdef1"
PARENT_SPAN_ID = "34567890abcdef12"
PRIVATE_SENTINEL = "fixture-private-response-and-credential"


def mapping(value: object) -> dict[str, object]:
    if not isinstance(value, dict) or any(not isinstance(key, str) for key in value):
        raise AssertionError("Expected a string-keyed object")
    return {key: item for key, item in value.items() if isinstance(key, str)}


def records(value: object) -> list[dict[str, object]]:
    if not isinstance(value, list):
        raise TypeError("Expected a list of objects")
    return [mapping(item) for item in value]


def text(value: object) -> str:
    if not isinstance(value, str):
        raise TypeError("Expected a string")
    return value


class TelemetryReadTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.state = Path(temporary.name) / "missing-state"
        self.requests: list[tuple[str, str, bytes, str | None]] = []
        self.routes: dict[str, tuple[int, bytes]] = {}
        case = self

        class Handler(BaseHTTPRequestHandler):
            def answer(self) -> None:
                body = self.rfile.read(int(self.headers.get("Content-Length", "0")))
                case.requests.append(
                    (self.command, self.path, body, self.headers.get("Authorization"))
                )
                code, response = case.routes.get(
                    self.path, (404, b"unknown fixture route")
                )
                self.send_response(code)
                self.send_header("Content-Length", str(len(response)))
                self.end_headers()
                self.wfile.write(response)

            def do_GET(self) -> None:
                self.answer()

            def do_POST(self) -> None:
                self.answer()

            def log_message(self, format: str, *_arguments: object) -> None:
                pass

        self.server = ThreadingHTTPServer(("127.0.0.1", 0), Handler)
        self.server.daemon_threads = True
        thread = threading.Thread(
            target=lambda: self.server.serve_forever(poll_interval=0.05), daemon=True
        )
        thread.start()
        self.addCleanup(self.close_server, thread)
        self.base = f"http://127.0.0.1:{self.server.server_port}"
        self.search = "/api/default/_search?type=traces"
        self.configuration: dict[str, object] = {
            "schema_version": 1,
            "configuration_id": "fixture-generation",
            "organization": "default",
            "streams": {"traces": "pse_traces"},
            "backend_http_endpoint": self.base,
        }
        self.services: dict[str, dict[str, object]] = {
            "collector": {
                "active_state": "active",
                "process": {
                    "pid": 1234,
                    "start_time_ticks": 4321,
                    "boot_id": "fixture-boot",
                    "incarnation": "fixture-process",
                },
                "ownership": "verified",
            }
        }
        self.services["openobserve"] = {
            "active_state": "active",
            "ownership": "verified",
            "process": {
                "pid": 2345,
                "start_ticks": 5432,
                "invocation_id": "backend-incarnation",
            },
        }
        self.inspected = {
            "status": "configured",
            "configuration": self.configuration,
            "services": self.services,
            "retention_days": 7,
            "versions": {"openobserve": "1.1.2"},
            "endpoints": {
                "collector_health": self.base + "/",
                "collector_metrics": self.base + "/metrics",
            },
        }

    def close_server(self, thread: threading.Thread) -> None:
        self.server.shutdown()
        self.server.server_close()
        thread.join()

    def lookup(self, *, limit: int = 10) -> dict[str, object]:
        with (
            patch.object(telemetry_service, "inspect", return_value=self.inspected),
            patch.object(
                telemetry_service,
                "backend_access",
                return_value=(
                    self.base,
                    {"Authorization": "Basic " + PRIVATE_SENTINEL},
                ),
            ),
        ):
            return telemetry_query.lookup_trace(
                self.state, TRACE_ID, start_time_us=100, end_time_us=200, limit=limit
            )

    def observe_health(self) -> dict[str, object]:
        with patch.object(telemetry_service, "inspect", return_value=self.inspected):
            return telemetry.health(self.state)

    def test_failed_service_inspection_preserves_local_reader(self) -> None:
        with (
            patch.object(
                telemetry_service,
                "inspect",
                side_effect=telemetry_service.TelemetryServiceError("private detail"),
            ),
            patch.object(
                telemetry_process, "local_status", return_value={"status": "disabled"}
            ),
        ):
            answer = telemetry.health(self.state)
        self.assertEqual(mapping(answer["producer"])["status"], "disabled")
        self.assertEqual(
            mapping(answer["service"])["reason"], "selected_service_inspection_failed"
        )
        self.assertEqual(mapping(answer["pipeline"])["status"], "unavailable")
        self.assertNotIn("private detail", json.dumps(answer))
        self.assertEqual(self.requests, [])

    def test_restarted_collector_counters_are_not_attributed_to_old_incarnation(
        self,
    ) -> None:
        self.routes.update(
            {
                "/": (200, b"ok"),
                "/healthz": (200, b"ok"),
                "/metrics": (200, b"otelcol_exporter_sent_spans_total 7\n"),
            }
        )
        changed = {
            **self.inspected,
            "services": {
                **self.services,
                "collector": {**self.services["collector"], "process": {"pid": 9876}},
            },
        }
        with patch.object(
            telemetry_service, "inspect", side_effect=[self.inspected, changed]
        ):
            answer = telemetry.health(self.state)
        pipeline = mapping(answer["pipeline"])
        self.assertEqual(mapping(pipeline["collector"])["status"], "stale")
        self.assertEqual(mapping(pipeline["collector_delivery"])["status"], "stale")
        self.assertNotIn("samples", mapping(pipeline["collector_delivery"]))
        self.assertEqual(mapping(pipeline["backend"])["status"], "responding")

    def test_configuration_command_honors_explicit_state(self) -> None:
        self.state.mkdir()
        (self.state / "configuration.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "configuration_id": "explicit-selection",
                    "otlp_http_endpoint": self.base,
                }
            )
        )
        output = io.StringIO()
        with redirect_stdout(output), patch.dict("os.environ", {}, clear=True):
            self.assertEqual(
                telemetry.main(["configuration", "--state", str(self.state)]), 0
            )
        self.assertEqual(
            json.loads(output.getvalue())["configuration_id"], "explicit-selection"
        )

    def test_id_time_and_limit_validation_precedes_backend(self) -> None:
        cases = (
            {"trace_id": "' OR 1=1 --"},
            {"trace_id": "0" * 32},
            {"trace_id": TRACE_ID.upper()},
            {"trace_id": TRACE_ID + "; DROP TABLE spans"},
            {"start_time_us": -1},
            {"start_time_us": True},
            {"end_time_us": 100},
            {"end_time_us": "200"},
            {"limit": 0},
            {"limit": 1001},
            {"limit": True},
        )
        with (
            patch.object(
                telemetry_service,
                "inspect",
                side_effect=AssertionError("validation entered service"),
            ),
            patch.object(
                telemetry_service,
                "backend_access",
                side_effect=AssertionError("validation entered backend"),
            ),
        ):
            for changed in cases:
                values = {
                    "trace_id": TRACE_ID,
                    "start_time_us": 100,
                    "end_time_us": 200,
                    "limit": 10,
                    **changed,
                }
                with self.subTest(changed=changed), self.assertRaises(ValueError):
                    telemetry_query.lookup_trace(
                        self.state,
                        values["trace_id"],  # pyrefly: ignore [bad-argument-type] -- intentional invalid API input
                        start_time_us=values["start_time_us"],  # pyrefly: ignore [bad-argument-type] -- intentional invalid API input
                        end_time_us=values["end_time_us"],  # pyrefly: ignore [bad-argument-type] -- intentional invalid API input
                        limit=values["limit"],  # pyrefly: ignore [bad-argument-type] -- intentional invalid API input
                    )
        self.assertEqual(self.requests, [])

    def test_stream_and_organization_injection_refused_before_http(self) -> None:
        for field, invalid in (
            ("organization", "default/other"),
            ("streams", {"traces": "pse_traces; DROP TABLE facts"}),
        ):
            with (
                self.subTest(field=field),
                patch.object(
                    telemetry_service,
                    "inspect",
                    return_value={
                        **self.inspected,
                        "configuration": {**self.configuration, field: invalid},
                    },
                ),
                patch.object(
                    telemetry_service,
                    "backend_access",
                    side_effect=AssertionError("invalid identifiers reached backend"),
                ),
                self.assertRaises(ValueError),
            ):
                telemetry_query.lookup_trace(
                    self.state, TRACE_ID, start_time_us=100, end_time_us=200
                )
        self.assertEqual(self.requests, [])

    def test_actual_search_query_interval_and_normalized_references(self) -> None:
        self.routes[self.search] = (
            200,
            json.dumps(
                {
                    "hits": [
                        {
                            "trace_id": TRACE_ID,
                            "span_id": SPAN_ID,
                            "reference_parent_span_id": PARENT_SPAN_ID,
                            "reference_parent_trace_id": PARENT_TRACE_ID,
                            "service_service_instance_id": "producer-incarnation",
                            "operation_name": "owner.dispatch",
                            "_timestamp": 125,
                            "unselected_payload": PRIVATE_SENTINEL,
                        }
                    ],
                    "total": 999999,
                }
            ).encode(),
        )
        answer = self.lookup()
        self.assertEqual(answer["status"], "available")
        self.assertEqual(
            answer["records"],
            [
                {
                    "trace_id": TRACE_ID,
                    "span_id": SPAN_ID,
                    "parent_span_id": PARENT_SPAN_ID,
                    "parent_trace_id": PARENT_TRACE_ID,
                    "service_instance_id": "producer-incarnation",
                    "operation_name": "owner.dispatch",
                    "timestamp_us": 125,
                }
            ],
        )
        method, target, content, authorization = self.requests[0]
        self.assertEqual(method, "POST")
        self.assertEqual(target, self.search)
        self.assertEqual(authorization, "Basic " + PRIVATE_SENTINEL)
        document = json.loads(content)
        self.assertEqual(document["query"]["start_time"], 100)
        self.assertEqual(document["query"]["end_time"], 200)
        self.assertEqual(document["query"]["size"], 10)
        self.assertEqual(document["query"]["from"], 0)
        self.assertIn(
            "FROM pse_traces WHERE trace_id = '" + TRACE_ID + "'",
            document["query"]["sql"],
        )
        self.assertIn("ORDER BY _timestamp,span_id", document["query"]["sql"])
        self.assertFalse(answer["complete"])
        self.assertIsNone(answer["population_total"])
        source = mapping(answer["source"])
        self.assertEqual(source["configuration_id"], "fixture-generation")
        self.assertEqual(source["retention_days"], 7)
        self.assertIn("no atomic snapshot", text(source["source_cut"]))
        self.assertNotIn(PRIVATE_SENTINEL, json.dumps(answer))

    def test_partial_empty_and_row_limit_never_claim_complete_population(self) -> None:
        cases = (
            ({"hits": []}, 10, "empty", "bounded_live_selection"),
            ({"hits": [{"span_id": SPAN_ID}]}, 1, "available", "row_limit_reached"),
            (
                {"hits": [{"span_id": SPAN_ID}], "is_partial": True},
                10,
                "partial",
                "backend_partial",
            ),
            (
                {"hits": [], "function_error": PRIVATE_SENTINEL},
                10,
                "partial",
                "backend_partial",
            ),
        )
        for payload, limit, status, coverage in cases:
            with self.subTest(status=status, coverage=coverage):
                self.routes[self.search] = (
                    200,
                    json.dumps({"total": 1000000, **payload}).encode(),
                )
                answer = self.lookup(limit=limit)
                self.assertEqual(answer["status"], status)
                self.assertEqual(answer["coverage"], coverage)
                self.assertEqual(answer["returned_rows"], len(records(payload["hits"])))
                self.assertEqual(answer["row_limit"], limit)
                self.assertFalse(answer["complete"])
                self.assertIsNone(answer["population_total"])
                self.assertIn("not observed", text(answer["missing_record_meaning"]))
                self.assertNotIn(PRIVATE_SENTINEL, json.dumps(answer))

    def test_refusal_bad_json_shape_and_response_bound_do_not_leak_bodies(self) -> None:
        cases = (
            (403, PRIVATE_SENTINEL.encode(), "backend_refused"),
            (200, ("not JSON: " + PRIVATE_SENTINEL).encode(), "backend_read_failed"),
            (
                200,
                json.dumps({"hits": PRIVATE_SENTINEL}).encode(),
                "backend_read_failed",
            ),
            (
                200,
                json.dumps({"hits": [PRIVATE_SENTINEL]}).encode(),
                "backend_read_failed",
            ),
            (200, json.dumps({"hits": [{}, {}]}).encode(), "backend_read_failed"),
            (
                200,
                json.dumps({"hits": [], "padding": PRIVATE_SENTINEL * 100}).encode(),
                "backend_read_failed",
            ),
        )
        with patch.object(telemetry_investigation, "MAX_RESPONSE_BYTES", 512):
            for code, response, reason in cases:
                with self.subTest(
                    code=code, reason=reason, response_size=len(response)
                ):
                    self.routes[self.search] = (code, response)
                    answer = self.lookup(limit=1)
                    self.assertEqual(answer["status"], "unavailable")
                    self.assertEqual(answer["reason"], reason)
                    self.assertEqual(answer["records"], [])
                    self.assertFalse(answer["complete"])
                    self.assertNotIn(PRIVATE_SENTINEL, json.dumps(answer))

    def test_missing_service_health_is_passive_and_preserves_unknown_meaning(
        self,
    ) -> None:
        with ExitStack() as stack:
            for name in (
                "setup",
                "install",
                "start",
                "restart",
                "enable",
                "_systemctl",
            ):
                stack.enter_context(
                    patch.object(
                        telemetry_service,
                        name,
                        side_effect=AssertionError("passive health performed effects"),
                    )
                )
            stack.enter_context(
                patch.object(
                    telemetry_process,
                    "bootstrap",
                    side_effect=AssertionError("passive health initialized SDK"),
                )
            )
            stack.enter_context(patch("scripts.telemetry_process._STATE.active", None))
            answer = telemetry.health(
                self.state, environment={"PSE_TELEMETRY_STATE": str(self.state)}
            )
        self.assertEqual(
            answer["pipeline"],
            {"status": "unavailable", "reason": "telemetry_not_configured"},
        )
        self.assertEqual(mapping(answer["producer"])["status"], "not_initialized")
        self.assertEqual(mapping(answer["producer"])["queue_occupancy"], "unavailable")
        self.assertEqual(mapping(answer["delivery"])["backend_acceptance"], "unknown")
        self.assertEqual(
            mapping(answer["delivery"])["query_visibility"], "not_exercised"
        )
        self.assertEqual(self.requests, [])
        self.assertFalse(self.state.exists())

    def test_unowned_listener_cannot_become_selected_pipeline_evidence(self) -> None:
        self.services["collector"]["ownership"] = "unavailable"
        self.services["openobserve"]["process"] = None
        answer = self.observe_health()
        stages = mapping(answer["pipeline"])
        self.assertEqual(mapping(stages["collector"])["status"], "unavailable")
        self.assertEqual(mapping(stages["backend"])["status"], "unavailable")
        self.assertNotIn("collector_delivery", stages)
        self.assertEqual(self.requests, [])

    def test_trace_lookup_refuses_unowned_backend_before_credentials_or_http(
        self,
    ) -> None:
        for backend in (
            {"ownership": "verified", "process": None},
            {"ownership": "unavailable", "process": {"pid": 999}},
            None,
        ):
            with (
                self.subTest(backend=backend),
                patch.object(
                    telemetry_service,
                    "inspect",
                    return_value={
                        **self.inspected,
                        "services": {"openobserve": backend},
                    },
                ),
                patch.object(telemetry_service, "backend_access") as credentials,
                patch.object(telemetry_query, "request") as request,
            ):
                answer = telemetry_query.lookup_trace(
                    self.state, TRACE_ID, start_time_us=100, end_time_us=200
                )
                self.assertEqual(answer["status"], "unavailable")
                self.assertEqual(
                    answer["reason"], "selected_backend_not_active_or_owned"
                )
                self.assertEqual(answer["records"], [])
                credentials.assert_not_called()
                request.assert_not_called()

    def test_trace_lookup_discards_records_after_backend_restarts(self) -> None:
        restarted = {
            **self.inspected,
            "services": {
                "openobserve": {**self.services["openobserve"], "process": {"pid": 999}}
            },
        }
        with (
            patch.object(
                telemetry_service, "inspect", side_effect=[self.inspected, restarted]
            ),
            patch.object(
                telemetry_service, "backend_access", return_value=(self.base, {})
            ),
            patch.object(
                telemetry_query,
                "request",
                return_value=(200, b'{"hits":[{"trace_id":"foreign"}]}'),
            ),
        ):
            answer = telemetry_query.lookup_trace(
                self.state, TRACE_ID, start_time_us=100, end_time_us=200
            )
        self.assertEqual(answer["status"], "stale")
        self.assertEqual(answer["reason"], "selected_backend_changed_during_read")
        self.assertEqual(answer["records"], [])

    def test_total_deadline_bounds_slow_headers_and_body(self) -> None:
        def drip_handler(
            mode: str, stopping: threading.Event
        ) -> type[BaseHTTPRequestHandler]:
            class Drip(BaseHTTPRequestHandler):
                def do_GET(self) -> None:
                    data = (
                        b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\n"
                        if mode == "headers"
                        else b"x" * 1000
                    )
                    try:
                        if mode == "body":
                            self.send_response(200)
                            self.send_header("Content-Length", "1000")
                            self.end_headers()
                        for byte in data:
                            self.wfile.write(bytes([byte]))
                            self.wfile.flush()
                            if stopping.wait(0.03):
                                break
                    except (OSError, ConnectionError):
                        pass

                def log_message(self, format: str, *_arguments: object) -> None:
                    pass

            return Drip

        for mode in ("headers", "body"):
            stopping = threading.Event()
            server = ThreadingHTTPServer(("127.0.0.1", 0), drip_handler(mode, stopping))
            server.daemon_threads = True
            thread = threading.Thread(target=server.serve_forever, daemon=True)
            thread.start()
            try:
                started = time.monotonic()
                with (
                    patch.object(telemetry_investigation, "TIMEOUT_SECONDS", 0.15),
                    self.assertRaises(OSError),
                ):
                    telemetry_query.request(f"http://127.0.0.1:{server.server_port}/")
                self.assertLess(time.monotonic() - started, 1)
            finally:
                stopping.set()
                server.shutdown()
                server.server_close()
                thread.join(2)

    def test_failed_health_stage_does_not_erase_independent_samples_or_identity(
        self,
    ) -> None:
        self.routes["/healthz"] = (503, PRIVATE_SENTINEL.encode())
        self.routes["/"] = (200, b"ok")
        metrics = """# TYPE otelcol_exporter_sent_spans_total counter
    otelcol_exporter_sent_spans_total{exporter="otlphttp/openobserve",service_instance_id="collector-incarnation"} 3
    otelcol_exporter_queue_size{exporter="otlphttp/openobserve"} 2
    otelcol_receiver_refused_spans_total{receiver="otlp"} NaN
    unrelated_metric{secret="not-selected"} 999
"""
        self.routes["/metrics"] = (200, metrics.encode())
        answer = self.observe_health()
        stages = mapping(answer["pipeline"])
        self.assertEqual(mapping(stages["backend"])["status"], "refused")
        self.assertEqual(mapping(stages["collector"])["status"], "responding")
        delivery = mapping(stages["collector_delivery"])
        self.assertEqual(delivery["status"], "observed")
        self.assertEqual(delivery["configuration_id"], "fixture-generation")
        self.assertEqual(delivery["process"], self.services["collector"]["process"])
        self.assertEqual(len(records(delivery["samples"])), 2)
        samples = {
            str(sample["name"]): sample for sample in records(delivery["samples"])
        }
        self.assertEqual(samples["otelcol_exporter_sent_spans_total"]["value"], 3)
        self.assertEqual(
            mapping(samples["otelcol_exporter_sent_spans_total"]["labels"])[
                "service_instance_id"
            ],
            "collector-incarnation",
        )
        self.assertEqual(
            delivery["query_visibility"], "not_established_by_exporter_counters"
        )
        self.assertEqual(mapping(answer["delivery"])["backend_acceptance"], "unknown")
        self.assertEqual(
            mapping(answer["delivery"])["query_visibility"], "not_exercised"
        )
        self.assertNotIn(PRIVATE_SENTINEL, json.dumps(answer))
        self.assertNotIn("unrelated_metric", json.dumps(answer))

    def test_collector_metrics_unavailable_and_series_bound_are_explicit(self) -> None:
        self.routes["/healthz"] = (200, b"ok")
        self.routes["/"] = (503, b"unavailable")
        cases = (
            (503, PRIVATE_SENTINEL.encode()),
            (200, b"\xff"),
            (200, b"otelcol_exporter_sent_spans_total{bad syntax 3"),
            (
                200,
                "\n".join(
                    f'otelcol_exporter_sent_spans_total{{worker="{index}"}} 1'
                    for index in range(257)
                ).encode(),
            ),
        )
        for code, response in cases:
            with self.subTest(code=code, bytes=len(response)):
                self.routes["/metrics"] = (code, response)
                answer = self.observe_health()
                stages = mapping(answer["pipeline"])
                self.assertEqual(mapping(stages["backend"])["status"], "responding")
                self.assertEqual(mapping(stages["collector"])["status"], "refused")
                self.assertEqual(
                    mapping(stages["collector_delivery"])["status"], "unavailable"
                )
                self.assertNotIn("samples", mapping(stages["collector_delivery"]))
                self.assertEqual(
                    mapping(answer["delivery"])["backend_acceptance"], "unknown"
                )
                self.assertNotIn(PRIVATE_SENTINEL, json.dumps(answer))

    def test_cli_invalid_lookup_has_useful_safe_diagnostic(self) -> None:
        stream = io.StringIO()
        with (
            redirect_stdout(stream),
            patch.object(
                telemetry_service,
                "inspect",
                side_effect=AssertionError("invalid CLI selected backend"),
            ),
        ):
            code = telemetry.main(
                [
                    "lookup-trace",
                    "--state",
                    str(self.state),
                    "--trace-id",
                    PRIVATE_SENTINEL,
                    "--start-us",
                    "100",
                    "--end-us",
                    "200",
                ]
            )
        self.assertEqual(code, 1)
        answer = json.loads(stream.getvalue())
        self.assertEqual(answer["code"], "pse.telemetry.operation_refused")
        self.assertEqual(answer["operation"], "lookup-trace")
        self.assertIn("native results", answer["remedy"])
        self.assertNotIn(PRIVATE_SENTINEL, stream.getvalue())

    def test_cli_missing_service_health_is_successful_diagnostic_read(self) -> None:
        stream = io.StringIO()
        with (
            redirect_stdout(stream),
            patch.object(
                telemetry_process,
                "bootstrap",
                side_effect=AssertionError("CLI health initialized SDK"),
            ),
        ):
            code = telemetry.main(["health", "--state", str(self.state)])
        self.assertEqual(code, 0)
        answer = json.loads(stream.getvalue())
        self.assertEqual(
            mapping(answer["pipeline"])["reason"], "telemetry_not_configured"
        )
        self.assertEqual(mapping(answer["delivery"])["backend_acceptance"], "unknown")
        self.assertFalse(self.state.exists())


if __name__ == "__main__":
    unittest.main()

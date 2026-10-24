# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pure telemetry selection tests: no SDK, native extension or live services."""

# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.telemetry_config import resolve_process


class ConfigurationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.state = Path(self.temporary.name)
        self.env = {"PSE_TELEMETRY_STATE": str(self.state)}

    def test_no_selection_disables_without_importing_sdk_or_native(self) -> None:
        settings = resolve_process("test", environment=self.env)
        self.assertTrue(settings.disabled)
        self.assertFalse(settings.traces.enabled)
        import sys
        self.assertNotIn("pse._native", sys.modules)
        self.assertEqual(settings.effective()["delivery"], "not_observed")

    def test_selection_precedence_none_and_redaction(self) -> None:
        (self.state / "configuration.json").write_text(json.dumps({"schema_version": 1, "otlp_http_endpoint": "http://127.0.0.1:4318", "configuration_id": "fixture"}))
        settings = resolve_process("test", environment={**self.env, "OTEL_EXPORTER_OTLP_ENDPOINT": "https://example.test/secret-path", "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT": "https://trace.test/private", "OTEL_EXPORTER_OTLP_HEADERS": "Authorization=secret-token", "OTEL_EXPORTER_OTLP_TRACES_HEADERS": "x-specific=trace-secret", "OTEL_EXPORTER_OTLP_TIMEOUT": "0.5", "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT": "0.1", "OTEL_LOGS_EXPORTER": "none", "OTEL_RESOURCE_ATTRIBUTES": "service.instance.id=spoofed,secret=private-resource"})
        self.assertEqual(settings.traces.endpoint, "https://trace.test/private")
        self.assertEqual(settings.metrics.endpoint, "https://example.test/secret-path/v1/metrics")
        self.assertEqual(settings.traces.headers, (("x-specific", "trace-secret"),))
        self.assertEqual(settings.traces.timeout_ms, 100)
        self.assertEqual(settings.metrics.timeout_ms, 500)
        self.assertFalse(settings.logs.enabled)
        rendered = json.dumps(settings.effective())
        for secret in ("private-resource", "secret-token", "trace-secret", "secret-path", "/private", "spoofed"):
            self.assertNotIn(secret, rendered)
        self.assertNotEqual(dict(settings.resource)["service.instance.id"], "spoofed")
        self.assertEqual(settings.native_settings()["process_incarnation"], dict(settings.resource)["service.instance.id"])

    def test_configuration_is_explicit_selection_but_disabled_wins(self) -> None:
        (self.state / "configuration.json").write_text(json.dumps({"schema_version": 1, "otlp_http_endpoint": "http://127.0.0.1:4318"}))
        self.assertTrue(resolve_process("test", environment=self.env).traces.enabled)
        disabled = resolve_process("test", environment={**self.env, "OTEL_SDK_DISABLED": "true"})
        self.assertTrue(disabled.disabled)
        self.assertFalse(disabled.traces.enabled)
        self.assertIsNone(disabled.traces.endpoint)

    def test_unsupported_transport_and_unbounded_queue_refused(self) -> None:
        for values in ({"OTEL_EXPORTER_OTLP_PROTOCOL": "grpc"}, {"OTEL_BSP_MAX_QUEUE_SIZE": "0"}, {"OTEL_BSP_MAX_QUEUE_SIZE": "0.5"}, {"OTEL_BSP_MAX_QUEUE_SIZE": "999999999"}, {"OTEL_BSP_MAX_QUEUE_SIZE": "1", "OTEL_BSP_MAX_EXPORT_BATCH_SIZE": "2"}, {"OTEL_EXPORTER_OTLP_TIMEOUT": "nan"}):
            with self.subTest(values=values), self.assertRaises(ValueError):
                resolve_process("test", environment={**self.env, "OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:4318", **values})

    def test_instances_stable_per_process_and_distinct_in_actual_child(self) -> None:
        first = resolve_process("first", environment=self.env)
        second = resolve_process("second", environment=self.env)
        self.assertEqual(dict(first.resource)["service.instance.id"], dict(second.resource)["service.instance.id"])
        import sys
        child = subprocess.run([sys.executable, "-c", "from scripts.telemetry_config import process_incarnation; print(process_incarnation())"], check=True, capture_output=True, text=True)
        self.assertNotEqual(child.stdout.strip(), dict(first.resource)["service.instance.id"])
        with patch("scripts.telemetry_config.os.getpid", return_value=os.getpid() + 1):
            self.assertNotEqual(dict(resolve_process("child", environment=self.env).resource)["service.instance.id"], dict(first.resource)["service.instance.id"])


if __name__ == "__main__":
    unittest.main()

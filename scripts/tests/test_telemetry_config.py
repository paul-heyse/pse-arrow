# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pure telemetry selection tests: no SDK, native extension or live services."""

# ruff: noqa: PT009, PT027 -- stdlib unittest controls

from __future__ import annotations

import json
import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import msgspec

from scripts.generated.telemetry_settings import Settings
from scripts.telemetry_config import resolve_process


class ConfigurationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.state = Path(self.temporary.name)
        self.env = {"PSE_TELEMETRY_STATE": str(self.state)}

    def test_no_selection_disables_without_importing_sdk_or_native(self) -> None:
        # Other controls legitimately import the installed extension. A fresh
        # process tests bootstrap imports independently of suite ordering.
        child = subprocess.run(
            [
                sys.executable,
                "-c",
                (
                    "import json,sys; from scripts.telemetry_config import resolve_process; "
                    "s=resolve_process('test',environment={'PSE_TELEMETRY_STATE':sys.argv[1]}); "
                    "print(json.dumps({'disabled':s.wire.disabled,'traces':s.wire.traces.enabled,"
                    "'delivery':s.effective()['delivery'],'native':'pse._native' in sys.modules,"
                    "'sdk':any(n.startswith('opentelemetry.sdk') for n in sys.modules)}))"
                ),
                str(self.state),
            ],
            capture_output=True,
            text=True,
            check=False,
            timeout=15,
        )
        self.assertEqual(child.returncode, 0, child.stderr)
        self.assertEqual(
            json.loads(child.stdout),
            {
                "disabled": True,
                "traces": False,
                "delivery": "not_observed",
                "native": False,
                "sdk": False,
            },
        )

    def test_selection_precedence_none_and_redaction(self) -> None:
        (self.state / "configuration.json").write_text(
            json.dumps(
                {
                    "schema_version": 1,
                    "otlp_http_endpoint": "http://127.0.0.1:4318",
                    "configuration_id": "fixture",
                }
            )
        )
        settings = resolve_process(
            "test",
            environment={
                **self.env,
                "OTEL_EXPORTER_OTLP_ENDPOINT": "https://example.test/secret-path",
                "OTEL_EXPORTER_OTLP_TRACES_ENDPOINT": "https://trace.test/private",
                "OTEL_EXPORTER_OTLP_HEADERS": "Authorization=secret-token",
                "OTEL_EXPORTER_OTLP_TRACES_HEADERS": "x-specific=trace-secret",
                "OTEL_EXPORTER_OTLP_TIMEOUT": "0.5",
                "OTEL_EXPORTER_OTLP_TRACES_TIMEOUT": "0.1",
                "OTEL_LOGS_EXPORTER": "none",
                "OTEL_RESOURCE_ATTRIBUTES": "service.instance.id=spoofed,secret=private-resource",
            },
        )
        self.assertEqual(settings.wire.traces.endpoint, "https://trace.test/private")
        self.assertEqual(
            settings.wire.metrics.endpoint,
            "https://example.test/secret-path/v1/metrics",
        )
        self.assertEqual(settings.wire.traces.headers, {"x-specific": "trace-secret"})
        self.assertEqual(settings.wire.traces.timeout_ms, 100)
        self.assertEqual(settings.wire.metrics.timeout_ms, 500)
        self.assertFalse(settings.wire.logs.enabled)
        rendered = json.dumps(settings.effective())
        for secret in (
            "private-resource",
            "secret-token",
            "trace-secret",
            "secret-path",
            "/private",
            "spoofed",
        ):
            self.assertNotIn(secret, rendered)
        self.assertNotEqual(
            dict(settings.wire.resource)["service.instance.id"], "spoofed"
        )
        self.assertEqual(
            json.loads(settings.native_json())["process_incarnation"],
            dict(settings.wire.resource)["service.instance.id"],
        )

    def test_generated_private_wire_and_representations_preserve_redaction(
        self,
    ) -> None:
        settings = resolve_process(
            "test",
            environment={
                **self.env,
                "OTEL_EXPORTER_OTLP_ENDPOINT": "https://example.test/private-secret-path",
                "OTEL_EXPORTER_OTLP_HEADERS": "Authorization=credential-secret",
                "OTEL_RESOURCE_ATTRIBUTES": "private=resource-secret",
            },
        )
        wire = settings.native_json()
        decoded = msgspec.json.decode(wire, type=Settings)
        self.assertEqual(decoded, settings.wire)
        self.assertEqual(set(json.loads(wire)), set(Settings.__struct_fields__))
        self.assertNotIn("selection", json.loads(wire))
        self.assertNotIn("configuration_id", json.loads(wire))
        for secret in ("private-secret-path", "credential-secret", "resource-secret"):
            self.assertIn(secret, wire)
            self.assertNotIn(secret, repr(settings))
            self.assertNotIn(secret, repr(settings.wire.traces))
        field_name = "disabled"
        with self.assertRaises(AttributeError):
            setattr(settings.wire, field_name, True)
        detached = settings.wire
        detached.resource["service.instance.id"] = "mutated"
        detached.traces.headers["Authorization"] = "mutated"
        self.assertEqual(settings.native_json(), wire)
        self.assertEqual(settings.wire, decoded)

    def test_invalid_environment_errors_do_not_render_values(self) -> None:
        for key in (
            "OTEL_EXPORTER_OTLP_TIMEOUT",
            "OTEL_TRACES_SAMPLER_ARG",
            "OTEL_EXPORTER_OTLP_ENDPOINT",
        ):
            with self.subTest(key=key), self.assertRaises(ValueError) as caught:
                resolve_process(
                    "test", environment={**self.env, key: "credential-secret"}
                )
            self.assertNotIn("credential-secret", str(caught.exception))

    def test_settings_document_bound_precedes_sdk_composition(self) -> None:
        with self.assertRaisesRegex(ValueError, "64 KiB"):
            resolve_process(
                "test", environment={**self.env, "OTEL_SERVICE_NAME": "x" * 65536}
            )

    def test_source_only_bootstrap_imports_no_scientific_or_sdk_package(self) -> None:
        subprocess.run(
            [
                sys.executable,
                "-c",
                "from scripts.telemetry_config import resolve_process; import sys; resolve_process('test', environment={'PSE_TELEMETRY_STATE':'/nonexistent-pse-telemetry-test'}); assert not any(name == 'pse' or name.startswith(('pse.', 'opentelemetry')) for name in sys.modules)",
            ],
            check=True,
            capture_output=True,
            text=True,
        )

    def test_configuration_is_explicit_selection_but_disabled_wins(self) -> None:
        (self.state / "configuration.json").write_text(
            json.dumps(
                {"schema_version": 1, "otlp_http_endpoint": "http://127.0.0.1:4318"}
            )
        )
        self.assertTrue(
            resolve_process("test", environment=self.env).wire.traces.enabled
        )
        disabled = resolve_process(
            "test", environment={**self.env, "OTEL_SDK_DISABLED": "true"}
        )
        self.assertTrue(disabled.wire.disabled)
        self.assertFalse(disabled.wire.traces.enabled)
        self.assertIsNone(disabled.wire.traces.endpoint)

    def test_unsupported_transport_and_unbounded_queue_refused(self) -> None:
        for values in (
            {"OTEL_EXPORTER_OTLP_PROTOCOL": "grpc"},
            {"OTEL_BSP_MAX_QUEUE_SIZE": "0"},
            {"OTEL_BSP_MAX_QUEUE_SIZE": "0.5"},
            {"OTEL_BSP_MAX_QUEUE_SIZE": "999999999"},
            {"OTEL_BSP_MAX_QUEUE_SIZE": "1", "OTEL_BSP_MAX_EXPORT_BATCH_SIZE": "2"},
            {"OTEL_EXPORTER_OTLP_TIMEOUT": "nan"},
        ):
            with self.subTest(values=values), self.assertRaises(ValueError):
                resolve_process(
                    "test",
                    environment={
                        **self.env,
                        "OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:4318",
                        **values,
                    },
                )

    def test_instances_stable_per_process_and_distinct_in_actual_child(self) -> None:
        first = resolve_process("first", environment=self.env)
        second = resolve_process("second", environment=self.env)
        self.assertEqual(
            dict(first.wire.resource)["service.instance.id"],
            dict(second.wire.resource)["service.instance.id"],
        )
        child = subprocess.run(
            [
                sys.executable,
                "-c",
                "from scripts.telemetry_config import process_incarnation; print(process_incarnation())",
            ],
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertNotEqual(
            child.stdout.strip(), dict(first.wire.resource)["service.instance.id"]
        )
        with patch("scripts.telemetry_config.os.getpid", return_value=os.getpid() + 1):
            self.assertNotEqual(
                dict(resolve_process("child", environment=self.env).wire.resource)[
                    "service.instance.id"
                ],
                dict(first.wire.resource)["service.instance.id"],
            )


if __name__ == "__main__":
    unittest.main()

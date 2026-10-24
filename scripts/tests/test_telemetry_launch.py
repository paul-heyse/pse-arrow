# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Passive native launch selection through both existing environment owners."""

# ruff: noqa: PT009 -- stdlib unittest controls
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import pse_env, surreal_server, telemetry_launch


class TelemetryLaunchTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.environment = {
            "PSE_TELEMETRY_STATE": str(self.root / "unselected"),
            telemetry_launch.SETTINGS: "stale-private-settings",
        }

    def test_unselected_and_malformed_configuration_cannot_reuse_stale_settings(
        self,
    ) -> None:
        telemetry_launch.configure(self.root, self.environment)
        self.assertEqual(self.environment[telemetry_launch.STATUS], "disabled")
        self.assertNotIn(telemetry_launch.SETTINGS, self.environment)
        self.environment.update(
            {
                telemetry_launch.SETTINGS: "stale",
                "OTEL_EXPORTER_OTLP_ENDPOINT": "invalid",
            }
        )
        telemetry_launch.configure(self.root, self.environment)
        self.assertEqual(
            self.environment[telemetry_launch.STATUS], "initialization_refused"
        )
        self.assertNotIn(telemetry_launch.SETTINGS, self.environment)

    def test_base_environment_resolves_caller_over_local_and_keeps_carrier_private(
        self,
    ) -> None:
        caller = {
            **self.environment,
            "OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:4318",
            "OTEL_EXPORTER_OTLP_HEADERS": "authorization=fixture-secret",
            "OTEL_LOGS_EXPORTER": "none",
        }
        selected = pse_env.base_environment(
            self.root, caller, {"OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:9999"}
        )
        settings = json.loads(selected[telemetry_launch.SETTINGS])
        self.assertEqual(
            settings["traces"]["endpoint"], "http://127.0.0.1:4318/v1/traces"
        )
        self.assertEqual(
            settings["traces"]["headers"], {"authorization": "fixture-secret"}
        )
        self.assertFalse(settings["logs"]["enabled"])
        with (
            patch.object(pse_env, "compose", return_value=selected),
            patch.object(pse_env, "local_keys", return_value={}),
        ):
            rendered = pse_env.render(self.root, caller)
        self.assertNotIn(telemetry_launch.SETTINGS, rendered)
        self.assertNotIn("fixture-secret", rendered)

    def test_worker_uses_same_resolver_without_changing_scientific_allocation(
        self,
    ) -> None:
        inherited = {
            **self.environment,
            "OTEL_EXPORTER_OTLP_ENDPOINT": "http://127.0.0.1:4318",
            "PSE_MEMORY_MAX": "stale",
            "PSE_NATIVE_OPERATION": "stale",
        }
        with patch.object(
            surreal_server, "systemd_environment", return_value=inherited
        ):
            selected = surreal_server.worker_environment(
                self.root,
                3,
                {"native_worker_memory_bytes": 123456},
                self.root / "handoff",
            )
        self.assertEqual(selected[telemetry_launch.STATUS], "configured")
        self.assertEqual(selected["PSE_NATIVE_WORKER_SLOT"], "3")
        self.assertEqual(selected["PSE_NATIVE_WORKER_MEMORY_BYTES"], "123456")
        self.assertEqual(selected["PSE_NATIVE_HANDOFF"], str(self.root / "handoff"))
        self.assertNotIn("PSE_MEMORY_MAX", selected)
        self.assertNotIn("PSE_NATIVE_OPERATION", selected)
        self.assertEqual(
            json.loads(selected[telemetry_launch.SETTINGS])["traces"]["endpoint"],
            "http://127.0.0.1:4318/v1/traces",
        )


if __name__ == "__main__":
    unittest.main()

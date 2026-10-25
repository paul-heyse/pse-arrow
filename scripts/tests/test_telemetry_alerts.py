# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Native alert fault states and explicit control boundaries."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import unittest
from unittest.mock import patch

from scripts import telemetry_alerts as alerts


class TelemetryAlertsTests(unittest.TestCase):
    def test_no_observation_never_implies_healthy(self) -> None:
        now = 1_000_000_000
        enabled = {"enabled": True}
        self.assertEqual(alerts.evaluation_status(None, now, [], now), "missing")
        self.assertEqual(
            alerts.evaluation_status({"enabled": False}, now, [], now), "disabled"
        )
        self.assertEqual(
            alerts.evaluation_status(enabled, None, [], now), "missing_data"
        )
        self.assertEqual(
            alerts.evaluation_status(enabled, now - 121_000_000, [], now), "stale"
        )
        self.assertEqual(
            alerts.evaluation_status(enabled, now, [], now), "evaluation_unavailable"
        )
        self.assertEqual(
            alerts.evaluation_status(
                enabled, now, [{"timestamp": now - 121_000_000}], now
            ),
            "backlogged",
        )
        self.assertEqual(
            alerts.evaluation_status(
                enabled, now, [{"timestamp": now, "status": "Error"}], now
            ),
            "evaluation_failed",
        )
        self.assertEqual(
            alerts.evaluation_status(
                enabled, now, [{"timestamp": now, "status": "Completed"}], now
            ),
            "evaluated",
        )
        self.assertEqual(
            alerts.evaluation_status(
                enabled | {"last_triggered_at": now},
                now,
                [{"timestamp": now - 180_000_000}],
                now,
            ),
            "evaluated_history_pending",
        )

    def test_backend_unavailable_read_has_no_control_effect(self) -> None:
        with (
            patch.object(
                alerts._Selection,  # noqa: SLF001 -- Exercise the native alert incarnation fence and refusal boundary.
                "open",
                side_effect=alerts.AlertError("backend unavailable"),
            ),
            patch.object(
                alerts.telemetry_service,
                "start",
                side_effect=AssertionError("implicit start"),
            ),
        ):
            result = alerts.read(start_time_us=1, end_time_us=2)
        self.assertEqual(result["status"], "unavailable")
        self.assertFalse(result["complete"])
        self.assertEqual(result["records"], [])

    def test_invalid_interval_or_limit_precedes_backend_access(self) -> None:
        with patch.object(
            alerts._Selection,  # noqa: SLF001 -- Exercise the native alert incarnation fence and refusal boundary.
            "open",
            side_effect=AssertionError("unexpected backend access"),
        ):
            for start, end, limit in (
                (0, 0, 1),
                (2, 1, 1),
                (0, alerts.MAX_INTERVAL_US + 1, 1),
                (1, 2, 1001),
                (True, 2, 1),
            ):
                with (
                    self.subTest(start=start, end=end, limit=limit),
                    self.assertRaises(ValueError),
                ):
                    alerts.read(start_time_us=start, end_time_us=end, limit=limit)

    def test_changed_backend_process_refuses_delivery_and_control(self) -> None:
        image = {
            "configuration_id": "a",
            "services": {"openobserve": {"process": {"pid": 3}}},
        }
        selected = alerts._Selection(  # noqa: SLF001 -- Exercise the native alert incarnation fence and refusal boundary.
            None, None, image, "http://127.0.0.1:5080", {"Authorization": "private"}
        )
        changed = {
            "configuration_id": "a",
            "services": {"openobserve": {"process": {"pid": 4}}},
        }
        with (
            patch.object(alerts.telemetry_service, "inspect", return_value=changed),
            patch.object(
                alerts.telemetry_query,
                "request",
                side_effect=AssertionError("request after process change"),
            ),
            self.assertRaisesRegex(alerts.AlertError, "incarnation changed"),
        ):
            selected.api("/api/default/alerts/templates", {"name": "selected"})

    def test_native_error_does_not_echo_credentials_or_backend_body(self) -> None:
        image = {
            "configuration_id": "a",
            "services": {"openobserve": {"process": {"pid": 3}}},
        }
        selected = alerts._Selection(  # noqa: SLF001 -- Exercise the native alert incarnation fence and refusal boundary.
            None, None, image, "http://127.0.0.1:5080", {"Authorization": "secret"}
        )
        with (
            patch.object(alerts.telemetry_service, "inspect", return_value=image),
            patch.object(
                alerts.telemetry_query,
                "request",
                return_value=(400, b"secret backend diagnostic"),
            ),
            self.assertRaises(alerts.AlertError) as result,
        ):
            selected.api("/api/default/alerts/templates", {"name": "selected"})
        self.assertNotIn("secret", str(result.exception))


if __name__ == "__main__":
    unittest.main()

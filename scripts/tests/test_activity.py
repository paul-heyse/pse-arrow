# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Read-only observation distinguishes unavailable, partial and empty results."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import contextlib
import io
import json
import os
import subprocess
import unittest
from unittest.mock import patch

from scripts import activity


class ActivityTests(unittest.TestCase):
    def observe(
        self, responses: list[object], *, structured: bool = True
    ) -> tuple[int, str, str]:
        output = io.StringIO()
        errors = io.StringIO()
        with (
            patch.object(activity, "systemctl", side_effect=responses),
            patch.object(
                activity,
                "build_processes",
                return_value=[(42, "cargo", "g", "cargo check")],
            ),
            patch.object(
                activity.pse_env, "limits", return_value=[("pse.slice", "123")]
            ),
            patch.object(
                activity.pse_env.host,
                "acquire",
                side_effect=AssertionError("admission called"),
            ),
            patch.object(
                activity.pse_env.build_environment,
                "configure",
                side_effect=AssertionError("compiler preparation called"),
            ),
            contextlib.redirect_stdout(output),
            contextlib.redirect_stderr(errors),
        ):
            status = activity.main(["--json"] if structured else [])
        return (
            status,
            output.getvalue(),
            errors.getvalue(),
        )

    def test_successfully_empty_units(self) -> None:
        status, output, _ = self.observe(["[]"])
        result = json.loads(output)
        self.assertEqual(status, 0)
        self.assertEqual(result["units"], [])
        self.assertTrue(result["units_available"])

    def test_failure_timeout_and_malformed_units_preserve_independent_observations(
        self,
    ) -> None:
        for response in (
            ValueError("manager failed"),
            subprocess.TimeoutExpired("systemctl", 10),
            "not-json",
            "{}",
            "",
            '[{"unit":"pse-x.scope"}]',
        ):
            with self.subTest(response=response):
                status, output, _ = self.observe([response])
                result = json.loads(output)
                self.assertEqual(status, 1)
                self.assertIsNone(result["units"])
                self.assertFalse(result["units_available"])
                self.assertEqual(result["build_processes"][0]["pid"], 42)
                self.assertEqual(result["aggregate_limits"], [["pse.slice", "123"]])
                self.assertTrue(result["observation_errors"])

    def test_partial_disappearance_retains_other_unit(self) -> None:
        properties = "MemoryCurrent=0\nCPUUsageNSec=0\nActiveEnterTimestampMonotonic=0\nControlGroup=/g\nSlice=pse.slice\nLoadState=loaded\nActiveState=active\n"
        status, output, _ = self.observe(
            [
                '[{"unit":"pse-first.scope","active":"active"},{"unit":"pse-gone.scope","active":"active"}]',
                properties,
                properties.replace("LoadState=loaded", "LoadState=not-found"),
            ]
        )
        result = json.loads(output)
        self.assertEqual(status, 1)
        self.assertFalse(result["units_available"])
        self.assertEqual(
            [unit["unit"] for unit in result["units"]], ["pse-first.scope"]
        )
        self.assertIn("disappeared", result["observation_errors"][0])

    def test_transport_return_code_is_checked(self) -> None:
        with (
            patch.object(
                activity.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 1, stdout="[]"),
            ),
            self.assertRaisesRegex(ValueError, "failed"),
        ):
            activity.systemctl("list-units")

    def test_manager_address_defaults_without_workload_preparation(self) -> None:
        with (
            patch.dict(os.environ, {}, clear=True),
            patch.object(
                activity.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0, stdout="[]"),
            ) as run,
            patch.object(
                activity.pse_env.build_environment,
                "configure",
                side_effect=AssertionError("compiler"),
            ),
            patch.object(
                activity.pse_env.host,
                "acquire",
                side_effect=AssertionError("admission"),
            ),
        ):
            self.assertEqual(activity.systemctl("list-units"), "[]")
        self.assertIn("DBUS_SESSION_BUS_ADDRESS", run.call_args.kwargs["env"])
        self.assertIn("XDG_RUNTIME_DIR", run.call_args.kwargs["env"])

    def test_text_failure_does_not_claim_empty_units(self) -> None:
        status, output, errors = self.observe(
            [ValueError("manager failed")], structured=False
        )
        self.assertEqual(status, 1)
        self.assertNotIn("no pse-* units", output)
        self.assertIn("units unavailable", errors)
        self.assertIn("cargo check", output)


if __name__ == "__main__":
    unittest.main()

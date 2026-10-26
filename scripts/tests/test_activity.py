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
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import operation_lifetime
from scripts import activity


class ActivityTests(unittest.TestCase):
    def test_admission_explanation_is_detached_and_preserves_parked_intent(
        self,
    ) -> None:
        output = io.StringIO()
        snapshot = {
            "version": 1,
            "owners": {"a" * 32: {"class": "reference"}},
            "parked_services": ["/fixture/service"],
        }
        with (
            patch.object(activity, "units", return_value=([], [])),
            patch.object(activity, "build_processes", return_value=[]),
            patch.object(activity.pse_env, "limits", return_value=[]),
            patch.object(
                activity.host_admission, "readonly_snapshot", return_value=snapshot
            ),
            patch.object(
                activity.host_admission,
                "admission_observations",
                return_value=[
                    {
                        "status": "waiting",
                        "reason": "exclusive lane occupied",
                        "age_seconds": 3,
                    }
                ],
            ),
            patch.object(
                activity.host_admission,
                "reconcile",
                side_effect=AssertionError("reconcile"),
            ),
            patch.object(
                activity.host_admission, "status", side_effect=AssertionError("status")
            ),
            patch.object(
                activity.host_admission,
                "acquire",
                side_effect=AssertionError("acquire"),
            ),
            patch.object(
                activity.pse_env.build_environment,
                "configure",
                side_effect=AssertionError("compiler"),
            ),
            contextlib.redirect_stdout(output),
        ):
            self.assertEqual(
                activity.main(["--json", "--admission", "--owner", "a" * 32]), 0
            )
        result = json.loads(output.getvalue())["explanation"]
        self.assertEqual(result["mode"], "detached-no-write")
        self.assertEqual(result["parked_services"], ["/fixture/service"])
        self.assertEqual(result["attempts"][0]["age_seconds"], 3)
        self.assertEqual(
            result["inotify"]["max_user_watches"]["remaining_capacity"], "unknown"
        )

    def test_empty_scope_retains_manager_lifetime_explanation(self) -> None:
        fields = {
            "InvocationID": "a" * 32,
            "ControlGroup": "/empty",
            "LoadState": "loaded",
            "ActiveState": "active",
        }
        with (
            patch.object(
                operation_lifetime, "unit_observation", return_value=fields
            ),
            patch.object(operation_lifetime, "populated", return_value=False),
        ):
            observed, errors = activity.units(["pse-empty.scope"])
        self.assertEqual(errors, [])
        assert observed is not None
        self.assertEqual(observed[0]["population"], "empty")
        self.assertIn("user manager", observed[0]["wait_explanation"])

    def test_checkpoint_is_atomic_throttled_and_keeps_original_deadline(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary) / "host"
            deadline = time.monotonic() + 10
            checkpoint = activity.host_admission.AdmissionCheckpoint(
                directory, activity.host_admission.select("reference"), deadline
            )
            checkpoint.publish(
                "waiting", "exclusive occupied", {"a" * 32: {"class": "reference"}}
            )
            before = checkpoint.path.read_bytes(), checkpoint.path.stat().st_mtime_ns
            checkpoint.publish(
                "waiting", "exclusive occupied", {"a" * 32: {"class": "reference"}}
            )
            self.assertEqual(
                before,
                (checkpoint.path.read_bytes(), checkpoint.path.stat().st_mtime_ns),
            )
            checkpoint.publish("admitted", None, allocation="b" * 32)
            image = activity.host_admission.admission_observations(
                directory, attempt=checkpoint.path.stem
            )[0]
            self.assertEqual(image["deadline"], deadline)
            self.assertEqual(image["status"], "admitted")
            self.assertEqual(image["allocation"], "b" * 32)
            self.assertTrue(image["caller_alive"])

    def observe(
        self,
        responses: list[object],
        *,
        structured: bool = True,
        selection: list[str] | None = None,
    ) -> tuple[int, str, str]:
        output = io.StringIO()
        errors = io.StringIO()
        with (
            patch.object(
                operation_lifetime, "manager_observation", side_effect=responses
            ),
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
            arguments = ["--json"] if structured else []
            for unit in selection or []:
                arguments.extend(("--unit", unit))
            status = activity.main(arguments)
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
        properties = "InvocationID=abc\nMemoryCurrent=0\nCPUUsageNSec=0\nActiveEnterTimestampMonotonic=0\nControlGroup=/g\nSlice=pse.slice\nLoadState=loaded\nActiveState=active\n"
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

    def test_selected_units_bypass_global_listing_and_deduplicate_in_order(
        self,
    ) -> None:
        fields = {
            "InvocationID": "abc",
            "ControlGroup": "/g",
            "LoadState": "loaded",
            "ActiveState": "active",
            "MemoryCurrent": "0",
            "CPUUsageNSec": "0",
            "ActiveEnterTimestampMonotonic": "0",
            "Slice": "pse.slice",
        }
        with (
            patch.object(
                activity, "systemctl", side_effect=AssertionError("global list")
            ),
            patch.object(
                operation_lifetime, "unit_observation", return_value=fields
            ) as observe,
        ):
            status, output, _ = self.observe(
                [], selection=["pse-z.scope", "pse-a.service", "pse-z.scope"]
            )
        result = json.loads(output)
        self.assertEqual(status, 0)
        self.assertEqual(result["unit_selection"], ["pse-z.scope", "pse-a.service"])
        self.assertEqual(
            [call.args[0] for call in observe.call_args_list], result["unit_selection"]
        )
        self.assertTrue(result["units_available"])
        self.assertEqual(result["units"][0]["InvocationID"], "abc")

    def test_selected_partial_failure_preserves_healthy_unit_and_independent_results(
        self,
    ) -> None:
        properties = "InvocationID=abc\nControlGroup=/g\nLoadState=loaded\nActiveState=active\nMemoryCurrent=0\nCPUUsageNSec=0\nActiveEnterTimestampMonotonic=0\nSlice=pse.slice\n"
        for failure in (
            properties.replace("LoadState=loaded", "LoadState=not-found"),
            properties.replace("LoadState=loaded\n", ""),
            properties + "invalid-property\n",
            ValueError("missing manager"),
        ):
            with self.subTest(failure=failure):
                status, output, _ = self.observe(
                    [failure, properties],
                    selection=["pse-gone.scope", "pse-good.scope"],
                )
                result = json.loads(output)
                self.assertEqual(status, 1)
                self.assertFalse(result["units_available"])
                self.assertEqual(
                    [unit["unit"] for unit in result["units"]], ["pse-good.scope"]
                )
                self.assertEqual(result["build_processes"][0]["pid"], 42)
                self.assertIn(
                    "pse-gone.scope unavailable", result["observation_errors"][0]
                )

    def test_selection_rejects_nonidentities_before_observation(self) -> None:
        for unit in (
            "--all",
            "pse-*.scope",
            "pse-x.scope ",
            "pse-x\n.scope",
            "x.service",
            "pse-x.slice",
            "pse-.scope",
        ):
            with (
                self.subTest(unit=unit),
                patch.object(
                    activity, "units", side_effect=AssertionError("observation")
                ),
                contextlib.redirect_stderr(io.StringIO()),
                self.assertRaises(SystemExit) as refused,
            ):
                activity.main(["--unit", unit])
            self.assertEqual(refused.exception.code, 2)

    def test_independent_failures_do_not_change_selected_unit_completeness(
        self,
    ) -> None:
        fields = {
            "InvocationID": "abc",
            "ControlGroup": "/g",
            "LoadState": "loaded",
            "ActiveState": "active",
        }
        for owner, name in (
            (activity, "build_processes"),
            (activity.pse_env, "limits"),
        ):
            output = io.StringIO()
            with (
                patch.object(
                    operation_lifetime, "unit_observation", return_value=fields
                ),
                patch.object(activity, "build_processes", return_value=[]),
                patch.object(activity.pse_env, "limits", return_value=[]),
                patch.object(
                    owner, name, side_effect=OSError("independent unavailable")
                ),
                contextlib.redirect_stdout(output),
            ):
                status = activity.main(["--json", "--unit", "pse-healthy.scope"])
            result = json.loads(output.getvalue())
            self.assertEqual(status, 1)
            self.assertTrue(result["units_available"])
            self.assertEqual(result["units"][0]["unit"], "pse-healthy.scope")

    def test_pressure_selection_retains_unavailable_without_hiding_unit(self) -> None:
        pressure = activity.pressure_observation
        host = pressure.Sample(
            "observed", pressure.Identity("boot", None, None, None), 1, 2, 3, {}
        )
        missing = pressure.Sample(
            "unavailable", None, 1, 2, 3, {}, reason="permission denied"
        )
        output = io.StringIO()
        with (
            patch.object(
                activity,
                "units",
                return_value=(
                    [
                        {
                            "unit": "pse-selected.scope",
                            "ControlGroup": "/selected",
                            "InvocationID": "abc",
                        }
                    ],
                    [],
                ),
            ),
            patch.object(
                operation_lifetime,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ControlGroup": "/selected",
                    "InvocationID": "abc",
                },
            ),
            patch.object(pressure, "sample", side_effect=[host, missing]),
            patch.object(activity, "build_processes", return_value=[]),
            patch.object(activity.pse_env, "limits", return_value=[]),
            contextlib.redirect_stdout(output),
        ):
            code = activity.main(
                ["--json", "--pressure", "--unit", "pse-selected.scope"]
            )
        result = json.loads(output.getvalue())
        self.assertEqual(code, 1)
        self.assertTrue(result["units_available"])
        self.assertEqual(result["pressure"]["host"]["status"], "observed")
        self.assertEqual(
            result["pressure"]["selected"]["/selected"]["reason"], "permission denied"
        )
        self.assertTrue(result["observation_errors"])

    def test_changed_unit_does_not_link_new_incarnation_pressure(self) -> None:
        pressure = activity.pressure_observation
        sample = pressure.Sample(
            "observed",
            pressure.Identity("boot", "/selected", 1, 10),
            1,
            2,
            3,
            {"cpu.stat": pressure.Reading("available", {"usage_usec": 1})},
        )
        with (
            patch.object(pressure, "sample", return_value=sample),
            patch.object(
                operation_lifetime,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ControlGroup": "/selected",
                    "InvocationID": "new",
                },
            ),
        ):
            observed = activity.unit_pressure(
                {
                    "unit": "pse-selected.scope",
                    "ControlGroup": "/selected",
                    "InvocationID": "old",
                }
            )
        self.assertEqual(observed.status, "unavailable")
        self.assertIsNone(observed.identity)
        self.assertEqual(observed.fields, {})
        if observed.reason is None:
            self.fail("Expected the original incarnation-change reason")
        self.assertIn("incarnation changed", observed.reason)


if __name__ == "__main__":
    unittest.main()

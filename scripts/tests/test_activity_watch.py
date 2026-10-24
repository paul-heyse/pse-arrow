# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact detached watches reconcile owner truth, including loss and reconnect gaps."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls
from __future__ import annotations

import contextlib
import io
import json
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import activity, activity_watch as watch, test_resources, test_run


class WatchTests(unittest.TestCase):
    def test_cursor_wait_change_and_reconnect_current_state(self) -> None:
        selected = {"kind": "unit", "handle": "pse-exact.scope"}
        first = watch.watch(selected, observe=lambda *_: {"state": "running", "stage": "build"})
        with patch.object(watch.time, "sleep"):
            same = watch.watch(selected, cursor=first["cursor"], max_wait=0, observe=lambda *_: {"state": "running", "stage": "build"})
        self.assertFalse(same["changed"])
        self.assertEqual(same["reason"], "timeout")
        end = watch.watch(selected, cursor=same["cursor"], observe=lambda *_: {"state": "terminal-without-result"})
        self.assertTrue(end["changed"])
        self.assertEqual(end["reconciliation"]["history"], "unavailable")
        self.assertIn("missing", end["reconciliation"]["gap"])
        self.assertEqual(end["reconciliation"]["previous_observed_unix_ns"], same["observed_unix_ns"])

    def test_cursor_bound_to_selection_and_wait_bounds(self) -> None:
        first = watch.watch({"kind": "attempt", "handle": "a" * 32}, observe=lambda *_: {})
        for cursor in (first["cursor"], "!invalid", "x" * 16385):
            with self.assertRaises(ValueError):
                watch.watch({"kind": "attempt", "handle": "b" * 32}, cursor=cursor)
        for bound in (-1, 31, float("inf"), float("nan")):
            with self.assertRaises(ValueError):
                watch.watch({}, max_wait=bound)

    def test_heartbeat_ages_do_not_wake_but_real_facts_do(self) -> None:
        self.assertEqual(watch.revision({"stage": "compiler", "observed_at": 1, "age_seconds": 2}), watch.revision({"stage": "compiler", "observed_at": 10, "age_seconds": 3}))
        self.assertNotEqual(watch.revision({"stage": "compiler"}), watch.revision({"stage": "linker"}))

    def test_unit_disappearance_and_environment_failure_are_unavailable(self) -> None:
        selected = {"kind": "unit", "handle": "pse-missing.scope"}
        for response in ({"LoadState": "not-found"}, OSError("missing systemctl"), subprocess.TimeoutExpired("systemctl", 0.1)):
            with patch.object(watch.native_operation, "unit_observation", side_effect=response if isinstance(response, Exception) else None, return_value=response):
                result = watch.snapshot(selected, time.monotonic() + 1)
            self.assertEqual(result["state"], "observation-unavailable")
            self.assertIsNone(result["result_reference"])

    def test_inactive_unit_does_not_claim_result_or_success(self) -> None:
        with patch.object(watch.native_operation, "unit_observation", return_value={"LoadState": "loaded", "ActiveState": "inactive", "Result": "success"}):
            result = watch.snapshot({"kind": "unit", "handle": "pse-ended.scope"}, time.monotonic() + 1)
        self.assertEqual(result["state"], "terminal-without-result")
        self.assertIsNone(result["result_reference"])

    def test_actual_admission_owner_and_missing_attempt(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            checkpoint = watch.host_admission.AdmissionCheckpoint(Path(temporary), watch.host_admission.select("reference"), time.monotonic() + 5)
            selected = {"kind": "attempt", "handle": checkpoint.path.stem, "admission_directory": temporary}
            self.assertEqual(watch.snapshot(selected, time.monotonic() + 1)["state"], "observation-unavailable")
            checkpoint.publish("waiting", "exclusive lane occupied")
            waiting = watch.watch(selected, max_wait=0)
            self.assertEqual(waiting["snapshot"]["state"], "waiting")
            checkpoint.publish("admitted", None, allocation="b" * 32)
            admitted = watch.watch(selected, cursor=waiting["cursor"])
            self.assertEqual(admitted["snapshot"]["state"], "admitted")
            self.assertEqual(admitted["snapshot"]["attempt"]["allocation"], "b" * 32)
            self.assertIsNone(admitted["snapshot"]["result_reference"])

    def test_product_terminal_is_genuine_even_if_build_observation_unavailable(self) -> None:
        with tempfile.TemporaryDirectory() as temporary, patch.object(test_resources, "registry", return_value=Path(temporary) / "registry"):
            path = test_resources.invocation("python", [{"name": "suite::one"}])
            selected = watch.selection("run", str(path))
            before = watch.watch(selected, max_wait=0)
            self.assertEqual(before["snapshot"]["state"], "running")
            test_run.finish(path, {"exit_code": 0, "results": [{"nodeid": "suite::one", "status": "passed"}]}, "runner")
            with patch.object(watch.build_observation, "observations", side_effect=PermissionError("snapshot denied")):
                result = watch.watch(selected, cursor=before["cursor"])
            self.assertEqual(result["snapshot"]["state"], "result-complete")
            self.assertEqual(result["snapshot"]["result_reference"]["run_path"], str(path))
            self.assertEqual(result["snapshot"]["result_reference"]["terminal_identity"], test_resources.storage().terminal(path.stem)[0])
            self.assertIn("denied", result["snapshot"]["build_error"])

    def test_product_process_disappearance_does_not_supply_terminal(self) -> None:
        with tempfile.TemporaryDirectory() as temporary, patch.object(test_resources, "registry", return_value=Path(temporary) / "registry"):
            path = test_resources.invocation("python", [{"name": "suite::one"}])
            selected = watch.selection("run", str(path))
            with patch.object(watch.host_admission, "caller_alive", return_value=False):
                lost = watch.snapshot(selected, time.monotonic() + 1)
            self.assertEqual(lost["state"], "observation-lost")
            self.assertIsNone(lost["result_reference"])
            self.assertIsNone(lost["run"]["terminal"])
            with test_resources.storage().invocation_edit(path.stem) as state:
                state["phase"] = "abandoned"
            self.assertEqual(watch.snapshot(selected, time.monotonic() + 1)["state"], "observation-lost")

    def test_interrupted_cli_detaches_and_leaves_actual_child_alive(self) -> None:
        with subprocess.Popen([sys.executable, "-c", "import time; time.sleep(10)"]) as child:
            try:
                output = io.StringIO()
                with patch.object(watch, "watch", side_effect=KeyboardInterrupt), contextlib.redirect_stdout(output):
                    code = activity.main(["--watch", "--unit", "pse-live.scope"])
                self.assertEqual(code, 130)
                self.assertEqual(json.loads(output.getvalue())["workload_action"], "none")
                self.assertIsNone(child.poll())
            finally:
                child.terminate()
                child.wait(timeout=2)

    def test_cli_is_independent_of_admission_compiler_and_global_units(self) -> None:
        output = io.StringIO()
        with patch.object(activity, "units", side_effect=AssertionError("global units")), patch.object(activity.pse_env.host, "acquire", side_effect=AssertionError("admission")), patch.object(activity.pse_env.build_environment, "configure", side_effect=AssertionError("compiler")), patch.object(watch.native_operation, "unit_observation", side_effect=OSError("manager unavailable")), contextlib.redirect_stdout(output):
            self.assertEqual(activity.main(["--watch", "--unit", "pse-live.scope", "--max-wait", "0"]), 1)
        self.assertEqual(json.loads(output.getvalue())["snapshot"]["state"], "observation-unavailable")

    def test_cli_requires_single_exact_owner(self) -> None:
        for args in (["--watch"], ["--watch", "--unit", "pse-a.scope", "--unit", "pse-b.scope"], ["--watch", "--run", "a" * 32, "--attempt", "b" * 32], ["--cursor", "x"]):
            with contextlib.redirect_stderr(io.StringIO()), self.assertRaises(SystemExit) as refused:
                activity.main(args)
            self.assertEqual(refused.exception.code, 2)


if __name__ == "__main__":
    unittest.main()

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""A successful command alone does not qualify native instrumentation."""

# ruff: noqa: PT009 -- these tests use unittest assertions.
import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import highs_lifecycle_diagnostic as diagnostic
from scripts.highs_lifecycle_diagnostic import qualify_asan


class LifecycleQualificationTests(unittest.TestCase):
    def test_normal_lifecycle_needs_both_instrumented_suppliers_and_actual_completion(
        self,
    ) -> None:
        normal = {"exit_code": 0, "timed_out": False}
        self.assertTrue(qualify_asan(normal, True, True))
        self.assertFalse(qualify_asan(normal, False, True))
        self.assertFalse(qualify_asan(normal, True, False))
        self.assertFalse(qualify_asan({"exit_code": -9, "timed_out": True}, True, True))
        self.assertFalse(qualify_asan({"exit_code": 1, "timed_out": False}, True, True))

    def test_refused_or_failed_build_never_claims_unsupported_instrumentation(self) -> None:
        for code, expected in [(125, "admission-refused"), (2, "build-failed")]:
            with tempfile.TemporaryDirectory() as directory:
                output = Path(directory) / "capture"
                observed = {
                    "exit_code": code,
                    "timed_out": False,
                    "ownership": {"owned_drain": "complete"},
                }
                with (
                    patch("sys.argv", ["highs-lifecycle", str(output)]),
                    patch.object(diagnostic, "run", return_value=observed),
                    patch.object(
                        diagnostic.subprocess,
                        "check_output",
                        return_value="host: x86_64-unknown-linux-gnu\n",
                    ),
                    contextlib.redirect_stdout(io.StringIO()),
                ):
                    self.assertEqual(diagnostic.main(), 1)
                record = json.loads((output / "lifecycle.json").read_text())
                self.assertEqual(record["status"], expected)
                self.assertEqual(record["build"]["exit_code"], code)
                self.assertTrue(record["sensitivity_control"].startswith("not_run:"))
                self.assertNotIn("normal", record)

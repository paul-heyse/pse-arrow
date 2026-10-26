# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Native progress supervision retains cancellation and diagnoses actual stalls."""

from __future__ import annotations

import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import patch

from scripts import host_metadata
from scripts import recovery_operation as recovery


class RecoveryOperationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.request = self.enterContext(tempfile.TemporaryFile())
        self.response = self.enterContext(tempfile.TemporaryFile())
        self.errors = self.enterContext(tempfile.TemporaryFile())
        self.checkpoint = self.directory / "progress.json"
        self.children: list[subprocess.Popen] = []
        original = subprocess.Popen

        def launch(*args: object, **options: object) -> subprocess.Popen:
            child = original(*args, **options)
            self.children.append(child)
            return child

        self.enterContext(
            patch.object(recovery.subprocess, "Popen", side_effect=launch)
        )

    def run_child(self, **options: object) -> int:
        return recovery.run(
            [sys.executable, "-c", "import time; time.sleep(0.6)"],
            self.request,
            self.response,
            self.errors,
            self.checkpoint,
            **options,
        )

    def test_continuing_byte_progress_outlives_the_stall_interval(self) -> None:
        started = time.monotonic()
        calls = 0

        def observe(_path: Path) -> object:
            nonlocal calls
            calls += 1
            if calls == 1:
                return None
            elapsed = time.monotonic() - started
            return SimpleNamespace(
                attempt="selected",
                sequence=int(elapsed * 10),
                phase=SimpleNamespace(
                    value="complete" if elapsed >= 0.5 else "fingerprint"
                ),
            )

        with patch.object(recovery, "observation", side_effect=observe):
            self.assertEqual(self.run_child(stall_seconds=0.25), 0)
        self.assertGreater(time.monotonic() - started, 0.5)
        self.assertIsNotNone(self.children[0].poll())

    def test_stale_checkpoint_does_not_extend_an_unresponsive_attempt(self) -> None:
        stale = SimpleNamespace(
            attempt="previous", sequence=500, phase=SimpleNamespace(value="fingerprint")
        )
        with (
            patch.object(recovery, "observation", return_value=stale),
            self.assertRaisesRegex(recovery.RecoveryError, "stalled.*explicit resume"),
        ):
            self.run_child(stall_seconds=0.15)
        self.assertIsNotNone(self.children[0].poll())

    def test_original_caller_deadline_settles_the_exact_native_child(self) -> None:
        with (
            patch.object(recovery, "observation", return_value=None),
            self.assertRaises(host_metadata.AdmissionError),
        ):
            self.run_child(deadline=time.monotonic() + 0.15)
        self.assertIsNotNone(self.children[0].poll())

    def test_progress_regression_refuses_without_waiting_for_success(self) -> None:
        def observed(sequence: int) -> SimpleNamespace:
            return SimpleNamespace(
                attempt="selected",
                sequence=sequence,
                phase=SimpleNamespace(value="fingerprint"),
            )

        with (
            patch.object(
                recovery, "observation", side_effect=[None, observed(3), observed(2)]
            ),
            self.assertRaisesRegex(recovery.RecoveryError, "authority changed"),
        ):
            self.run_child()
        self.assertIsNotNone(self.children[0].poll())

    def test_success_without_native_complete_is_refused(self) -> None:
        with (
            patch.object(recovery, "observation", return_value=None),
            self.assertRaisesRegex(recovery.RecoveryError, "omitted complete"),
        ):
            self.run_child()


if __name__ == "__main__":
    unittest.main()

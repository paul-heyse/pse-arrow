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
from typing import TYPE_CHECKING, ParamSpec, TypeVar

if TYPE_CHECKING:
    from collections.abc import Callable
from unittest.mock import patch

import pytest

from scripts import host_metadata
from scripts import recovery_operation as recovery

Parameters = ParamSpec("Parameters")
Result = TypeVar("Result")


def record_calls(
    function: Callable[Parameters, Result], results: list[Result]
) -> Callable[Parameters, Result]:
    def recorded(*args: Parameters.args, **kwargs: Parameters.kwargs) -> Result:
        result = function(*args, **kwargs)
        results.append(result)
        return result

    return recorded


class RecoveryOperationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.request = self.enterContext(tempfile.TemporaryFile())
        self.response = self.enterContext(tempfile.TemporaryFile())
        self.errors = self.enterContext(tempfile.TemporaryFile())
        self.checkpoint = self.directory / "progress.json"
        self.children: list[subprocess.Popen] = []
        self.enterContext(
            patch.object(
                recovery.subprocess,
                "Popen",
                side_effect=record_calls(subprocess.Popen, self.children),
            )
        )

    def run_child(
        self, *, deadline: float | None = None, stall_seconds: float = 120
    ) -> int:
        return recovery.run(
            [sys.executable, "-c", "import time; time.sleep(0.6)"],
            self.request,
            self.response,
            self.errors,
            self.checkpoint,
            deadline=deadline,
            stall_seconds=stall_seconds,
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
            assert self.run_child(stall_seconds=0.25) == 0
        assert time.monotonic() - started > 0.5
        assert self.children[0].poll() is not None

    def test_stale_checkpoint_does_not_extend_an_unresponsive_attempt(self) -> None:
        stale = SimpleNamespace(
            attempt="previous", sequence=500, phase=SimpleNamespace(value="fingerprint")
        )
        with (
            patch.object(recovery, "observation", return_value=stale),
            pytest.raises(recovery.RecoveryError, match=r"stalled.*explicit resume"),
        ):
            self.run_child(stall_seconds=0.15)
        assert self.children[0].poll() is not None

    def test_original_caller_deadline_settles_the_exact_native_child(self) -> None:
        with (
            patch.object(recovery, "observation", return_value=None),
            pytest.raises(host_metadata.AdmissionError),
        ):
            self.run_child(deadline=time.monotonic() + 0.15)
        assert self.children[0].poll() is not None

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
            pytest.raises(recovery.RecoveryError, match="authority changed"),
        ):
            self.run_child()
        assert self.children[0].poll() is not None

    def test_success_without_native_complete_is_refused(self) -> None:
        with (
            patch.object(recovery, "observation", return_value=None),
            pytest.raises(recovery.RecoveryError, match="omitted complete"),
        ):
            self.run_child()


if __name__ == "__main__":
    unittest.main()

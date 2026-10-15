# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Once-only audit orchestration keeps failed and interrupted science distinct."""
# ruff: noqa: PT009, PT027 -- stdlib tooling controls

from __future__ import annotations

import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from types import SimpleNamespace
from unittest.mock import Mock, patch

from scripts import case_audit, case_measure


class AuditTests(unittest.TestCase):
    def test_missing_outcome_and_nonzero_exit_cannot_be_success(self) -> None:
        self.assertEqual(case_audit.reconcile_outcome(0, None)["status"], "failed")
        self.assertEqual(
            case_audit.reconcile_outcome(1, {"status": "completed"})["status"], "failed"
        )
        interrupted = case_audit.reconcile_outcome(
            1, {"status": "interrupted", "cleanup": "confirmed"}
        )
        self.assertEqual(interrupted["status"], "interrupted")
        self.assertEqual(interrupted["cleanup"], "confirmed")

    def test_audit_launches_one_process_with_separate_diagnostic_directory(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            program = (
                "import os,json,pathlib; "
                "p=pathlib.Path(os.environ['PSE_CASE_AUDIT_DIR']); "
                "assert os.environ['PSE_DIAGNOSTICS_DIR']==str(p/'checkpoints'); "
                "(p/'audit-outcome.json').write_text(json.dumps({'status':'interrupted','native_joined':True})); "
                "raise SystemExit(2)"
            )
            with patch.object(case_audit, "Observer") as observer:
                observer.return_value.finish.return_value = {"inspection_windows": 1}
                result = case_audit.run_case(
                    [sys.executable, "-c", program],
                    cwd=directory,
                    env=dict(os.environ),
                    directory=directory,
                )
            self.assertEqual(result["status"], "interrupted")
            self.assertTrue(result["native_joined"])
            self.assertEqual(result["exit_code"], 2)
            self.assertEqual(
                json.loads((directory / "diagnostics/observation.json").read_text()),
                result,
            )

    def test_audit_requires_once_only_explicit_nonrepeated_selection(self) -> None:
        for arguments in (
            ["out", "--audit-diagnostics"],
            ["out", "--smoke", "--audit-diagnostics"],
            ["out", "--smoke", "--audit-diagnostics", "--case", "a", "--case", "a"],
            ["out", "--smoke", "--audit-diagnostics", "--case", "document-admission"],
        ):
            with (
                patch.object(sys, "argv", ["case_measure", *arguments]),
                self.assertRaises(SystemExit) as exit_error,
            ):
                case_measure.main()
            self.assertEqual(exit_error.exception.code, 2)

    def test_foreground_interrupt_requests_stop_and_waits_for_child_cleanup(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            child = (
                "import os,pathlib,time,signal,json; "
                "p=pathlib.Path(os.environ['PSE_CASE_AUDIT_DIR']); "
                "signal.signal(signal.SIGINT,lambda *_: (p/'bad-signal').write_text('signal')); "
                "(p/'ready').write_text('ready')\n"
                "while not (p/'stop.request').exists(): time.sleep(.02)\n"
                "(p/'audit-outcome.json').write_text(json.dumps({'status':'interrupted','cleanup':'confirmed'}))\n"
            )
            parent = (
                "from scripts import case_audit; from pathlib import Path; from unittest.mock import patch; import os,sys\n"
                "with patch.object(case_audit,'Observer') as observer:\n"
                " observer.return_value.finish.return_value={}\n"
                " try: case_audit.run_case([sys.executable,'-c',sys.argv[2]],cwd=Path.cwd(),env=dict(os.environ),directory=Path(sys.argv[1]))\n"
                " except KeyboardInterrupt: pass\n"
            )
            with subprocess.Popen(
                [sys.executable, "-c", parent, str(directory), child],
                cwd=case_measure.ROOT,
                start_new_session=True,
            ) as process:
                deadline = time.monotonic() + 10
                while not (directory / "diagnostics/ready").exists():
                    self.assertIsNone(
                        process.poll(), "audit parent exited before child was ready"
                    )
                    self.assertLess(time.monotonic(), deadline)
                    time.sleep(0.02)
                os.killpg(process.pid, signal.SIGINT)
                self.assertEqual(process.wait(timeout=10), 0)
            self.assertFalse((directory / "diagnostics/bad-signal").exists())
            self.assertEqual(
                json.loads((directory / "diagnostics/audit-outcome.json").read_text())[
                    "cleanup"
                ],
                "confirmed",
            )

    def test_managed_discovery_requires_registered_database_and_live_start_identity(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            directory = Path(temporary)
            checkpoint = directory / "worker.json"
            checkpoint.write_text("retained worker prefix")
            (directory / "managed-target.json").write_text(
                json.dumps({"state": str(directory), "database": "owned"})
            )
            (directory / "primary-receiver.json").write_text(
                json.dumps(
                    {"pid": os.getpid(), "ready": True, "canonical_database": "owned"}
                )
            )
            start = int(
                Path(f"/proc/{os.getpid()}/stat")
                .read_text()
                .rsplit(")", 1)[1]
                .split()[19]
            )
            for matches in (False, True):
                observer = object.__new__(case_audit.Observer)
                observer.directory = directory
                observer.managed_checkpoint = None
                observer.managed_operations = []
                observer.reads = 1
                observer.failures = []
                observer.list_request = Mock()
                observer.request = Mock()
                observer.write = Mock()
                process = SimpleNamespace(
                    locator=SimpleNamespace(
                        pid=os.getpid(), start_ticks=start if matches else start + 1
                    ),
                    checkpoint=str(checkpoint),
                    operations=[SimpleNamespace(id="operation", outcome=None)],
                    partial=False,
                )
                observer.diagnostics = SimpleNamespace(
                    list_processes=Mock(
                        return_value=SimpleNamespace(processes=[process], next=None)
                    ),
                    inspect_checkpoint=Mock(return_value="document"),
                    InspectionError=RuntimeError,
                )
                observer.inspect_managed()
                self.assertEqual(
                    observer.managed_checkpoint, checkpoint if matches else None
                )
                self.assertEqual(
                    observer.diagnostics.inspect_checkpoint.call_count, int(matches)
                )
                self.assertEqual(observer.failures, [])


if __name__ == "__main__":
    unittest.main()

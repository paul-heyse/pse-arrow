# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One terminal composer with authentic child status and signal forwarding."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import resource_store as files
from scripts import test_resources as resources
from scripts import test_run


class TerminalComposerTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        selected = patch.dict(
            os.environ, {"PSE_TEST_RESOURCE_STATE": str(self.root / "registry")}
        )
        selected.start()
        self.addCleanup(selected.stop)
        self.path = resources.invocation("python", [{"name": "control.py::test_case"}])
        self.report = self.path.with_suffix(".xml")

    def test_actual_child_status_survives_missing_report_reconciliation(self) -> None:
        for status in (0, 7):
            with self.subTest(status=status):
                path = resources.invocation(
                    "python", [{"name": "control.py::test_case"}]
                )
                with patch.object(test_run, "finish", wraps=test_run.finish) as final:
                    code = test_run.execute_selected(
                        path,
                        [sys.executable, "-c", f"raise SystemExit({status})"],
                        os.environ,
                        path.with_suffix(".xml"),
                        selected=[{"nodeid": "control.py::test_case"}],
                    )
                self.assertEqual(code, status)
                final.assert_called_once()
                evidence = files.read(path.with_suffix(".terminal.json"))
                self.assertEqual(evidence["exit_code"], status)
                self.assertEqual(evidence["process"]["child_status"], status)
                self.assertTrue(evidence["report_errors"])
                _identity, terminal = resources.storage().terminal(
                    resources.read_invocation(path)["nonce"]
                )
                self.assertFalse(terminal["complete"])

    def test_launch_exception_and_keyboard_interrupt_reconcile_once(self) -> None:
        for error in (OSError("spawn failed"), KeyboardInterrupt()):
            path = resources.invocation("python", [])

            def launch(_observation: dict[str, object]) -> int:
                raise error

            with (
                self.subTest(error=type(error).__name__),
                patch.object(test_run, "finish", wraps=test_run.finish) as final,
            ):
                with self.assertRaises(type(error)):
                    test_run.execute_selected(
                        path,
                        ["unlaunched"],
                        os.environ,
                        path.with_suffix(".xml"),
                        launch=launch,
                    )
                final.assert_called_once()
            evidence = files.read(path.with_suffix(".terminal.json"))
            self.assertEqual(
                evidence["exit_code"],
                -signal.SIGINT if isinstance(error, KeyboardInterrupt) else None,
            )
            self.assertTrue(evidence["report_errors"])
            self.assertEqual(
                resources.storage().terminal(resources.read_invocation(path)["nonce"])[
                    1
                ]["complete"],
                False,
            )

    def test_inherited_python_child_never_composes_terminal(self) -> None:
        with patch.object(
            test_run, "execute_selected", side_effect=AssertionError("second composer")
        ):
            self.assertEqual(
                test_run.run_python(
                    [sys.executable, "-c", "raise SystemExit(7)"],
                    env={**os.environ, resources.MARKER: str(self.path)},
                ),
                7,
            )
        state = files.read(
            resources.storage().path(
                "invocation-state", resources.read_invocation(self.path)["nonce"]
            )
        )
        self.assertEqual(state["phase"], "open")

    def test_ordinary_python_terminal_carries_actual_extension_observation(
        self,
    ) -> None:
        artifact = {
            "binary": "/selected/_native.so",
            "sha256": "observed",
            "features": ["force-validate"],
        }
        environment = {
            name: value
            for name, value in os.environ.items()
            if name != resources.MARKER
        }
        with patch.object(test_run, "execute_selected", return_value=7) as run:
            self.assertEqual(
                test_run.run_python(
                    [sys.executable, "-m", "pytest"], env=environment, artifact=artifact
                ),
                7,
            )
        self.assertEqual(
            run.call_args.kwargs["details"], {"artifacts": {"python": artifact}}
        )

    def test_python_observer_parent_owns_generated_report_and_extension_observation(
        self,
    ) -> None:
        command = [
            sys.executable,
            "-m",
            "scripts.python_tests",
            "--functional-observer-child",
        ]
        artifact = {
            "binary": "/selected/_native.so",
            "sha256": "observed",
            "features": ["force-validate"],
        }
        environment = {
            name: value
            for name, value in os.environ.items()
            if name != resources.MARKER
        }
        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(test_run, "execute_selected", return_value=7) as run,
        ):
            self.assertEqual(
                test_run.python_observer(
                    command,
                    [],
                    lambda _observation: 7,
                    terminal_owner="runner",
                    default_report=True,
                    artifact=artifact,
                ),
                7,
            )
            self.assertNotIn(resources.MARKER, os.environ)
            self.assertNotIn("PSE_TEST_ENUMERATION", os.environ)
        path, forwarded_command, forwarded, report = run.call_args.args
        self.assertEqual(resources.read_invocation(path)["terminal_owner"], "runner")
        self.assertEqual(forwarded_command, command)
        self.assertEqual(command[-1], "--junitxml=" + str(report))
        self.assertEqual(forwarded[resources.MARKER], str(path))
        self.assertEqual(forwarded["PSE_NATIVE_EXPECTED_SHA256"], "observed")
        self.assertEqual(
            run.call_args.kwargs["details"], {"artifacts": {"python": artifact}}
        )

    def test_signal_during_terminal_freeze_preserves_precommit_interruption_and_postcommit_truth(
        self,
    ) -> None:
        from scripts import native_operation

        for phase in ("before-freeze", "after-freeze"):
            with self.subTest(phase=phase):
                path = resources.invocation(
                    "python", [{"name": "control.py::test_case"}]
                )
                report = path.with_suffix(".xml")
                environment = {
                    key: value
                    for key, value in os.environ.items()
                    if not key.startswith("PSE_NATIVE_")
                }
                original = test_run.finish
                fired = False

                def launch(observation: dict[str, object]) -> int:
                    report.write_text(
                        '<testsuites><testsuite><testcase classname="control.py" name="test_case"/></testsuite></testsuites>'
                    )
                    observation.update(child_status=0, owned_drain="complete")
                    return 0

                def finish(*args, **kwargs):
                    nonlocal fired
                    if not fired:
                        fired = True
                        if phase == "before-freeze":
                            os.kill(os.getpid(), signal.SIGTERM)
                        else:
                            callback = kwargs["on_frozen"]

                            def committed() -> None:
                                callback()
                                os.kill(os.getpid(), signal.SIGTERM)

                            kwargs["on_frozen"] = committed
                    return original(*args, **kwargs)

                with (
                    patch.dict(os.environ, environment, clear=True),
                    patch.object(native_operation, "scope_owner", return_value=None),
                    patch.object(test_run, "finish", side_effect=finish),
                ):
                    code = test_run.execute_selected(
                        path,
                        ["phase-control"],
                        environment,
                        report,
                        selected=[{"class": "control.py", "name": "test_case"}],
                        launch=launch,
                    )
                evidence = files.read(path.with_suffix(".terminal.json"))
                terminal = resources.storage().terminal(path.stem)[1]
                self.assertEqual(code, 143 if phase == "before-freeze" else 0)
                self.assertEqual(evidence["exit_code"], code)
                self.assertEqual(terminal["evidence"]["workload_exit"], code)
                self.assertEqual(terminal["complete"], phase == "after-freeze")
                self.assertEqual(
                    evidence["status"],
                    "passed" if phase == "after-freeze" else "failed",
                )

    def test_non_native_ignored_sigterm_escalates_within_original_clock(self) -> None:
        from scripts import native_operation

        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("PSE_NATIVE_")
        }
        child = "import os,signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); os.kill(os.getppid(),signal.SIGTERM); time.sleep(30)"
        started = time.monotonic()
        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(native_operation, "scope_owner", return_value=None),
            patch.object(native_operation, "cancel_foreground_handoffs"),
        ):
            code = test_run.execute_selected(
                self.path,
                [sys.executable, "-c", child],
                environment,
                self.report,
                selected=[{"nodeid": "control.py::test_case"}],
            )
        self.assertLess(time.monotonic() - started, 11)
        evidence = files.read(self.path.with_suffix(".terminal.json"))
        self.assertEqual(code, 143)
        self.assertEqual(evidence["process"]["child_status"], -signal.SIGKILL)
        self.assertEqual(evidence["process"]["forwarded_signal"], signal.SIGTERM)
        self.assertFalse(resources.storage().terminal(self.path.stem)[1]["complete"])

    def test_non_native_forwarded_signal_survives_child_passing_report_and_zero_exit(
        self,
    ) -> None:
        from scripts import native_operation

        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("PSE_NATIVE_")
        }
        xml = '<testsuites><testsuite><testcase classname="control.py" name="test_case"/></testsuite></testsuites>'
        child = "\n".join(
            [
                "import os,signal,time; from pathlib import Path",
                "signal.signal(signal.SIGTERM,lambda *_args:None)",
                "os.kill(os.getppid(),signal.SIGTERM)",
                "time.sleep(.1)",
                f"Path({str(self.report)!r}).write_text({xml!r})",
            ]
        )
        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(native_operation, "scope_owner", return_value=None),
            patch.object(native_operation, "cancel_foreground_handoffs"),
        ):
            code = test_run.execute_selected(
                self.path,
                [sys.executable, "-c", child],
                environment,
                self.report,
                selected=[{"class": "control.py", "name": "test_case"}],
            )
        evidence = files.read(self.path.with_suffix(".terminal.json"))
        self.assertEqual(code, 143)
        self.assertEqual(evidence["process"]["child_status"], 0)
        self.assertEqual(evidence["process"]["forwarded_signal"], signal.SIGTERM)
        self.assertEqual(evidence["status"], "failed")
        self.assertFalse(resources.storage().terminal(self.path.stem)[1]["complete"])

    def test_sigterm_forwards_owned_process_tree_and_freezes_incomplete(self) -> None:
        ready = self.root / "ready"
        grandchild_ready = self.root / "grandchild-ready"
        settled = self.root / "grandchild-settled"
        pid = self.root / "grandchild-pid"
        grandchild = (
            "import os,signal,time; from pathlib import Path; "
            f"Path({str(pid)!r}).write_text(str(os.getpid())); "
            f"signal.signal(signal.SIGTERM,lambda *_:(Path({str(settled)!r}).write_text('signal'),exit(0))); "
            f"Path({str(grandchild_ready)!r}).write_text('ready'); "
            "time.sleep(30)"
        )
        child = "\n".join(
            [
                "import os,signal,subprocess,sys,time; from pathlib import Path",
                f"grandchild=subprocess.Popen([sys.executable,'-c',{grandchild!r}])",
                "def settle(signum, frame):",
                "    grandchild.wait(timeout=3)",
                "    signal.signal(signum,signal.SIG_DFL)",
                "    os.kill(os.getpid(),signum)",
                "signal.signal(signal.SIGTERM,settle)",
                f"Path({str(ready)!r}).write_text('ready')",
                "time.sleep(30)",
            ]
        )
        code = "\n".join(
            [
                "import os,sys",
                "from pathlib import Path",
                "from scripts import native_operation, test_run",
                # This control owns a real process tree without a systemd scope.
                "native_operation.scope_owner=lambda **_options:None",
                "native_operation.cancel_foreground_handoffs=lambda **_options:None",
                f"path=Path({str(self.path)!r})",
                f"result=test_run.execute_selected(path,[sys.executable,'-c',{child!r}],os.environ,path.with_suffix('.xml'),selected=[{{'nodeid':'control.py::test_case'}}])",
                "raise SystemExit(result)",
            ]
        )
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("PSE_NATIVE_")
        }
        parent = subprocess.Popen(
            [sys.executable, "-c", code],
            env=environment,
            stdout=subprocess.PIPE,
            stderr=subprocess.PIPE,
            text=True,
        )
        try:
            deadline = time.monotonic() + 5
            while (
                not (ready.exists() and grandchild_ready.exists())
                and time.monotonic() < deadline
            ):
                time.sleep(0.01)
            self.assertTrue(ready.exists() and grandchild_ready.exists())
            parent.send_signal(signal.SIGTERM)
            _stdout, stderr = parent.communicate(timeout=5)
            self.assertEqual(parent.returncode, 143, stderr)
            deadline = time.monotonic() + 2
            while not settled.exists() and time.monotonic() < deadline:
                time.sleep(0.01)
            self.assertTrue(settled.exists())
            evidence = files.read(self.path.with_suffix(".terminal.json"))
            self.assertEqual(evidence["exit_code"], 143)
            self.assertEqual(evidence["process"]["child_status"], -signal.SIGTERM)
            self.assertEqual(evidence["process"]["forwarded_signal"], signal.SIGTERM)
            self.assertTrue(evidence["report_errors"])
            self.assertEqual(evidence["process"]["owned_drain"], "unobserved")
            frozen = resources.storage().terminal(
                resources.read_invocation(self.path)["nonce"]
            )[1]
            self.assertFalse(frozen["complete"])
        finally:
            if parent.poll() is None:
                parent.kill()
                parent.communicate(timeout=3)


if __name__ == "__main__":
    unittest.main()

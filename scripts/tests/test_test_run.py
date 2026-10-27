# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""One terminal composer with authentic child status and signal forwarding."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import hashlib
import json
import os
import signal
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import (
    native_operation,
    native_tests,
    operation_lifetime,
    scientific_inputs,
    test_run,
)
from scripts import resource_store as files
from scripts import test_resources as resources

if TYPE_CHECKING:
    from collections.abc import Callable, Mapping, Sequence


class TerminalComposerTests(unittest.TestCase):
    def test_pure_selection_uses_prepared_execution_without_worker_or_store_observer(
        self,
    ) -> None:
        provenance = self.root / "native.json"
        provenance.write_text("{}")
        selection = provenance.with_name("native-selected.json")
        inventory = {
            "rust-suites": {
                "pse-ids": {
                    "binary-id": "pse-ids::lib",
                    "testcases": {
                        "tests::identity": {"filter-match": {"status": "matches"}}
                    },
                }
            }
        }
        selection.write_text(json.dumps(inventory))
        execution = ["--binaries-metadata", "/captured/binaries.json"]
        environment = {
            **os.environ,
            "PSE_NATIVE_PROVENANCE": str(provenance),
            "RUST_MIN_STACK": "16777216",
            "PSE_TEST_REQUIRED_ROLES": "",
        }
        with (
            patch.object(
                native_tests, "ordinary_rust_capture", return_value=(execution, None)
            ),
            patch.object(native_tests, "rust_fixture_bindings", return_value={}),
            patch.object(
                test_run.validation_receipts, "verify_native_profile"
            ) as verify,
            patch.object(test_run, "run_rust", return_value=7) as run,
            patch("scripts.surreal_server.observer") as observer,
        ):
            self.assertEqual(
                test_run.observer_rust(["cargo", "nextest", "run"], environment), 7
            )
        verify.assert_called_once_with({})
        observer.assert_not_called()
        self.assertEqual(run.call_args.args[0], ["cargo", "nextest", "run", *execution])
        self.assertEqual(run.call_args.kwargs["inventory"], inventory)
        self.assertEqual(run.call_args.kwargs["effects"], "canonical")
        self.assertEqual(
            run.call_args.kwargs["env"]["PSE_NATIVE_SELECTION"], str(selection)
        )

        # A missing worker is still a refusal for actual canonical/native demand,
        # rather than permission to bypass the observer or its lifecycle owner.
        inventory["rust-suites"]["pse-ids"]["binary-id"] = "pse-runtime::lib"
        inventory["rust-suites"]["pse-ids"]["testcases"] = {
            "workflow::durable_tests::control": {"filter-match": {"status": "matches"}}
        }
        selection.write_text(json.dumps(inventory))
        with (
            patch.object(
                native_tests, "ordinary_rust_capture", return_value=(execution, None)
            ),
            patch.object(native_tests, "rust_fixture_bindings", return_value={}),
            patch.object(test_run, "run_rust") as run,
            self.assertRaisesRegex(resources.ResourceError, "requires its worker"),
        ):
            test_run.observer_rust(["cargo", "nextest", "run"], environment)
        run.assert_not_called()

    def test_canonical_only_selection_keeps_observer_and_restores_stale_worker(
        self,
    ) -> None:
        provenance = self.root / "native.json"
        selection = provenance.with_name("native-selected.json")
        inventory = {
            "rust-suites": {
                "runtime": {
                    "binary-id": "pse-runtime::lib",
                    "binary-path": sys.executable,
                    "testcases": {
                        "workflow::result_inspection::canonical_result_inspection_server_unit::stored_result": {
                            "filter-match": {"status": "matches"}
                        }
                    },
                }
            }
        }
        selection.write_text(json.dumps(inventory))
        environment = {
            **os.environ,
            "PSE_NATIVE_PROVENANCE": str(provenance),
            "PSE_SURREAL_STATE": str(self.root / "service"),
            "PSE_WORKER_BINARY": "/stale/worker",
            "RUST_MIN_STACK": "16777216",
            "PSE_TEST_REQUIRED_ROLES": "",
        }
        # This control owns a fresh Rust invocation, independently of the host
        # suite's outer Python invocation. Its parent authority must be restored.
        environment.pop(resources.MARKER, None)
        prior_invocation = os.environ.get(resources.MARKER)

        def launch(_state: Path, command: list[str], **_kwargs: object) -> int:
            self.assertNotIn("PSE_WORKER_BINARY", os.environ)
            self.assertEqual(
                os.environ["PSE_TEST_EXECUTION_PROFILE"], "exclusive-observer"
            )
            self.assertEqual(command[2:4], ["scripts.test_run", "--observer-child"])
            return 7

        def execute(
            _invocation: Path,
            _command: Sequence[str],
            actual: Mapping[str, str],
            _report: Path,
            *,
            launch: Callable[[dict[str, object]], int] | None = None,
            **_options: object,
        ) -> int:
            self.assertNotIn("PSE_WORKER_BINARY", actual)
            if launch is None:
                self.fail("Expected the original canonical observer callback")
            return launch({})

        with (
            patch.dict(os.environ, {"PSE_WORKER_BINARY": "/stale/worker"}),
            patch.object(
                native_tests, "ordinary_rust_capture", return_value=(["prepared"], None)
            ),
            patch.object(native_tests, "rust_fixture_bindings", return_value={}),
            patch.object(
                test_run,
                "rust_report_command",
                return_value=(["cargo", "nextest", "run", "prepared"], self.report),
            ),
            patch.object(test_run, "execute_selected", side_effect=execute),
            patch("scripts.surreal_server.observer", side_effect=launch) as observer,
            patch.object(test_run, "run_rust") as direct,
        ):
            self.assertEqual(
                test_run.observer_rust(["cargo", "nextest", "run"], environment), 7
            )
            self.assertEqual(os.environ["PSE_WORKER_BINARY"], "/stale/worker")
        observer.assert_called_once()
        direct.assert_not_called()
        self.assertEqual(os.environ.get(resources.MARKER), prior_invocation)

    def test_workerless_observer_child_uses_the_same_strict_preparation_owner(
        self,
    ) -> None:
        with (
            patch.dict(
                os.environ,
                {
                    test_run.RUST_OBSERVER: "1",
                    "PSE_TEST_EXECUTION_PROFILE": "exclusive-observer",
                    "PSE_NATIVE_PROVENANCE": str(self.root / "native.json"),
                },
                clear=True,
            ),
            patch.object(native_tests, "managed_rust_run", return_value=4) as verified,
        ):
            self.assertEqual(test_run.observer_child(["prepared"]), 4)
            verified.assert_called_once_with(
                ["prepared"], self.root / "native.json", require_worker=False
            )
            os.environ.pop("PSE_NATIVE_PROVENANCE")
            with self.assertRaisesRegex(
                resources.ResourceError, "preparation provenance"
            ):
                test_run.observer_child(["prepared"])

    def test_observer_carries_selected_roles_and_restores_parent_environment(
        self,
    ) -> None:
        provenance = self.root / "native.json"
        selection = provenance.with_name("native-selected.json")
        selection.write_text("{}")
        environment = {
            **os.environ,
            "PSE_NATIVE_PROVENANCE": str(provenance),
            "PSE_SURREAL_STATE": str(self.root / "service"),
            "RUST_MIN_STACK": "16777216",
            "PSE_TEST_REQUIRED_ROLES": "operator-graph,operator-query",
        }
        environment.pop(resources.MARKER, None)
        prior_invocation = os.environ.get(resources.MARKER)
        prior = os.environ.get("PSE_TEST_REQUIRED_ROLES")

        def launch(_state: Path, _command: list[str], **_kwargs: object) -> int:
            self.assertEqual(
                os.environ["PSE_TEST_REQUIRED_ROLES"], "operator-graph,operator-query"
            )
            return 4

        def execute(
            _path: Path,
            _command: Sequence[str],
            _environment: Mapping[str, str],
            _report: Path,
            *,
            launch: Callable[[dict[str, object]], int] | None = None,
            **_options: object,
        ) -> int:
            if launch is None:
                self.fail("Expected the original launch callback")
            return launch({})

        with (
            patch.object(
                native_tests,
                "ordinary_rust_capture",
                return_value=(["prepared"], self.root / "worker"),
            ),
            patch.object(native_tests, "rust_fixture_bindings", return_value={}),
            patch.object(
                test_run.validation_receipts, "native_inventory", return_value={}
            ),
            patch.object(
                test_run,
                "rust_selected",
                return_value=[{"binary_id": "pse-operations::lib", "name": "operator"}],
            ),
            patch.object(
                test_run,
                "rust_report_command",
                return_value=(["cargo", "nextest", "run", "prepared"], self.report),
            ),
            patch.object(test_run, "execute_selected", side_effect=execute),
            patch("scripts.surreal_server.observer", side_effect=launch),
        ):
            self.assertEqual(
                test_run.observer_rust(["cargo", "nextest", "run"], environment), 4
            )
        self.assertEqual(os.environ.get("PSE_TEST_REQUIRED_ROLES"), prior)
        self.assertEqual(os.environ.get(resources.MARKER), prior_invocation)

    def test_rust_report_configuration_precedes_libtest_separator(self) -> None:
        command, report = test_run.rust_report_command(
            self.path, ["cargo", "nextest", "run", "--", "chosen", "--exact"]
        )
        self.assertLess(command.index("--config-file"), command.index("--"))
        self.assertEqual(command[-3:], ["--", "chosen", "--exact"])
        self.assertEqual(report, self.path.with_suffix(".xml"))

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

    def test_managed_status_refuses_missing_report_and_retains_child_status(
        self,
    ) -> None:
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
                self.assertEqual(code, status or 1)
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

            def launch(
                _observation: dict[str, object], error: BaseException = error
            ) -> int:
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
        self.assertEqual(forwarded["_PSE_NATIVE_PUBLISHED_IMAGE"], artifact["binary"])
        self.assertEqual(
            run.call_args.kwargs["details"], {"artifacts": {"python": artifact}}
        )

    def test_signal_during_terminal_freeze_preserves_precommit_interruption_and_postcommit_truth(
        self,
    ) -> None:
        for phase in ("before-freeze", "after-freeze"):
            with self.subTest(phase=phase):
                self.assert_signal_freeze_case(phase)

    def assert_signal_freeze_case(self, phase: str) -> None:
        path = resources.invocation("python", [{"name": "control.py::test_case"}])
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
            # This in-process fixture writes immediately after the
            # composer's wall-clock start. Use an actual post-write
            # timestamp instead of the kernel's cached inode time.
            written = time.time_ns()
            os.utime(report, ns=(written, written))
            observation.update(child_status=0, owned_drain="complete")
            return 0

        def finish(
            selected_path: Path,
            check: dict[str, object],
            terminal_owner: str,
            *,
            deadline: float | None = None,
            deadline_supplier: Callable[[], float | None] | None = None,
            on_frozen: Callable[[], None] | None = None,
            deferred: list[str] | None = None,
        ) -> bool:
            nonlocal fired
            if not fired:
                started = check["started"]
                if not isinstance(started, (int, float)):
                    raise TypeError("Composer start must be a wall-clock timestamp")
                self.assertGreaterEqual(
                    report.stat().st_mtime,
                    started,
                    {"phase": phase, "check": check},
                )
                self.assertEqual(check["report_errors"], [], check)
                self.assertEqual(check["exit_code"], 0, check)
                fired = True
                if phase == "before-freeze":
                    os.kill(os.getpid(), signal.SIGTERM)
                else:
                    if on_frozen is None:
                        raise TypeError("Freeze control requires its commit callback")
                    callback = on_frozen

                    def committed() -> None:
                        callback()
                        os.kill(os.getpid(), signal.SIGTERM)

                    on_frozen = committed
            return original(
                selected_path,
                check,
                terminal_owner,
                deadline=deadline,
                deadline_supplier=deadline_supplier,
                on_frozen=on_frozen,
                deferred=deferred,
            )

        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(operation_lifetime, "scope_owner", return_value=None),
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
        diagnostics = {
            "phase": phase,
            "report_mtime_ns": report.stat().st_mtime_ns,
            "evidence": evidence,
            "terminal": terminal,
        }
        self.assertTrue(fired, diagnostics)
        self.assertEqual(code, 143 if phase == "before-freeze" else 0)
        self.assertEqual(evidence["exit_code"], code)
        self.assertEqual(terminal["evidence"]["workload_exit"], code)
        self.assertEqual(terminal["complete"], phase == "after-freeze", diagnostics)
        self.assertEqual(
            evidence["status"],
            "passed" if phase == "after-freeze" else "failed",
        )

    def test_projection_deadlines_and_postfreeze_signal_never_acknowledge_partial_work(
        self,
    ) -> None:
        for phase in ("original-clock", "after-freeze", "before-ack"):
            with self.subTest(phase=phase):
                self.assert_projection_deadline_case(phase)

    def assert_projection_deadline_case(self, phase: str) -> None:
        path = resources.invocation("python", [{"name": "control.py::test_case"}])
        report = path.with_suffix(".xml")
        store = resources.storage()
        members = [
            store.register(
                [str(self.root), f"{path.stem}_{index}", "evidence"],
                {
                    "invocation": path.stem,
                    "test": "control.py::test_case",
                    "state": str(self.root),
                    "database": f"{path.stem}_{index}",
                    "kind": "evidence",
                    "cleanup": "retained",
                    "disposition": "incomplete",
                    "pin": True,
                    "drained": True,
                    "references": {},
                    "borrows": {},
                    "units": [],
                },
                invocation=path.stem,
            )
            for index in range(5)
        ]
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("PSE_NATIVE_")
        }
        now = [100.0]
        original_finish = test_run.finish
        original_save = files.FileStore._save  # noqa: SLF001 -- inject elapsed projection work
        original_publish = files.publish

        def launch(observation: dict[str, object]) -> int:
            report.write_text(
                '<testsuites><testsuite><testcase classname="control.py" name="test_case"/></testsuite></testsuites>'
            )
            written = time.time_ns()
            os.utime(report, ns=(written, written))
            observation.update(child_status=0, owned_drain="complete")
            if phase == "original-clock":
                observation["cancellation_deadline"] = 105.0
            return 0

        def finish(
            selected_path: Path,
            check: dict[str, object],
            terminal_owner: str,
            *,
            deadline: float | None = None,
            deadline_supplier: Callable[[], float | None] | None = None,
            on_frozen: Callable[[], None] | None = None,
            deferred: list[str] | None = None,
        ) -> bool:
            if on_frozen is None:
                raise TypeError("Freeze control requires its commit callback")
            callback = on_frozen

            def frozen() -> None:
                callback()
                if phase == "after-freeze":
                    os.kill(os.getpid(), signal.SIGTERM)

            return original_finish(
                selected_path,
                check,
                terminal_owner,
                deadline=deadline,
                deadline_supplier=deadline_supplier,
                on_frozen=frozen,
                deferred=deferred,
            )

        def save(selected: files.FileStore, entry: files.Entry) -> None:
            original_save(selected, entry)
            if phase != "before-ack":
                now[0] += 3 if phase == "original-clock" else 6

        def publish(destination: Path, value: Mapping[str, object]) -> None:
            original_publish(destination, value)
            if phase == "before-ack" and destination == path.with_suffix(
                ".terminal.json"
            ):
                os.kill(os.getpid(), signal.SIGTERM)
                now[0] += 11

        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(files.time, "monotonic", side_effect=lambda: now[0]),
            patch.object(operation_lifetime, "scope_owner", return_value=None),
            patch.object(
                native_operation, "note_composer_cancellation"
            ) as cancellation,
            patch.object(native_operation, "acknowledge_composer") as acknowledge,
            patch.object(test_run, "finish", side_effect=finish),
            patch.object(files.FileStore, "_save", new=save),
            patch.object(files, "publish", side_effect=publish),
        ):
            code = test_run.execute_selected(
                path,
                ["projection-control"],
                environment,
                report,
                selected=[{"class": "control.py", "name": "test_case"}],
                launch=launch,
            )
        acknowledge.assert_not_called()
        if phase != "original-clock":
            cancellation.assert_called_once()
            self.assertEqual(cancellation.call_args.args, (signal.SIGTERM, 110.0))
        self.assertEqual(code, 0 if phase == "before-ack" else 1)
        state = store.invocation_snapshot(path.stem)
        self.assertTrue(state["terminal"]["complete"])
        self.assertEqual(state["terminal"]["evidence"]["workload_exit"], 0)
        self.assertEqual(state["terminal"]["evidence"]["report_errors"], [])
        evidence = files.read(path.with_suffix(".terminal.json"))
        if phase == "before-ack":
            self.assertEqual(state["projection"], "complete")
            self.assertEqual(evidence["status"], "passed")
        else:
            self.assertNotIn("projection", state)
            projected = sum(
                "terminal_identity" in store.lookup(resource).facts
                for resource in members
            )
            self.assertGreater(projected, 0)
            self.assertLess(projected, len(members))
            self.assertEqual(evidence["status"], "failed")
            self.assertTrue(
                any(
                    "deadline exhausted" in error for error in evidence["report_errors"]
                )
            )

    def test_non_native_ignored_sigterm_escalates_within_original_clock(self) -> None:
        environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("PSE_NATIVE_")
        }
        child = "import os,signal,time; signal.signal(signal.SIGTERM,signal.SIG_IGN); os.kill(os.getppid(),signal.SIGTERM); time.sleep(30)"
        started = time.monotonic()
        original_run = native_operation.run

        def bounded_run(
            command: list[str],
            env: dict[str, str],
            *,
            cwd: Path,
            observation: dict[str, object],
        ) -> int:
            # Exercise actual signal delivery/escalation under an existing caller
            # cancellation clock, without waiting out the production ten-second grace.
            deadline = time.monotonic() + 2
            return original_run(
                command,
                env,
                cwd=cwd,
                observation=observation,
                cleanup_clock=lambda: deadline,
            )

        with (
            patch.dict(os.environ, environment, clear=True),
            patch.object(operation_lifetime, "scope_owner", return_value=None),
            patch.object(native_operation, "cancel_foreground_handoffs"),
            patch.object(native_operation, "run", side_effect=bounded_run) as run,
        ):
            code = test_run.execute_selected(
                self.path,
                [sys.executable, "-c", child],
                environment,
                self.report,
                selected=[{"nodeid": "control.py::test_case"}],
            )
        run.assert_called_once()
        self.assertLess(time.monotonic() - started, 4)
        evidence = files.read(self.path.with_suffix(".terminal.json"))
        self.assertEqual(code, 143)
        self.assertEqual(evidence["process"]["child_status"], -signal.SIGKILL)
        self.assertEqual(evidence["process"]["forwarded_signal"], signal.SIGTERM)
        self.assertFalse(resources.storage().terminal(self.path.stem)[1]["complete"])

    def test_non_native_forwarded_signal_survives_child_passing_report_and_zero_exit(
        self,
    ) -> None:
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
            patch.object(operation_lifetime, "scope_owner", return_value=None),
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
                "from scripts import native_operation, native_tests, operation_lifetime, scientific_inputs, test_run",
                # This control owns a real process tree without a systemd scope.
                "operation_lifetime.scope_owner=lambda **_options:None",
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


@patch(
    "scripts.cargo_tools.selected_identity",
    new=lambda name: {
        "name": name,
        "version": "0.9.146",
        "path": "/owned/cargo-nextest",
        "sha256": "selected",
    },
)
class OrdinaryRustScientificInputTests(unittest.TestCase):
    """Exercise ordinary execution independently of native observer preparation."""

    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.environment = {
            "PSE_TEST_RESOURCE_STATE": str(self.root / "registry"),
            "RUST_MIN_STACK": str(16 << 20),
        }
        selected = patch.dict(os.environ, self.environment, clear=True)
        selected.start()
        self.addCleanup(selected.stop)
        self.events: list[str] = []
        self.captured: Path | None = None
        self.capsule = {
            "schema": "scientific-inputs-v1",
            "files": {"packages/reference/unit/models/law.pse": list(b"captured law")},
            "families": {
                "control": {
                    "packages": ["packages/reference/unit"],
                    "documents": ["packages/reference/unit/models/law.pse"],
                    "substitutions": [],
                    "projection": "declarations",
                }
            },
            "directories": {},
            "package_headers": [],
        }

    def inventory(
        self, package: str = "pse-compiler", *, selected: bool = True
    ) -> dict:
        return {
            "rust-suites": {
                "control": {
                    "binary-id": f"{package}::lib",
                    "binary-path": "/prebuilt/control",
                    "testcases": {
                        "scientific_control": {
                            "filter-match": {
                                "status": "matches" if selected else "mismatch",
                            }
                        }
                    },
                }
            }
        }

    def test_fresh_local_selection_prepares_once_and_executes_retained_metadata(
        self,
    ) -> None:
        inventory = self.inventory()

        def prepare(
            command: list[str],
            provenance: Path,
            selection: Path,
            _environment: Mapping[str, str],
            **options: str | bool | None,
        ) -> tuple[list[str], Path | None]:
            self.assertEqual(command, ["cargo", "nextest", "run", "-p", "pse-compiler"])
            self.assertEqual(
                options,
                {"effects": "native-local", "package_scope": True, "canonical": False},
            )
            selection.write_text(json.dumps(inventory))
            self.captured = provenance.with_suffix(".scientific.json")
            self.captured.write_text(json.dumps(self.capsule))
            self.events.append("prepare")
            return [
                "--binaries-metadata",
                "/retained/binaries.json",
                "--cargo-metadata",
                "/retained/cargo.json",
            ], None

        with (
            patch.object(test_run.validation, "fresh_output", return_value=self.root),
            patch.object(
                native_tests, "ordinary_rust_capture", side_effect=prepare
            ) as prepared,
            patch.object(
                native_tests,
                "rust_fixture_bindings",
                side_effect=lambda *_args: {scientific_inputs.ROLE: str(self.captured)},
            ),
            patch.object(
                native_tests,
                "prepared_rust_inventory",
                side_effect=lambda _args, _env, **kwargs: kwargs["inventory"],
            ) as admitted,
            patch.object(test_run, "execute_selected", return_value=7) as execution,
            patch.object(
                test_run,
                "observer_rust",
                side_effect=AssertionError("local observer acquisition"),
            ),
            patch.object(
                test_run.subprocess,
                "run",
                side_effect=AssertionError("fallback list/build"),
            ),
        ):
            self.assertEqual(
                test_run.run_rust(
                    ["cargo", "nextest", "run", "-p", "pse-compiler"],
                    env=self.environment,
                    effects="native-local",
                    package_scope=True,
                ),
                7,
            )
        prepared.assert_called_once()
        admitted.assert_called_once()
        run = execution.call_args.args[1]
        self.assertIn("--binaries-metadata", run)
        self.assertIn("--cargo-metadata", run)
        self.assertNotIn("-p", run)
        self.assertEqual(
            execution.call_args.kwargs["details"]["scientific_inputs"]["path"],
            str(self.captured),
        )

    def test_retained_inventory_uses_prepared_input_without_listing_or_recapture(
        self,
    ) -> None:
        capsule = self.root / "retained.json"
        capsule.write_text(json.dumps(self.capsule))
        inventory = self.inventory()
        environment = {**self.environment, scientific_inputs.ROLE: str(capsule)}
        command = [
            "cargo",
            "nextest",
            "run",
            "--binaries-metadata",
            "/retained/binaries.json",
            "--cargo-metadata",
            "/retained/cargo.json",
        ]

        def validate_prepared(
            arguments: list[str], actual_environment: dict, *, inventory: dict
        ) -> dict:
            self.assertEqual(arguments, command[3:])
            self.assertEqual(
                actual_environment,
                environment | {test_run.EXECUTION_EFFECTS: "native-local"},
            )
            self.assertEqual(inventory, self.inventory())
            return inventory

        with (
            patch.object(
                native_tests, "prepared_rust_inventory", side_effect=validate_prepared
            ) as prepared,
            patch.object(
                test_run.subprocess, "run", side_effect=AssertionError("re-enumeration")
            ) as listed,
            patch.object(
                scientific_inputs, "capture", side_effect=AssertionError("recapture")
            ) as captured,
            patch.object(test_run, "execute_selected", return_value=0) as execute,
        ):
            self.assertEqual(
                test_run.run_rust(
                    command,
                    env=environment,
                    inventory=inventory,
                    effects="native-local",
                ),
                0,
            )
        prepared.assert_called_once()
        listed.assert_not_called()
        captured.assert_not_called()
        self.assertEqual(
            execute.call_args.args[2][scientific_inputs.ROLE], str(capsule)
        )
        self.assertEqual(
            execute.call_args.kwargs["details"]["scientific_inputs"]["sha256"],
            hashlib.sha256(capsule.read_bytes()).hexdigest(),
        )

    def test_canonical_retained_catalogue_is_prepared_before_observer_acquisition(
        self,
    ) -> None:
        inventory = self.inventory("pse-operations")
        retained = [
            "--binaries-metadata",
            "/retained/binaries.json",
            "--cargo-metadata",
            "/retained/cargo.json",
        ]
        command = [
            "cargo",
            "nextest",
            "run",
            "-p",
            "pse-operations",
            "--features",
            "pse-operations/canonical-tests",
        ]
        worker = self.root / "worker"
        state = self.root / "service"
        environment = {
            **self.environment,
            "PSE_NATIVE_OPERATION": "/authenticated/operation",
            "PSE_SURREAL_STATE": str(state),
        }

        def prepare(
            actual: list[str],
            provenance: Path,
            selection: Path,
            actual_environment: Mapping[str, str],
            **options: str | bool | None,
        ) -> tuple[list[str], Path | None]:
            self.assertEqual(actual, command)
            self.assertEqual(
                options,
                {"effects": "canonical", "package_scope": False, "canonical": True},
            )
            self.assertEqual(actual_environment["PSE_SURREAL_STATE"], str(state))
            selection.write_text(json.dumps(inventory))
            provenance.write_text("{}")
            self.events.append("prepare")
            return retained, worker

        def compose(
            _path: Path,
            _command: Sequence[str],
            _environment: Mapping[str, str],
            _report: Path,
            *,
            launch: Callable[[dict[str, object]], int] | None = None,
            **_options: object,
        ) -> int:
            self.events.append("compose")
            if launch is None:
                self.fail("Expected the original launch callback")
            return launch({})

        def observe(actual_state: Path, child: list[str], **_options: object) -> int:
            self.assertEqual(self.events, ["prepare", "compose"])
            self.assertEqual(actual_state, state)
            self.assertEqual(
                child[:4],
                [sys.executable, "-m", "scripts.test_run", "--observer-child"],
            )
            self.assertIn("--binaries-metadata", child)
            self.assertIn("--cargo-metadata", child)
            self.assertNotIn("--features", child)
            self.assertNotIn("-p", child)
            self.assertEqual(
                json.loads(Path(os.environ["PSE_NATIVE_SELECTION"]).read_text()),
                inventory,
            )
            self.assertEqual(os.environ["PSE_WORKER_BINARY"], str(worker))
            self.events.append("observe")
            return 9

        with (
            patch.object(test_run.validation, "fresh_output", return_value=self.root),
            patch.object(
                native_tests, "ordinary_rust_capture", side_effect=prepare
            ) as prepared,
            patch.object(native_tests, "rust_fixture_bindings", return_value={}),
            patch.object(test_run, "execute_selected", side_effect=compose),
            patch("scripts.surreal_server.observer", side_effect=observe) as observer,
            patch.object(
                test_run.subprocess,
                "run",
                side_effect=AssertionError("duplicate listing or build"),
            ),
        ):
            self.assertEqual(test_run.run_rust(command, env=environment), 9)
        prepared.assert_called_once()
        observer.assert_called_once()
        self.assertEqual(self.events, ["prepare", "compose", "observe"])
        self.assertNotIn("PSE_NATIVE_SELECTION", os.environ)
        self.assertNotIn("PSE_WORKER_BINARY", os.environ)

    def test_retained_metadata_without_catalogue_refuses_before_preparation_or_invocation(
        self,
    ) -> None:
        for effects in ("native-local", "canonical"):
            with (
                self.subTest(effects=effects),
                patch.object(native_tests, "ordinary_rust_capture") as prepare,
                patch.object(resources, "invocation") as invocation,
                self.assertRaisesRegex(ValueError, "requires its prepared inventory"),
            ):
                test_run.run_rust(
                    [
                        "cargo",
                        "nextest",
                        "run",
                        "--binaries-metadata",
                        "/retained/binaries.json",
                        "--cargo-metadata",
                        "/retained/cargo.json",
                    ],
                    env=self.environment,
                    effects=effects,
                )
            prepare.assert_not_called()
            invocation.assert_not_called()


if __name__ == "__main__":
    unittest.main()

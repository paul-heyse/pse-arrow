# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Injected caller hangs and pre-snapshot primary ownership, without native work.

Run this file directly with the checkout Python to test only private lifecycle
controls, independently of the installed extension and canonical substrate.
"""

import importlib.util
import os
import subprocess
import sys
import tempfile
import time
import unittest
from contextlib import AbstractContextManager
from pathlib import Path
from unittest.mock import patch

import pytest

ROOT = str(Path(__file__).resolve().parents[3])
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

SPEC = importlib.util.spec_from_file_location(
    "managed_study_fixture_controls",
    Path(__file__).with_name("managed_study_fixture.py"),
)
assert SPEC is not None
assert SPEC.loader is not None
FIXTURE = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = FIXTURE
SPEC.loader.exec_module(FIXTURE)


@pytest.mark.unit
class ServiceControllerRoutingTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.owner = Path(temporary.name)
        self.state = self.owner / ".receivers" / "context" / "isolated"
        self.state.mkdir(mode=0o700, parents=True)
        self.generation = self.owner / ".generations" / ("b" * 64)
        self.script = self.generation / "scripts" / "surreal_server.py"
        for directory in (
            self.owner / ".generations",
            self.generation,
            self.script.parent,
        ):
            directory.mkdir(mode=0o700)
        # No script-content hash is evaluated by this reader. These are the
        # construction owner's retained declaration and exact selected role.
        FIXTURE._write_private(self.script, "admitted storage entry point")
        self.controller = {
            "supervisor_executable": sys.executable,
            "supervisor_script": str(self.script),
            "supervisor_sha256": "2" * 64,
        }
        self.config = {"service_supervisor": self.controller}
        self.receiver = {
            "service_state": str(self.owner),
            "service_supervisor": self.controller,
            "primary_receiver": {
                "supervisor_script": str(
                    self.owner
                    / ".generations"
                    / ("c" * 64)
                    / "scripts/surreal_server.py"
                ),
                "worker_executable": "/independent/admitted-worker",
            },
        }
        FIXTURE._write_private(self.state / "config.json", self.receiver)
        self.manifest = {
            "version": 2,
            "identity": "b" * 64,
            "files": {"scripts/surreal_server.py": "2" * 64},
        }
        FIXTURE._write_private(self.generation / "generation.json", self.manifest)

    def test_distinct_receiver_uses_actual_service_controller_without_source_rehash(
        self,
    ) -> None:
        # Only the three controller strings participate in this projection.
        # Legitimate execution/producer metadata belongs to its separate role.
        self.config["service_supervisor"] = {
            **self.controller,
            "worker_metadata": {"producer": "structured"},
        }
        self.receiver["service_supervisor"] = {
            **self.controller,
            "producer_metadata": {"context": "independent"},
        }
        with (
            patch.object(
                FIXTURE.server, "config_for", return_value=self.config
            ) as config,
            patch.object(
                FIXTURE.server,
                "verify_generation",
                side_effect=AssertionError("source rescan"),
            ),
            patch.object(
                FIXTURE.server,
                "file_digest",
                side_effect=AssertionError("program rehash"),
            ),
        ):
            selected = FIXTURE.server.admitted_service_controller(
                self.state, self.receiver
            )
        self.assertEqual(selected, self.controller)
        self.assertNotEqual(
            selected["supervisor_script"],
            self.receiver["primary_receiver"]["supervisor_script"],
        )
        config.assert_called_once_with(self.owner)
        self.assertEqual(
            self.receiver["primary_receiver"]["worker_executable"],
            "/independent/admitted-worker",
        )

    def test_changed_foreign_or_missing_controller_refuses(self) -> None:
        for reason in ("missing", "foreign", "receiver", "manifest"):
            controller = self.controller.copy()
            receiver = self.receiver.copy()
            config = self.config.copy()
            manifest = self.manifest.copy()
            if reason == "missing":
                config = {}
            elif reason == "foreign":
                controller["supervisor_script"] = str(self.owner / "foreign.py")
                receiver["service_supervisor"] = controller
                config["service_supervisor"] = controller
            elif reason == "receiver":
                receiver["service_supervisor"] = {
                    **controller,
                    "supervisor_script": "/foreign.py",
                }
            else:
                manifest["identity"] = "d" * 64
            FIXTURE._write_private(self.generation / "generation.json", manifest)
            with (
                self.subTest(reason=reason),
                patch.object(FIXTURE.server, "config_for", return_value=config),
                self.assertRaises(FIXTURE.server.SupervisorError),
            ):
                FIXTURE.server.admitted_service_controller(self.state, receiver)


@pytest.mark.unit
class ManagedStudyControlTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        root = Path(self.temporary.name)
        self.state = root / "state"
        self.state.mkdir(mode=0o700)
        self.resources = FIXTURE.test_resources
        registry = patch.object(
            self.resources.service_borrowers, "registry", return_value=root / "registry"
        )
        registry.start()
        self.addCleanup(registry.stop)
        config = patch.object(
            FIXTURE.server,
            "config_for",
            return_value={
                "instance_id": "owned-service",
                "namespace": "pse",
                "admission": "open",
                "accepting_writes": True,
            },
        )
        config.start()
        self.addCleanup(config.stop)
        self.selected_identity = (
            os.environ.get("PYTEST_CURRENT_TEST", "").rsplit(" (", 1)[0] or self.id()
        )
        # This private invocation owns mocked control metadata only. Its explicit
        # canonical demand does not alter pytest's LOCAL requirement or admit
        # native science, an actual receiver or the canonical substrate.
        self.invocation = self.resources.invocation(
            "python",
            [{"name": self.selected_identity}],
            requirements={
                self.selected_identity: {
                    "effects": ["canonical"],
                    "fixtures": [],
                    "capacity": "canonical",
                    "roles": [],
                }
            },
        )
        environment = patch.dict(
            os.environ,
            {
                self.resources.MARKER: str(self.invocation),
                "PSE_TEST_EXECUTION_EFFECTS": "canonical",
            },
        )
        environment.start()
        self.addCleanup(environment.stop)
        self.fixture = FIXTURE.ManagedStudyFixture(
            str(self.state), test=self.selected_identity
        )
        self.fixture.database = f"canonical_test_{self.fixture.nonce}"
        self.fixture.controls_resource = self.resources.register(
            {
                "state": str(self.state),
                "database": self.fixture.database,
                "kind": "controls",
                "directory": str(self.fixture.directory),
                "test": self.selected_identity,
            }
        )
        self.fixture.directory.mkdir(mode=0o700)
        with FIXTURE.server.state_lock(self.state):
            FIXTURE._write_private(
                self.fixture.lock,
                {
                    "nonce": self.fixture.nonce,
                    "pid": os.getpid(),
                    "start": FIXTURE.operation_lifetime.start_identity(os.getpid()),
                },
            )
            self.fixture.lock_owned = True
        self.held = FIXTURE.NativeEntries(
            self.fixture.nonce,
            self.fixture.database,
            os.getpid(),
            tuple(FIXTURE.NativeEntry(str(index), False, False) for index in range(16)),
            16,
            16,
        )

    def retire_controls(self) -> None:
        resource = self.fixture.controls_resource
        record = self.resources.resource_status(resource)
        assert record["drained"] is True
        assert record["pin"] is True
        assert self.fixture.directory.exists()
        assert self.resources.reclaim(resource) == "retention pin"
        assert self.fixture.directory.exists()
        assert self.resources.finalize(
            self.invocation,
            {self.selected_identity: "pass"},
            complete=True,
            terminal_owner="runner",
        ) == [resource]
        assert self.resources.reclaim(resource) == "removed"
        assert not self.fixture.directory.exists()

    def caller(self, body: str) -> list[str]:
        script = (
            "import importlib.util,sys,time,signal; from pathlib import Path; "
            f"sys.path.insert(0, {ROOT!r}); "
            "spec=importlib.util.spec_from_file_location("
            "'managed_study_fixture_controls', "
            f"{str(Path(__file__).with_name('managed_study_fixture.py'))!r}); "
            "fixture=importlib.util.module_from_spec(spec); "
            "sys.modules[spec.name]=fixture; "
            "spec.loader.exec_module(fixture); "
            "directory=Path(sys.argv[1]); "
            "fixture._write_private(directory/'calling.json', "
            "{'nonce':sys.argv[2],'started':time.monotonic()});\n" + body
        )
        return [
            sys.executable,
            "-c",
            script,
            str(self.fixture.directory),
            self.fixture.nonce,
        ]

    def test_control_invocation_refuses_native_and_foreign_test_demand(self) -> None:
        self.resources.require_effect(
            {"test": self.selected_identity}, effect="canonical"
        )
        with pytest.raises(
            self.resources.ResourceError, match="Undeclared native demand"
        ):
            self.resources.require_effect(
                {"test": self.selected_identity}, effect="native"
            )
        with pytest.raises(
            self.resources.ResourceError, match="not uniquely collected"
        ):
            self.resources.require_effect(
                {"test": self.selected_identity + "::foreign"}
            )
        self.fixture.close()
        self.retire_controls()

    def test_close_preserves_lock_published_by_another_nonce(self) -> None:
        record = FIXTURE._read_object(self.fixture.lock)
        record["nonce"] = "another-qualification-owner"
        FIXTURE._write_private(self.fixture.lock, record)
        self.fixture.close()
        assert FIXTURE._read_object(self.fixture.lock) == record
        assert not self.fixture.lock_owned
        self.retire_controls()

    def test_malformed_lock_is_retained_instead_of_accepted_as_owner(self) -> None:
        self.fixture.lock.write_bytes(b"")
        with pytest.raises(FIXTURE.msgspec.DecodeError):
            self.fixture.close()
        assert self.fixture.lock.exists()
        assert self.fixture.lock_owned
        assert self.fixture.directory.exists()
        record = self.resources.resource_status(self.fixture.controls_resource)
        assert record["pin"] is True

    def test_hanging_public_caller_receives_sigint_and_joins_before_cleanup(
        self,
    ) -> None:
        interrupted = self.state / "interrupted"
        command = self.caller(
            "def interrupted(signum, frame):\n"
            f"    Path({str(interrupted)!r}).touch()\n"
            "    time.sleep(0.05)\n"
            "    raise KeyboardInterrupt\n"
            "signal.signal(signal.SIGINT, interrupted)\n"
            "while True: time.sleep(0.01)\n"
        )
        began = time.monotonic()
        with (
            patch.object(self.fixture, "observation", return_value=self.held),
            pytest.raises(AssertionError, match="public study operation timed out"),
        ):
            self.fixture.call(command, lambda _value: None, timeout=0.15)
        assert time.monotonic() - began < 2
        assert interrupted.exists()
        assert self.fixture.caller.returncode is not None
        assert not self.fixture.lock.exists()
        self.retire_controls()

    def test_unjoined_caller_retains_controls_lock_and_exact_pid(self) -> None:
        command = self.caller(
            "signal.signal(signal.SIGINT, signal.SIG_IGN)\n"
            "while True: time.sleep(0.01)\n"
        )
        self.fixture.drain_deadline = time.monotonic() + 0.4
        try:
            with (
                patch.object(self.fixture, "observation", return_value=self.held),
                pytest.raises(AssertionError, match="did not drain; controls retained"),
            ):
                self.fixture.call(command, lambda _value: None, timeout=0.1)
            caller = self.fixture.caller
            assert caller.poll() is None
            assert self.fixture.directory.exists()
            assert self.fixture.lock.exists()
            receipt = FIXTURE._read_object(self.fixture.directory / "caller.json")
            assert receipt == {"pid": caller.pid, "nonce": self.fixture.nonce}
            retained_deadline = self.fixture.drain_deadline
            with pytest.raises(
                AssertionError, match="did not drain; controls retained"
            ):
                self.fixture.close()
            assert self.fixture.drain_deadline == retained_deadline
        finally:
            # This is an injected Python sleeper, with no native owner or future.
            self.fixture.caller.terminate()
            self.fixture.caller.wait(timeout=2)

    def test_child_assertion_is_not_swallowed(self) -> None:
        command = self.caller("raise AssertionError('retained scientific assertion')\n")
        with (
            patch.object(self.fixture, "observation", return_value=self.held),
            pytest.raises(AssertionError, match="retained scientific assertion"),
        ):
            self.fixture.call(command, lambda _value: None)
        assert self.fixture.caller.returncode == 1
        assert not self.fixture.lock.exists()
        self.retire_controls()

    def test_child_setup_is_bounded_before_public_call_marker(self) -> None:
        with pytest.raises(AssertionError, match="public caller setup timed out"):
            self.fixture.call(
                [sys.executable, "-c", "import time; time.sleep(10)"],
                lambda _value: None,
                setup_timeout=0.1,
            )
        assert self.fixture.caller.returncode is not None
        assert not self.fixture.lock.exists()
        self.retire_controls()

    def primary(
        self, body: str, *, directory: Path | None = None
    ) -> subprocess.Popen[bytes]:
        selected = self.fixture.directory if directory is None else directory
        process = subprocess.Popen(
            [
                sys.executable,
                "-c",
                body,
                "--qualification-native-entry",
                str(selected),
                "--canonical-database",
                self.fixture.database,
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )

        def cleanup() -> None:
            # These injected startup processes have no native owners.
            if process.poll() is None:
                process.terminate()
            process.wait(timeout=2)

        self.addCleanup(cleanup)
        return process

    def service(
        self, process: subprocess.Popen[bytes]
    ) -> tuple[AbstractContextManager[object], AbstractContextManager[object]]:
        original_read = Path.read_text

        def read(
            path: Path, encoding: str | None = None, errors: str | None = None
        ) -> str:
            if str(path) == "/sys/fs/cgroup/managed-fixture-test/cgroup.events":
                return f"populated {int(process.poll() is None)}\n"
            if str(path) == f"/proc/{process.pid}/cgroup":
                return "0::/managed-fixture-test\n"
            return original_read(path, encoding=encoding, errors=errors)

        def observe(
            command: list[str], **_kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess(
                command,
                0,
                "LoadState=loaded\nActiveState=active\n"
                f"ControlGroup=/managed-fixture-test\nMainPID={process.pid}\n",
                "",
            )

        return patch.object(Path, "read_text", read), patch.object(
            FIXTURE.subprocess, "run", observe
        )

    def test_pre_snapshot_primary_must_exit_before_controls_are_removed(self) -> None:
        process = self.primary(
            "from pathlib import Path; import sys,time; directory=Path(sys.argv[2]);\n"
            "while not (directory/'stop.json').exists(): time.sleep(0.005)\n"
            "assert (directory/'release.json').exists()\n"
            "time.sleep(0.05)\n"
            "assert directory.exists()\n"
        )
        self.fixture.primary_attempted = True
        with self.service(process)[0], self.service(process)[1]:
            self.fixture.close()
        assert process.wait(timeout=2) == 0
        assert not self.fixture.lock.exists()
        self.retire_controls()

    def test_malformed_snapshot_does_not_release_a_live_primary_allocation(
        self,
    ) -> None:
        process = self.primary("import time; time.sleep(10)")
        self.fixture.primary_attempted = True
        (self.fixture.directory / "entries.json").write_bytes(b"{bad snapshot")
        (self.fixture.directory / "entries.json").chmod(0o600)
        self.fixture.drain_deadline = time.monotonic() + 0.15
        with (
            self.service(process)[0],
            self.service(process)[1],
            pytest.raises(
                AssertionError,
                match="qualified primary did not drain; controls retained",
            ),
        ):
            self.fixture.close()
        assert process.poll() is None
        assert self.fixture.lock.exists()
        assert self.fixture.directory.exists()
        assert (self.fixture.directory / "stop.json").exists()

    def test_unrelated_primary_is_not_adopted_or_signalled(self) -> None:
        process = self.primary(
            "import time; time.sleep(10)", directory=self.state / "unrelated"
        )
        self.fixture.primary_attempted = True
        with (
            self.service(process)[0],
            self.service(process)[1],
            pytest.raises(AssertionError, match="not associated with this fixture"),
        ):
            self.fixture.close()
        assert self.fixture.lock.exists()
        assert self.fixture.directory.exists()
        assert not (self.fixture.directory / "stop.json").exists()
        assert process.poll() is None

    def test_handoff_refuses_foreign_live_observer_before_stopping_same_database(
        self,
    ) -> None:
        from scripts import (  # noqa: PLC0415 -- pure control owner
            case_measure,
            surreal_server,
        )

        process = self.primary("import time; time.sleep(10)")
        observed = Path(f"/proc/{process.pid}")
        FIXTURE._write_private(
            self.state / "primary-observer.json",
            {
                "pid": process.pid,
                "start": (observed / "stat").read_text().rsplit(")", 1)[1].split()[19],
                "group": "/same-observer-group",
            },
        )
        with (
            patch.object(case_measure, "stop_managed_primary") as stop,
            patch.object(
                surreal_server,
                "primary_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "ControlGroup": "/primary",
                },
            ),
            pytest.raises(AssertionError, match="another live caller owns"),
        ):
            self.fixture._handoff_prior_observer()
        stop.assert_not_called()
        assert process.poll() is None

    def test_handoff_refuses_live_primary_without_current_observer_ownership(
        self,
    ) -> None:
        from scripts import (  # noqa: PLC0415 -- pure control owner
            case_measure,
            surreal_server,
        )

        with (
            patch.object(case_measure, "stop_managed_primary") as stop,
            patch.object(
                surreal_server,
                "primary_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "ControlGroup": "/primary",
                },
            ),
            pytest.raises(AssertionError, match="owned by another observer"),
        ):
            self.fixture._handoff_prior_observer()
        stop.assert_not_called()

    def test_handoff_releases_only_current_observer_after_prior_primary_drained(
        self,
    ) -> None:
        from scripts import (  # noqa: PLC0415 -- pure control owner
            case_measure,
            surreal_server,
        )

        process = Path(f"/proc/{os.getpid()}")
        FIXTURE._write_private(
            self.state / "primary-observer.json",
            {
                "pid": os.getpid(),
                "start": (process / "stat").read_text().rsplit(")", 1)[1].split()[19],
                "group": next(
                    line.removeprefix("0::")
                    for line in (process / "cgroup").read_text().splitlines()
                    if line.startswith("0::")
                ),
            },
        )
        with (
            patch.object(case_measure, "stop_managed_primary") as stop,
            patch.object(surreal_server, "config_for", return_value={}),
            patch.object(
                surreal_server,
                "primary_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "inactive",
                    "ControlGroup": "",
                },
            ),
        ):
            self.fixture._handoff_prior_observer()
        stop.assert_not_called()
        assert not (self.state / "primary-observer.json").exists()


if __name__ == "__main__":
    unittest.main()

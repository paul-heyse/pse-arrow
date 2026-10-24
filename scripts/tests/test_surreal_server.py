# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Scoped controls for owned SurrealDB state and lifecycle boundaries."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import errno
import fcntl
import importlib.util
import io
import json
import os
import shutil
import stat
import subprocess
import sys
import tempfile
import threading
import time
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import MagicMock, call, patch

from scripts import host_admission as host
from scripts import surreal_server as server
from scripts.tests import canonical_recovery_check as recovery
from scripts.tests.server_receipt_fixture import fixture_build_context, fixture_receipt
from scripts.tests.test_surrealism_module import fixture_module

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping
    from types import ModuleType
    from typing import TextIO


def readiness_fixture(phase: server.Readiness) -> server.ServiceObservation:
    return server.ServiceObservation(
        phase,
        Path("/fixture"),
        "fixture",
        "fixture.service",
        "controlled observation",
        manager={"ActiveState": "active"}
        if phase in {server.Readiness.READY, server.Readiness.STARTING}
        else {},
        listener_owned=phase in {server.Readiness.READY, server.Readiness.STARTING},
        protocol_ready=phase is server.Readiness.READY,
    )


def record(value: object) -> dict[str, object]:
    if not isinstance(value, dict):
        raise TypeError("Expected an original owner record")
    return value


class AdministrativeTransportTests(unittest.TestCase):
    def test_preserves_multiline_comment_and_literal_bytes_and_explicit_errors(
        self,
    ) -> None:
        query = (
            "// retained comment\nRETURN 'line\\n// literal';\nTHROW 'fixture refusal';"
        )
        outcomes = [
            {"status": "OK", "result": "line\\n// literal"},
            {"status": "ERR", "result": "fixture refusal"},
        ]
        with (
            patch.object(server, "config_for", return_value={"port": 18080}),
            patch.object(
                server,
                "read_json",
                return_value={"username": "fixture", "password": "private"},
            ),
            patch.object(
                server.urllib.request,
                "urlopen",
                return_value=io.BytesIO(json.dumps(outcomes).encode()),
            ) as send,
        ):
            self.assertEqual(
                server.administrative_results(
                    Path("/fixture"),
                    query,
                    time.monotonic() + 5,
                    namespace="owned",
                    database="fixture",
                ),
                outcomes,
            )
        request = send.call_args.args[0]
        self.assertEqual(request.data, query.encode())
        self.assertEqual(request.get_header("Surreal-ns"), "owned")
        self.assertEqual(request.get_header("Surreal-db"), "fixture")
        with (
            patch.object(server, "administrative_results", return_value=outcomes),
            self.assertRaisesRegex(server.SupervisorError, "failed or incomplete"),
        ):
            server.administrative_query(Path("/fixture"), query, time.monotonic() + 5)

    def test_refuses_malformed_or_oversize_statement_envelopes(self) -> None:
        for payload in (
            b"not json",
            b"{}",
            b"[]",
            b"[null]",
            b'[{"status":"OK"}]',
            b'[{"status":[],"result":null}]',
            b'[{"status":"ERR","result":null}]',
            b'[{"status":"unknown","result":null}]',
            b" " * (server.MESSAGE_BYTES + 1),
        ):
            with (
                self.subTest(payload_size=len(payload)),
                patch.object(server, "config_for", return_value={"port": 18080}),
                patch.object(
                    server,
                    "read_json",
                    return_value={"username": "fixture", "password": "private"},
                ),
                patch.object(
                    server.urllib.request, "urlopen", return_value=io.BytesIO(payload)
                ),
                self.assertRaisesRegex(
                    server.SupervisorError, "invalid or incomplete|exceeded|valid JSON"
                ),
            ):
                server.administrative_results(
                    Path("/fixture"),
                    "RETURN 1;",
                    time.monotonic() + 5,
                    namespace="owned",
                    database="fixture",
                )


class LifecycleClockTests(unittest.TestCase):
    def test_lifecycle_deadline_narrows_nested_clock_and_restores_on_failure(
        self,
    ) -> None:
        with (
            patch.object(server.time, "monotonic", return_value=100),
            patch.object(server._STARTUP, "deadline", None, create=True),  # noqa: SLF001 -- observe exact nested clock restoration
        ):
            self.assertIsNone(server._STARTUP.deadline)  # noqa: SLF001 -- establish the original absent clock
            with server.lifecycle_deadline(140) as outer:
                self.assertEqual(outer, 140)
                with server.lifecycle_deadline(160) as inner:
                    self.assertEqual(inner, 140)
                    self.assertEqual(server._STARTUP.deadline, 140)  # noqa: SLF001 -- later offers cannot renew the original clock
                with (
                    self.assertRaisesRegex(ValueError, "controlled failure"),
                    server.lifecycle_deadline(120),
                ):
                    self.assertEqual(server._STARTUP.deadline, 120)  # noqa: SLF001 -- earlier offers narrow the shared control clock
                    raise ValueError("controlled failure")
                self.assertEqual(server._STARTUP.deadline, 140)  # noqa: SLF001 -- nested failure restores the enclosing clock
            self.assertIsNone(server._STARTUP.deadline)  # noqa: SLF001 -- final exit restores the original absence

    def test_lifecycle_deadline_refuses_nonfinite_offers(self) -> None:
        for deadline in (float("nan"), float("inf")):
            with (
                self.subTest(deadline=deadline),
                self.assertRaisesRegex(
                    server.SupervisorError, "finite absolute deadline"
                ),
                server.lifecycle_deadline(deadline),
            ):
                self.fail("nonfinite clock entered lifecycle composition")

    def test_expired_readmission_refuses_before_reservation_or_publication(
        self,
    ) -> None:
        with (
            patch.object(server.time, "monotonic", return_value=100),
            patch.object(server._STARTUP, "deadline", None, create=True),  # noqa: SLF001 -- original control clock is absent
            patch.object(server, "lifecycle_reservation") as reserve,
            patch.object(server, "publish_generation") as publish,
            self.assertRaises(server.DeadlineExpiredError),
        ):
            server.readmit_supervisor(Path("/selected"), deadline=99)
        reserve.assert_not_called()
        publish.assert_not_called()

    def test_readmission_expiry_during_drain_prevents_late_publication(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            state = Path(directory)
            config = {"admission": "quiesced", "accepting_writes": False}
            with (
                patch.object(server.time, "monotonic", return_value=100) as clock,
                patch.object(server._STARTUP, "deadline", None, create=True),  # noqa: SLF001 -- assert public readmission restores its offered clock
                patch.object(
                    server,
                    "lifecycle_reservation",
                    return_value=contextlib.nullcontext(),
                ),
                patch.object(server, "config_for", return_value=config),
                patch.object(server, "active", return_value=False),
                patch.object(server, "close_context_admission"),
                patch.object(
                    server,
                    "all_contexts_drained",
                    side_effect=lambda _state: setattr(clock, "return_value", 141),
                ),
                patch.object(server, "publish_generation") as publish,
                self.assertRaises(server.DeadlineExpiredError),
            ):
                server.readmit_supervisor(state, deadline=140)
            publish.assert_not_called()
            self.assertFalse((state / ".profile-history").exists())


class SurrealSupervisorTests(unittest.TestCase):
    def test_recovery_equivalence_accepts_exact_relocation_and_fresh_incarnation(
        self,
    ) -> None:
        config = self.reference_fixture()
        restored = server.checked_directory(self.root / "restored")
        shutil.copytree(self.state / ".generations", restored / ".generations")
        candidate = server.object_mapping(
            server.reroot_restored(
                server.fresh_database_identity(config), str(self.state), restored
            )
        )
        recovery.recovery_equivalence(
            config, candidate, self.state, restored, fresh=True
        )
        with self.assertRaisesRegex(server.SupervisorError, "current incarnation"):
            recovery.recovery_equivalence(
                config,
                {**candidate, "instance_id": config["instance_id"]},
                self.state,
                restored,
                fresh=True,
            )
        with self.assertRaisesRegex(server.SupervisorError, "readiness or admission"):
            recovery.recovery_equivalence(
                config,
                {**candidate, "restart_qualified": True},
                self.state,
                restored,
                fresh=True,
            )
        for changes in (
            {"resources": {}},
            {"interpretation": "different"},
            {"tooling_fixture": not config["tooling_fixture"]},
            {"unclassified": True},
        ):
            with (
                self.subTest(changes=changes),
                self.assertRaises(server.SupervisorError),
            ):
                recovery.recovery_equivalence(
                    config, {**candidate, **changes}, self.state, restored, fresh=True
                )

    def test_recovery_equivalence_refuses_changed_receiver_bytes_association_and_permissions(
        self,
    ) -> None:
        config = self.reference_fixture()
        original = server.object_mapping(config["primary_receiver"])
        restored = server.checked_directory(self.root / "restored")
        shutil.copytree(self.state / ".generations", restored / ".generations")
        candidate = server.object_mapping(
            server.reroot_restored(original, str(self.state), restored)
        )
        recovery.receiver_equivalence(
            original, candidate, self.state, restored, primary=True
        )
        for changes in (
            {"supervisor_executable": "/different/python"},
            {"worker_executable": str(restored) + "-other/bin/pse-worker"},
            {"worker_sha256": "changed"},
            {"extra": "field"},
        ):
            with (
                self.subTest(changes=changes),
                self.assertRaises(server.SupervisorError),
            ):
                recovery.receiver_equivalence(
                    original,
                    {**candidate, **changes},
                    self.state,
                    restored,
                    primary=True,
                )
        worker = Path(str(candidate["worker_executable"]))
        saved = worker.read_bytes()
        worker.write_bytes(saved + b"changed")
        with self.assertRaisesRegex(server.SupervisorError, "closure changed"):
            recovery.receiver_equivalence(
                original, candidate, self.state, restored, primary=True
            )
        worker.write_bytes(saved)
        for artifact, mode, diagnostic in (
            (worker, 0o750, "execution mode or private ownership"),
            (worker.parent, 0o750, "permissions"),
        ):
            with self.subTest(artifact=artifact):
                before = artifact.stat().st_mode & 0o777
                artifact.chmod(mode)
                try:
                    with self.assertRaisesRegex(server.SupervisorError, diagnostic):
                        recovery.receiver_equivalence(
                            original, candidate, self.state, restored, primary=True
                        )
                finally:
                    artifact.chmod(before)
        manifest = worker.parents[1] / "generation.json"
        declared = server.read_json(manifest)
        server.write_json(manifest, {**declared, "identity": "changed"})
        with self.assertRaisesRegex(server.SupervisorError, "identity"):
            recovery.receiver_equivalence(
                original, candidate, self.state, restored, primary=True
            )

    def test_recovery_receiver_refuses_intermediate_and_manifest_symlinks(self) -> None:
        config = self.reference_fixture()
        original = server.object_mapping(config["primary_receiver"])
        restored = server.checked_directory(self.root / "restored")
        shutil.copytree(self.state / ".generations", restored / ".generations")
        candidate = server.object_mapping(
            server.reroot_restored(original, str(self.state), restored)
        )
        recovery.receiver_equivalence(
            original, candidate, self.state, restored, primary=True
        )
        for root, receiver in ((self.state, original), (restored, candidate)):
            generation = Path(str(receiver["worker_executable"])).parents[1]
            for artifact in (
                generation / "bin",
                generation / "scripts",
                generation / "generation.json",
            ):
                with self.subTest(root=root, artifact=artifact):
                    # Bytes, ownership and modes stay equal. Only the actual
                    # association leaves the declared generation through a link.
                    relocated = root / ("redirected-" + artifact.name)
                    artifact.rename(relocated)
                    artifact.symlink_to(
                        relocated, target_is_directory=relocated.is_dir()
                    )
                    try:
                        with self.assertRaisesRegex(server.SupervisorError, "symlinks"):
                            recovery.receiver_equivalence(
                                original, candidate, self.state, restored, primary=True
                            )
                    finally:
                        artifact.unlink()
                        relocated.rename(artifact)
        recovery.receiver_equivalence(
            original, candidate, self.state, restored, primary=True
        )

    def test_held_metadata_lock_expires_without_late_admission(self) -> None:
        config = self.initialized()
        program = "import fcntl,sys; f=open(sys.argv[1], 'a'); fcntl.flock(f, fcntl.LOCK_EX); print('held',flush=True); sys.stdin.readline()"
        child = subprocess.Popen(
            [sys.executable, "-c", program, str(self.state / ".supervisor.lock")],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
        )
        try:
            assert child.stdout is not None
            self.assertEqual(child.stdout.readline().strip(), "held")
            for control in ("join", "reservation", "context"):
                with self.subTest(control=control):
                    started = server.time.monotonic()
                    deadline = started + 0.05
                    with self.assertRaises(server.DeadlineExpiredError):
                        if control == "join":
                            server.wait_reservation(
                                self.state,
                                self.state / "absent.json",
                                "absent",
                                deadline,
                            )
                        elif control == "reservation":
                            with server.lifecycle_reservation(
                                self.state, purpose="startup", deadline=deadline
                            ):
                                self.fail("expired lifecycle was admitted")
                        else:
                            with server.context_admission(
                                self.state, deadline=deadline
                            ):
                                self.fail("expired context was admitted")
                    self.assertLess(server.time.monotonic() - started, 0.5)
                    self.assertFalse((self.state / "lifecycle-owner.json").exists())
                    self.assertEqual(server.config_for(self.state), config)
        finally:
            child.communicate("release\n", timeout=5)
            self.assertEqual(child.returncode, 0)

    def test_lock_rechecks_clock_after_acquisition_before_effects(self) -> None:
        self.initialized()
        original = server.fcntl.flock

        def delayed(descriptor: int, operation: int) -> None:
            original(descriptor, operation)
            server.time.sleep(0.04)

        with (
            patch.object(server.fcntl, "flock", side_effect=delayed),
            self.assertRaises(server.DeadlineExpiredError),
            server.lifecycle_reservation(
                self.state, purpose="startup", deadline=server.time.monotonic() + 0.02
            ),
        ):
            self.fail("post-acquisition expiry admitted effects")
        self.assertFalse((self.state / "lifecycle-owner.json").exists())

    def test_expired_owned_lifecycle_retains_cleanup_and_releases_only_its_reservation(
        self,
    ) -> None:
        self.initialized()
        deadline = server.time.monotonic() + 0.02
        calls = []

        def cleanup(state: Path) -> None:
            self.assertEqual(state, self.state)
            self.assertIsNone(server._STARTUP.deadline)  # noqa: SLF001 -- cleanup consumes its own existing lifetime
            self.assertTrue((state / "lifecycle-owner.json").exists())
            calls.append(state)

        with (
            patch.object(server._STARTUP, "deadline", deadline, create=True),  # noqa: SLF001 -- original caller clock
            patch.object(server, "drain_maintenance_listener", side_effect=cleanup),
            self.assertRaises(server.DeadlineExpiredError),
            server.lifecycle_reservation(
                self.state, purpose="startup", deadline=deadline
            ),
        ):
            server.time.sleep(0.04)
            server.remaining(deadline)
        self.assertEqual(calls, [self.state])
        self.assertFalse((self.state / "lifecycle-owner.json").exists())

    def test_live_reservation_observations_distinguish_operation_purpose(self) -> None:
        config = self.initialized()
        for purpose, phase in (
            ("startup", server.Readiness.STARTING),
            ("stop", server.Readiness.STOPPING),
            ("maintenance", server.Readiness.MAINTENANCE),
            (None, server.Readiness.UNAVAILABLE),
        ):
            selected = {
                "nonce": "selected",
                "pid": os.getpid(),
                "start": server.native_operation.start_identity(os.getpid()),
                "purpose": purpose,
            }
            server.write_json(self.state / "lifecycle-owner.json", selected)
            with (
                self.subTest(purpose=purpose),
                patch.object(server, "systemctl") as manager,
            ):
                observed = server.service_readiness(self.state, config)
                self.assertEqual(observed.phase, phase)
                self.assertEqual(observed.instance_id, config["instance_id"])
                self.assertEqual(observed.state, self.state)
                self.assertEqual(observed.reservation, selected)
                self.assertIn("reservation", observed.cause)
                manager.assert_not_called()
        (self.state / "lifecycle-owner.json").unlink()

    def test_external_activation_requires_exact_owned_launch(self) -> None:
        config = self.initialized()
        invocation = "a" * 32
        manager = subprocess.CompletedProcess(
            [],
            0,
            f"ActiveState=activating\nControlGroup=/owned\nInvocationID={invocation}\n",
            "",
        )
        launch = {"binding": {"invocation": invocation, "group": "/owned", "inode": 17}}
        with (
            patch.object(server, "systemctl", return_value=manager),
            patch.object(server, "recorded_storage_owner", return_value=(None, launch)),
            patch.object(server.host_admission, "group_identity", return_value=17),
        ):
            observed = server.service_readiness(self.state, config)
            self.assertEqual(observed.phase, server.Readiness.STARTING)
            self.assertEqual(observed.manager["InvocationID"], invocation)
            launch["binding"]["invocation"] = "b" * 32
            self.assertEqual(
                server.service_readiness(self.state, config).phase,
                server.Readiness.MISMATCH,
            )
        with (
            patch.object(server, "systemctl", return_value=manager),
            patch.object(
                server,
                "recorded_storage_owner",
                side_effect=server.SupervisorError("launch proof unavailable"),
            ),
        ):
            observed = server.service_readiness(self.state, config)
            self.assertEqual(observed.phase, server.Readiness.UNAVAILABLE)
            self.assertIn("launch proof unavailable", observed.cause)

    def test_typed_readiness_separates_absence_unavailable_and_mismatch(self) -> None:
        config = self.initialized()
        self.assertEqual(
            server.service_readiness(self.state, config).phase,
            server.Readiness.ACTIONABLE_ABSENT,
        )
        active = subprocess.CompletedProcess(
            [], 0, "ActiveState=active\nControlGroup=/owned\n", ""
        )
        with (
            patch.object(server, "systemctl", return_value=active),
            patch.object(server, "listening_inodes", return_value={"owned"}),
        ):
            for error in (
                PermissionError("denied"),
                FileNotFoundError("process vanished"),
                server.SupervisorError("ledger unavailable"),
            ):
                with (
                    self.subTest(error=error),
                    patch.object(server, "_owns_listener", side_effect=error),
                ):
                    self.assertEqual(
                        server.service_readiness(self.state, config).phase,
                        server.Readiness.UNAVAILABLE,
                    )
            with patch.object(server, "_owns_listener", return_value=False):
                self.assertEqual(
                    server.service_readiness(self.state, config).phase,
                    server.Readiness.MISMATCH,
                )
        with patch.object(
            server,
            "systemctl",
            return_value=subprocess.CompletedProcess([], 1, "", "unavailable"),
        ):
            self.assertEqual(
                server.service_readiness(self.state, config).phase,
                server.Readiness.UNAVAILABLE,
            )
        with patch.object(server, "listening_inodes", return_value={"unowned"}):
            self.assertEqual(
                server.service_readiness(self.state, config).phase,
                server.Readiness.MISMATCH,
            )

    def test_stop_keeps_allocation_until_exact_zombie_is_reaped(self) -> None:
        config = self.initialized()
        child = subprocess.Popen([sys.executable, "-c", "pass"])
        self.addCleanup(child.wait)
        os.waitid(os.P_PID, child.pid, os.WEXITED | os.WNOWAIT)
        identity = server.native_operation.start_identity(child.pid)
        self.assertEqual(
            Path(f"/proc/{child.pid}/stat").read_text().rsplit(")", 1)[1].split()[0],
            "Z",
        )
        server.write_json(
            self.state / "server-process.json",
            {"pid": child.pid, "start": identity},
        )
        deadline = server.time.monotonic() + 10

        def observation(
            *arguments: str, **_kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            output = (
                "ActiveState=inactive\nControlGroup=\n"
                if "--property=ControlGroup" in arguments
                and "--property=ActiveState" in arguments
                else ""
                if "--property=ControlGroup" in arguments
                else "inactive\n"
            )
            return subprocess.CompletedProcess([], 0, output, "")

        def reap(_duration: float) -> None:
            released.assert_not_called()
            self.assertEqual(server._STARTUP.deadline, deadline)  # noqa: SLF001 -- verify the original stop clock is retained
            self.assertEqual(
                server.native_operation.start_identity(child.pid), identity
            )
            child.wait()

        with (
            patch.object(server, "systemctl", side_effect=observation),
            patch.object(server, "all_contexts_drained"),
            patch.object(server, "listening_inodes", return_value=set()),
            patch.object(server, "release_stopped_service") as released,
            patch.object(server._STARTUP, "deadline", deadline, create=True),  # noqa: SLF001 -- inject only the captured stop clock
            patch.object(server.time, "sleep", side_effect=reap) as wait,
        ):
            self.assertEqual(
                server.service_readiness(self.state, config).phase,
                server.Readiness.MISMATCH,
            )
            server._stop(self.state, config)  # noqa: SLF001 -- isolate the exact process drain boundary
            wait.assert_called_once_with(0.05)
            released.assert_called_once_with(self.state, config)
            self.assertEqual(
                server.service_readiness(self.state, config).phase,
                server.Readiness.ACTIONABLE_ABSENT,
            )

    def test_stop_releases_when_recorded_process_is_gone_or_replaced(self) -> None:
        config = self.initialized()
        server.write_json(
            self.state / "server-process.json", {"pid": 12345, "start": "old"}
        )
        for identity in (FileNotFoundError("reaped"), "different lifetime"):
            with (
                self.subTest(identity=identity),
                patch.object(server, "all_contexts_drained"),
                patch.object(
                    server.native_operation,
                    "start_identity",
                    side_effect=[identity],
                ),
                patch.object(server, "release_stopped_service") as released,
                patch.object(server.time, "sleep") as wait,
            ):
                server._stop(self.state, config)  # noqa: SLF001 -- exercise exact lifetime retirement without lifecycle effects
                wait.assert_not_called()
                released.assert_called_once_with(self.state, config)

    def test_stop_original_deadline_expiry_keeps_exact_process_allocation(self) -> None:
        config = self.initialized()
        server.write_json(
            self.state / "server-process.json", {"pid": 12345, "start": "old"}
        )
        deadline = server.time.monotonic() - 1
        with (
            patch.object(server, "all_contexts_drained"),
            patch.object(server.native_operation, "start_identity", return_value="old"),
            patch.object(server._STARTUP, "deadline", deadline, create=True),  # noqa: SLF001 -- inject the expired original stop clock
            patch.object(server, "release_stopped_service") as released,
            patch.object(server.time, "sleep") as wait,
            self.assertRaisesRegex(server.SupervisorError, "original admission clock"),
        ):
            server._stop(self.state, config)  # noqa: SLF001 -- verify allocation retention on exact drain expiry
        released.assert_not_called()
        wait.assert_not_called()

    def test_unknown_or_mismatched_readiness_never_reserves_or_starts(self) -> None:
        config = self.initialized()
        for observation in (
            readiness_fixture(server.Readiness.UNAVAILABLE),
            readiness_fixture(server.Readiness.MISMATCH),
        ):
            with (
                self.subTest(observation=observation),
                patch.object(server, "service_readiness", return_value=observation),
                patch.object(server, "lifecycle_reservation") as reservation,
                patch.object(server, "_start") as launch,
                self.assertRaisesRegex(server.SupervisorError, "startup refused"),
            ):
                server.start(self.state, config)
            reservation.assert_not_called()
            launch.assert_not_called()

    def test_ready_start_returns_without_lifecycle_mutation(self) -> None:
        config = self.initialized()
        with (
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.READY),
            ),
            patch.object(server, "lifecycle_reservation") as reservation,
            patch.object(server, "_start") as launch,
        ):
            server.start(self.state, config)
        reservation.assert_not_called()
        launch.assert_not_called()

    def test_concurrent_start_borrower_joins_release_and_reobserves(self) -> None:
        config = self.initialized()
        entered, joining, release, completed = (threading.Event() for _ in range(4))
        original_wait = server.wait_reservation
        deadline = server.time.monotonic() + 10

        def readiness(*_args: object, **_kwargs: object) -> server.ServiceObservation:
            if completed.is_set():
                return readiness_fixture(server.Readiness.READY)
            return (
                readiness_fixture(server.Readiness.STARTING)
                if entered.is_set()
                else readiness_fixture(server.Readiness.ACTIONABLE_ABSENT)
            )

        def launch(*_args: object, **kwargs: object) -> None:
            self.assertEqual(kwargs["deadline"], deadline)
            entered.set()
            self.assertTrue(release.wait(5))
            completed.set()

        def wait(state: Path, path: Path, nonce: object, until: float) -> None:
            self.assertEqual(until, deadline)
            self.assertEqual(server.read_json(path)["nonce"], nonce)
            joining.set()
            original_wait(state, path, nonce, until)

        with (
            patch.object(server, "service_readiness", side_effect=readiness),
            patch.object(server, "_start", side_effect=launch) as launched,
            patch.object(server, "wait_reservation", side_effect=wait),
            ThreadPoolExecutor(max_workers=2) as pool,
        ):
            owner = pool.submit(
                server.start, self.state, dict(config), deadline=deadline
            )
            self.assertTrue(entered.wait(5))
            borrower = pool.submit(
                server.start, self.state, dict(config), deadline=deadline
            )
            try:
                self.assertTrue(joining.wait(5))
            finally:
                release.set()
            owner.result(timeout=5)
            borrower.result(timeout=5)
        launched.assert_called_once()
        self.assertFalse((self.state / "lifecycle-owner.json").exists())

    def test_cli_start_and_ensure_join_live_owner(self) -> None:
        self.initialized()

        def exercise(command: str) -> None:
            entered, joining, release, completed = (threading.Event() for _ in range(4))
            original_wait = server.wait_reservation

            def readiness(
                *_args: object, **_kwargs: object
            ) -> server.ServiceObservation:
                if completed.is_set():
                    return readiness_fixture(server.Readiness.READY)
                return (
                    readiness_fixture(server.Readiness.STARTING)
                    if entered.is_set()
                    else readiness_fixture(server.Readiness.ACTIONABLE_ABSENT)
                )

            def launch(*_args: object, **_kwargs: object) -> None:
                entered.set()
                self.assertTrue(release.wait(5))
                completed.set()

            def wait(state: Path, path: Path, nonce: object, deadline: float) -> None:
                joining.set()
                original_wait(state, path, nonce, deadline)

            with (
                self.subTest(command=command),
                patch.object(server, "service_readiness", side_effect=readiness),
                patch.object(server, "_start", side_effect=launch) as launched,
                patch.object(server, "wait_reservation", side_effect=wait),
                patch.object(server, "public_status", return_value={}),
                patch("builtins.print"),
                ThreadPoolExecutor(max_workers=2) as pool,
            ):
                arguments = [command, "--state", str(self.state)]
                owner = pool.submit(server.main, arguments)
                self.assertTrue(entered.wait(5))
                borrower = pool.submit(server.main, arguments)
                try:
                    self.assertTrue(joining.wait(5))
                finally:
                    release.set()
                self.assertEqual(owner.result(timeout=5), 0)
                self.assertEqual(borrower.result(timeout=5), 0)
                launched.assert_called_once()
            self.assertFalse((self.state / "lifecycle-owner.json").exists())

        for command in ("start", "ensure"):
            exercise(command)

    def test_cli_ready_start_and_ensure_skip_outer_reservation(self) -> None:
        self.initialized()
        for command in ("start", "ensure"):
            with (
                self.subTest(command=command),
                patch.object(
                    server,
                    "service_readiness",
                    return_value=readiness_fixture(server.Readiness.READY),
                ),
                patch.object(server, "lifecycle_reservation") as reservation,
                patch.object(server, "_start") as launch,
                patch.object(server, "public_status", return_value={}),
                patch("builtins.print"),
            ):
                self.assertEqual(server.main([command, "--state", str(self.state)]), 0)
            reservation.assert_not_called()
            launch.assert_not_called()

    def test_cli_recover_retains_strict_outer_reservation(self) -> None:
        self.initialized()
        with (
            server.lifecycle_reservation(self.state),
            patch.object(server, "reset_failure_window") as reset,
            patch.object(server, "start") as start,
            patch("builtins.print"),
            ThreadPoolExecutor(max_workers=1) as pool,
        ):
            refusal = pool.submit(server.main, ["recover", "--state", str(self.state)])
            self.assertEqual(refusal.result(timeout=5), 1)
        reset.assert_not_called()
        start.assert_not_called()

    def test_concurrent_primary_admission_joins_existing_completion(self) -> None:
        self.reference_fixture()
        entered, joining, release, completed = (threading.Event() for _ in range(4))
        original_wait = server.wait_reservation
        launches = []

        def ensure(*_args: object, **_kwargs: object) -> dict[str, object]:
            if not completed.is_set():
                launches.append(True)
                entered.set()
                self.assertTrue(release.wait(5))
                completed.set()
            return {"ready": True}

        def wait(state: Path, path: Path, nonce: object, until: float) -> None:
            self.assertEqual(path.name, "primary-admission.json")
            joining.set()
            original_wait(state, path, nonce, until)

        with (
            patch.object(server, "_ensure_primary", side_effect=ensure),
            patch.object(server, "wait_reservation", side_effect=wait),
            ThreadPoolExecutor(max_workers=2) as pool,
        ):
            owner = pool.submit(server.ensure_primary, self.state)
            self.assertTrue(entered.wait(5))
            borrower = pool.submit(server.ensure_primary, self.state)
            try:
                self.assertTrue(joining.wait(5))
            finally:
                release.set()
            self.assertTrue(owner.result(timeout=5)["ready"])
            self.assertTrue(borrower.result(timeout=5)["ready"])
        self.assertEqual(launches, [True])
        self.assertFalse((self.state / "primary-admission.json").exists())

    def test_released_owner_unknown_observation_does_not_create_replacement_reservation(
        self,
    ) -> None:
        config = self.initialized()
        path = self.state / "lifecycle-owner.json"
        server.write_json(
            path,
            {
                "nonce": "owner",
                "pid": os.getpid(),
                "start": server.native_operation.start_identity(os.getpid()),
                "boot": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
            },
        )

        def release(*_args: object) -> None:
            path.unlink()

        with (
            patch.object(
                server,
                "service_readiness",
                side_effect=[
                    readiness_fixture(server.Readiness.STARTING),
                    readiness_fixture(server.Readiness.UNAVAILABLE),
                ],
            ),
            patch.object(server, "wait_reservation", side_effect=release),
            patch.object(server, "_start") as launch,
            patch.object(server, "write_json", wraps=server.write_json) as publication,
            self.assertRaisesRegex(server.SupervisorError, "unavailable"),
        ):
            server.start(self.state, config)
        launch.assert_not_called()
        publication.assert_not_called()

    def test_interrupted_start_owner_reobserves_and_replaces_only_stale_reservation(
        self,
    ) -> None:
        config = self.initialized()
        path = self.state / "lifecycle-owner.json"
        server.write_json(
            path,
            {
                "nonce": "interrupted",
                "pid": os.getpid(),
                "start": "not-current",
                "boot": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
            },
        )

        def launch(*_args: object, **_kwargs: object) -> None:
            self.assertNotEqual(server.read_json(path)["nonce"], "interrupted")

        with patch.object(server, "_start", side_effect=launch) as launched:
            server.start(self.state, config)
        launched.assert_called_once()
        self.assertFalse(path.exists())

    def test_start_borrower_preserves_original_deadline_and_selection(self) -> None:
        config = self.initialized()
        path = self.state / "lifecycle-owner.json"
        server.write_json(
            path,
            {
                "nonce": "owner",
                "pid": os.getpid(),
                "start": server.native_operation.start_identity(os.getpid()),
                "boot": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
            },
        )
        with (
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.STARTING),
            ),
            patch.object(server.time, "monotonic", side_effect=[1.0, 1.0, 3.0]),
            patch.object(server, "_start") as launch,
            self.assertRaisesRegex(server.SupervisorError, "original admission clock"),
        ):
            server.start(self.state, config, deadline=2.0)
        launch.assert_not_called()
        self.assertEqual(server.read_json(path)["nonce"], "owner")

        def changed(*_args: object) -> None:
            path.unlink()
            server.write_json(
                self.state / "config.json",
                {
                    **config,
                    "resources": server.resources(
                        8 * server.GIB, 2 * server.GIB, 2, server.GIB
                    ),
                },
            )

        with (
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.STARTING),
            ),
            patch.object(server, "wait_reservation", side_effect=changed),
            patch.object(server, "_start") as launch,
            self.assertRaisesRegex(server.SupervisorError, "selection changed"),
        ):
            server.start(self.state, config)
        launch.assert_not_called()

    def test_failed_resume_retains_parking_and_closed_admission(self) -> None:
        config = self.initialized()
        config.update(parked=True, admission="quiesced", accepting_writes=False)
        server.write_json(self.state / "config.json", config)

        def manager(*args: str, **_kwargs: object) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess(
                [], 1 if args[0] == "start" else 0, "", ""
            )

        with (
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
            ),
            patch.object(server, "service_allocation", return_value=self.allocation),
            patch.object(server, "materialize_service"),
            patch.object(server, "systemctl", side_effect=manager),
            self.assertRaisesRegex(server.SupervisorError, "Cannot start"),
        ):
            server.start(self.state, config)
        selected = server.config_for(self.state)
        self.assertTrue(selected["parked"])
        self.assertFalse(selected["accepting_writes"])
        self.assertEqual(selected["admission"], "quiesced")
        self.allocation.release.assert_called_once()
        self.assertFalse((self.state / "lifecycle-owner.json").exists())

    def test_start_reobserves_after_actual_reservation_before_launch(self) -> None:
        config = self.initialized()
        with (
            patch.object(
                server,
                "service_readiness",
                side_effect=[
                    readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                    readiness_fixture(server.Readiness.UNAVAILABLE),
                ],
            ),
            patch.object(server, "service_allocation") as allocation,
            patch.object(server, "materialize_service") as materialize,
            self.assertRaisesRegex(server.SupervisorError, "unavailable"),
        ):
            server.start(self.state, config)
        allocation.assert_not_called()
        materialize.assert_not_called()
        self.assertFalse((self.state / "lifecycle-owner.json").exists())

    def test_new_launch_reobserves_unavailable_listener_under_original_clock(
        self,
    ) -> None:
        config = self.initialized()
        server.close_context_admission(self.state)
        config.update(server.config_for(self.state))
        self.assertFalse(config["accepting_writes"])
        with (
            patch.object(
                server,
                "service_readiness",
                side_effect=[
                    readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                    readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                    readiness_fixture(server.Readiness.UNAVAILABLE),
                    readiness_fixture(server.Readiness.STARTING),
                ],
            ) as observed,
            patch.object(server, "service_allocation", return_value=self.allocation),
            patch.object(server, "materialize_service"),
            patch.object(server, "listener_ready", return_value=True),
            patch.object(server, "establish_protocol_readiness"),
            patch.object(server.time, "sleep") as pause,
        ):
            server.start(self.state, config)
        self.assertEqual(observed.call_count, 4)
        pause.assert_called_once()
        self.assertEqual(server.config_for(self.state)["admission"], "open")
        self.assertEqual(config, server.config_for(self.state))
        self.assertTrue(config["accepting_writes"])

    def test_retired_receiver_evidence_needs_exact_dead_drained_borrower_state(
        self,
    ) -> None:
        self.initialized()
        base = {
            "kind": "namespace",
            "state": str(self.state / ".receivers/retired/context"),
            "cleanup": "retained",
            "drained": True,
            "live": False,
            "borrows": {},
        }
        for changes, allowed in (
            ({}, True),
            ({"drained": False}, False),
            ({"drained": "true"}, False),
            ({"live": True}, False),
            ({"creation_borrower": {"nonce": "pending"}}, False),
            ({"borrows": {"active": {}}}, False),
            ({"cleanup": "removing", "cleaner": {"live": True}}, False),
        ):
            record = {**base, **changes}
            before = json.loads(json.dumps(record))
            with (
                self.subTest(changes=changes),
                patch.object(
                    server.test_resources,
                    "service_resources",
                    return_value={"retained": record},
                ),
                patch.object(
                    server.test_resources,
                    "borrower_alive",
                    side_effect=lambda row: row.get("live", False),
                ),
            ):
                if allowed:
                    server.require_borrowers_drained(self.state)
                else:
                    with self.assertRaises(server.SupervisorError):
                        server.require_borrowers_drained(self.state)
            self.assertEqual(record, before)

    def test_start_consumes_acknowledged_payload_freeze_in_caller_snapshot(
        self,
    ) -> None:
        template = self.initialized()
        for already_running in (False, True):
            with self.subTest(already_running=already_running):
                config = dict(template)
                config.update(
                    payload_frozen=True, admission="quiesced", accepting_writes=False
                )
                server.write_json(self.state / "config.json", config)
                observations = (
                    [readiness_fixture(server.Readiness.READY)] * 2
                    if already_running
                    else [
                        readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                        readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                        readiness_fixture(server.Readiness.STARTING),
                    ]
                )
                with (
                    patch.object(server, "service_readiness", side_effect=observations),
                    patch.object(
                        server, "service_allocation", return_value=self.allocation
                    ),
                    patch.object(server, "materialize_service"),
                    patch.object(server, "establish_protocol_readiness"),
                    patch.object(server, "payload_query", return_value=True) as thaw,
                ):
                    server.start(self.state, config)
                thaw.assert_called_once()
                self.assertNotIn("payload_frozen", config)
                self.assertEqual(config, server.config_for(self.state))
                with patch.object(
                    server,
                    "service_readiness",
                    return_value=readiness_fixture(server.Readiness.READY),
                ):
                    server.start(self.state, config)
                self.assertNotIn("payload_frozen", server.config_for(self.state))

    def test_parked_partial_reopen_failure_preserves_acknowledged_primary_thaw(
        self,
    ) -> None:
        config = self.initialized()
        config.update(
            payload_frozen=True,
            parked=True,
            admission="quiesced",
            accepting_writes=False,
        )
        server.write_json(self.state / "config.json", config)
        receiver = self.state / ".receivers/partial/context"
        receiver.mkdir(mode=0o700, parents=True)
        server.write_json(
            receiver / "config.json",
            {**config, "service_state": str(self.state), "database": "receiver"},
        )
        with (
            patch.object(
                server, "managed_contexts", return_value=[self.state, receiver]
            ),
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.READY),
            ),
            patch.object(server, "payload_query", side_effect=[True, False]) as thaw,
            self.assertRaisesRegex(
                server.SupervisorError, "File admission reopen was not acknowledged"
            ),
        ):
            server._start(  # noqa: SLF001 -- inject partial multi-context reopen into the reserved startup owner
                self.state, config, validation=False, deadline=time.monotonic() + 30
            )
        self.assertEqual(thaw.call_count, 2)
        self.assertEqual(
            [entry.args[0] for entry in thaw.call_args_list], [self.state, receiver]
        )
        primary = server.config_for(self.state)
        remaining = server.config_for(receiver)
        self.assertNotIn("payload_frozen", primary)
        self.assertTrue(remaining["payload_frozen"])
        self.assertTrue(primary["parked"])
        self.assertEqual(config, primary)
        for selected in (primary, remaining):
            self.assertFalse(selected["accepting_writes"])
            self.assertEqual(selected["admission"], "quiesced")

    def test_validation_launch_keeps_published_admission_closed(self) -> None:
        config = self.initialized()
        server.close_context_admission(self.state, admission="validation_required")
        config.update(server.config_for(self.state))
        with (
            patch.object(
                server,
                "service_readiness",
                side_effect=[
                    readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                    readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                    readiness_fixture(server.Readiness.STARTING),
                ],
            ),
            patch.object(server, "service_allocation", return_value=self.allocation),
            patch.object(server, "materialize_service"),
            patch.object(server, "establish_protocol_readiness"),
            patch.object(server, "open_context_admission") as opened,
        ):
            server.start(self.state, config, validation=True)
        opened.assert_not_called()
        self.assertEqual(config["admission"], "validation_required")
        self.assertFalse(config["accepting_writes"])
        self.assertEqual(config, server.config_for(self.state))

    def test_unknown_launched_lifetime_expiry_does_not_stop_or_release_its_allocation(
        self,
    ) -> None:
        config = self.initialized()
        with (
            server.lifecycle_reservation(self.state),
            patch.object(
                server,
                "service_readiness",
                side_effect=[
                    readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                    readiness_fixture(server.Readiness.UNAVAILABLE),
                ],
            ),
            patch.object(server, "service_allocation", return_value=self.allocation),
            patch.object(server, "materialize_service"),
            patch.object(
                server,
                "remaining",
                side_effect=[1.0, server.SupervisorError("original admission clock")],
            ),
            patch.object(server.time, "monotonic", return_value=0.0),
            patch.object(
                server,
                "systemctl",
                return_value=subprocess.CompletedProcess([], 0, "", ""),
            ) as manager,
            self.assertRaisesRegex(server.SupervisorError, "original admission clock"),
        ):
            server._start(self.state, config, validation=False, deadline=1.0)  # noqa: SLF001 -- exercise already-reserved launch expiry
        self.assertNotIn("stop", [call.args[0] for call in manager.call_args_list])
        self.allocation.release.assert_not_called()

    def test_selected_recovery_receiver_copies_only_frozen_admitted_closure(
        self,
    ) -> None:
        config = self.reference_fixture()
        receiver = server.checked_primary(config)
        generation = Path(receiver["supervisor_script"]).parents[1]
        (generation / "unrelated").write_text("not part of the admitted closure")
        restored = server.checked_directory(self.root / "restored")
        (restored / ".generations").mkdir(mode=0o700)
        copied = recovery.copy_selected_receiver(config, self.state, restored)
        self.assertFalse(
            (restored / ".generations" / generation.name / "unrelated").exists()
        )
        recovery.receiver_equivalence(
            receiver, copied, self.state, restored, primary=True
        )

    def test_listener_ownership_survives_unrelated_descriptor_disappearance(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            proc = Path(directory)
            (proc / "fd").mkdir()
            vanished = proc / "fd/1"
            listener = proc / "fd/2"
            vanished.symlink_to("socket:[7]")
            listener.symlink_to("socket:[42]")

            def descriptors() -> Generator[Path, None, None]:
                vanished.unlink()
                yield vanished
                yield listener

            with patch.object(Path, "iterdir", return_value=descriptors()):
                self.assertTrue(server.process_owns_listening_socket(proc, {"42"}))
            self.assertFalse(server.process_owns_listening_socket(proc, {"99"}))
            listener.unlink()
            self.assertFalse(server.process_owns_listening_socket(proc, {"42"}))

    def test_listener_ownership_does_not_ignore_descriptor_access_refusal(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            proc = Path(directory)
            (proc / "fd").mkdir()
            (proc / "fd/1").symlink_to("socket:[42]")
            with (
                patch.object(Path, "readlink", side_effect=PermissionError("refused")),
                self.assertRaises(PermissionError),
            ):
                server.process_owns_listening_socket(proc, {"42"})

    def test_parking_stopped_materialized_resident_preserves_explicit_stop(
        self,
    ) -> None:
        self.reference_fixture()
        config = server.config_for(self.state)
        config.update(resident=True, unit_materialized=True, parked=False)
        server.write_json(self.state / "config.json", config)
        with (
            patch.object(
                server.native_operation,
                "unit_observation",
                return_value={"LoadState": "loaded", "ActiveState": "inactive"},
            ),
            patch.object(server, "all_contexts_drained") as drained,
            patch.object(server, "stop") as stop,
        ):
            self.assertFalse(server.park_service(self.state))
        self.assertFalse(server.config_for(self.state)["parked"])
        drained.assert_not_called()
        stop.assert_not_called()

    def test_parking_refuses_replaced_running_incarnation(self) -> None:
        self.reference_fixture()
        config = server.config_for(self.state)
        config.update(resident=True, unit_materialized=True, parked=False)
        server.write_json(self.state / "config.json", config)
        binding: host.BoundUnit = {
            "group": "/owned",
            "invocation": "a" * 32,
            "inode": 123,
        }
        with (
            patch.object(
                server.native_operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "ControlGroup": "/owned",
                    "InvocationID": "b" * 32,
                },
            ),
            patch.object(host, "group_identity", return_value=123),
            patch.object(server, "stop") as stop,
            self.assertRaisesRegex(server.SupervisorError, "lifetime changed"),
        ):
            server.park_service(self.state, expected_binding=binding)
        self.assertFalse(server.config_for(self.state)["parked"])
        stop.assert_not_called()

    def test_parking_consumes_exact_running_binding_before_stop(self) -> None:
        self.reference_fixture()
        config = server.config_for(self.state)
        config.update(resident=True, unit_materialized=True, parked=False)
        server.write_json(self.state / "config.json", config)
        binding: host.BoundUnit = {
            "group": "/owned",
            "invocation": "a" * 32,
            "inode": 123,
        }

        def stop(_state: Path, selected: dict[str, object]) -> None:
            self.assertTrue(selected["parked"])
            self.assertTrue(server.config_for(self.state)["parked"])

        with (
            patch.object(
                server.native_operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "ControlGroup": "/owned",
                    "InvocationID": "a" * 32,
                },
            ),
            patch.object(host, "group_identity", return_value=123),
            patch.object(server, "all_contexts_drained"),
            patch.object(server, "stop", side_effect=stop) as stopped,
        ):
            self.assertTrue(server.park_service(self.state, expected_binding=binding))
        stopped.assert_called_once()

    def test_parking_rechecks_invocation_and_inode_after_context_drain(self) -> None:
        self.reference_fixture()
        config = server.config_for(self.state)
        config.update(resident=True, unit_materialized=True, parked=False)
        server.write_json(self.state / "config.json", config)
        binding: host.BoundUnit = {
            "group": "/owned",
            "invocation": "a" * 32,
            "inode": 123,
        }
        for changed in ("invocation", "inode"):
            observed = {
                "LoadState": "loaded",
                "ActiveState": "active",
                "ControlGroup": "/owned",
                "InvocationID": "a" * 32,
            }
            inode = [123]

            def drain(
                _state: Path,
                changed: str = changed,
                observed: dict[str, str] = observed,
                inode: list[int] = inode,
            ) -> None:
                if changed == "invocation":
                    observed["InvocationID"] = "b" * 32
                else:
                    inode[0] = 456

            with (
                self.subTest(changed=changed),
                patch.object(
                    server.native_operation,
                    "unit_observation",
                    side_effect=lambda _, observed=observed: dict(observed),
                ),
                patch.object(
                    host, "group_identity", side_effect=lambda _, inode=inode: inode[0]
                ),
                patch.object(server, "all_contexts_drained", side_effect=drain),
                patch.object(server, "stop") as stop,
                self.assertRaisesRegex(
                    server.SupervisorError, "changed while draining"
                ),
            ):
                server.park_service(self.state, expected_binding=binding)
            stop.assert_not_called()
            self.assertFalse(server.config_for(self.state)["parked"])

    def test_explicit_stop_clears_resume_premise_before_withdrawing_queue(self) -> None:
        self.reference_fixture()
        config = server.config_for(self.state)
        config["parked"] = True
        server.write_json(self.state / "config.json", config)
        args = server.parser().parse_args(
            ["stop", "--state", str(self.state), "--drained"]
        )

        def withdraw(state: Path) -> None:
            self.assertEqual(state, self.state)
            self.assertFalse(server.config_for(state)["parked"])

        with (
            patch.object(
                host, "withdraw_parked_service", side_effect=withdraw
            ) as withdrawn,
            patch.object(server, "stop") as stopped,
            patch.object(server, "public_status", return_value={}),
            patch("builtins.print"),
        ):
            self.assertEqual(server.dispatch(args), 0)
        withdrawn.assert_called_once_with(self.state)
        stopped.assert_called_once()

    def setUp(self) -> None:
        fixture_build_context(self)
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.state = self.root / "state"
        module_install = patch.object(
            server.surrealism_module,
            "install",
            side_effect=lambda selected, _tools: fixture_module(
                self.root / "modules", str(selected["binary_sha256"])
            ),
        )
        module_install.start()
        self.addCleanup(module_install.stop)
        native_cache = patch.dict(
            os.environ,
            {
                "PSE_NATIVE_CACHE": str(self.root / "native-cache"),
                "PSE_TEST_RESOURCE_STATE": str(self.root / "test-resources"),
            },
        )
        native_cache.start()
        self.addCleanup(native_cache.stop)
        # Supervisor unit controls borrow a declared owner without touching the
        # live machine ledger. Host admission itself has its own owner tests.
        self.allocation = MagicMock(spec=host.Allocation)
        self.allocation.directory = self.root / "admission"
        self.allocation.nonce = "a" * 32
        self.allocation.profile = host.select("reference")
        self.allocation.environment.return_value = {}
        self.host_inherit = host.inherit
        self.host_snapshot = host.readonly_snapshot
        for target, name, value in (
            (host, "inherit", self.allocation),
            (host, "enforce_allocation", None),
            (host, "enforce_parent", None),
            (
                host,
                "readonly_snapshot",
                {"owners": {self.allocation.nonce: {"units": {}}}},
            ),
        ):
            owned = patch.object(target, name, return_value=value)
            owned.start()
            self.addCleanup(owned.stop)

        def manager_result(
            *arguments: str, check: bool = True, **_kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            self.assertIsInstance(check, bool)
            output = (
                "LoadState=not-found\nActiveState=inactive\nControlGroup=\nMemoryMax=infinity\n"
                if "--property=LoadState" in arguments
                else (
                    "ActiveState=inactive\nControlGroup=\n"
                    if "--property=ActiveState" in arguments
                    and "--property=ControlGroup" in arguments
                    else "inactive\n"
                )
            )
            return subprocess.CompletedProcess([], 0, output, "")

        manager = patch.object(server, "systemctl", side_effect=manager_result)
        manager.start()
        self.addCleanup(manager.stop)

    def initialized(self, *, fresh: bool = False) -> dict[str, object]:
        args = server.parser().parse_args(
            [
                "setup",
                "--state",
                str(self.state),
                "--interpretation",
                "pse.substrate.v1",
                "--memory-mib",
                "4096",
                "--server-memory-mib",
                "2048",
                "--native-workers",
                "2",
                "--native-worker-memory-mib",
                "1024",
                "--port",
                "18080",
            ]
        )
        binary = self.root / "surreal"
        binary.write_text("#!/bin/sh\nexit 0\n")
        binary.chmod(0o700)
        with (
            patch.object(
                server,
                "install",
                return_value={
                    "version": "3.3.4",
                    "surrealmx_version": "1.0.0",
                    **fixture_receipt(),
                    "binary": str(binary),
                    "binary_sha256": server.file_digest(binary),
                    "upstream_revision": server.surreal_source.REVISION,
                },
            ),
            patch.object(server, "active", return_value=False),
        ):
            server.setup(args)
        initialized = server.read_json(self.state / "config.json")
        self.assertEqual(initialized["admission"], "validation_required")
        self.assertFalse(initialized["accepting_writes"])
        # These controls exercise an explicitly initialized ordinary service.
        if not fresh:
            initialized.pop("fresh_root_bootstrap")
            initialized.update(admission="open", accepting_writes=True)
        server.write_json(self.state / "config.json", initialized)
        (self.state / "database").mkdir()
        (self.state / "database" / "fixture").write_bytes(
            b"acknowledged fixed operation identity"
        )
        return server.config_for(self.state)

    def frozen_source(self, config: dict[str, object]) -> tuple[Path, ModuleType]:
        supervisor = server.object_mapping(config["service_supervisor"])
        script = Path(str(supervisor["supervisor_script"]))
        spec = importlib.util.spec_from_file_location(
            "frozen_service_source_fixture", script.parent / "surreal_source.py"
        )
        assert spec is not None
        assert spec.loader is not None
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        return script, module

    def test_frozen_config_admits_only_its_verified_bound_runtime(self) -> None:
        config = self.initialized()
        script, frozen = self.frozen_source(config)
        with (
            patch.object(server, "SCRIPT", script),
            patch.object(server, "surreal_source", frozen),
            patch.object(
                frozen, "build_context", side_effect=AssertionError("compiler scan")
            ),
        ):
            self.assertEqual(server.config_for(self.state), config)
        # The runtime path cannot be selected by a controller outside the tree.
        with patch.object(server.surreal_source, "admitted_runtime_receipt") as runtime:
            self.assertEqual(server.config_for(self.state), config)
        runtime.assert_not_called()

    def test_frozen_config_refuses_wrong_binding_generation_bytes_and_receipt_key(
        self,
    ) -> None:
        config = self.initialized()
        script, frozen = self.frozen_source(config)
        original = json.loads(json.dumps(config))
        for change in (
            "script",
            "sha256",
            "executable",
            "effective_key",
            "captured_bytes",
        ):
            candidate = json.loads(json.dumps(original))
            if change == "script":
                candidate["service_supervisor"]["supervisor_script"] = str(
                    self.state / "fake/scripts/surreal_server.py"
                )
            elif change == "sha256":
                candidate["service_supervisor"]["supervisor_sha256"] = "0" * 64
            elif change == "executable":
                candidate["service_supervisor"]["supervisor_executable"] = (
                    "/different/python"
                )
            elif change == "effective_key":
                candidate["server"]["effective_key"] = "0" * 64
            captured = script.parent.parent / ".cargo/config.toml"
            before = captured.read_bytes()
            if change == "captured_bytes":
                captured.write_bytes(before + b"\n# changed\n")
            server.write_json(self.state / "config.json", candidate)
            try:
                with (
                    self.subTest(change=change),
                    patch.object(server, "SCRIPT", script),
                    patch.object(server, "surreal_source", frozen),
                    self.assertRaises(server.SupervisorError),
                ):
                    server.config_for(self.state)
            finally:
                captured.write_bytes(before)
                server.write_json(self.state / "config.json", original)

    def test_frozen_supervisor_cannot_admit_same_receipt_for_another_state(
        self,
    ) -> None:
        config = self.initialized()
        script, frozen = self.frozen_source(config)
        other = self.root / "other-state"
        other.mkdir(mode=0o700)
        server.write_json(other / "config.json", config)
        with (
            patch.object(server, "SCRIPT", script),
            patch.object(server, "surreal_source", frozen),
            self.assertRaisesRegex(
                server.SupervisorError, "bound immutable service generation"
            ),
        ):
            server.config_for(other)

    def test_frozen_runtime_preserves_actual_binary_digest_check_before_launch(
        self,
    ) -> None:
        config = self.initialized()
        script, frozen = self.frozen_source(config)
        binary = Path(str(server.object_mapping(config["server"])["binary"]))
        binary.write_bytes(b"changed executable")
        unit = server.unit_name(self.state)
        binding = {"group": "/controlled/group", "invocation": "b" * 32, "inode": 1}
        with (
            patch.object(server, "SCRIPT", script),
            patch.object(server, "surreal_source", frozen),
            patch.object(
                server._STARTUP,  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
                "deadline",
                time.monotonic() + 30,
                create=True,
            ),
            patch.object(server, "storage_launch", return_value={"binding": binding}),
            patch.object(
                server, "resume_storage_allocation", return_value=self.allocation
            ),
            patch.object(
                host,
                "allocation_metadata",
                return_value=contextlib.nullcontext(
                    {"owners": {self.allocation.nonce: {"units": {unit: binding}}}}
                ),
            ),
            patch.object(server.os, "sched_setaffinity"),
            patch.object(server, "BoundedLog") as log,
            self.assertRaisesRegex(server.SupervisorError, "Storage binary differs"),
        ):
            # This is the actual pre-launch branch, after frozen config validation.
            server.serve(self.state)
        log.assert_not_called()

    def check_recovery_probe_retirement(
        self, mode: str, *, unknown_committed: bool = False
    ) -> None:
        if not (self.state / "config.json").is_file():
            self.initialized()
        history = self.state / ".recovery-controls" / "historical" / "context.json"
        history.parent.mkdir(parents=True, exist_ok=True)
        historical = {"outcome": "incomplete", "database": "historical_probe"}
        server.write_json(history, historical)
        clocks: list[float] = []
        selected: list[Path] = []

        def query(
            _state: Path,
            sql: str,
            deadline: float,
            *,
            namespace: str,
            database: str,
            select_context: bool = True,
        ) -> list[dict[str, object]]:
            clocks.append(deadline)
            self.assertGreater(deadline, server.time.monotonic())
            self.assertEqual(namespace, "pse_recovery")
            path = (
                self.state
                / ".recovery-controls"
                / database.removeprefix("recovery_")
                / "context.json"
            )
            context = server.read_json(path)
            selected.append(path)
            self.assertEqual(context["database"], database)
            acknowledged = {
                "operation": context["operation"],
                "payload": "acknowledged-v1",
            }
            unknown = {
                "operation": context["unknown_operation"],
                "payload": "unknown-v1",
            }
            if sql.startswith("DEFINE NAMESPACE"):
                self.assertFalse(select_context)
                return [
                    {"status": "OK", "result": None},
                    {
                        "status": "OK",
                        "result": {"namespace": namespace, "database": None},
                    },
                    {"status": "OK", "result": None},
                ]
            if sql.startswith("UPSERT"):
                return [
                    {"status": "OK", "result": None},
                    {"status": "OK", "result": acknowledged},
                ]
            if sql.startswith("SELECT"):
                return [
                    {"status": "OK", "result": acknowledged},
                    {"status": "OK", "result": unknown if unknown_committed else None},
                ]
            self.assertEqual(
                sql, f"USE NS `{namespace}`; REMOVE DATABASE `{database}`; INFO FOR NS;"
            )
            self.assertFalse(select_context)
            self.assertEqual(context["outcome"], "settled")
            self.assertTrue(context["acknowledged_survived"])
            self.assertEqual(
                context["unknown_outcome"],
                "committed" if unknown_committed else "not_committed",
            )
            self.assertEqual(context["retirement"], "pending")
            if mode == "unavailable":
                raise OSError("retirement response unavailable")
            if mode == "expired":
                raise server.DeadlineExpiredError("original control clock expired")
            result: list[dict[str, object]] = [
                {"status": "OK", "result": {"namespace": namespace, "database": None}},
                {"status": "OK", "result": None},
                {
                    "status": "OK",
                    "result": {"databases": {"historical_probe": "retained"}},
                },
            ]
            if mode == "partial":
                return result[:2]
            if mode == "wrong-context":
                result[0]["result"] = {"namespace": namespace, "database": database}
            if mode == "failed-statement":
                result[1]["status"] = "ERR"
            if mode == "missing-result":
                result[1].pop("result")
            if mode == "malformed-catalog":
                result[2]["result"] = {}
            if mode == "still-present":
                result[2]["result"] = {"databases": {database: "not retired"}}
            return result

        with (
            patch.object(server, "all_contexts_drained"),
            patch.object(server, "recovery_generation", return_value="same-generation"),
            patch.object(server, "start") as start,
            patch.object(server, "stop"),
            patch.object(server, "reset_failure_window"),
            patch.object(server, "administrative_query", side_effect=query),
            patch.object(server, "abandon_administrative_response"),
            patch.object(server, "materialize_service") as materialize,
            patch.object(server, "public_status", return_value={"qualified": True}),
        ):
            if mode == "passed":
                self.assertEqual(
                    server.qualify_recovery(self.state), {"qualified": True}
                )
            else:
                with self.assertRaises((server.SupervisorError, OSError)):
                    server.qualify_recovery(self.state)
                materialize.assert_not_called()
        self.assertEqual(len(clocks), 4)
        self.assertEqual(len(set(clocks)), 1)
        self.assertTrue(
            all(call.kwargs["deadline"] == clocks[0] for call in start.call_args_list)
        )
        context = server.read_json(selected[-1])
        self.assertEqual(server.read_json(history), historical)
        self.assertEqual(
            context["retirement"], "complete" if mode == "passed" else "pending"
        )
        self.assertEqual(
            context["outcome"], "passed" if mode == "passed" else "settled"
        )
        self.assertEqual(
            server.config_for(self.state)["restart_qualified"], mode == "passed"
        )
        if mode == "passed":
            proof = server.read_json(self.state / "recovery-qualification.json")
            self.assertTrue(
                server.object_mapping(proof["administrative_context"])["retired"]
            )
            self.assertEqual(proof["unknown_outcome"], context["unknown_outcome"])
        else:
            self.assertFalse((self.state / "recovery-qualification.json").exists())

    def test_recovery_retires_only_its_settled_probe_before_positive_qualification(
        self,
    ) -> None:
        for committed in (False, True):
            with self.subTest(unknown_committed=committed):
                self.check_recovery_probe_retirement(
                    "passed", unknown_committed=committed
                )

    def test_recovery_uncertain_probe_retirement_keeps_evidence_and_restart_disabled(
        self,
    ) -> None:
        for mode in (
            "partial",
            "wrong-context",
            "failed-statement",
            "missing-result",
            "malformed-catalog",
            "still-present",
            "unavailable",
            "expired",
        ):
            with self.subTest(mode=mode):
                self.check_recovery_probe_retirement(mode)

    def test_initialize_uses_exact_existing_owner_and_settles_closed(self) -> None:
        config = self.initialized()
        config.update(admission="validation_required", accepting_writes=False)
        server.write_json(self.state / "config.json", config)

        def install(
            state: Path, selected: dict[str, object], command: list[str]
        ) -> None:
            self.assertEqual(command, ["/current/xtask", "canonical-init", str(state)])
            pending = server.object_mapping(selected["derived_rebuild_pending"])
            self.assertEqual(pending["kind"], "initialize")
            self.assertEqual(selected["instance_id"], config["instance_id"])
            self.assertFalse(selected["accepting_writes"])
            self.assertTrue(
                server.reservation_live(
                    server.read_json(state / "lifecycle-owner.json")
                )
            )
            pending["phase"] = "validated"
            server.write_json(state / "config.json", selected)

        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "initialize_rebuild", side_effect=install),
            patch.object(server, "drain_maintenance_listener"),
        ):
            server.initialize(
                self.state, ["/current/xtask", "canonical-init", str(self.state)]
            )
        selected = server.config_for(self.state)
        self.assertEqual(selected["admission"], "quiesced")
        self.assertFalse(selected["accepting_writes"])
        self.assertNotIn("derived_rebuild_pending", selected)
        self.assertEqual(selected["instance_id"], config["instance_id"])

    def test_initialize_failure_retains_same_pending_target_for_retry(self) -> None:
        config = self.initialized()
        config.update(admission="validation_required", accepting_writes=False)
        server.write_json(self.state / "config.json", config)
        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "drain_maintenance_listener"),
            patch.object(
                server,
                "initialize_rebuild",
                side_effect=server.SupervisorError("interrupted"),
            ),
        ):
            pending: dict[str, object] | None = None
            for _ in range(2):
                with self.assertRaisesRegex(server.SupervisorError, "interrupted"):
                    server.initialize(self.state, ["/current/xtask"])
                selected = server.config_for(self.state)
                current = server.object_mapping(selected["derived_rebuild_pending"])
                if pending is not None:
                    self.assertEqual(current, pending)
                pending = current
                self.assertEqual(selected["instance_id"], config["instance_id"])
                self.assertFalse(selected["accepting_writes"])

    def test_initialize_listener_failure_preserves_published_generation_and_materialization(
        self,
    ) -> None:
        config = self.initialized()
        config.update(admission="validation_required", accepting_writes=False)
        server.write_json(self.state / "config.json", config)
        worker = self.root / "captured-worker"
        worker.write_bytes(b"fixture worker")
        replacement = server.publish_generation(self.state, worker)
        self.assertNotEqual(replacement, config["service_supervisor"])

        def failed_listener(_state: Path) -> str:
            current = server.config_for(self.state)
            current.update(service_supervisor=replacement, unit_materialized=True)
            server.write_json(self.state / "config.json", current)
            raise server.SupervisorError("listener setup failed after publication")

        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "drain_maintenance_listener"),
            patch.object(server, "maintenance_endpoint", side_effect=failed_listener),
            self.assertRaisesRegex(
                server.SupervisorError, "listener setup failed after publication"
            ),
        ):
            server.initialize(self.state, ["/current/xtask"])
        retained = server.config_for(self.state)
        self.assertEqual(retained["service_supervisor"], replacement)
        self.assertTrue(retained["unit_materialized"])
        self.assertFalse(retained["accepting_writes"])
        self.assertEqual(
            record(retained["derived_rebuild_pending"])["phase"], "initializing"
        )

    def test_materialization_lost_ack_recovers_exact_private_declaration_only_after_drain(
        self,
    ) -> None:
        config = self.initialized()
        config.update(admission="validation_required", accepting_writes=False)
        server.write_json(self.state / "config.json", config)
        with patch.dict(
            os.environ, {"XDG_CONFIG_HOME": str(self.root / "user-config")}
        ):
            server.materialize_service(self.state, config, self.allocation)
            declaration = (
                self.root / "user-config/systemd/user" / server.unit_name(self.state)
            )
            before = declaration.read_bytes()
            config.pop("unit_materialized")
            server.write_json(self.state / "config.json", config)
            with (
                server.lifecycle_reservation(self.state),
                patch.object(server, "all_contexts_drained") as drain,
                patch.object(
                    server,
                    "service_readiness",
                    return_value=server.ServiceObservation(
                        server.Readiness.ACTIONABLE_ABSENT,
                        self.state,
                        config["instance_id"],
                        server.unit_name(self.state),
                        "exact drained fixture",
                        manager={"ActiveState": "inactive", "ControlGroup": ""},
                    ),
                ),
            ):
                server.materialize_service(self.state, config, self.allocation)
            drain.assert_called_once_with(self.state)
            self.assertEqual(declaration.read_bytes(), before)
            self.assertTrue(server.config_for(self.state)["unit_materialized"])

    def test_materialization_lost_ack_refuses_changed_foreign_or_undrained_unit(
        self,
    ) -> None:
        config = self.initialized()
        config.update(admission="validation_required", accepting_writes=False)
        server.write_json(self.state / "config.json", config)
        with patch.dict(
            os.environ, {"XDG_CONFIG_HOME": str(self.root / "user-config")}
        ):
            server.materialize_service(self.state, config, self.allocation)
            declaration = (
                self.root / "user-config/systemd/user" / server.unit_name(self.state)
            )
            before = declaration.read_bytes()
            config.pop("unit_materialized")
            server.write_json(self.state / "config.json", config)
            for change in ("content", "mode", "symlink", "undrained"):
                with self.subTest(change=change):
                    if change == "content":
                        declaration.write_bytes(before + b"# foreign declaration\n")
                    elif change == "mode":
                        declaration.chmod(0o644)
                    elif change == "symlink":
                        declaration.unlink()
                        target = self.root / "redirected-unit"
                        target.write_bytes(before)
                        target.chmod(0o600)
                        declaration.symlink_to(target)
                    with (
                        server.lifecycle_reservation(self.state),
                        patch.object(server, "all_contexts_drained"),
                        patch.object(
                            server,
                            "service_readiness",
                            return_value=readiness_fixture(server.Readiness.MISMATCH),
                        ),
                        self.assertRaisesRegex(
                            server.SupervisorError, "materialization|inactive drained"
                        ),
                    ):
                        server.materialize_service(self.state, config, self.allocation)
                    self.assertNotIn("unit_materialized", server.config_for(self.state))
                    if declaration.is_symlink():
                        declaration.unlink()
                    declaration.write_bytes(before)
                    declaration.chmod(0o600)

    def interrupted_initializer_launch(
        self,
    ) -> tuple[dict[str, object], dict[str, object]]:
        config = self.initialized()
        owner = {
            "nonce": "a" * 32,
            "purpose": "maintenance",
            "pid": 999999,
            "start": "dead",
        }
        config.update(
            admission="validation_required",
            accepting_writes=False,
            derived_rebuild_pending={
                "kind": "initialize",
                "phase": "initializing",
                "initializer": owner,
                "target_namespace": config["namespace"],
                "target_database": config["database"],
            },
        )
        worker = self.root / "launch-worker"
        worker.write_bytes(b"fixture captured generation")
        replacement = server.publish_generation(self.state, worker)
        launch = {
            "allocation": str(self.root / "admission" / ("b" * 32)),
            "generation": config["instance_id"],
            "deadline": time.monotonic() + 30,
            "supervisor_generation": replacement,
            "server_generation": config["server"],
            "service_class": config.get("service_class", "functional"),
            "server_memory_bytes": record(config["resources"])["server_memory_bytes"],
        }
        server.write_json(self.state / "config.json", config)
        server.write_json(self.state / "service-launch.json", launch)
        server.write_json(
            self.state / ".maintenance-listener.json",
            {
                "instance_id": config["instance_id"],
                "owner": owner,
                "supervisor": replacement,
                "server": config["server"],
                "resources": config["resources"],
                "credentials_sha256": server.file_digest(
                    self.state / "credentials.json"
                ),
                "root_bootstrap": None,
            },
        )
        return config, launch

    def test_interrupted_initializer_witness_authenticates_stale_config_for_drain_only(
        self,
    ) -> None:
        config, launch = self.interrupted_initializer_launch()
        with (
            patch.object(server, "start") as start,
            patch.object(server, "materialize_service") as materialize,
        ):
            selected = server.storage_launch_drain_config(self.state, config, launch)
        self.assertEqual(
            selected["service_supervisor"], launch["supervisor_generation"]
        )
        self.assertEqual(
            {**selected, "service_supervisor": config["service_supervisor"]}, config
        )
        self.assertEqual(server.config_for(self.state), config)
        start.assert_not_called()
        materialize.assert_not_called()

    def test_interrupted_initializer_drain_refuses_changed_owner_generation_or_bound_launch(
        self,
    ) -> None:
        config, launch = self.interrupted_initializer_launch()
        marker_path = self.state / ".maintenance-listener.json"
        original = server.read_json(marker_path)
        for changed in ("owner", "generation", "bound", "mode", "symlink", "moved"):
            with self.subTest(changed=changed):
                marker = json.loads(json.dumps(original))
                candidate = dict(launch)
                if changed == "owner":
                    record(marker["owner"])["nonce"] = "c" * 32
                elif changed == "generation":
                    marker["supervisor"] = config["service_supervisor"]
                elif changed == "bound":
                    candidate["binding"] = {"group": "/recorded/group"}
                elif changed == "moved":
                    marker["supervisor"]["supervisor_script"] = str(
                        self.root
                        / "foreign/.generations"
                        / ("d" * 64)
                        / "scripts/surreal_server.py"
                    )
                    candidate["supervisor_generation"] = marker["supervisor"]
                server.write_json(marker_path, marker)
                if changed == "mode":
                    marker_path.chmod(0o644)
                elif changed == "symlink":
                    marker_path.rename(self.state / "other-marker.json")
                    marker_path.symlink_to(self.state / "other-marker.json")
                with self.assertRaises(server.SupervisorError):
                    server.storage_launch_drain_config(self.state, config, candidate)
                if marker_path.is_symlink():
                    marker_path.unlink()
                server.write_json(marker_path, original)
        self.assertEqual(server.config_for(self.state), config)

    def test_interrupted_initializer_drain_refuses_changed_observations(self) -> None:
        config, launch = self.interrupted_initializer_launch()
        verify = server.verify_generation

        def changed(directory: Path) -> server.GenerationVerification:
            verified = verify(directory)
            path = self.state / ".maintenance-listener.json"
            marker = server.read_json(path)
            record(marker["owner"])["nonce"] = "c" * 32
            server.write_json(path, marker)
            return verified

        with (
            patch.object(server, "verify_generation", side_effect=changed),
            self.assertRaisesRegex(server.SupervisorError, "observations changed"),
        ):
            server.storage_launch_drain_config(self.state, config, launch)

    def test_non_initializer_maintenance_marker_preserves_exact_history_drain_route(
        self,
    ) -> None:
        config, launch = self.interrupted_initializer_launch()
        config["admission"] = "quiesced"
        record(config["derived_rebuild_pending"])["kind"] = "rebuild"
        record(config["derived_rebuild_pending"])["phase"] = "accounts"
        server.write_json(self.state / "config.json", config)
        history = self.state / ".profile-history"
        history.mkdir(mode=0o700)
        prior = {**config, "service_supervisor": launch["supervisor_generation"]}
        server.write_json(history / (("d" * 32) + ".json"), prior)
        with patch.object(
            server,
            "interrupted_initializer_drain_config",
            side_effect=AssertionError("unrelated initializer route"),
        ):
            self.assertEqual(
                server.storage_launch_drain_config(self.state, config, launch), prior
            )
        self.assertEqual(server.config_for(self.state), config)

    def test_initializer_child_failure_drains_under_existing_lifetime(self) -> None:
        config = self.initialized()
        config.update(admission="validation_required", accepting_writes=False)
        server.write_json(self.state / "config.json", config)
        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "drain_maintenance_listener") as drain,
            patch.object(
                server, "maintenance_endpoint", return_value="http://127.0.0.1:18080"
            ) as listener,
            patch.object(server, "establish_protocol_readiness") as readiness,
            patch.object(server, "run_file_effect", return_value=1) as child,
            patch.object(server, "stop") as stop,
            self.assertRaisesRegex(server.SupervisorError, "initializer failed"),
        ):
            server.initialize(self.state, ["/current/xtask"])
        listener.assert_called_once()
        readiness.assert_called_once()
        drain.assert_called_with(self.state)
        stop.assert_not_called()
        arguments, options = child.call_args
        self.assertEqual(arguments[0], ["/current/xtask"])
        pending = server.object_mapping(
            server.config_for(self.state)["derived_rebuild_pending"]
        )
        self.assertEqual(
            arguments[1]["PSE_CANONICAL_INITIALIZER"],
            server.object_mapping(pending["initializer"])["nonce"],
        )
        self.assertIsInstance(options["deadline"], float)

    def test_fresh_engine_retires_only_closed_drained_state_and_rotates_authority(
        self,
    ) -> None:
        old = self.initialized()
        old.update(admission="quiesced", accepting_writes=False)
        server.write_json(self.state / "config.json", old)
        old_payload = server.payload_files.root(self.state, old) / "generated.bin"
        old_payload.write_bytes(b"disposable generated scientific values")
        with (
            patch.object(server, "install", return_value=old["server"]),
            patch.object(server, "all_contexts_drained") as drain,
            patch.object(server, "active", return_value=False),
            patch.object(server, "stop"),
        ):
            server.fresh_engine(self.state, self.root / "tools")
        selected = server.config_for(self.state)
        self.assertNotEqual(selected["instance_id"], old["instance_id"])
        self.assertNotEqual(selected["payload_incarnation"], old["payload_incarnation"])
        self.assertNotEqual(selected["database"], old["database"])
        self.assertEqual(selected["admission"], "validation_required")
        self.assertFalse(selected["accepting_writes"])
        self.assertEqual(selected["endpoint"], "grpc://127.0.0.1:18080")
        self.assertFalse((self.state / "database").exists())
        self.assertFalse(old_payload.exists())
        self.assertFalse((self.state / "fresh-engine-pending.json").exists())
        self.assertGreaterEqual(drain.call_count, 1)

    def test_fresh_engine_rebinds_existing_owned_service_file(self) -> None:
        old = self.initialized()
        old.update(admission="quiesced", accepting_writes=False, unit_materialized=True)
        server.write_json(self.state / "config.json", old)
        manager = self.root / "manager"
        unit = manager / "systemd/user" / server.unit_name(self.state)
        unit.parent.mkdir(parents=True)
        unit.write_text("[Service]\nDescription=old owned generation\n")
        with (
            patch.object(server, "install", return_value=old["server"]),
            patch.object(server, "all_contexts_drained"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "stop"),
            patch.dict(os.environ, {"XDG_CONFIG_HOME": str(manager)}),
        ):
            server.fresh_engine(self.state, self.root / "tools")
            selected = server.config_for(self.state)
            self.assertTrue(selected["unit_materialized"])
            with (
                patch.object(server, "verify_generation"),
                patch.object(server, "recovery_qualified", return_value=False),
                patch.object(
                    server,
                    "storage_placement",
                    return_value={
                        "slice": "pse.slice",
                        "memory_bytes": 2 * server.GIB,
                        "cpu_threads": 2,
                    },
                ),
            ):
                server.materialize_service(self.state, selected, self.allocation)
            self.assertIn(str(selected["instance_id"]), unit.read_text())

    def test_prior_mx_inputs_are_retirement_only(self) -> None:
        old = self.initialized()
        old.update(admission="quiesced", accepting_writes=False)
        old_server = dict(server.object_mapping(old["server"]))
        old_server["inputs"] = {"retired": "previous exact MX generation"}
        old["server"] = old_server
        server.write_json(self.state / "config.json", old)
        with self.assertRaisesRegex(server.SupervisorError, "admitted source-built"):
            server.config_for(self.state)
        current = {**old_server, **fixture_receipt()}
        with (
            patch.object(server, "install", return_value=current),
            patch.object(server, "all_contexts_drained"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "stop"),
        ):
            server.fresh_engine(self.state, self.root / "tools")
        self.assertEqual(server.config_for(self.state)["server"], current)

    def test_fresh_bootstrap_binds_exact_owner_and_never_infers_freshness(self) -> None:
        config = self.initialized(fresh=True)
        config.update(admission="validation_required", accepting_writes=False)
        server.write_json(self.state / "config.json", config)
        with server.lifecycle_reservation(self.state):
            owner = server.read_json(self.state / "lifecycle-owner.json")
            with self.assertRaisesRegex(
                server.SupervisorError, "explicit initialization"
            ):
                server.fresh_root_authority(self.state, config)
            config["surrealkit_rollout_pending"] = {
                "phase": "prepared",
                "sql_outcome": "not_submitted",
                "instance_id": config["instance_id"],
                "owner": owner,
            }
            self.assertEqual(
                server.fresh_root_authority(self.state, config),
                config["fresh_root_bootstrap"],
            )
            for change in ({"owner": {"nonce": "wrong"}}, {"sql_outcome": "unknown"}):
                selected = {
                    **config,
                    "surrealkit_rollout_pending": {
                        **server.object_mapping(config["surrealkit_rollout_pending"]),
                        **change,
                    },
                }
                with self.assertRaises(server.SupervisorError):
                    server.fresh_root_authority(self.state, selected)
            config.pop("fresh_root_bootstrap")
            self.assertIsNone(server.fresh_root_authority(self.state, config))

    def test_fresh_rollout_bootstrap_acknowledges_without_ordinary_readiness(
        self,
    ) -> None:
        config = self.initialized(fresh=True)
        authority = config["fresh_root_bootstrap"]
        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "start"),
            patch.object(server, "drain_maintenance_listener"),
            patch.object(server, "establish_protocol_readiness") as authenticate,
            server.lifecycle_reservation(self.state),
        ):
            owner = server.read_json(self.state / "lifecycle-owner.json")
            config["surrealkit_rollout_pending"] = {
                "phase": "prepared",
                "sql_outcome": "not_submitted",
                "instance_id": config["instance_id"],
                "owner": owner,
            }
            server.write_json(self.state / "config.json", config)
            server.maintenance_endpoint(self.state)
            listener = server.read_json(self.state / ".maintenance-listener.json")
            self.assertEqual(listener["root_bootstrap"], authority)
            self.assertEqual(listener["owner"], owner)
            self.assertFalse(authenticate.call_args.kwargs["record_readiness"])
            selected = server.config_for(self.state)
            self.assertNotIn("fresh_root_bootstrap", selected)
            self.assertFalse(selected["accepting_writes"])
            self.assertFalse((self.state / "protocol-readiness.json").exists())

    def test_fresh_engine_refuses_open_or_undrained_before_disposal(self) -> None:
        old = self.initialized()
        with patch.object(server, "install") as build:
            with self.assertRaisesRegex(server.SupervisorError, "closed"):
                server.fresh_engine(self.state, self.root / "tools")
            build.assert_not_called()
        old.update(admission="quiesced", accepting_writes=False)
        server.write_json(self.state / "config.json", old)
        with (
            patch.object(
                server,
                "all_contexts_drained",
                side_effect=server.SupervisorError("undrained"),
            ),
            patch.object(server, "discard_generated_tree") as discard,
            self.assertRaisesRegex(server.SupervisorError, "undrained"),
        ):
            server.fresh_engine(self.state, self.root / "tools")
        discard.assert_not_called()
        self.assertTrue((self.state / "database" / "fixture").exists())

    def test_fresh_engine_resume_keeps_exact_target_after_interrupted_disposal(
        self,
    ) -> None:
        old = self.initialized()
        old.update(admission="quiesced", accepting_writes=False)
        server.write_json(self.state / "config.json", old)
        with (
            patch.object(server, "install", return_value=old["server"]) as build,
            patch.object(server, "all_contexts_drained"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "stop"),
        ):
            with (
                patch.object(
                    server,
                    "discard_generated_tree",
                    side_effect=server.SupervisorError("interrupted"),
                ),
                self.assertRaisesRegex(server.SupervisorError, "interrupted"),
            ):
                server.fresh_engine(self.state, self.root / "tools")
            pending = server.read_json(self.state / "fresh-engine-pending.json")
            with self.assertRaisesRegex(server.SupervisorError, "incomplete"):
                server.config_for(self.state)
            server.fresh_engine(self.state, self.root / "tools")
            build.assert_called_once()
        selected = server.config_for(self.state)
        self.assertEqual(selected, pending["target"])
        self.assertEqual(
            server.read_json(self.state / "credentials.json"), pending["credentials"]
        )

    def test_fresh_engine_legacy_profile_is_retirement_only(self) -> None:
        old = self.initialized()
        budget = server.object_mapping(old["resources"])
        old_budget = {
            key: value
            for key, value in budget.items()
            if not key.startswith("surrealmx_") and key != "grpc_max_attached_sessions"
        }
        old_budget.update(
            rocksdb_block_cache_bytes=2 * server.GIB // 4,
            rocksdb_write_buffer_bytes=32 * server.MIB,
            rocksdb_write_buffers=2,
        )
        old.update(
            profile_version=2,
            endpoint="ws://127.0.0.1:18080",
            websocket_max_message_bytes=server.MESSAGE_BYTES,
            resources=old_budget,
            admission="quiesced",
            accepting_writes=False,
        )
        old.pop("rpc_max_message_bytes")
        old.pop("payload_incarnation")
        server.write_json(self.state / "config.json", old)
        with self.assertRaisesRegex(server.SupervisorError, "fresh-engine"):
            server.config_for(self.state)
        with (
            patch.object(server, "install", return_value=old["server"]),
            patch.object(server, "all_contexts_drained"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "stop"),
        ):
            server.fresh_engine(self.state, self.root / "tools")
        self.assertEqual(server.config_for(self.state)["profile_version"], 3)
        self.assertNotEqual(
            server.config_for(self.state).get("payload_incarnation"), None
        )

    def test_joint_budget_rejects_overallocation(self) -> None:
        with self.assertRaises(server.SupervisorError):
            server.resources(2 * server.GIB, server.GIB, 2, server.GIB)
        allocation = server.resources(4 * server.GIB, 2 * server.GIB, 2, server.GIB)
        self.assertLess(
            server.integer(allocation["surrealmx_memory_limit_bytes"]),
            server.integer(allocation["memory_threshold_bytes"]),
        )
        self.assertLess(
            server.integer(allocation["memory_threshold_bytes"]),
            server.integer(allocation["server_memory_bytes"]),
        )

    def current_observer_registration(
        self, *, pid: int | None = None
    ) -> dict[str, object]:
        self.initialized()
        selected = os.getpid() if pid is None else pid
        process = Path(f"/proc/{selected}")
        record: dict[str, object] = {
            "pid": selected,
            "start": (process / "stat").read_text().rsplit(")", 1)[1].split()[19],
            "group": next(
                line.removeprefix("0::")
                for line in (process / "cgroup").read_text().splitlines()
                if line.startswith("0::")
            ),
        }
        server.write_json(self.state / "primary-observer.json", record)
        return record

    def test_release_current_observer_removes_only_own_drained_registration(
        self,
    ) -> None:
        self.current_observer_registration()
        stopped = {
            "LoadState": "loaded",
            "ActiveState": "inactive",
            "ControlGroup": "/empty-primary",
        }
        with (
            patch.object(server, "primary_observation", return_value=stopped),
            patch.object(server, "group_populated", return_value=False) as population,
        ):
            server.release_current_observer(self.state)
        population.assert_called_once_with("/empty-primary")
        self.assertFalse((self.state / "primary-observer.json").exists())

    def test_release_current_observer_refuses_foreign_live_pid(self) -> None:
        child = subprocess.Popen([sys.executable, "-c", "import time; time.sleep(10)"])
        try:
            record = self.current_observer_registration(pid=child.pid)
            with self.assertRaisesRegex(server.SupervisorError, "does not own"):
                server.release_current_observer(self.state)
            self.assertEqual(
                server.read_json(self.state / "primary-observer.json"), record
            )
            self.assertIsNone(child.poll())
        finally:
            child.terminate()
            child.wait(timeout=2)

    def test_release_current_observer_refuses_stale_process_identity(self) -> None:
        record = self.current_observer_registration()
        for key, value in (("start", "stale-start"), ("group", "/another-observer")):
            with self.subTest(key=key):
                changed = {**record, key: value}
                server.write_json(self.state / "primary-observer.json", changed)
                with self.assertRaisesRegex(server.SupervisorError, "identity changed"):
                    server.release_current_observer(self.state)
                self.assertEqual(
                    server.read_json(self.state / "primary-observer.json"), changed
                )

    def test_release_current_observer_refuses_live_or_populated_primary(self) -> None:
        record = self.current_observer_registration()
        for state, populated in (
            ("active", False),
            ("activating", False),
            ("inactive", True),
            ("failed", True),
        ):
            with (
                self.subTest(state=state, populated=populated),
                patch.object(
                    server,
                    "primary_observation",
                    return_value={
                        "LoadState": "loaded",
                        "ActiveState": state,
                        "ControlGroup": "/primary",
                    },
                ),
                patch.object(server, "group_populated", return_value=populated),
                self.assertRaisesRegex(
                    server.SupervisorError, "stopped, drained primary"
                ),
            ):
                server.release_current_observer(self.state)
            self.assertEqual(
                server.read_json(self.state / "primary-observer.json"), record
            )

    def test_release_current_observer_rechecks_identity_after_primary_observation(
        self,
    ) -> None:
        record = self.current_observer_registration()
        changed = {**record, "start": "changed-during-observation"}

        def observation(_state: Path) -> dict[str, str]:
            server.write_json(self.state / "primary-observer.json", changed)
            return {
                "LoadState": "loaded",
                "ActiveState": "inactive",
                "ControlGroup": "",
            }

        with (
            patch.object(server, "primary_observation", side_effect=observation),
            self.assertRaisesRegex(server.SupervisorError, "identity changed"),
        ):
            server.release_current_observer(self.state)
        self.assertEqual(
            server.read_json(self.state / "primary-observer.json"), changed
        )

    def test_reference_execution_materializes_exact_case_profile(self) -> None:
        allocation = server.reference_resources()
        execution = allocation["execution"]
        self.assertIsInstance(execution, dict)
        if not isinstance(execution, dict):
            self.fail("materialized execution must be an object")
        self.assertEqual(execution["pool_memory_bytes"], 128 * server.GIB)
        self.assertEqual(execution["worker_bytes"], 16 * server.GIB)
        self.assertEqual(execution["cpu_threads"], 16)
        self.assertEqual(execution["case_lanes"], 16)
        self.assertEqual(execution["math_jobs"], 32)
        self.assertEqual(allocation["native_workers"], 1)
        self.assertEqual(allocation["native_worker_memory_bytes"], 140 * server.GIB)
        self.assertEqual(allocation["total_memory_bytes"], 160 * server.GIB)
        with self.assertRaises(server.SupervisorError):
            server.resources(
                156 * server.GIB, 16 * server.GIB, 1, 140 * server.GIB, execution
            )

    def reference_fixture(self) -> dict[str, object]:
        config = self.initialized()
        worker = self.root / "pse-worker"
        worker.write_text("#!/bin/sh\nexit 0\n")
        worker.chmod(0o700)
        selected = patch.dict(os.environ, {"PSE_WORKER_BINARY": str(worker)})
        selected.start()
        self.addCleanup(selected.stop)
        config["resources"] = server.reference_resources()
        config["primary_receiver"] = server.publish_generation(self.state, worker)
        server.write_json(self.state / "config.json", config)
        return config

    def test_generation_publish_verifies_identical_concurrent_winner(self) -> None:
        for collision in (errno.EEXIST, errno.ENOTEMPTY):
            with self.subTest(errno=collision):
                state = self.root / str(collision)
                state.mkdir()

                def concurrent_winner(
                    pending: Path, destination: Path, collision: int = collision
                ) -> Path:
                    self.assertFalse(destination.exists())
                    shutil.copytree(pending, destination)
                    raise OSError(collision, "concurrent immutable generation")

                with patch.object(Path, "rename", new=concurrent_winner):
                    generation = server.publish_generation(state)
                destination = Path(generation["supervisor_script"]).parents[1]
                server.verify_generation(destination)
                pending = list((state / ".generations").glob("pending-*"))
                self.assertEqual(len(pending), 1)
                self.assertTrue((pending[0] / "generation.json").exists())

    def test_generation_publish_rejects_corrupt_concurrent_winner(self) -> None:
        self.state.mkdir()

        def concurrent_winner(pending: Path, destination: Path) -> Path:
            self.assertFalse(destination.exists())
            shutil.copytree(pending, destination)
            (destination / "scripts/surreal_server.py").write_text("corrupt winner")
            raise OSError(errno.ENOTEMPTY, "concurrent immutable generation")

        with (
            patch.object(Path, "rename", new=concurrent_winner),
            self.assertRaisesRegex(server.SupervisorError, "closure changed"),
        ):
            server.publish_generation(self.state)
        self.assertEqual(len(list((self.state / ".generations").glob("pending-*"))), 1)

    def test_generation_publish_propagates_other_rename_errors(self) -> None:
        self.state.mkdir()
        with (
            patch.object(
                Path, "rename", side_effect=OSError(errno.EACCES, "rename denied")
            ),
            self.assertRaises(OSError) as failure,
        ):
            server.publish_generation(self.state)
        self.assertEqual(failure.exception.errno, errno.EACCES)
        self.assertEqual(len(list((self.state / ".generations").glob("pending-*"))), 1)

    def test_changed_primary_executable_requires_readmission(self) -> None:
        config = self.reference_fixture()
        server.checked_primary(config)
        Path(server.recorded_primary(config)["worker_executable"]).write_text(
            "#!/bin/sh\nexit 1\n"
        )
        with self.assertRaises(server.SupervisorError):
            server.checked_primary(config)

    def test_primary_environment_selects_worker_receipt_and_native_inputs(self) -> None:
        config = self.reference_fixture()
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            self.fail("reference resources are not an object")
        receiver = server.checked_primary(config)
        receipt = self.root / "worker-role.json"
        receipt.write_text("worker admission belongs to the deployment owner")
        with patch.dict(
            os.environ,
            {
                "PSE_PRODUCER_RECEIPT": "/python-role.json",
                "PSE_WORKER_PRODUCER_RECEIPT": str(receipt),
                "PSE_WORKER_BINARY": str(self.root / "pse-worker"),
                "PSE_NATIVE_PROVIDER_RECEIPT": "/provider.json",
                "IPOPT_DIR": "/admitted/solver",
                "SYMBOLICA_LICENSE": "private-test-value",
                "PSE_NATIVE_OPERATION": "/caller-operation.json",
            },
            clear=True,
        ):
            environment, names = server.primary_environment(
                self.state, allocation, receiver
            )
        self.assertEqual(environment["PSE_PRODUCER_RECEIPT"], str(receipt))
        self.assertEqual(environment["PSE_NATIVE_PROVIDER_RECEIPT"], "/provider.json")
        self.assertEqual(environment["IPOPT_DIR"], "/admitted/solver")
        self.assertNotIn("PSE_NATIVE_OPERATION", environment)
        arguments = server.primary_service_environment(environment, names)
        for name in (
            "PSE_PRODUCER_RECEIPT",
            "PSE_NATIVE_PROVIDER_RECEIPT",
            "IPOPT_DIR",
            "SYMBOLICA_LICENSE",
        ):
            self.assertIn(f"--setenv={name}", arguments)
        self.assertFalse(
            any("private-test-value" in argument for argument in arguments)
        )
        removed = next(
            argument
            for argument in arguments
            if argument.startswith("--property=UnsetEnvironment=")
        )
        self.assertIn("PSE_NATIVE_OPERATION", removed)

    def test_primary_environment_without_worker_role_removes_generic_receipt(
        self,
    ) -> None:
        config = self.reference_fixture()
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            self.fail("reference resources are not an object")
        receiver = server.checked_primary(config)
        with patch.dict(
            os.environ, {"PSE_PRODUCER_RECEIPT": "/python-role.json"}, clear=True
        ):
            environment, names = server.primary_environment(
                self.state, allocation, receiver
            )
        self.assertNotIn("PSE_PRODUCER_RECEIPT", environment)
        arguments = server.primary_service_environment(environment, names)
        self.assertNotIn("--setenv=PSE_PRODUCER_RECEIPT", arguments)
        removed = next(
            argument
            for argument in arguments
            if argument.startswith("--property=UnsetEnvironment=")
        )
        self.assertIn("PSE_PRODUCER_RECEIPT", removed)

    def test_selected_worker_mismatch_refuses_before_primary_launch_or_reuse(
        self,
    ) -> None:
        self.reference_fixture()
        other = self.root / "other-worker"
        other.write_text("#!/bin/sh\nexit 1\n")
        other.chmod(0o700)
        # Selected bytes must match the admitted immutable receiving artifact.
        with (
            patch.dict(os.environ, {"PSE_WORKER_BINARY": str(other)}, clear=True),
            patch.object(server, "ensure_execution_placement") as placement,
            patch.object(server, "primary_ready") as readiness,
            patch.object(server.subprocess, "run") as launch,
            self.assertRaises(server.SupervisorError),
        ):
            server.ensure_primary(self.state)
        placement.assert_not_called()
        readiness.assert_not_called()
        launch.assert_not_called()

    def test_primary_receipt_reuse_checks_actual_path_and_observed_bytes(self) -> None:
        receipt = self.root / "worker.json"
        receipt.write_bytes(b"actual worker receipt")
        process = self.root / "proc-worker"
        process.mkdir()
        (process / "environ").write_bytes(
            b"OTHER=ignored\0PSE_PRODUCER_RECEIPT=" + os.fsencode(receipt) + b"\0"
        )
        environment = {"PSE_WORKER_PRODUCER_RECEIPT": str(receipt)}
        marker: dict[str, object] = {
            "producer_receipt_path": str(receipt),
            "producer_receipt_sha256": server.file_digest(receipt),
        }
        self.assertTrue(server.primary_receipt_ready(process, marker, environment))
        self.assertFalse(server.primary_receipt_ready(process, marker, {}))
        alternate = self.root / "other-worker.json"
        alternate.write_bytes(receipt.read_bytes())
        self.assertFalse(
            server.primary_receipt_ready(
                process, marker, {"PSE_WORKER_PRODUCER_RECEIPT": str(alternate)}
            )
        )
        receipt.write_bytes(b"changed worker receipt")
        self.assertFalse(server.primary_receipt_ready(process, marker, environment))
        (process / "environ").write_bytes(b"PSE_PRODUCER_RECEIPT=/python-role.json\0")
        marker["producer_receipt_sha256"] = server.file_digest(receipt)
        self.assertFalse(server.primary_receipt_ready(process, marker, environment))

    def test_primary_local_reuse_requires_actual_generic_receipt_absence(self) -> None:
        process = self.root / "proc-worker"
        process.mkdir()
        (process / "environ").write_bytes(b"OTHER=ignored\0")
        self.assertTrue(server.primary_receipt_ready(process, {}, {}))
        (process / "environ").write_bytes(b"PSE_PRODUCER_RECEIPT=\0")
        self.assertFalse(server.primary_receipt_ready(process, {}, {}))

    def test_primary_reuses_only_verified_ready_receiver(self) -> None:
        self.reference_fixture()
        observation = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "/owned",
        }
        with (
            patch.object(server, "ensure_execution_placement"),
            patch.object(server, "primary_observation", return_value=observation),
            patch.object(server, "primary_ready", return_value=True),
            patch.object(
                server, "primary_dispatch_owner", return_value={"controlled": "owner"}
            ),
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.READY),
            ),
            patch.object(server.subprocess, "run") as launch,
        ):
            self.assertTrue(server.ensure_primary(self.state)["ready"])
            launch.assert_not_called()

    def test_primary_generation_verification_is_shared_only_within_admission(
        self,
    ) -> None:
        config = self.reference_fixture()
        captures = []

        def admitted(*_args: object) -> dict[str, object]:
            self.assertTrue((self.state / "primary-admission.json").exists())
            before = digest.call_count
            server.checked_primary(config)
            count = digest.call_count - before
            self.assertGreater(count, 0)
            server.checked_primary(config)
            self.assertEqual(digest.call_count - before, count)
            captures.append(count)
            return {"ready": True}

        with (
            patch.object(server, "file_digest", wraps=server.file_digest) as digest,
            patch.object(server, "_ensure_primary", side_effect=admitted),
        ):
            server.ensure_primary(self.state)
            server.ensure_primary(self.state)
        self.assertEqual(captures[0], captures[1])
        self.assertIsNone(getattr(server._STARTUP, "verification", None))  # noqa: SLF001 -- owner evidence must not survive admission

    def test_primary_shared_generation_invalidates_changed_bytes_path_link_and_mode(
        self,
    ) -> None:
        config = self.reference_fixture()
        receiver = server.checked_primary(config)
        worker = Path(receiver["worker_executable"])
        original = worker.read_bytes()
        original_mode = worker.stat().st_mode
        manifest = worker.parents[1] / "generation.json"

        for mutation in (
            "same-inode-restored-mtime",
            "mode",
            "widened-mode",
            "manifest-mode",
            "link",
            "path",
        ):

            def admitted(*_args: object, mutation: str = mutation) -> dict[str, object]:
                server.checked_primary(config)
                selected = json.loads(json.dumps(config))
                before = worker.stat()
                try:
                    if mutation == "same-inode-restored-mtime":
                        worker.write_bytes(b"x" * len(original))
                        os.utime(worker, ns=(before.st_atime_ns, before.st_mtime_ns))
                        self.assertEqual(worker.stat().st_ino, before.st_ino)
                        self.assertEqual(worker.stat().st_mtime_ns, before.st_mtime_ns)
                    elif mutation == "mode":
                        worker.chmod(0o600)
                    elif mutation == "widened-mode":
                        worker.chmod(0o755)
                    elif mutation == "manifest-mode":
                        manifest.chmod(0o644)
                    elif mutation == "link":
                        other = self.root / "unowned-worker"
                        other.write_bytes(original)
                        other.chmod(0o700)
                        worker.unlink()
                        worker.symlink_to(other)
                    else:
                        selected["primary_receiver"]["worker_executable"] = str(
                            self.root / "unowned-worker"
                        )
                    with self.assertRaises(server.SupervisorError):
                        server.checked_primary(selected)
                finally:
                    if worker.is_symlink():
                        worker.unlink()
                    worker.write_bytes(original)
                    worker.chmod(original_mode)
                    manifest.chmod(0o600)
                return {"ready": True}

            with (
                self.subTest(mutation=mutation),
                patch.object(server, "_ensure_primary", side_effect=admitted),
            ):
                server.ensure_primary(self.state)

    def test_primary_new_process_and_association_require_full_generation_capture(
        self,
    ) -> None:
        config = self.reference_fixture()
        observation = {
            "ActiveState": "active",
            "ControlGroup": "/owned",
            "MainPID": str(os.getpid()),
        }
        marker = {
            "pid": os.getpid(),
            "canonical_database": "selected",
            "nonce": "original",
        }
        server.write_json(self.state / "primary-receiver.json", marker)
        server.write_json(self.state / "primary-launch.json", {"nonce": "original"})

        def admitted(*_args: object) -> dict[str, object]:
            server.checked_primary(config)
            qualified = digest.call_count
            server.primary_ready(self.state, config, observation, "selected")
            initial = digest.call_count
            self.assertGreater(initial, 0)
            self.assertEqual(initial, qualified)
            server.primary_ready(self.state, config, observation, "selected")
            self.assertEqual(digest.call_count, initial)
            marker["nonce"] = "new-process"
            server.write_json(self.state / "primary-receiver.json", marker)
            server.write_json(
                self.state / "primary-launch.json", {"nonce": "new-process"}
            )
            server.primary_ready(self.state, config, observation, "selected")
            self.assertGreater(digest.call_count, initial)
            previous = digest.call_count
            server.primary_ready(self.state, config, observation, "other-database")
            self.assertGreater(digest.call_count, previous)
            previous = digest.call_count
            with patch.object(
                server.native_operation, "start_identity", return_value="new-start"
            ):
                server.primary_ready(self.state, config, observation, "other-database")
            self.assertGreater(digest.call_count, previous)
            previous = digest.call_count
            (self.state / "primary-receiver.json").unlink()
            server.primary_ready(self.state, config, observation, "selected")
            self.assertGreater(digest.call_count, previous)
            self.assertEqual(process.call_count, 6)
            return {"ready": True}

        with (
            patch.object(server, "file_digest", wraps=server.file_digest) as digest,
            patch.object(
                server, "_primary_process", return_value=os.getpid()
            ) as process,
            patch.object(server, "_ensure_primary", side_effect=admitted),
        ):
            server.ensure_primary(self.state)

    def test_generation_stream_capture_stops_on_original_clock_without_sealing(
        self,
    ) -> None:
        config = self.reference_fixture()
        generation = Path(
            server.primary_association(config)["supervisor_script"]
        ).parents[1]
        owner = server.PrimaryVerification()
        expired = server.DeadlineExpiredError("original control clock")
        with (
            patch.object(server._STARTUP, "deadline", 100, create=True),  # noqa: SLF001 -- inject the original admission clock
            patch.object(server._STARTUP, "verification", owner, create=True),  # noqa: SLF001 -- inspect unsealed cancelled evidence
            patch.object(server, "remaining", side_effect=expired),
            self.assertRaises(server.DeadlineExpiredError),
        ):
            server.verify_generation(generation)
        self.assertEqual(owner.generations, {})

        large = self.root / "chunked-content"
        large.write_bytes(b"x" * (3 * server.MIB))
        with (
            patch.object(server._STARTUP, "deadline", 100, create=True),  # noqa: SLF001 -- every chunk consumes the same clock
            patch.object(server, "remaining", side_effect=[1, 1, expired]) as clock,
            self.assertRaises(server.DeadlineExpiredError),
        ):
            server.file_digest(large)
        self.assertEqual(clock.call_args_list, [call(100)] * 3)

    def test_generation_capture_never_seals_partial_or_changed_inventory(self) -> None:
        config = self.reference_fixture()
        worker = Path(server.primary_association(config)["worker_executable"])
        generation = worker.parents[1]
        digest = server.file_digest
        for failure in ("cancel", "mutation"):
            owner = server.PrimaryVerification()
            calls: list[Path] = []

            def capture(
                path: Path, failure: str = failure, calls: list[Path] = calls
            ) -> str:
                result = digest(path)
                calls.append(path)
                if failure == "cancel" and len(calls) == 2:
                    raise server.DeadlineExpiredError("original clock during capture")
                if failure == "mutation" and path == worker:
                    information = worker.stat()
                    worker.write_bytes(b"x" * information.st_size)
                    os.utime(
                        worker, ns=(information.st_atime_ns, information.st_mtime_ns)
                    )
                return result

            with (
                self.subTest(failure=failure),
                patch.object(server._STARTUP, "verification", owner, create=True),  # noqa: SLF001 -- incomplete evidence must remain private and unsealed
                patch.object(server, "file_digest", side_effect=capture),
                self.assertRaises(server.SupervisorError),
            ):
                server.verify_generation(generation)
            self.assertGreaterEqual(len(calls), 2)
            self.assertEqual(owner.generations, {})

    def test_unmatched_live_primary_is_refused(self) -> None:
        self.reference_fixture()
        observation = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "/owned",
        }
        with (
            patch.object(server, "ensure_execution_placement"),
            patch.object(server, "primary_observation", return_value=observation),
            patch.object(server, "primary_ready", return_value=False),
            patch.object(server.subprocess, "run") as launch,
            self.assertRaises(server.SupervisorError),
        ):
            server.ensure_primary(self.state)
        launch.assert_not_called()

    def test_primary_cold_start_preserves_database_and_one_case_group(self) -> None:
        config = self.reference_fixture()
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        deadline = server.time.monotonic() + 5
        admission = []

        def prepare(
            unit: str, *, reservation: tuple[Path, str], deadline: float
        ) -> Path:
            self.assertGreater(deadline, server.time.monotonic())
            self.assertEqual(unit, server.primary_unit(self.state))
            self.assertEqual(reservation[0], self.state / "primary-admission.json")
            self.assertEqual(server.read_json(reservation[0])["nonce"], reservation[1])
            admission.append(reservation[1])
            return self.root / "handoff"

        with (
            patch.object(server._STARTUP, "deadline", deadline, create=True),  # noqa: SLF001 -- exercise the original inherited admission clock
            patch.object(server, "ensure_execution_placement"),
            patch.object(
                server, "primary_dispatch_owner", return_value={"controlled": "owner"}
            ),
            patch.object(server, "primary_observation", return_value=inactive),
            patch.object(
                server, "primary_ready", side_effect=[False, True]
            ) as readiness,
            patch.object(server, "settled_worker", return_value=inactive),
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.READY),
            ),
            patch(
                "scripts.native_operation.prepare_handoff",
                side_effect=prepare,
            ) as handoff,
            patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0),
            ) as launch,
        ):
            receipt = server.ensure_primary(self.state, database="isolated-study")
        handoff.assert_called_once_with(
            server.primary_unit(self.state),
            reservation=(self.state / "primary-admission.json", admission[0]),
            deadline=deadline,
        )
        self.assertEqual(receipt["canonical_database"], "isolated-study")
        self.assertEqual(readiness.call_args.args[-1], "isolated-study")
        command = launch.call_args.args[0]
        self.assertIn(f"--slice={server.execution_slice(self.state)}", command)
        self.assertIn(f"--property=MemoryMax={140 * server.GIB}", command)
        self.assertEqual(
            command[-4:],
            ["--maximum-in-flight", "16", "--canonical-database", "isolated-study"],
        )
        self.assertGreater(launch.call_args.kwargs["timeout"], 0)
        self.assertLessEqual(launch.call_args.kwargs["timeout"], 30)
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            self.fail("reference resources are not an object")
        self.assertEqual(allocation["execution"], receipt["execution"])

    def test_live_observer_launcher_reserves_the_single_foreground_allocation(
        self,
    ) -> None:
        self.reference_fixture()
        start = (
            Path(f"/proc/{os.getpid()}/stat").read_text().rsplit(")", 1)[1].split()[19]
        )
        server.write_json(
            self.state / "observer-launch.json",
            {"pid": os.getpid(), "start": start, "unit": "pse-observer-live.scope"},
        )
        with (
            patch.object(server, "ensure_execution_placement"),
            patch.object(server.subprocess, "call") as launch,
            self.assertRaises(server.SupervisorError),
        ):
            server.observer(self.state, ["python", "-m", "pytest"])
        launch.assert_not_called()

    def test_observer_scope_preserves_shared_profile_and_finite_cap(self) -> None:
        allocation = server.reference_resources()
        command = server.observer_scope_command(
            self.state,
            allocation,
            "pse-observer-owned.scope",
            ["python", "-m", "pytest", "-n", "0"],
        )
        self.assertIn(f"--slice={server.execution_slice(self.state)}", command)
        self.assertIn(f"--property=MemoryMax={4 * server.GIB}", command)
        self.assertIn("--property=MemorySwapMax=0", command)
        self.assertEqual(command[-5:], ["python", "-m", "pytest", "-n", "0"])
        with self.assertRaises(server.SupervisorError):
            server.observer_scope_command(
                self.state, allocation, "pse-observer-owned.scope", []
            )

    @contextlib.contextmanager
    def observer_control_fixture(
        self, memory: int = 144 * server.GIB
    ) -> Generator[tuple[dict[str, object], MagicMock, MagicMock], None, None]:
        group = "/owned/control"
        control = "pse-owned-control.scope"
        self.allocation.directory = Path(
            tempfile.mkdtemp(prefix="admission-", dir=self.root)
        )
        state: dict[str, object] = {
            "memory": memory,
            "group": group,
            "invocation": "b" * 32,
            "inode": 17,
        }
        ledger: dict[str, object] = {
            "owners": {
                self.allocation.nonce: {
                    "units": {
                        control: {"group": group, "invocation": "b" * 32, "inode": 17}
                    }
                }
            }
        }

        @contextlib.contextmanager
        def metadata(
            _directory: Path, **_options: object
        ) -> Generator[dict[str, object], None, None]:
            yield ledger

        def manager(
            *arguments: str, **_kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            if arguments[0] == "set-property":
                for argument in arguments:
                    if argument.startswith("MemoryMax="):
                        state["memory"] = int(argument.removeprefix("MemoryMax="))
                output = ""
            else:
                output = (
                    "LoadState=loaded\nActiveState=active\n"
                    f"ControlGroup={state['group']}\nInvocationID={state['invocation']}\n"
                    f"MemoryMax={state['memory']}\n"
                )
            return subprocess.CompletedProcess([], 0, output, "")

        with (
            patch.object(host, "allocation_metadata", side_effect=metadata),
            patch.object(server.native_operation, "process_group", return_value=group),
            patch.object(host, "group_identity", side_effect=lambda _: state["inode"]),
            patch.object(server, "systemctl", side_effect=manager) as calls,
            patch.object(
                server, "observer_allocation_drained", return_value=True
            ) as drained,
        ):
            yield state, calls, drained

    def test_observer_control_cap_restored_after_verified_drain(self) -> None:
        self.reference_fixture()
        observation: dict[str, object] = {}
        with (
            self.observer_control_fixture() as (state, _calls, _drained),
            patch.object(server, "ensure_execution_placement"),
            patch.object(
                server.native_operation, "prepare_handoff", return_value=None
            ) as handoff,
            patch.object(server, "systemd_environment", return_value={}),
            patch.object(server, "observer_scope_command", return_value=["probe"]),
            patch.object(server.native_operation, "run", return_value=0) as launch,
        ):
            self.assertEqual(
                server.observer(
                    self.state, ["probe"], "exclusive-observer", observation=observation
                ),
                0,
            )
            self.assertEqual(state["memory"], 144 * server.GIB)
        launch.assert_called_once()
        handoff.assert_called_once_with(
            self.allocation.register.call_args.args[0], foreground=True
        )
        self.allocation.release.assert_not_called()
        self.assertFalse((self.state / "observer-launch.json").exists())
        self.assertEqual(observation["owned_drain"], "complete")

    def test_parent_owned_observer_isolates_supervisor_from_gate_process_group(
        self,
    ) -> None:
        self.reference_fixture()
        server.native_operation.write_json(
            self.root / "handoff",
            {"version": server.native_operation.VERSION, "admissions": {}},
        )
        with (
            self.observer_control_fixture(),
            patch.object(server, "ensure_execution_placement"),
            patch.object(
                server.native_operation,
                "prepare_handoff",
                return_value=self.root / "handoff",
            ),
            patch.object(server, "systemd_environment", return_value={}),
            patch.object(server, "observer_scope_command", return_value=["probe"]),
            patch.object(server.native_operation, "run", return_value=0) as launch,
        ):
            self.assertEqual(
                server.observer(self.state, ["probe"], "exclusive-observer"), 0
            )
        self.assertEqual(
            launch.call_args.args[1]["PSE_NATIVE_HANDOFF"],
            str(self.root / "handoff"),
        )
        self.assertEqual(launch.call_args.kwargs["foreground"], self.root / "handoff")

    def test_observer_control_cap_restored_when_launch_raises(self) -> None:
        with self.observer_control_fixture() as (state, _calls, _drained):
            with (
                self.assertRaisesRegex(OSError, "launch failed"),
                server.observer_control_limit(
                    self.allocation, "pse-observer-owned.scope", "exclusive-observer"
                ),
            ):
                self.assertEqual(state["memory"], 8 * server.GIB)
                raise OSError("launch failed")
            self.assertEqual(state["memory"], 144 * server.GIB)

    def test_observer_control_survivor_retains_launch_guard(self) -> None:
        self.reference_fixture()
        with (
            self.observer_control_fixture() as (state, _calls, drained),
            patch.object(server, "ensure_execution_placement"),
            patch.object(server.native_operation, "prepare_handoff", return_value=None),
            patch.object(server, "systemd_environment", return_value={}),
            patch.object(server, "observer_scope_command", return_value=["probe"]),
            patch.object(server.native_operation, "run", return_value=0),
        ):
            drained.return_value = False
            self.assertEqual(
                server.observer(self.state, ["probe"], "exclusive-observer"), 0
            )
            self.assertEqual(state["memory"], 8 * server.GIB)
        self.allocation.release.assert_not_called()
        self.assertTrue((self.state / "observer-launch.json").exists())

    def test_observer_control_cap_kept_when_observer_survives(self) -> None:
        with self.observer_control_fixture() as (state, _calls, drained):
            drained.return_value = False
            with server.observer_control_limit(
                self.allocation, "pse-observer-owned.scope", "exclusive-observer"
            ):
                self.assertEqual(state["memory"], 8 * server.GIB)
            self.assertEqual(state["memory"], 8 * server.GIB)
        self.allocation.release.assert_not_called()

    def test_observer_control_cap_refuses_replaced_scope_before_restoration(
        self,
    ) -> None:
        for field, replacement in (
            ("group", "/replaced/control"),
            ("invocation", "c" * 32),
            ("inode", 18),
        ):
            with (
                self.subTest(field=field),
                self.observer_control_fixture() as (state, calls, _drained),
            ):
                with (
                    self.assertRaisesRegex(server.SupervisorError, "identity changed"),
                    server.observer_control_limit(
                        self.allocation,
                        "pse-observer-owned.scope",
                        "exclusive-observer",
                    ),
                ):
                    state[field] = replacement
                self.assertEqual(state["memory"], 8 * server.GIB)
                self.assertEqual(
                    sum(
                        call.args[0] == "set-property" for call in calls.call_args_list
                    ),
                    1,
                )

    def test_observer_control_cap_never_widens_an_originally_smaller_limit(
        self,
    ) -> None:
        with self.observer_control_fixture(2 * server.GIB) as (state, calls, _drained):
            with server.observer_control_limit(
                self.allocation, "pse-observer-owned.scope", "exclusive-observer"
            ):
                self.assertEqual(state["memory"], 2 * server.GIB)
            self.assertEqual(state["memory"], 2 * server.GIB)
            for call in calls.call_args_list:
                for argument in call.args:
                    if argument.startswith("MemoryMax="):
                        self.assertLessEqual(
                            int(argument.removeprefix("MemoryMax=")), 2 * server.GIB
                        )

    def test_observer_control_cap_excludes_overlapping_borrowers(self) -> None:
        with self.observer_control_fixture() as (state, calls, _drained):
            with server.observer_control_limit(
                self.allocation, "first-observer.scope", "exclusive-observer"
            ):
                with (
                    self.assertRaisesRegex(
                        server.SupervisorError, "already has a cap borrower"
                    ),
                    server.observer_control_limit(
                        self.allocation,
                        "different-state-observer.scope",
                        "exclusive-observer",
                    ),
                ):
                    self.fail("An overlapping observer must never launch")
                self.assertEqual(state["memory"], 8 * server.GIB)
                self.assertEqual(
                    sum(
                        call.args[0] == "set-property" for call in calls.call_args_list
                    ),
                    1,
                )
            self.assertEqual(state["memory"], 144 * server.GIB)
            self.assertEqual(
                server.read_json(
                    self.allocation.directory / "observer-control-borrows.json"
                ),
                {},
            )

    def test_observer_control_survivor_retains_borrow_reservation(self) -> None:
        with self.observer_control_fixture() as (state, _calls, drained):
            drained.return_value = False
            with server.observer_control_limit(
                self.allocation, "first-observer.scope", "exclusive-observer"
            ):
                pass
            self.assertEqual(state["memory"], 8 * server.GIB)
            with (
                self.assertRaisesRegex(
                    server.SupervisorError, "already has a cap borrower"
                ),
                server.observer_control_limit(
                    self.allocation,
                    "different-state-observer.scope",
                    "exclusive-observer",
                ),
            ):
                self.fail("A surviving observer must retain its reservation")

    def test_observer_control_cap_refuses_replaced_borrow_nonce(self) -> None:
        with self.observer_control_fixture() as (state, calls, _drained):
            with (
                self.assertRaisesRegex(
                    server.SupervisorError, "borrow identity changed"
                ),
                server.observer_control_limit(
                    self.allocation, "first-observer.scope", "exclusive-observer"
                ),
            ):
                path = self.allocation.directory / "observer-control-borrows.json"
                reservations = server.read_json(path)
                server.object_mapping(reservations["pse-owned-control.scope"])[
                    "nonce"
                ] = "c" * 32
                server.write_json(path, reservations)
            self.assertEqual(state["memory"], 8 * server.GIB)
            self.assertEqual(
                sum(call.args[0] == "set-property" for call in calls.call_args_list), 1
            )

    def test_observer_drain_requires_bound_kernel_lifetime(self) -> None:
        unit = "observer.scope"
        bound = {"group": "/owned/observer", "invocation": "b" * 32, "inode": 17}
        for label, units, identity, populated, invocation, expected in (
            ("absent", {}, 17, False, "b" * 32, False),
            ("unbound-not-found", {unit: {}}, 17, False, "", False),
            ("bound-empty", {unit: bound}, 17, False, "b" * 32, True),
            ("bound-populated", {unit: bound}, 17, True, "b" * 32, False),
            ("replaced-inode", {unit: bound}, 18, True, "c" * 32, True),
            ("changed-invocation", {unit: bound}, 17, False, "c" * 32, False),
        ):
            with self.subTest(lifetime=label):
                record = {
                    "boot": "d" * 8 + "-" + "-".join(["d" * 4] * 3) + "-" + "d" * 12,
                    "units": units,
                    "terminal_guards": {"f" * 32: {"phase": "registered"}},
                }
                original = json.loads(json.dumps(record))
                ledger: dict[str, object] = {"owners": {self.allocation.nonce: record}}

                @contextlib.contextmanager
                def metadata(
                    _directory: Path,
                    captured: dict[str, object] = ledger,
                    **_options: object,
                ) -> Generator[dict[str, object], None, None]:
                    yield captured

                with (
                    patch.object(host, "allocation_metadata", side_effect=metadata),
                    patch.object(host, "boot", return_value=record["boot"]),
                    patch.object(host, "group_identity", return_value=identity),
                    patch.object(host.operation, "populated", return_value=populated),
                    patch.object(
                        host.operation,
                        "unit_observation",
                        return_value={
                            "LoadState": "not-found"
                            if label in {"absent", "unbound-not-found"}
                            else "loaded",
                            "ActiveState": "active",
                            "InvocationID": invocation,
                        },
                    ),
                ):
                    self.assertEqual(
                        server.observer_allocation_drained(self.allocation, unit),
                        expected,
                    )
                    self.assertFalse(host.drained({**record, "released": True}))
                self.assertEqual(record, original)

    def test_observer_cleanup_restoration_uses_original_cancellation_clock(
        self,
    ) -> None:
        with self.observer_control_fixture() as (state, manager, drained):
            metadata_control = host.allocation_metadata
            assert isinstance(metadata_control, MagicMock)
            metadata = metadata_control.side_effect
            assert callable(metadata)
            calls: list[float | None] = []

            def bounded_metadata(
                directory: Path, *, deadline: float | None = None, **_options: object
            ) -> contextlib.AbstractContextManager[dict[str, object]]:
                calls.append(deadline)
                return metadata(directory)

            clock: list[float | None] = [None]
            with patch.object(
                host, "allocation_metadata", side_effect=bounded_metadata
            ):
                with server.observer_control_limit(
                    self.allocation,
                    "observer.scope",
                    "exclusive-observer",
                    cancellation_deadline=lambda: clock[0],
                ):
                    clock[0] = time.monotonic() + 1
                self.assertEqual(state["memory"], 144 * server.GIB)
            self.assertEqual(calls[-2:], [clock[0], clock[0]])
            self.assertEqual(drained.call_args.kwargs["deadline"], clock[0])
            restoration = [
                call
                for call in manager.call_args_list
                if call.kwargs.get("deadline") is not None
            ]
            self.assertTrue(restoration)
            self.assertTrue(
                all(call.kwargs["deadline"] == clock[0] for call in restoration)
            )

    def test_observer_physical_drain_refuses_contended_metadata_by_original_clock(
        self,
    ) -> None:
        owner = host.acquire(
            host.select("light"), directory=self.root / "private-admission"
        )
        descriptor = os.open(owner.directory / ".lock", os.O_RDWR)
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        started = time.monotonic()
        try:
            with self.assertRaises(host.AdmissionError):
                server.observer_allocation_drained(
                    owner, "unknown.scope", deadline=started + 0.03
                )
        finally:
            os.close(descriptor)
        self.assertLess(time.monotonic() - started, 0.2)
        with host.allocation_metadata(owner.directory) as ledger:
            self.assertIn(owner.nonce, ledger["owners"])

    def test_observer_unbound_launch_retains_cap_and_reservation(self) -> None:
        actual_drained = server.observer_allocation_drained
        with self.observer_control_fixture() as (state, _calls, drained):
            drained.side_effect = actual_drained
            with server.observer_control_limit(
                self.allocation, "unbound-observer.scope", "exclusive-observer"
            ):
                pass
            self.assertEqual(state["memory"], 8 * server.GIB)
            reservations = server.read_json(
                self.allocation.directory / "observer-control-borrows.json"
            )
            self.assertIn("pse-owned-control.scope", reservations)

    def test_observer_known_prelaunch_failure_restores_and_clears_reservations(
        self,
    ) -> None:
        self.reference_fixture()
        actual_drained = server.observer_allocation_drained
        with (
            self.observer_control_fixture() as (state, _calls, drained),
            patch.object(server, "ensure_execution_placement"),
            patch.object(
                server.native_operation,
                "prepare_handoff",
                side_effect=OSError("handoff failed"),
            ),
            patch.object(server.native_operation, "run") as launch,
        ):
            drained.side_effect = actual_drained
            with self.assertRaisesRegex(OSError, "handoff failed"):
                server.observer(self.state, ["probe"], "exclusive-observer")
            self.assertEqual(state["memory"], 144 * server.GIB)
            self.assertEqual(
                server.read_json(
                    self.allocation.directory / "observer-control-borrows.json"
                ),
                {},
            )
        launch.assert_not_called()
        self.assertFalse((self.state / "observer-launch.json").exists())

    def test_qualification_control_requires_private_owned_state_directory(self) -> None:
        self.reference_fixture()
        private = self.state / "native-entry-control"
        private.mkdir(mode=0o700)
        self.assertEqual(server.qualification_directory(self.state, private), private)
        with self.assertRaises(server.SupervisorError):
            server.qualification_directory(self.state, self.root)
        private.chmod(0o755)
        with self.assertRaises(server.SupervisorError):
            server.qualification_directory(self.state, private)
        private.chmod(0o700)
        alias = self.state / "entry-alias"
        alias.symlink_to(private, target_is_directory=True)
        with self.assertRaises(server.SupervisorError):
            server.qualification_directory(self.state, alias)

    def test_qualification_primary_launch_forwards_only_explicit_control(self) -> None:
        self.reference_fixture()
        private = self.state / "native-entry-control"
        private.mkdir(mode=0o700)
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        with (
            patch.object(server, "ensure_execution_placement"),
            patch.object(
                server, "primary_dispatch_owner", return_value={"controlled": "owner"}
            ),
            patch.object(server, "primary_observation", return_value=inactive),
            patch.object(
                server, "primary_ready", side_effect=[False, True]
            ) as readiness,
            patch.object(server, "settled_worker", return_value=inactive),
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.READY),
            ),
            patch(
                "scripts.native_operation.prepare_handoff",
                return_value=self.root / "handoff",
            ),
            patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0),
            ) as launch,
        ):
            server.ensure_primary(
                self.state, database="qualified-study", qualification=private
            )
        self.assertEqual(readiness.call_args.args[-1], private)
        self.assertEqual(
            launch.call_args.args[0][-2:],
            ["--qualification-native-entry", str(private)],
        )

    def test_qualification_cannot_use_nonprimary_command(self) -> None:
        args = server.parser().parse_args(
            [
                "start",
                "--state",
                str(self.state),
                "--qualification-native-entry",
                str(self.state / "control"),
            ]
        )
        with self.assertRaises(server.SupervisorError):
            server.dispatch(args)

    def test_primary_database_mismatch_cannot_claim_readiness(self) -> None:
        config = self.reference_fixture()
        server.write_json(
            self.state / "primary-launch.json", {"nonce": "actual-launch"}
        )
        server.write_json(
            self.state / "primary-receiver.json",
            {
                "ready": True,
                "nonce": "actual-launch",
                "pid": os.getpid(),
                "canonical_database": "another-study",
                **server.reference_execution(),
            },
        )
        observation = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "/owned",
            "MemoryMax": str(140 * server.GIB),
        }
        self.assertFalse(
            server.primary_ready(self.state, config, observation, "selected-study")
        )

    def test_primary_quiesce_signals_actual_receiver_without_killing_wrapper(
        self,
    ) -> None:
        self.reference_fixture()
        server.write_json(self.state / "primary-receiver.json", {"pid": 12345})
        args = server.parser().parse_args(["quiesce", "--state", str(self.state)])
        real_close = os.close

        def close_resource(descriptor: int) -> None:
            if descriptor != 87:
                real_close(descriptor)

        with (
            patch.object(server, "primary_observation", return_value={}),
            patch.object(server, "primary_drain_pid", return_value=12345),
            patch.object(server.os, "pidfd_open", return_value=87) as open_pid,
            patch.object(server.signal, "pidfd_send_signal") as signal_pid,
            patch.object(server.os, "close", side_effect=close_resource) as close_pid,
            patch.object(server, "active", return_value=False),
            patch("builtins.print"),
        ):
            self.assertEqual(server.dispatch(args), 0)
        open_pid.assert_called_once_with(12345)
        signal_pid.assert_called_once_with(87, server.signal.SIGINT)
        self.assertEqual(
            sum(call.args == (87,) for call in close_pid.call_args_list), 1
        )
        self.assertFalse(server.config_for(self.state)["accepting_writes"])

    def test_functional_no_primary_quiesce_keeps_worker_drain_checks(self) -> None:
        config = self.reference_fixture()
        config["primary_receiver"] = None
        server.write_json(self.state / "config.json", config)
        idle = {"LoadState": "not-found", "ActiveState": "inactive", "ControlGroup": ""}
        args = server.parser().parse_args(["quiesce", "--state", str(self.state)])
        with (
            patch.object(server, "primary_observation", return_value=idle),
            patch.object(
                server,
                "recorded_primary",
                side_effect=AssertionError("no primary was configured"),
            ),
            patch.object(server, "settled_worker", return_value=idle) as slots,
            patch.object(server, "active", return_value=False),
            patch("builtins.print"),
        ):
            self.assertEqual(server.dispatch(args), 0)
            closed = server.config_for(self.state)
            self.assertFalse(closed["accepting_writes"])
            server.workers_drained(self.state, closed)
        allocation = server.object_mapping(config["resources"])
        self.assertEqual(slots.call_count, server.integer(allocation["native_workers"]))

    def test_functional_no_primary_refuses_surviving_primary_or_worker(self) -> None:
        config = self.reference_fixture()
        config["primary_receiver"] = None
        idle = {"LoadState": "not-found", "ActiveState": "inactive", "ControlGroup": ""}
        live = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "/owned/live",
        }
        with self.assertRaisesRegex(server.SupervisorError, "Unconfigured primary"):
            server.primary_drain_pid(self.state, config, live)
        with (
            patch.object(server, "primary_observation", return_value=idle),
            patch.object(server, "settled_worker", return_value=live),
            self.assertRaisesRegex(
                server.SupervisorError, "Managed native worker slots"
            ),
        ):
            server.workers_drained(self.state, config)

    def test_functional_no_primary_still_refuses_live_observer(self) -> None:
        config = self.reference_fixture()
        config["primary_receiver"] = None
        server.write_json(self.state / "config.json", config)
        server.write_json(
            self.state / "primary-observer.json",
            {
                "pid": 999999999,
                "start": "0",
                "group": "/owned/observer",
            },
        )
        with (
            patch.object(server, "require_borrowers_drained") as borrowers,
            patch.object(server, "observer_launch_busy", return_value=False),
            patch.object(server, "group_populated", return_value=True),
            self.assertRaisesRegex(
                server.SupervisorError, "observer process group remains populated"
            ),
        ):
            server.all_contexts_drained(self.state)
        borrowers.assert_called_once_with(self.state)

    def test_missing_primary_configuration_is_not_explicit_no_primary(self) -> None:
        config = self.reference_fixture()
        del config["primary_receiver"]
        idle = {"LoadState": "not-found", "ActiveState": "inactive", "ControlGroup": ""}
        with self.assertRaisesRegex(
            server.SupervisorError, "configured primary receiver"
        ):
            server.primary_drain_pid(self.state, config, idle)

    def live_primary_fixture(
        self,
    ) -> tuple[dict[str, object], subprocess.Popen[bytes], dict[str, str]]:
        config = self.reference_fixture()
        worker = self.root / "pse-worker"
        shutil.copyfile(Path(sys.executable).resolve(), worker)
        config["primary_receiver"] = server.publish_generation(self.state, worker)
        server.write_json(self.state / "config.json", config)
        worker = Path(server.recorded_primary(config)["worker_executable"])
        server.write_json(
            self.state / "execution-placement.json", {"physical_cpus": list(range(16))}
        )
        nonce = "owned-live-launch"
        environment = {
            **os.environ,
            "PSE_PRIMARY_NONCE": nonce,
            "PYTHONHOME": sys.base_prefix,
        }
        environment.pop("PSE_PRODUCER_RECEIPT", None)
        child = subprocess.Popen(
            [str(worker), "-c", "import time; time.sleep(30)"],
            env=environment,
        )

        def stop_child() -> None:
            if child.poll() is None:
                child.terminate()
            child.wait(timeout=2)

        self.addCleanup(stop_child)
        server.write_json(self.state / "primary-launch.json", {"nonce": nonce})
        server.write_json(
            self.state / "primary-receiver.json",
            {
                "ready": True,
                "nonce": nonce,
                "pid": child.pid,
                "canonical_database": config["database"],
                **server.reference_execution(),
            },
        )
        group = next(
            line.removeprefix("0::")
            for line in Path(f"/proc/{child.pid}/cgroup").read_text().splitlines()
            if line.startswith("0::")
        )
        observation = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": group,
            "MemoryMax": str(140 * server.GIB),
        }
        return config, child, observation

    def test_live_primary_readiness_checks_current_admission(self) -> None:
        config, child, observation = self.live_primary_fixture()
        with (
            patch.object(server, "group_populated", return_value=True),
            patch.object(server, "role_affinity_ready", return_value=True),
            patch.object(
                server, "effective_limits", return_value=(140 * server.GIB, 16)
            ),
            patch.object(server, "systemd_environment", return_value={}),
        ):
            self.assertTrue(server.primary_ready(self.state, config, observation))
            self.assertEqual(
                server.primary_drain_pid(self.state, config, observation), child.pid
            )

    def test_primary_quiesce_drains_immutable_generation_after_source_rebuild(
        self,
    ) -> None:
        config, child, observation = self.live_primary_fixture()
        worker = self.root / "pse-worker"
        replacement = self.root / "rebuilt-worker"
        replacement.write_bytes(b"different next admission")
        replacement.replace(worker)
        self.assertEqual(
            str(Path(f"/proc/{child.pid}/exe").readlink()),
            server.recorded_primary(config)["worker_executable"],
        )
        self.assertEqual(
            server.file_digest(Path(f"/proc/{child.pid}/exe")),
            server.recorded_primary(config)["worker_sha256"],
        )
        args = server.parser().parse_args(["quiesce", "--state", str(self.state)])
        with (
            patch.object(server, "primary_observation", return_value=observation),
            patch.object(server, "group_populated", return_value=True),
            patch.object(server, "role_affinity_ready", return_value=True),
            patch.object(
                server, "effective_limits", return_value=(140 * server.GIB, 16)
            ),
            patch.object(server, "systemd_environment", return_value={}),
            patch.object(server.signal, "pidfd_send_signal") as signal_pid,
            patch.object(server, "active", return_value=False),
            patch("builtins.print"),
        ):
            self.assertTrue(server.primary_ready(self.state, config, observation))
            self.assertEqual(
                server.primary_drain_pid(self.state, config, observation), child.pid
            )
            self.assertEqual(server.dispatch(args), 0)
        self.assertEqual(signal_pid.call_count, 1)
        self.assertEqual(signal_pid.call_args.args[1], server.signal.SIGINT)
        self.assertIsNone(child.poll())
        self.assertEqual(server.config_for(self.state)["admission"], "quiescing")

    def test_primary_drain_refuses_changed_process_bytes_group_and_nonce(self) -> None:
        config, child, observation = self.live_primary_fixture()
        receiver = config["primary_receiver"]
        self.assertIsInstance(receiver, dict)
        if not isinstance(receiver, dict):
            self.fail("missing receiver")
        marker = server.read_json(self.state / "primary-receiver.json")
        with (
            patch.object(server, "group_populated", return_value=True),
            patch.object(server, "role_affinity_ready", return_value=True),
            patch.object(
                server, "effective_limits", return_value=(140 * server.GIB, 16)
            ),
        ):
            self.assertEqual(
                server.primary_drain_pid(self.state, config, observation), child.pid
            )
            with self.subTest("different admitted bytes"):
                receiver["worker_sha256"] = "0" * 64
                self.assertIsNone(
                    server.primary_drain_pid(self.state, config, observation)
                )
                receiver["worker_sha256"] = server.file_digest(
                    Path(f"/proc/{child.pid}/exe")
                )
            with self.subTest("foreign cgroup"):
                self.assertIsNone(
                    server.primary_drain_pid(
                        self.state, config, {**observation, "ControlGroup": "/foreign"}
                    )
                )
            with self.subTest("stale launch and marker"):
                server.write_json(
                    self.state / "primary-launch.json", {"nonce": "stale"}
                )
                server.write_json(
                    self.state / "primary-receiver.json", {**marker, "nonce": "stale"}
                )
                self.assertIsNone(
                    server.primary_drain_pid(self.state, config, observation)
                )

    def test_primary_quiesce_empty_group_with_missing_disk_worker(self) -> None:
        self.reference_fixture()
        (self.root / "pse-worker").unlink()
        args = server.parser().parse_args(["quiesce", "--state", str(self.state)])
        with (
            patch.object(server.os, "pidfd_open") as open_pid,
            patch.object(server, "active", return_value=False),
            patch("builtins.print"),
        ):
            self.assertEqual(server.dispatch(args), 0)
        open_pid.assert_not_called()
        self.assertFalse(server.config_for(self.state)["accepting_writes"])

    def test_primary_drain_refuses_malformed_recorded_admission(self) -> None:
        config = self.reference_fixture()
        config["primary_receiver"] = {"worker_executable": "/unowned"}
        with self.assertRaisesRegex(server.SupervisorError, "unsupported fields"):
            server.primary_drain_pid(self.state, config, {})

    def test_profile_ancestor_limits_are_read_from_actual_hierarchy(self) -> None:
        parent = self.root / "cgroups"
        child = parent / "primary"
        child.mkdir(parents=True)
        (parent / "memory.max").write_text(str(120 * server.GIB))
        (child / "memory.max").write_text(str(140 * server.GIB))
        (parent / "cpu.max").write_text("800000 100000")
        (child / "cpu.max").write_text("max 100000")
        memory, cpu = server.effective_limits(child)
        self.assertEqual(memory, 120 * server.GIB)
        self.assertEqual(cpu, 8)

    def test_insufficient_ancestor_cap_refuses_before_placement_mutation(self) -> None:
        group = self.root / "originating-allocation"
        group.mkdir()
        (group / "memory.max").write_text(str(120 * server.GIB))
        with (
            patch.object(server, "group_for_slice", return_value=group),
            patch.object(server, "physical_cpus", return_value=list(range(16))),
            patch.object(
                server, "effective_limits", return_value=(120 * server.GIB, 16)
            ),
            patch.object(server, "systemctl") as manager,
            self.assertRaises(server.SupervisorError),
        ):
            server.ensure_execution_placement(self.state, server.reference_resources())
        manager.assert_not_called()

    def test_placement_host_lookup_boundaries_share_original_deadline(self) -> None:
        self.reference_fixture()
        directory = self.allocation.directory
        directory.mkdir(mode=0o700)
        program = "import fcntl,os,sys; f=open(sys.argv[1], 'a'); os.chmod(sys.argv[1], 0o600); fcntl.flock(f, fcntl.LOCK_EX); print('held',flush=True); sys.stdin.readline()"
        child = subprocess.Popen(
            [sys.executable, "-c", program, str(directory / ".lock")],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
        )
        controls = (
            lambda: server.execution_slice(self.state),
            lambda: server.physical_cpus(1),
            lambda: server.selected_slice(self.state, {}),
            lambda: server.ensure_execution_placement(
                self.state, server.reference_resources()
            ),
        )
        try:
            assert child.stdout is not None
            self.assertEqual(child.stdout.readline().strip(), "held")
            for control in controls:
                deadline = server.time.monotonic() + 0.05
                with (
                    self.subTest(control=control),
                    patch.object(server._STARTUP, "deadline", deadline, create=True),  # noqa: SLF001 -- exercise the original inherited admission clock
                    patch.dict(
                        os.environ,
                        {host.MARKER: str(directory / self.allocation.nonce)},
                    ),
                    patch.object(
                        host, "inherit", side_effect=self.host_inherit
                    ) as inherit,
                    patch.object(
                        host, "readonly_snapshot", side_effect=self.host_snapshot
                    ),
                    patch.object(host, "enforce_allocation") as enforce,
                    self.assertRaises(server.DeadlineExpiredError),
                ):
                    control()
                self.assertEqual(inherit.call_args.kwargs["deadline"], deadline)
                enforce.assert_not_called()
                self.assertFalse((self.state / "execution-placement.json").exists())
        finally:
            child.communicate("release\n", timeout=5)
            self.assertEqual(child.returncode, 0)
        self.assertFalse((self.state / "execution-placement.json").exists())

    def test_placement_enforcement_and_publication_consume_original_clock(self) -> None:
        self.reference_fixture()
        group = self.root / "placement-group"
        group.mkdir()
        (group / "memory.max").write_text(str(160 * server.GIB))
        for boundary in ("enforcement", "publication"):
            clock = [10.0]

            def expire_enforcement(
                *_args: object,
                boundary: str = boundary,
                clock: list[float] = clock,
                **kwargs: object,
            ) -> None:
                self.assertEqual(kwargs["deadline"], 11)
                if boundary == "enforcement":
                    clock[0] = 12
                    raise host.AdmissionError("host control clock expired")

            def expire_publication(
                _group: Path, *, clock: list[float] = clock
            ) -> tuple[int, int]:
                clock[0] = 12
                return 160 * server.GIB, 16

            with (
                self.subTest(boundary=boundary),
                patch.object(server._STARTUP, "deadline", 11, create=True),  # noqa: SLF001 -- exercise the original inherited admission clock
                patch.object(
                    server.time, "monotonic", side_effect=lambda clock=clock: clock[0]
                ),
                patch.object(server, "physical_cpus", return_value=list(range(16))),
                patch.object(server, "group_for_slice", return_value=group),
                patch.object(
                    host, "enforce_allocation", side_effect=expire_enforcement
                ),
                patch.object(
                    server,
                    "host_memory",
                    return_value=(188 * server.GIB, 112 * server.GIB),
                ),
                patch.object(
                    server, "effective_limits", side_effect=expire_publication
                ),
                self.assertRaises(server.DeadlineExpiredError),
            ):
                server.ensure_execution_placement(
                    self.state, server.reference_resources()
                )
            self.assertFalse((self.state / "execution-placement.json").exists())

    def test_storage_admission_consumed_by_role_start_shares_original_clock(
        self,
    ) -> None:
        config = self.initialized()
        deadline = server.time.monotonic() + 5
        for exclusive in (False, True):
            owner = MagicMock(spec=host.Allocation)
            owner.directory = self.allocation.directory
            owner.nonce = self.allocation.nonce
            owner.profile = (
                host.select("reference")
                if exclusive
                else host.select(
                    "store-functional",
                    str(
                        server.object_mapping(config["resources"])[
                            "server_memory_bytes"
                        ]
                    ),
                )
            )
            row: dict[str, object] = {"units": {}}

            @contextlib.contextmanager
            def metadata(
                _directory: Path,
                *,
                deadline: float | None = None,
                offered: float = deadline,
                owner: host.Allocation = owner,
                row: dict[str, object] = row,
            ) -> Generator[dict[str, object], None, None]:
                self.assertEqual(deadline, offered)
                yield {"owners": {owner.nonce: row}}

            with (
                self.subTest(exclusive=exclusive),
                patch.object(
                    host, "inherit", return_value=owner if exclusive else None
                ) as inherit,
                patch.object(host, "acquire", return_value=owner) as acquire,
                patch.object(host, "allocation_metadata", side_effect=metadata),
            ):
                self.assertIs(
                    server.service_allocation(self.state, config, deadline), owner
                )
            self.assertEqual(inherit.call_args.kwargs["deadline"], deadline)
            owner.register.assert_called_once_with(
                server.unit_name(self.state), deadline=deadline
            )
            if exclusive:
                acquire.assert_not_called()
                self.assertEqual(row["borrowed_services"], [str(self.state)])
            else:
                self.assertEqual(acquire.call_args.kwargs["deadline"], deadline)
                self.assertEqual(row["service"], str(self.state))
            self.assertEqual(
                server.read_json(self.state / "service-launch.json")["deadline"],
                deadline,
            )

    def test_storage_resume_keeps_exact_predecessor_checks_on_original_clock(
        self,
    ) -> None:
        config = self.initialized()
        deadline = server.time.monotonic() + 5
        predecessor = {"group": "/owned", "invocation": "a" * 32, "inode": 17}
        processes = self.root / "cgroup.procs"
        for scenario in (
            "valid",
            "same-invocation",
            "replacement-group",
            "outside-membership",
            "surviving-descendant",
        ):
            observed = {
                "ControlGroup": "/replacement"
                if scenario == "replacement-group"
                else "/owned",
                "InvocationID": "a" * 32 if scenario == "same-invocation" else "b" * 32,
                "ActiveState": "activating",
            }
            processes.write_text(
                str(os.getpid())
                + ("\n99999999" if scenario == "surviving-descendant" else "")
            )
            with (
                self.subTest(scenario=scenario),
                patch.object(
                    server,
                    "recorded_storage_owner",
                    return_value=(self.allocation, {"binding": predecessor}),
                ) as record,
                patch.object(
                    server, "storage_unit_observation", return_value=observed
                ) as observe,
                patch.object(
                    server.native_operation,
                    "process_group",
                    return_value="/outside"
                    if scenario == "outside-membership"
                    else "/owned",
                ),
                patch.object(Path, "rglob", return_value=iter([processes])),
            ):
                if scenario == "valid":
                    self.assertIs(
                        server.resume_storage_allocation(
                            self.state, config, deadline=deadline
                        ),
                        self.allocation,
                    )
                else:
                    with self.assertRaises(server.SupervisorError):
                        server.resume_storage_allocation(
                            self.state, config, deadline=deadline
                        )
            record.assert_called_once_with(self.state, config, deadline=deadline)
            observe.assert_called_once_with(self.state, deadline=deadline)

    def test_storage_serve_consumed_host_controls_share_startup_or_resume_clock(
        self,
    ) -> None:
        config = self.initialized()
        unit = server.unit_name(self.state)
        parent_deadline = server.time.monotonic() + 5
        binding = {"group": "/owned", "invocation": "b" * 32, "inode": 18}
        for restart in (False, True):
            launch: dict[str, object] = {
                "allocation": str(self.allocation.directory / self.allocation.nonce),
                "deadline": parent_deadline - 1,
            }
            if restart:
                launch["binding"] = {**binding, "invocation": "a" * 32}
            server.write_json(self.state / "service-launch.json", launch)
            expected_deadline = parent_deadline if restart else parent_deadline - 1

            @contextlib.contextmanager
            def metadata(
                _directory: Path,
                *,
                deadline: float | None = None,
                expected_deadline: float = expected_deadline,
            ) -> Generator[dict[str, object], None, None]:
                self.assertEqual(deadline, expected_deadline)
                yield {"owners": {self.allocation.nonce: {"units": {unit: binding}}}}

            self.allocation.bind.reset_mock()
            with (
                self.subTest(restart=restart),
                patch.object(server._STARTUP, "deadline", parent_deadline, create=True),  # noqa: SLF001 -- exercise the original inherited admission clock
                patch.object(server, "storage_launch", return_value=launch),
                patch.object(
                    server, "resume_storage_allocation", return_value=self.allocation
                ) as resume,
                patch.object(host, "inherit", return_value=self.allocation) as inherit,
                patch.object(server, "storage_placement"),
                patch.object(host, "allocation_metadata", side_effect=metadata),
                patch.object(host, "enforce_parent") as enforce,
                patch.object(
                    server.os,
                    "sched_setaffinity",
                    side_effect=RuntimeError("stop before payload"),
                ),
                self.assertRaisesRegex(RuntimeError, "stop before payload"),
            ):
                server.serve(self.state)
            if restart:
                resume.assert_called_once_with(
                    self.state, config, deadline=parent_deadline
                )
                inherit.assert_not_called()
            else:
                self.assertEqual(
                    inherit.call_args.kwargs["deadline"], expected_deadline
                )
            self.allocation.bind.assert_called_once_with(
                unit, deadline=expected_deadline
            )
            self.assertEqual(enforce.call_args.kwargs["deadline"], expected_deadline)

    def test_storage_hash_expiry_prevents_payload_dispatch(self) -> None:
        config = self.initialized()
        deadline = server.time.monotonic() + 5
        clock = [deadline - 1]
        unit = server.unit_name(self.state)
        binding = {"group": "/owned", "invocation": "b" * 32, "inode": 18}
        launch = {
            "allocation": str(self.allocation.directory / self.allocation.nonce),
            "deadline": deadline,
            "binding": binding,
        }
        server.write_json(self.state / "service-launch.json", launch)

        @contextlib.contextmanager
        def metadata(
            _directory: Path, *, deadline: float | None = None
        ) -> Generator[dict[str, object], None, None]:
            self.assertEqual(deadline, launch["deadline"])
            yield {"owners": {self.allocation.nonce: {"units": {unit: binding}}}}

        def hash_after_expiry(_path: Path) -> str:
            clock[0] = deadline + 1
            return str(server.object_mapping(config["server"])["binary_sha256"])

        with (
            patch.object(server, "config_for", return_value=config),
            patch.object(server._STARTUP, "deadline", deadline, create=True),  # noqa: SLF001 -- exercise the original inherited admission clock
            patch.object(
                server.time, "monotonic", side_effect=lambda clock=clock: clock[0]
            ),
            patch.object(server, "storage_launch", return_value=launch),
            patch.object(
                server, "resume_storage_allocation", return_value=self.allocation
            ),
            patch.object(server, "storage_placement"),
            patch.object(host, "allocation_metadata", side_effect=metadata),
            patch.object(server.os, "sched_setaffinity"),
            patch.object(
                server, "file_digest", side_effect=hash_after_expiry
            ) as digest,
            patch.object(server.subprocess, "Popen") as dispatch,
            self.assertRaises(server.DeadlineExpiredError),
        ):
            server.serve(self.state)
        digest.assert_called_once_with(
            Path(str(server.object_mapping(config["server"])["binary"]))
        )
        dispatch.assert_not_called()
        self.assertFalse((self.state / "server-process.json").exists())

    def test_host_capacity_below_cap_refuses_before_placement_mutation(self) -> None:
        # Host capacity belongs to admission; the supervisor must respect its
        # refusal before reading or changing any placement state.
        with (
            patch.object(server, "physical_cpus", return_value=list(range(16))),
            patch.object(
                host,
                "enforce_allocation",
                side_effect=host.AdmissionError(
                    "Host physical memory cannot admit this owner"
                ),
            ),
            patch.object(server, "group_for_slice") as group,
            patch.object(server, "systemctl") as manager,
            self.assertRaisesRegex(host.AdmissionError, "Host physical memory"),
        ):
            server.ensure_execution_placement(self.state, server.reference_resources())
        group.assert_not_called()
        manager.assert_not_called()

    def test_available_memory_is_observed_without_upfront_reservation(self) -> None:
        self.reference_fixture()
        group = self.root / "cgroups/profile"
        group.mkdir(parents=True)
        (group / "memory.max").write_text(str(160 * server.GIB))
        (group / "memory.current").write_text(str(2 * server.GIB))
        (group / "cpuset.cpus.effective").write_text("0-15")
        with (
            patch.object(server, "physical_cpus", return_value=list(range(16))),
            patch.object(server, "group_for_slice", return_value=group),
            patch.object(
                server,
                "effective_limits",
                return_value=(160 * server.GIB, 16),
            ),
            patch.object(
                server, "host_memory", return_value=(188 * server.GIB, 112 * server.GIB)
            ),
            patch.object(server, "systemctl") as manager,
        ):
            server.ensure_execution_placement(self.state, server.reference_resources())
        manager.assert_not_called()
        observation = server.read_json(self.state / "execution-placement.json")
        self.assertEqual(observation["available_plus_owned_bytes"], 114 * server.GIB)
        self.assertEqual(observation["memory_max_bytes"], 160 * server.GIB)
        self.assertFalse(observation["physically_reserved"])

    def test_readiness_marker_without_kernel_association_is_not_readiness(self) -> None:
        config = self.reference_fixture()
        execution = server.reference_execution()
        server.write_json(
            self.state / "primary-launch.json", {"nonce": "actual-launch"}
        )
        server.write_json(
            self.state / "primary-receiver.json",
            {
                "ready": True,
                "nonce": "actual-launch",
                "pid": os.getpid(),
                "canonical_database": config["database"],
                **execution,
            },
        )
        observation = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "/different",
            "MemoryMax": str(140 * server.GIB),
        }
        with patch.object(server, "group_populated", return_value=True):
            self.assertFalse(server.primary_ready(self.state, config, observation))

    def test_missing_cpuset_uses_verified_role_affinity(self) -> None:
        self.reference_fixture()
        group = self.root / "cgroups/profile"
        group.mkdir(parents=True)
        (group / "memory.max").write_text(str(160 * server.GIB))
        with (
            patch.object(server, "physical_cpus", return_value=list(range(16))),
            patch.object(server, "group_for_slice", return_value=group),
            patch.object(
                server,
                "effective_limits",
                return_value=(160 * server.GIB, 16),
            ),
            patch.object(
                server, "host_memory", return_value=(188 * server.GIB, 20 * server.GIB)
            ),
            patch.object(server, "systemctl"),
        ):
            server.ensure_execution_placement(self.state, server.reference_resources())
        observation = server.read_json(self.state / "execution-placement.json")
        self.assertEqual(observation["cpu_placement"], "process-affinity")
        self.assertEqual(observation["physical_cpus"], list(range(16)))

    def test_role_affinity_is_inherited_and_reads_actual_thread_escape(self) -> None:
        execution: dict[str, object] = {"cpu_threads": 2}
        allowed = sorted(os.sched_getaffinity(0))
        cpus = server.physical_cpus(2)
        program = (
            "import os, sys, threading\n"
            "ready = threading.Event(); escape = threading.Event(); changed = threading.Event(); stop = threading.Event()\n"
            "def worker():\n"
            " ready.set(); escape.wait(); os.sched_setaffinity(0, "
            + repr(allowed)
            + "); changed.set(); stop.wait()\n"
            "thread = threading.Thread(target=worker); thread.start(); ready.wait()\n"
            "print('ready', flush=True); sys.stdin.readline(); escape.set(); changed.wait()\n"
            "print('escaped', flush=True); sys.stdin.readline(); stop.set(); thread.join()\n"
        )
        command = server.role_command(
            {"execution": execution}, [sys.executable, "-c", program]
        )
        with subprocess.Popen(
            command, stdin=subprocess.PIPE, stdout=subprocess.PIPE, text=True
        ) as child:
            if child.stdin is None or child.stdout is None:
                raise AssertionError("child diagnostic pipes absent")
            try:
                self.assertEqual(child.stdout.readline().strip(), "ready")
                self.assertEqual(os.sched_getaffinity(child.pid), set(cpus))
                self.assertTrue(server.role_affinity_ready(child.pid, execution))
                child.stdin.write("escape\n")
                child.stdin.flush()
                self.assertEqual(child.stdout.readline().strip(), "escaped")
                self.assertEqual(os.sched_getaffinity(child.pid), set(cpus))
                self.assertFalse(server.role_affinity_ready(child.pid, execution))
                child.stdin.write("stop\n")
                child.stdin.flush()
                self.assertEqual(child.wait(timeout=5), 0)
            finally:
                if child.poll() is None:
                    child.kill()
                    child.wait(timeout=5)

    def test_profile_worker_uses_one_shared_parent_and_finite_process_cap(self) -> None:
        allocation = server.reference_resources()
        command = server.worker_scope_command(
            self.state, 0, allocation, ["/built/pse-worker"]
        )
        self.assertIn(f"--slice={server.execution_slice(self.state)}", command)
        self.assertIn(f"--property=MemoryMax={140 * server.GIB}", command)
        self.assertIn("--property=TasksMax=2048", command)
        environment = server.worker_environment(self.state, 0, allocation)
        self.assertEqual(
            environment["PSE_NATIVE_WORKER_MEMORY_BYTES"], str(140 * server.GIB)
        )

    def quiesced_fixture(self) -> dict[str, object]:
        config = self.initialized()
        config["admission"] = "quiesced"
        config["accepting_writes"] = False
        server.write_json(self.state / "config.json", config)
        return config

    def reconfigure(self, *options: str) -> int:
        args = server.parser().parse_args(
            ["reconfigure", "--state", str(self.state), *options]
        )
        with (
            patch.object(server, "active", return_value=False),
            patch("builtins.print"),
        ):
            return server.dispatch(args)

    def test_reconfigure_preserves_identity_credentials_and_database(self) -> None:
        before = self.quiesced_fixture()
        credentials = (self.state / "credentials.json").read_bytes()
        fixture = (self.state / "database/fixture").read_bytes()
        units = [server.worker_unit(self.state, slot) for slot in range(2)]
        with (
            patch.object(server, "start") as started,
            patch.object(server, "stop") as stopped,
            patch.object(server, "install") as installed,
        ):
            self.assertEqual(
                self.reconfigure(
                    "--memory-mib",
                    "24576",
                    "--server-memory-mib",
                    "8192",
                    "--native-worker-memory-mib",
                    "8192",
                ),
                0,
            )
        started.assert_not_called()
        stopped.assert_not_called()
        installed.assert_not_called()
        after = server.config_for(self.state)
        self.assertEqual(
            after["resources"],
            server.resources(24 * server.GIB, 8 * server.GIB, 2, 8 * server.GIB),
        )
        before.pop("resources")
        after.pop("resources")
        self.assertEqual(after, before)
        self.assertEqual((self.state / "credentials.json").read_bytes(), credentials)
        self.assertEqual((self.state / "database/fixture").read_bytes(), fixture)
        self.assertEqual(
            units, [server.worker_unit(self.state, slot) for slot in range(2)]
        )

    def test_reconfigure_freezes_new_receiver_after_verified_offline_drain(
        self,
    ) -> None:
        before = self.quiesced_fixture()
        worker = self.root / "replacement-worker"
        worker.write_bytes(b"#!/bin/sh\nexit 0\n")
        worker.chmod(0o700)
        events = []
        publish = server.publish_generation

        def freeze(state: Path, selected: Path) -> dict[str, str]:
            self.assertEqual(events, ["drained"])
            self.assertEqual(server.config_for(state), before)
            self.assertEqual(selected, worker)
            events.append("published")
            return publish(state, selected)

        with (
            patch.object(
                server,
                "workers_drained",
                side_effect=lambda *_args: events.append("drained"),
            ),
            patch.object(server, "publish_generation", side_effect=freeze),
        ):
            self.reconfigure(
                "--execution-profile", "wide", "--worker-executable", str(worker)
            )
        self.assertEqual(events, ["drained", "published"])
        selected = server.config_for(self.state)
        receiver = server.recorded_primary(selected)
        frozen = Path(receiver["worker_executable"])
        self.assertIn(self.state / ".generations", frozen.parents)
        self.assertNotEqual(frozen, worker)
        self.assertEqual(frozen.read_bytes(), worker.read_bytes())
        worker.write_bytes(b"replacement checkout bytes")
        self.assertEqual(server.checked_primary(selected), receiver)
        self.assertEqual(frozen.read_bytes(), b"#!/bin/sh\nexit 0\n")
        self.assertEqual(selected["resources"], server.execution_resources("wide"))

    def test_reconfigure_drain_refusal_does_not_publish_receiver_generation(
        self,
    ) -> None:
        self.quiesced_fixture()
        worker = self.root / "replacement-worker"
        worker.write_bytes(b"#!/bin/sh\nexit 0\n")
        worker.chmod(0o700)
        before = (self.state / "config.json").read_bytes()
        with (
            patch.object(
                server,
                "workers_drained",
                side_effect=server.SupervisorError("not drained"),
            ),
            patch.object(server, "publish_generation") as publish,
            self.assertRaisesRegex(server.SupervisorError, "not drained"),
        ):
            self.reconfigure(
                "--execution-profile", "wide", "--worker-executable", str(worker)
            )
        publish.assert_not_called()
        self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_reconfigure_omitted_values_preserve_saved_allocation(self) -> None:
        config = self.quiesced_fixture()
        # Bytes need not be MiB-aligned: omission preserves exact persisted values.
        config["resources"] = server.resources(
            8 * server.GIB + 1, 3 * server.GIB + 1, 3, server.GIB + 1
        )
        server.write_json(self.state / "config.json", config)
        self.reconfigure("--memory-mib", "10240")
        self.assertEqual(
            server.config_for(self.state)["resources"],
            server.resources(10 * server.GIB, 3 * server.GIB + 1, 3, server.GIB + 1),
        )
        before = (self.state / "config.json").read_bytes()
        self.reconfigure()
        self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_reconfigure_requires_existing_owned_state_without_creation(self) -> None:
        with self.assertRaises(OSError):
            self.reconfigure("--memory-mib", "8192")
        self.assertFalse(self.state.exists())
        self.state.mkdir()
        server.write_json(self.state / "config.json", {"owner": "unrelated"})
        before = (self.state / "config.json").read_bytes()
        with self.assertRaises(server.SupervisorError):
            self.reconfigure("--memory-mib", "8192")
        self.assertEqual((self.state / "config.json").read_bytes(), before)
        self.assertFalse((self.state / ".supervisor.lock").exists())

    def test_reconfigure_refuses_nonquiesced_admission_unchanged(self) -> None:
        config = self.quiesced_fixture()
        for admission, accepting in (
            ("open", True),
            ("quiescing", False),
            ("validation_required", False),
            ("quiesced", True),
        ):
            with self.subTest(admission=admission, accepting=accepting):
                config["admission"] = admission
                config["accepting_writes"] = accepting
                server.write_json(self.state / "config.json", config)
                before = (self.state / "config.json").read_bytes()
                with self.assertRaisesRegex(server.SupervisorError, "Quiesce"):
                    self.reconfigure("--memory-mib", "8192")
                self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_reconfigure_refuses_active_or_unverified_server_unchanged(self) -> None:
        self.quiesced_fixture()
        before = (self.state / "config.json").read_bytes()
        for code, output, populated in (
            (0, "ActiveState=active\nControlGroup=/fixture\n", False),
            (0, "ActiveState=activating\nControlGroup=\n", False),
            (0, "ActiveState=inactive\nControlGroup=/fixture\n", True),
            (1, "ActiveState=inactive\nControlGroup=\n", False),
            (0, "ActiveState=inactive\n", False),
        ):
            with (
                self.subTest(code=code, output=output, populated=populated),
                patch.object(
                    server,
                    "systemctl",
                    return_value=subprocess.CompletedProcess([], code, output, ""),
                ),
                patch.object(server, "group_populated", return_value=populated),
                patch.object(server, "workers_drained") as drained,
                self.assertRaisesRegex(server.SupervisorError, "verified stopped"),
            ):
                self.reconfigure("--memory-mib", "8192")
            drained.assert_not_called()
            self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_reconfigure_refuses_undrained_or_unverifiable_workers_unchanged(
        self,
    ) -> None:
        self.quiesced_fixture()
        before = (self.state / "config.json").read_bytes()
        busy = {
            "LoadState": "loaded",
            "ActiveState": "inactive",
            "ControlGroup": "/worker",
        }
        with (
            patch.object(server, "worker_observation", return_value=busy),
            patch.object(server, "group_populated", side_effect=bool),
            self.assertRaisesRegex(server.SupervisorError, "remain active"),
        ):
            self.reconfigure("--memory-mib", "8192")
        self.assertEqual((self.state / "config.json").read_bytes(), before)
        with (
            patch.object(
                server,
                "worker_observation",
                side_effect=server.SupervisorError("unverifiable"),
            ),
            self.assertRaisesRegex(server.SupervisorError, "unverifiable"),
        ):
            self.reconfigure("--memory-mib", "8192")
        self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_reconfigure_refuses_incomplete_module_pool_before_effects(self) -> None:
        config = self.quiesced_fixture()
        before = (self.state / "config.json").read_bytes()
        credentials = (self.state / "credentials.json").read_bytes()
        payload = (self.state / "database/fixture").read_bytes()
        args = server.parser().parse_args(
            ["reconfigure", "--state", str(self.state), "--server-memory-mib", "1024"]
        )
        # This allocation fits the ordinary process envelope, but cannot admit
        # the actual selected four-controller module pool. Use its real owner.
        server.resources(4 * server.GIB, server.GIB, 2, server.GIB)
        with (
            patch.object(server, "systemctl") as manager,
            patch.object(server, "workers_drained") as drained,
            patch.object(server, "publish_generation") as publish,
            patch.object(server, "write_json") as write,
            self.assertRaisesRegex(
                server.SupervisorError, "complete bounded study module pool"
            ),
        ):
            server.reconfigure(self.state, config, args)
        manager.assert_not_called()
        drained.assert_not_called()
        publish.assert_not_called()
        write.assert_not_called()
        self.assertEqual((self.state / "config.json").read_bytes(), before)
        self.assertEqual((self.state / "credentials.json").read_bytes(), credentials)
        self.assertEqual((self.state / "database/fixture").read_bytes(), payload)
        self.assertEqual(server.config_for(self.state), config)

    def test_reconfigure_invalid_or_partial_budget_does_not_mutate(self) -> None:
        self.quiesced_fixture()
        before = (self.state / "config.json").read_bytes()
        for options in (
            ("--server-memory-mib", "4096"),
            ("--memory-mib", "1024"),
            ("--memory-mib", "0"),
            ("--server-memory-mib", "511"),
            ("--native-worker-memory-mib", "63"),
            ("--memory-mib", str(2**63 // server.MIB)),
        ):
            with (
                self.subTest(options=options),
                patch.object(server, "workers_drained") as drained,
                self.assertRaises(server.SupervisorError),
            ):
                self.reconfigure(*options)
            drained.assert_not_called()
            self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_reconfigure_rejects_explicit_worker_count_and_identity_flags(self) -> None:
        self.quiesced_fixture()
        before = (self.state / "config.json").read_bytes()
        for flag, value in (
            ("--native-workers", "2"),
            ("--native-workers", "3"),
            ("--port", "18081"),
            ("--version", "v3.3.0"),
            ("--interpretation", "pse.substrate.v1"),
            ("--interpretation", ""),
            ("--tool-root", str(self.root / "tools")),
        ):
            with (
                self.subTest(flag=flag, value=value),
                self.assertRaisesRegex(server.SupervisorError, "memory options only"),
            ):
                self.reconfigure(flag, value, "--memory-mib", "8192")
            self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_reconfigure_reservation_excludes_owner_without_holding_drain_lock(
        self,
    ) -> None:
        self.quiesced_fixture()
        locked = False
        original_drain = server.workers_drained

        @server.contextlib.contextmanager
        def lock(_state: Path, **_kwargs: object) -> Generator[None, None, None]:
            nonlocal locked
            self.assertFalse(locked)
            locked = True
            try:
                yield
            finally:
                locked = False

        def drain(state: Path, config: dict[str, object]) -> None:
            # Kernel-associated reservation excludes another lifecycle owner,
            # while the metadata lock is free throughout drain IPC.
            self.assertFalse(locked)
            reservation = server.read_json(state / "lifecycle-owner.json")
            self.assertEqual(reservation["pid"], os.getpid())

            def competing_owner() -> None:
                with (
                    self.assertRaisesRegex(
                        server.SupervisorError, "Another live owner"
                    ),
                    server.lifecycle_reservation(state),
                ):
                    self.fail("A second live lifecycle owner was admitted")

            with ThreadPoolExecutor(max_workers=1) as competing:
                competing.submit(competing_owner).result(timeout=5)
            original_drain(state, config)

        with (
            patch.object(server, "state_lock", side_effect=lock),
            patch.object(server, "workers_drained", side_effect=drain),
        ):
            self.reconfigure("--memory-mib", "8192")
        self.assertFalse(locked)
        self.assertFalse((self.state / "lifecycle-owner.json").exists())
        self.assertEqual(
            server.object_mapping(server.config_for(self.state)["resources"])[
                "total_memory_bytes"
            ],
            8 * server.GIB,
        )

    def test_explicit_start_uses_reconfigured_budget_and_reopens_admission(
        self,
    ) -> None:
        self.quiesced_fixture()
        self.reconfigure(
            "--memory-mib",
            "32768",
            "--server-memory-mib",
            "16384",
            "--native-worker-memory-mib",
            "8192",
        )
        with (
            patch.object(server, "active", return_value=False),
            patch.object(
                server,
                "service_readiness",
                side_effect=[
                    readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                    readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                    readiness_fixture(server.Readiness.STARTING),
                    readiness_fixture(server.Readiness.READY),
                ],
            ),
            patch.object(server, "establish_protocol_readiness"),
            patch.object(server, "service_allocation", return_value=self.allocation),
            patch.dict(os.environ, {"XDG_CONFIG_HOME": str(self.root / "user-config")}),
            patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0),
            ) as started,
            patch("builtins.print"),
        ):
            self.assertEqual(
                server.dispatch(
                    server.parser().parse_args(["start", "--state", str(self.state)])
                ),
                0,
            )
        unit = self.root / "user-config/systemd/user" / server.unit_name(self.state)
        self.assertIn(f"MemoryMax={16 * server.GIB}", unit.read_text())
        self.assertIn("/.generations/", unit.read_text())
        started.assert_not_called()
        config = server.config_for(self.state)
        self.assertTrue(config["accepting_writes"])
        self.assertEqual(config["admission"], "open")
        environment = server.server_environment(
            config, {"username": "user", "password": "secret"}
        )
        self.assertEqual(
            environment["SURREAL_GRPC_MAX_MESSAGE_SIZE"], str(server.MESSAGE_BYTES)
        )
        self.assertTrue(
            "memory_limit=" + str(8 * server.GIB)
            in server.database_endpoint(self.state)
        )
        self.assertEqual(environment["SURREAL_MEMORY_THRESHOLD"], str(12 * server.GIB))
        allocation = config["resources"]
        if not isinstance(allocation, dict):
            self.fail("recorded resource allocation must be an object")
        self.assertEqual(
            server.worker_environment(self.state, 0, allocation)[
                "PSE_NATIVE_WORKER_MEMORY_BYTES"
            ],
            str(8 * server.GIB),
        )

    def test_private_credentials_and_idempotent_setup(self) -> None:
        config = self.initialized()
        self.assertEqual(
            config["resources"],
            server.resources(4 * server.GIB, 2 * server.GIB, 2, server.GIB),
        )
        self.assertEqual(config["port"], 18080)
        credentials = self.state / "credentials.json"
        before = credentials.read_bytes()
        self.assertEqual(credentials.stat().st_mode & 0o777, 0o600)
        args = server.parser().parse_args(
            [
                "setup",
                "--state",
                str(self.state),
                "--interpretation",
                "pse.substrate.v1",
            ]
        )
        with (
            patch.object(server, "install") as install,
            patch.object(server, "active", return_value=False),
        ):
            status = server.setup(args)
        install.assert_not_called()
        self.assertEqual(credentials.read_bytes(), before)
        self.assertNotIn(json.loads(before)["password"], json.dumps(status))

    def test_nonempty_unowned_state_is_preserved(self) -> None:
        self.state.mkdir()
        sentinel = self.state / "unrelated"
        sentinel.write_text("preserve me")
        with self.assertRaises(server.SupervisorError):
            server.checked_directory(self.state, empty=True)
        self.assertEqual(sentinel.read_text(), "preserve me")

    def test_symlinked_state_is_refused(self) -> None:
        target = self.root / "unrelated"
        target.mkdir()
        self.state.symlink_to(target, target_is_directory=True)
        with self.assertRaises(server.SupervisorError):
            server.checked_directory(self.state)

    def test_lexical_normalization_cannot_hide_a_symlinked_parent(self) -> None:
        target = self.root / "unrelated"
        target.mkdir()
        link = self.root / "linked"
        link.symlink_to(target, target_is_directory=True)
        alias = link / ".." / "state"
        with self.assertRaises(server.SupervisorError):
            server.checked_directory(alias)
        self.assertFalse(self.state.exists())

    def test_server_install_refuses_unselected_version_before_build(self) -> None:
        with patch.object(server.surreal_source, "install") as build:
            for version in ("v3.3.0", "v3.3.5", "latest", "../surreal"):
                with self.assertRaises(server.SupervisorError):
                    server.install(version, self.root / "tools")
            build.assert_not_called()

    def test_worker_scope_gets_independent_native_owner_and_launch_guard(self) -> None:
        allocation: dict[str, object] = {"native_worker_memory_bytes": server.GIB}
        with patch.dict(
            "os.environ",
            {
                "PSE_NATIVE_OPERATION": "/parent/record",
                "PSE_NATIVE_HANDOFF": "/stale/record",
            },
        ):
            environment = server.worker_environment(
                self.state, 0, allocation, self.root / "pending.json"
            )
        self.assertNotIn("PSE_NATIVE_OPERATION", environment)
        self.assertEqual(
            environment["PSE_NATIVE_HANDOFF"], str(self.root / "pending.json")
        )
        command = server.worker_scope_command(
            self.state, 0, allocation, ["/worker", "--jobs", "1"]
        )
        self.assertIn(
            str(Path(server.__file__).resolve().with_name("native_operation.py")),
            command,
        )
        self.assertIn("solver,klu,isolation,uno,petsc", command)
        self.assertEqual(command[-4:], ["--", "/worker", "--jobs", "1"])

    def test_listener_health_does_not_establish_authenticated_websocket_readiness(
        self,
    ) -> None:
        config = self.initialized()
        with patch.object(server, "listener_ready", return_value=True):
            self.assertFalse(server.ready(self.state, config))

    def test_module_artifact_absence_or_tamper_closes_ordinary_admission(self) -> None:
        config = self.initialized()
        absent = dict(config)
        absent.pop("study_module")
        server.write_json(self.state / "config.json", absent)
        with self.assertRaisesRegex(server.SupervisorError, "absent or mismatched"):
            server.config_for(self.state)
        server.write_json(self.state / "config.json", config)
        artifact = Path(str(server.object_mapping(config["study_module"])["artifact"]))
        artifact.chmod(0o600)
        artifact.write_bytes(b"unadmitted replacement")
        artifact.chmod(0o400)
        with self.assertRaisesRegex(server.SupervisorError, "absent or mismatched"):
            server.config_for(self.state)

    def test_module_environment_uses_only_private_artifact_and_finite_server_budget(
        self,
    ) -> None:
        config = self.initialized()
        with patch.dict(
            os.environ,
            {
                "SURREAL_CAPS_ALLOW_EXPERIMENTAL": "*",
                "SURREAL_BUCKET_FOLDER_ALLOWLIST": "/",
                "SURREAL_SURREALISM_MAX_TOTAL_CONTROLLERS": "999999",
            },
        ):
            environment = server.server_environment(
                config, {"username": "user", "password": "secret"}
            )
        self.assertEqual(
            environment["SURREAL_CAPS_ALLOW_EXPERIMENTAL"], "files,surrealism"
        )
        artifact = Path(str(server.study_module(config)["artifact"]))
        self.assertEqual(
            environment["SURREAL_BUCKET_FOLDER_ALLOWLIST"], str(artifact.parent)
        )
        self.assertEqual(
            environment["SURREAL_SURREALISM_MAX_MEMORY"], str(64 * server.MIB)
        )
        self.assertEqual(environment["SURREAL_SURREALISM_MAX_TOTAL_CONTROLLERS"], "4")
        self.assertEqual(environment["SURREAL_SURREALISM_CACHE_SIZE"], "1")
        self.assertEqual(environment["SURREAL_SURREALISM_MAX_FS_BYTES"], "0")
        self.assertEqual(environment["SURREAL_SURREALISM_MAX_KV_ENTRIES"], "0")
        self.assertEqual(environment["SURREAL_SURREALISM_MAX_KV_VALUE_BYTES"], "0")
        with self.assertRaisesRegex(
            server.SupervisorError, "complete bounded study module pool"
        ):
            server.module_runtime_allocation({"server_memory_bytes": 512 * server.MIB})

    def test_qualification_module_selection_requires_offline_drain_and_restores_private_allowlist(
        self,
    ) -> None:
        config = self.initialized()
        config.update(accepting_writes=False, admission="quiesced")
        server.write_json(self.state / "config.json", config)
        qualification = fixture_module(
            self.root / "modules",
            str(server.object_mapping(config["server"])["binary_sha256"]),
            qualification=True,
        )
        record = Path(str(qualification["artifact"])).parent / "module.json"
        with (
            patch.object(server, "private_offline_state") as offline,
            patch.object(server, "all_contexts_drained") as drained,
            patch.object(server, "public_status", return_value={}),
            patch.object(server, "active", return_value=False),
        ):
            server.select_module_qualification(self.state, record)
            selected = server.config_for(self.state)
            self.assertEqual(selected["study_module_qualification"], qualification)
            environment = server.server_environment(
                selected, {"username": "user", "password": "secret"}
            )
            parents = environment["SURREAL_BUCKET_FOLDER_ALLOWLIST"].split(os.pathsep)
            self.assertEqual(
                parents,
                [
                    str(Path(str(server.study_module(config)["artifact"])).parent),
                    str(record.parent),
                ],
            )
            offline.assert_called_once()
            drained.assert_called_once()
            with (
                patch.object(server, "active", return_value=True),
                self.assertRaisesRegex(server.SupervisorError, "stopped closed"),
            ):
                server.select_module_qualification(self.state, None)
            self.assertIn("study_module_qualification", server.config_for(self.state))
            server.select_module_qualification(self.state, None)
            selected = server.config_for(self.state)
            self.assertNotIn("study_module_qualification", selected)
            self.assertEqual(
                server.server_environment(
                    selected, {"username": "user", "password": "secret"}
                )["SURREAL_BUCKET_FOLDER_ALLOWLIST"],
                parents[0],
            )

    def test_long_hold_selection_is_explicit_and_does_not_relax_production_limits(
        self,
    ) -> None:
        config = self.initialized()
        config.update(accepting_writes=False, admission="quiesced")
        server.write_json(self.state / "config.json", config)
        builder = str(server.object_mapping(config["server"])["binary_sha256"])
        receipt = fixture_module(
            self.root / "modules", builder, qualification=True, long_hold=True
        )
        record = Path(str(receipt["artifact"])).parent / "module.json"
        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "all_contexts_drained") as drained,
            patch.object(server, "public_status", return_value={}),
            patch.object(server, "active", return_value=False),
        ):
            with self.assertRaisesRegex(server.SupervisorError, "explicit long-hold"):
                server.select_module_qualification(self.state, record)
            self.assertNotIn(
                "study_module_qualification", server.config_for(self.state)
            )
            server.select_module_qualification(self.state, record, long_hold=True)
            selected = server.config_for(self.state)
            environment = server.server_environment(
                selected, {"username": "user", "password": "secret"}
            )
            self.assertEqual(
                environment["SURREAL_SURREALISM_MAX_EXECUTION_TIME"], "6000"
            )
            self.assertEqual(
                server.object_mapping(server.study_module(selected)["limits"])[
                    "max_execution_time_ms"
                ],
                1000,
            )
            self.assertEqual(
                environment["SURREAL_SURREALISM_MAX_TOTAL_CONTROLLERS"], "4"
            )
            self.assertEqual(environment["SURREAL_SURREALISM_CACHE_SIZE"], "1")
            self.assertEqual(
                environment["SURREAL_SURREALISM_MAX_MEMORY"], str(64 * server.MIB)
            )
            with self.assertRaisesRegex(
                server.SupervisorError, "explicit qualification receipt"
            ):
                server.select_module_qualification(self.state, None, long_hold=True)
            server.select_module_qualification(self.state, None)
            restored = server.config_for(self.state)
            self.assertEqual(
                server.server_environment(
                    restored, {"username": "user", "password": "secret"}
                )["SURREAL_SURREALISM_MAX_EXECUTION_TIME"],
                "1000",
            )
            short = fixture_module(self.root / "modules", builder, qualification=True)
            short_record = Path(str(short["artifact"])).parent / "module.json"
            with self.assertRaisesRegex(server.SupervisorError, "explicit long-hold"):
                server.select_module_qualification(
                    self.state, short_record, long_hold=True
                )
            self.assertEqual(drained.call_count, 4)

    def test_long_hold_cli_is_restricted_to_explicit_qualification_selection(
        self,
    ) -> None:
        arguments = server.parser().parse_args(["status", "--long-hold"])
        with self.assertRaisesRegex(
            server.SupervisorError, "only available with module-qualification"
        ):
            server.dispatch(arguments)

    def test_grpc_readiness_proof_binds_invocation_generation_and_credentials(
        self,
    ) -> None:
        config = self.initialized()
        observed = subprocess.CompletedProcess([], 0, "actual-invocation\n", "")
        with patch.object(server, "systemctl", return_value=observed):
            proof = {
                "schema": "native-grpc-readiness-v1",
                "datastore": server.database_endpoint(self.state),
                "instance_id": config["instance_id"],
                "invocation": "actual-invocation",
                "binary_sha256": server.object_mapping(config["server"])[
                    "binary_sha256"
                ],
                "study_module_sha256": server.study_module(config)["sha256"],
                "credentials_sha256": server.file_digest(
                    self.state / "credentials.json"
                ),
            }
            server.write_json(self.state / "protocol-readiness.json", proof)
            self.assertTrue(server.protocol_ready(self.state, config))
            for key in (
                "instance_id",
                "invocation",
                "binary_sha256",
                "credentials_sha256",
                "study_module_sha256",
            ):
                with self.subTest(binding=key):
                    server.write_json(
                        self.state / "protocol-readiness.json",
                        {**proof, key: "different-admission"},
                    )
                    self.assertFalse(server.protocol_ready(self.state, config))
                    with (
                        patch.object(server, "active", return_value=True),
                        patch.object(server, "owns_listener", return_value=True),
                    ):
                        self.assertFalse(
                            server.public_status(self.state, config)[
                                "authenticated_grpc_ready"
                            ]
                        )

    def test_public_websocket_readiness_consumes_one_selected_observation(self) -> None:
        config = self.initialized()
        for phase in (
            server.Readiness.READY,
            server.Readiness.STARTING,
            server.Readiness.STOPPING,
            server.Readiness.MAINTENANCE,
            server.Readiness.UNAVAILABLE,
            server.Readiness.MISMATCH,
        ):
            selected = readiness_fixture(phase)
            with (
                self.subTest(phase=phase),
                patch.object(
                    server, "service_readiness", return_value=selected
                ) as observe,
                patch.object(server, "owns_listener") as listener,
                patch.object(server, "protocol_ready") as protocol,
            ):
                status = server.public_status(self.state, config)
                self.assertEqual(
                    status["active"], selected.manager.get("ActiveState") == "active"
                )
                self.assertEqual(status["listener_owned"], selected.listener_owned)
                self.assertEqual(
                    status["authenticated_grpc_ready"], selected.protocol_ready
                )
                lifecycle = server.object_mapping(status["lifecycle"])
                self.assertEqual(lifecycle["phase"], phase.value)
                self.assertEqual(lifecycle["cause"], selected.cause)
                observe.assert_called_once_with(self.state, config)
                listener.assert_not_called()
                protocol.assert_not_called()

    def test_owned_environment_does_not_inherit_authentication_bypass(self) -> None:
        config = self.initialized()
        with patch.dict(
            "os.environ",
            {
                "SURREAL_UNAUTHENTICATED": "true",
                "SURREAL_ROCKSDB_BLOCK_CACHE_SIZE": "99999999999",
            },
        ):
            environment = server.server_environment(
                config, {"username": "user", "password": "secret"}
            )
        self.assertNotIn("SURREAL_UNAUTHENTICATED", environment)
        self.assertEqual(environment["SURREAL_GRPC_MAX_MESSAGE_SIZE"], "4194304")
        self.assertNotIn("SURREAL_ROCKSDB_BLOCK_CACHE_SIZE", environment)
        self.assertEqual(environment["SURREAL_GRPC_MAX_ATTACHED_SESSIONS"], "256")
        self.assertEqual(environment["SURREAL_RUNTIME_WORKER_THREADS"], "4")

    def test_resource_override_outside_recorded_budget_is_refused(self) -> None:
        config = self.initialized()
        allocation = config["resources"]
        self.assertIsInstance(allocation, dict)
        if not isinstance(allocation, dict):
            self.fail("recorded resource allocation must be an object")
        allocation["surrealmx_memory_limit_bytes"] = 100 * server.GIB
        server.write_json(self.state / "config.json", config)
        with self.assertRaises(server.SupervisorError):
            server.config_for(self.state)

    def test_bounded_log_rotation_and_redaction(self) -> None:
        path = self.root / "server.log"
        log = server.BoundedLog(path, 32, 2, "secret")
        for _ in range(30):
            log.append(b"error contains secret\n")
        paths = list(self.root.glob("server.log*"))
        self.assertEqual(len(paths), 3)
        for item in paths:
            self.assertLessEqual(item.stat().st_size, 32)
            self.assertEqual(item.stat().st_mode & 0o777, 0o600)
            self.assertNotIn(b"secret", item.read_bytes())

    def application_inventory_fixture(self, *_args: object) -> dict[str, object]:
        return {
            "version": 1,
            "issuing_store": "00000000-0000-4000-8000-000000000001",
            "tables": {table: [] for table in server.APPLICATION_BACKUP_TABLES},
        }

    def backup_fixture(self) -> Path:
        config = self.initialized()
        config.update(
            interpretation=server.SUBSTRATE_INTERPRETATION,
            schema_interpretation=server.SUBSTRATE_INTERPRETATION,
        )
        server.write_json(self.state / "config.json", config)
        destination = self.root / "backup"
        with (
            patch.object(server, "active", return_value=False),
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
        ):
            server.backup(self.state, config, destination)
        return destination

    def empty_catalog(self, scope: str) -> dict[str, object]:
        fields = {
            "ROOT": (
                "accesses",
                "defaults",
                "namespaces",
                "nodes",
                "system",
                "users",
                "config",
            ),
            "NS": ("accesses", "databases", "users"),
            "DB": (
                "accesses",
                "apis",
                "analyzers",
                "buckets",
                "functions",
                "modules",
                "models",
                "params",
                "tables",
                "users",
                "configs",
                "sequences",
            ),
        }
        return {field: {} for field in fields[scope]}

    @contextlib.contextmanager
    def native_maintenance(
        self, *, retained_science: dict[str, list[dict[str, object]]] | None = None
    ) -> Generator[list[tuple[str, str, str]], None, None]:
        """Model native acknowledgments and science preserved by native export/import."""
        calls: list[tuple[str, str, str]] = []
        inventory: dict[Path, dict[str, set[str]]] = {}
        science: dict[tuple[Path, str, str], dict[str, list[dict[str, object]]]] = {}
        exported_science: dict[str, list[dict[str, object]]] = {}
        domain_owners: dict[tuple[Path, str, str], dict[str, object]] = {}
        exported_domain_owner: dict[str, object] | None = None

        def known_databases(state: Path) -> dict[str, set[str]]:
            if state not in inventory:
                current = server.config_for(state)
                pending = server.object_mapping(
                    current.get("derived_rebuild_pending", {})
                )
                # Restored bytes initially contain only the recorded source. A fresh
                # target identity in config does not mean import has created it.
                inventory[state] = {
                    str(pending.get("namespace", current["namespace"])): {
                        str(pending.get("database", current["database"]))
                    }
                }
                science[
                    (
                        state,
                        str(pending.get("namespace", current["namespace"])),
                        str(pending.get("database", current["database"])),
                    )
                ] = {
                    table: [dict(row) for row in rows]
                    for table, rows in (retained_science or {}).items()
                }
            return inventory[state]

        def query(
            state: Path,
            config: dict[str, object],
            _credentials: dict[str, object],
            body: str,
        ) -> dict[str, object]:
            calls.append((str(config["namespace"]), str(config["database"]), body))
            self.assertFalse(server.config_for(state)["accepting_writes"])
            known_databases(state)
            selected_science = science.get(
                (state, str(config["namespace"]), str(config["database"])), {}
            )
            for table in selected_science:
                if body == "LET $value=SELECT * FROM " + table + ";":  # noqa: S608
                    # Fixture-owned table names match the validated transport query.
                    return {"value": [dict(row) for row in selected_science[table]]}
            if (
                body
                == "LET $value=(SELECT * FROM ONLY canonical_interpretations:current).issuing_store;"
            ):
                return {"value": self.application_inventory_fixture()["issuing_store"]}
            if body.startswith("LET $rows=SELECT * FROM "):
                return {"value": []}
            prefix = "LET $domain=fn::pse_maintenance_v1::"
            if body.startswith(prefix):
                function, encoded = body[len(prefix) :].split("(", 1)
                suffix = "); LET $value=IF $domain=NONE {NULL} ELSE {$domain};"
                self.assertTrue(encoded.endswith(suffix))
                encoded = encoded.removesuffix(suffix)
                if function == "claim" and encoded.endswith(",NONE"):
                    encoded = encoded.removesuffix(",NONE") + ",null"
                arguments = json.loads("[" + encoded + "]")
                self.assertIs(type(arguments[0]), int)
                key = (state, str(config["namespace"]), str(config["database"]))
                previous = domain_owners.get(key)
                if function == "current":
                    self.assertEqual(len(arguments), 1)
                    return {"value": dict(previous) if previous else None}
                if function == "claim":
                    self.assertEqual(len(arguments), 7)
                    self.assertEqual(arguments[6], previous)
                    current = dict(
                        zip(
                            ("operation", "generation", "phase"),
                            arguments[1:4],
                            strict=True,
                        )
                    )
                    domain_owners[key] = current
                    return {"value": dict(current)}
                self.assertIsNotNone(previous)
                assert previous is not None
                self.assertEqual(
                    arguments[1:4],
                    [previous["operation"], previous["generation"], previous["phase"]],
                )
                if function == "page":
                    self.assertEqual(len(arguments), 6)
                    self.assertIn(arguments[4], {"rotate", "rehome"})
                    # This fixture contains no creation authorities or file-instance
                    # placements to mutate; science and future borrowers are retained.
                    return {
                        "value": {"count": 0, "after": arguments[5], "complete": True}
                    }
                self.assertEqual(function, "rehome_complete")
                self.assertEqual(len(arguments), 4)
                return {"value": True}
            if body.startswith("REMOVE DATABASE "):
                self.assertIsNone(config["database"])
                databases = known_databases(state)[str(config["namespace"])]
                matching = [
                    name
                    for name in databases
                    if body.startswith(
                        "REMOVE DATABASE " + server.sql_identifier(name) + ";"
                    )
                ]
                self.assertEqual(len(matching), 1)
                databases.remove(matching[0])
            return {"value": True}

        def catalog(
            state: Path,
            config: dict[str, object],
            _credentials: dict[str, object],
            scope: str,
        ) -> dict[str, object]:
            calls.append(
                (
                    str(config.get("namespace")),
                    str(config.get("database")),
                    "INFO FOR " + scope + ";",
                )
            )
            current = server.config_for(state)
            self.assertFalse(current["accepting_writes"])
            databases = known_databases(state)
            response = self.empty_catalog(scope)
            if scope == "ROOT":
                response["namespaces"] = {name: {} for name in databases}
                response["users"] = {
                    str(server.read_json(state / "credentials.json")["username"]): {}
                }
            elif scope == "NS":
                response["databases"] = {
                    name: {} for name in databases.get(str(config["namespace"]), set())
                }
            return response

        def copy(
            arguments: list[str], **options: object
        ) -> subprocess.CompletedProcess[str]:
            nonlocal exported_science, exported_domain_owner
            if arguments[1] == "--verify-payload-inventory":
                entries = [
                    json.loads(line)
                    for line in Path(arguments[2]).read_text().splitlines()
                ]
                for entry in entries:
                    self.assertEqual(
                        set(entry),
                        {
                            "instance",
                            "digest",
                            "bytes",
                            "encoding",
                            "batch_count",
                            "schema",
                            "interpretation",
                            "path",
                        },
                    )
                    self.assertEqual(
                        Path(entry["path"]).stat().st_size, int(entry["bytes"])
                    )
                return subprocess.CompletedProcess(
                    arguments,
                    0,
                    json.dumps({"verified_payload_files": len(entries)}),
                    "",
                )
            self.assertIn(arguments[1], {"export", "import"})
            calls.append((arguments[6], arguments[8], arguments[1]))
            if arguments[1] == "export":
                state = Path(str(options["cwd"]))
                known_databases(state)
                source_owner = domain_owners.get((state, arguments[6], arguments[8]))
                exported_domain_owner = dict(source_owner) if source_owner else None
                exported_science = {
                    table: [dict(row) for row in rows]
                    for table, rows in science.get(
                        (state, arguments[6], arguments[8]), {}
                    ).items()
                }
                Path(arguments[-1]).write_text(
                    "current authored records and retained admitted analyses"
                )
            else:
                self.assertTrue(Path(arguments[-1]).is_file())
                state = Path(str(options["cwd"]))
                known_databases(state).setdefault(arguments[6], set()).add(arguments[8])
                science[(state, arguments[6], arguments[8])] = {
                    table: [dict(row) for row in rows]
                    for table, rows in exported_science.items()
                }
                if exported_domain_owner is not None:
                    domain_owners[(state, arguments[6], arguments[8])] = dict(
                        exported_domain_owner
                    )
            return subprocess.CompletedProcess(arguments, 0, "", "")

        with (
            patch.object(server, "active", return_value=False),
            patch.object(server, "maintenance_query", side_effect=query),
            patch.object(server, "maintenance_catalog", side_effect=catalog),
            patch.object(
                server, "maintenance_endpoint", return_value="http://127.0.0.1:18080"
            ),
            patch.object(server.subprocess, "run", side_effect=copy),
        ):
            yield calls

    def test_fixed_worker_slots_survive_launcher_identity_and_enforce_capacity(
        self,
    ) -> None:
        self.initialized()
        child = MagicMock()
        child.poll.return_value = None
        child.wait.return_value = 17
        idle = {"LoadState": "not-found", "ActiveState": "inactive", "ControlGroup": ""}
        busy = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "",
            "MemoryMax": str(server.GIB),
        }

        def observe(_state: Path, slot: int, **_kwargs: object) -> dict[str, str]:
            if slot == 0:
                return busy
            return idle if spawn.call_count == 0 else busy

        with (
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.READY),
            ),
            patch.object(server, "worker_observation", side_effect=observe),
            patch.object(server.subprocess, "Popen", return_value=child) as spawn,
        ):
            result = server.worker(self.state, ["/worker", "--jobs", "1"])
        self.assertEqual(result, 17)
        command = spawn.call_args.args[0]
        self.assertIn(f"--unit={server.worker_unit(self.state, 1)}", command)
        self.assertIn(f"--property=MemoryMax={server.GIB}", command)
        self.assertIn("--property=MemorySwapMax=0", command)
        self.assertEqual(command[-4:], ["--", "/worker", "--jobs", "1"])
        first_name = server.worker_unit(self.state, 1)
        config = server.config_for(self.state)
        config["instance_id"] = "a replacement supervisor identity"
        server.write_json(self.state / "config.json", config)
        self.assertEqual(server.worker_unit(self.state, 1), first_name)
        with (
            patch.object(server, "worker_observation", return_value=busy),
            patch.object(server.subprocess, "Popen") as refused,
            self.assertRaisesRegex(server.SupervisorError, "occupied"),
        ):
            server.worker(self.state, ["/worker"])
        refused.assert_not_called()

    def test_worker_wait_releases_lifecycle_lock_and_uses_recorded_budget(self) -> None:
        config = self.initialized()
        locked = False

        @server.contextlib.contextmanager
        def lock(_state: Path, **_kwargs: object) -> Generator[None, None, None]:
            nonlocal locked
            self.assertFalse(locked)
            locked = True
            try:
                yield
            finally:
                locked = False

        child = MagicMock()
        child.poll.return_value = None

        def wait() -> int:
            self.assertFalse(locked, "quiesce must be able to acquire lifecycle lock")
            return 0

        child.wait.side_effect = wait
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        active = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "",
            "MemoryMax": str(server.GIB),
        }
        with (
            patch.object(server, "state_lock", side_effect=lock),
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.READY),
            ),
            patch.object(
                server,
                "worker_observation",
                side_effect=[inactive, inactive, inactive, active],
            ),
            patch.object(server.subprocess, "Popen", return_value=child) as spawn,
            patch.dict("os.environ", {"PSE_MEMORY_MAX": "off"}),
        ):
            self.assertEqual(server.worker(self.state, ["/worker"]), 0)
        environment = spawn.call_args.kwargs["env"]
        self.assertNotIn("PSE_MEMORY_MAX", environment)
        self.assertEqual(environment["PSE_NATIVE_WORKER_MEMORY_BYTES"], str(server.GIB))
        self.assertEqual(environment["PSE_SURREAL_STATE"], str(self.state))
        config["accepting_writes"] = False
        config["admission"] = "quiescing"
        server.write_json(self.state / "config.json", config)
        with (
            patch.object(server.subprocess, "Popen") as spawn,
            self.assertRaisesRegex(server.SupervisorError, "closed"),
        ):
            server.worker(self.state, ["/worker"])
        spawn.assert_not_called()

    def test_worker_registration_observation_failure_joins_launcher_and_keeps_uncertain_owner(
        self,
    ) -> None:
        self.initialized()
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        unit = server.worker_unit(self.state, 0)
        binding = {"group": "/owned", "invocation": "a" * 32, "inode": 17}
        handoff = self.root / "worker-handoff.json"
        server.write_json(
            handoff,
            {
                "handoff": unit,
                "scope": {"unit": unit, "group": "/owned", "invocation": "a" * 32},
            },
        )
        for error_kind in ("deadline", "timeout"):
            for completion in (
                "complete",
                "unknown",
                "replacement-invocation",
                "replacement-group",
                "replacement-inode",
                "predecessor",
                "unbound-handoff",
            ):
                with self.subTest(error=error_kind, cleanup=completion):
                    server.write_json(
                        handoff,
                        {
                            "handoff": unit,
                            "scope": None
                            if completion == "unbound-handoff"
                            else {
                                "unit": unit,
                                "group": "/owned",
                                "invocation": "a" * 32,
                            },
                        },
                    )
                    child = MagicMock()
                    child.poll.return_value = None
                    child.wait.return_value = -9
                    joined = []
                    child.wait.side_effect = lambda joined=joined: (
                        joined.append(True) or -9
                    )

                    def observe(
                        _state: Path,
                        _slot: int,
                        *,
                        deadline: float | None = None,
                        error_kind: str = error_kind,
                    ) -> dict[str, str]:
                        if deadline is None:
                            return inactive
                        server._STARTUP.deadline = server.time.monotonic() - 1  # noqa: SLF001 -- expire only after actual Popen
                        if error_kind == "deadline":
                            raise server.DeadlineExpiredError(
                                "registration clock expired"
                            )
                        raise subprocess.TimeoutExpired("manager observation", 0)

                    @contextlib.contextmanager
                    def ledger(
                        _directory: Path,
                        *,
                        joined: list[bool] = joined,
                        **_kwargs: object,
                    ) -> Generator[dict[str, object], None, None]:
                        self.assertTrue(
                            joined, "launcher must be joined before manager cleanup"
                        )
                        yield {
                            "owners": {
                                self.allocation.nonce: {"units": {unit: binding}}
                            }
                        }

                    actual = {
                        "LoadState": "loaded",
                        "ActiveState": "active",
                        "ControlGroup": "/owned",
                        "InvocationID": "a" * 32,
                    }
                    if completion == "replacement-invocation":
                        actual["InvocationID"] = "b" * 32
                    if completion == "replacement-group":
                        actual["ControlGroup"] = "/replacement"
                    drained = server.native_operation.DrainObservation(
                        {"unit": unit, **binding},
                        server.native_operation.DrainPhase.COMPLETE
                        if completion == "complete"
                        else server.native_operation.DrainPhase.UNAVAILABLE,
                        "controlled cleanup observation",
                    )
                    with (
                        patch.object(server, "ensure_execution_placement"),
                        patch.object(server, "start"),
                        patch.object(
                            server.native_operation,
                            "prepare_handoff",
                            return_value=handoff,
                        ),
                        patch.object(server, "worker_observation", side_effect=observe),
                        patch.object(server.subprocess, "Popen", return_value=child),
                        patch.object(
                            host,
                            "readonly_snapshot",
                            return_value={
                                "owners": {
                                    self.allocation.nonce: {
                                        "units": {
                                            unit: binding
                                            if completion == "predecessor"
                                            else {}
                                        }
                                    }
                                }
                            },
                        ),
                        patch.object(host, "allocation_metadata", side_effect=ledger),
                        patch.object(
                            host,
                            "group_identity",
                            return_value=18
                            if completion == "replacement-inode"
                            else 17,
                        ),
                        patch.object(
                            server.native_operation,
                            "unit_observation",
                            return_value=actual,
                        ),
                        patch.object(
                            server.native_operation, "drained", return_value=drained
                        ),
                        patch.object(
                            server,
                            "systemctl",
                            return_value=subprocess.CompletedProcess([], 0),
                        ) as control,
                        self.assertRaises(
                            (server.DeadlineExpiredError, subprocess.TimeoutExpired)
                        ),
                    ):
                        server.worker(self.state, ["/fixture-worker"])
                    child.kill.assert_called_once()
                    child.wait.assert_called_once()
                    stops = [
                        call
                        for call in control.call_args_list
                        if call.args[0] == "stop"
                    ]
                    self.assertEqual(
                        len(stops),
                        0
                        if completion.startswith("replacement")
                        or completion in {"predecessor", "unbound-handoff"}
                        else 1,
                    )
                    pending = self.state / "worker-admission-0.json"
                    self.assertEqual(pending.exists(), completion != "complete")
                    if pending.exists():
                        retained = server.read_json(pending)
                        self.assertTrue(retained["unresolved"])
                        self.assertEqual(
                            retained["binding"],
                            None
                            if completion in {"predecessor", "unbound-handoff"}
                            else {"unit": unit, **binding},
                        )
                        retained.update(pid=99999999, start="departed launcher")
                        self.assertTrue(server.reservation_live(retained))
                        with (
                            patch.object(server, "require_borrowers_drained"),
                            patch.object(server, "reconcile_registration"),
                            self.assertRaisesRegex(
                                server.SupervisorError, "reservation remains active"
                            ),
                        ):
                            server.all_contexts_drained(self.state)
                        pending.unlink()

    def test_primary_acknowledged_launch_observation_timeout_retains_uncertain_admission(
        self,
    ) -> None:
        self.reference_fixture()
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        unit = server.primary_unit(self.state)
        binding = {"group": "/owned", "invocation": "a" * 32, "inode": 17}
        server.write_json(
            self.root / "owned-handoff",
            {
                "handoff": unit,
                "scope": {"unit": unit, "group": "/owned", "invocation": "a" * 32},
            },
        )
        for completion in (
            "complete",
            "unknown",
            "replacement",
            "predecessor",
            "unbound-handoff",
        ):
            with self.subTest(cleanup=completion):
                server.write_json(
                    self.root / "owned-handoff",
                    {
                        "handoff": unit,
                        "scope": None
                        if completion == "unbound-handoff"
                        else {"unit": unit, "group": "/owned", "invocation": "a" * 32},
                    },
                )
                observed_count = []

                def observe(
                    _state: Path, *, observed_count: list[bool] = observed_count
                ) -> dict[str, str]:
                    observed_count.append(True)
                    if len(observed_count) == 1:
                        return inactive
                    server._STARTUP.deadline = server.time.monotonic() - 1  # noqa: SLF001 -- acknowledged launch precedes expiry
                    raise subprocess.TimeoutExpired("primary readiness observation", 0)

                @contextlib.contextmanager
                def ledger(
                    _directory: Path, *, deadline: float | None = None
                ) -> Generator[dict[str, object], None, None]:
                    self.assertIsNotNone(deadline)
                    self.assertGreater(deadline, server.time.monotonic())
                    self.assertIsNone(server._STARTUP.deadline)  # noqa: SLF001 -- cleanup clock is separate
                    yield {
                        "owners": {self.allocation.nonce: {"units": {unit: binding}}}
                    }

                actual = {
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "ControlGroup": "/owned",
                    "InvocationID": "a" * 32,
                }
                if completion == "replacement":
                    actual["InvocationID"] = "b" * 32
                drained = server.native_operation.DrainObservation(
                    {"unit": unit, **binding},
                    server.native_operation.DrainPhase.COMPLETE
                    if completion == "complete"
                    else server.native_operation.DrainPhase.UNAVAILABLE,
                    "controlled cleanup observation",
                )
                with (
                    patch.object(server, "ensure_execution_placement"),
                    patch.object(server, "start"),
                    patch.object(
                        server.native_operation,
                        "prepare_handoff",
                        return_value=self.root / "owned-handoff",
                    ),
                    patch.object(server, "primary_observation", side_effect=observe),
                    patch.object(server, "primary_ready", return_value=False),
                    patch.object(server, "settled_worker", return_value=inactive),
                    patch.object(
                        server.subprocess,
                        "run",
                        return_value=subprocess.CompletedProcess([], 0),
                    ) as launched,
                    patch.object(
                        host,
                        "readonly_snapshot",
                        return_value={
                            "owners": {
                                self.allocation.nonce: {
                                    "units": {
                                        unit: binding
                                        if completion == "predecessor"
                                        else {}
                                    }
                                }
                            }
                        },
                    ),
                    patch.object(host, "allocation_metadata", side_effect=ledger),
                    patch.object(host, "group_identity", return_value=17),
                    patch.object(
                        server.native_operation, "unit_observation", return_value=actual
                    ),
                    patch.object(
                        server.native_operation, "drained", return_value=drained
                    ),
                    patch.object(
                        server,
                        "systemctl",
                        return_value=subprocess.CompletedProcess([], 0),
                    ) as control,
                    self.assertRaises(subprocess.TimeoutExpired),
                ):
                    server.ensure_primary(self.state)
                launched.assert_called_once()
                self.assertEqual(len(observed_count), 2)
                stops = [
                    call for call in control.call_args_list if call.args[0] == "stop"
                ]
                self.assertEqual(
                    len(stops),
                    0
                    if completion in {"replacement", "predecessor", "unbound-handoff"}
                    else 1,
                )
                pending = self.state / "primary-admission.json"
                self.assertEqual(pending.exists(), completion != "complete")
                if pending.exists():
                    retained = server.read_json(pending)
                    self.assertTrue(retained["unresolved"])
                    self.assertEqual(
                        retained["binding"],
                        None
                        if completion in {"predecessor", "unbound-handoff"}
                        else {"unit": unit, **binding},
                    )
                    retained.update(pid=99999999, start="departed launcher")
                    self.assertTrue(server.reservation_live(retained))
                    pending.unlink()

    def test_short_worker_reconciles_fresh_completed_owner_and_preserves_status(
        self,
    ) -> None:
        self.initialized()
        unit = server.worker_unit(self.state, 0)
        binding = {"group": "/owned", "invocation": "b" * 32, "inode": 17}
        predecessor = {**binding, "invocation": "a" * 32}
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        handoff = self.root / "fresh-handoff.json"
        server.write_json(
            handoff,
            {
                "handoff": unit,
                "scope": {"unit": unit, "group": "/owned", "invocation": "b" * 32},
            },
        )

        @contextlib.contextmanager
        def ledger(
            _directory: Path, **_kwargs: object
        ) -> Generator[dict[str, object], None, None]:
            yield {"owners": {self.allocation.nonce: {"units": {unit: binding}}}}

        for guarded in (False, True):
            for status in (0, 7, -15):
                with self.subTest(handoff=guarded, status=status):
                    child = MagicMock()
                    child.poll.return_value = status
                    child.wait.return_value = status
                    with (
                        patch.object(server, "start"),
                        patch.object(server, "ensure_execution_placement"),
                        patch.object(
                            server, "worker_observation", return_value=inactive
                        ),
                        patch.object(
                            server.native_operation,
                            "prepare_handoff",
                            return_value=handoff if guarded else None,
                        ),
                        patch.object(
                            host,
                            "readonly_snapshot",
                            return_value={
                                "owners": {
                                    self.allocation.nonce: {
                                        "units": {unit: predecessor}
                                    }
                                }
                            },
                        ),
                        patch.object(host, "allocation_metadata", side_effect=ledger),
                        patch.object(host, "group_identity", return_value=None),
                        patch.object(
                            server.native_operation,
                            "unit_observation",
                            return_value=inactive,
                        ),
                        patch.object(
                            server.native_operation,
                            "drained",
                            return_value=server.native_operation.DrainObservation(
                                binding,
                                server.native_operation.DrainPhase.COMPLETE,
                                "fresh owner drained",
                            ),
                        ),
                        patch.object(server.subprocess, "Popen", return_value=child),
                        patch.object(
                            server,
                            "systemctl",
                            return_value=subprocess.CompletedProcess([], 0),
                        ) as control,
                    ):
                        self.assertEqual(
                            server.worker(self.state, ["/short-worker"]),
                            status if status >= 0 else 128 - status,
                        )
                    child.wait.assert_called_once()
                    child.kill.assert_not_called()
                    self.assertFalse(
                        any(call.args[0] == "stop" for call in control.call_args_list)
                    )
                    self.assertFalse((self.state / "worker-admission-0.json").exists())

    def test_registration_cleanup_refuses_predecessor_and_unbound_fresh_handoff(
        self,
    ) -> None:
        self.initialized()
        binding: dict[str, object] = {
            "group": "/old",
            "invocation": "a" * 32,
            "inode": 17,
        }
        for unit in (
            server.worker_unit(self.state, 0),
            server.primary_unit(self.state),
        ):
            for stale in ("predecessor", "unbound-handoff"):
                with self.subTest(unit=unit, state=stale):
                    handoff = self.root / "new-handoff.json"
                    server.write_json(handoff, {"handoff": unit, "scope": None})

                    @contextlib.contextmanager
                    def ledger(
                        _directory: Path, *, unit: str = unit, **_kwargs: object
                    ) -> Generator[dict[str, object], None, None]:
                        yield {
                            "owners": {
                                self.allocation.nonce: {"units": {unit: binding}}
                            }
                        }

                    with (
                        patch.object(host, "allocation_metadata", side_effect=ledger),
                        patch.object(
                            server.native_operation,
                            "unit_observation",
                            return_value={"LoadState": "not-found"},
                        ) as observe,
                        patch.object(
                            server.native_operation,
                            "drained",
                            return_value=server.native_operation.DrainObservation(
                                binding,
                                server.native_operation.DrainPhase.COMPLETE,
                                "old owner drained",
                            ),
                        ) as drain,
                        patch.object(server, "systemctl") as control,
                        self.assertRaisesRegex(server.SupervisorError, "fresh"),
                    ):
                        server.stop_registered_owner(
                            unit,
                            self.allocation,
                            predecessor=binding if stale == "predecessor" else {},
                            handoff=handoff,
                        )
                    observe.assert_not_called()
                    drain.assert_not_called()
                    control.assert_not_called()

    def test_offline_drain_retries_only_retained_exact_registration_owner(self) -> None:
        self.initialized()
        unit = server.worker_unit(self.state, 0)
        binding = {"group": "/owned", "invocation": "b" * 32, "inode": 17}
        predecessor = {**binding, "invocation": "a" * 32}
        handoff = self.root / "fresh-handoff.json"
        server.write_json(
            handoff,
            {
                "handoff": unit,
                "scope": {"unit": unit, "group": "/owned", "invocation": "b" * 32},
            },
        )
        pending = self.state / "worker-admission-0.json"
        selected = {
            "nonce": "selected",
            "unit": unit,
            "boot": host.boot(),
            "unresolved": True,
            "allocation": str(self.allocation.directory / self.allocation.nonce),
            "handoff": str(handoff),
            "predecessor": predecessor,
        }

        @contextlib.contextmanager
        def ledger(
            _directory: Path, **_kwargs: object
        ) -> Generator[dict[str, object], None, None]:
            yield {"owners": {self.allocation.nonce: {"units": {unit: binding}}}}

        for completion in ("unknown", "replacement", "complete"):
            server.write_json(pending, selected)
            actual = {
                "LoadState": "loaded",
                "ActiveState": "active",
                "ControlGroup": "/owned",
                "InvocationID": "b" * 32,
            }
            if completion == "replacement":
                actual["InvocationID"] = "c" * 32
            drained = server.native_operation.DrainObservation(
                binding,
                server.native_operation.DrainPhase.COMPLETE
                if completion == "complete"
                else server.native_operation.DrainPhase.UNAVAILABLE,
                "retry owner drain",
            )
            with (
                self.subTest(completion=completion),
                patch.object(server, "require_borrowers_drained"),
                patch.object(server, "workers_drained"),
                patch.object(host, "allocation_metadata", side_effect=ledger),
                patch.object(host, "group_identity", return_value=17),
                patch.object(
                    server.native_operation, "unit_observation", return_value=actual
                ),
                patch.object(server.native_operation, "drained", return_value=drained),
                patch.object(
                    server, "systemctl", return_value=subprocess.CompletedProcess([], 0)
                ) as control,
            ):
                if completion == "complete":
                    server.all_contexts_drained(self.state)
                    self.assertFalse(pending.exists())
                else:
                    with self.assertRaises(server.SupervisorError):
                        server.all_contexts_drained(self.state)
                    self.assertEqual(
                        server.read_json(pending),
                        {**selected, "binding": {"unit": unit, **binding}},
                    )
                self.assertEqual(
                    any(call.args[0] == "stop" for call in control.call_args_list),
                    completion != "replacement",
                )

    def test_registration_recovery_held_host_ledger_expires_without_late_drain(
        self,
    ) -> None:
        self.initialized()
        directory = self.allocation.directory
        directory.mkdir(mode=0o700)
        pending = self.state / "worker-admission-0.json"
        selected = {
            "nonce": "selected",
            "unit": server.worker_unit(self.state, 0),
            "boot": host.boot(),
            "unresolved": True,
            "allocation": str(directory / self.allocation.nonce),
            "predecessor": {},
            "handoff": None,
        }
        server.write_json(pending, selected)
        program = "import fcntl,os,sys; f=open(sys.argv[1], 'a'); os.chmod(sys.argv[1], 0o600); fcntl.flock(f, fcntl.LOCK_EX); print('held',flush=True); sys.stdin.readline()"
        child = subprocess.Popen(
            [sys.executable, "-c", program, str(directory / ".lock")],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
        )
        try:
            assert child.stdout is not None
            self.assertEqual(child.stdout.readline().strip(), "held")
            started = server.time.monotonic()
            deadline = started + 0.05
            with (
                patch.object(server._STARTUP, "deadline", deadline, create=True),  # noqa: SLF001 -- exercise the original inherited admission clock
                patch.object(server, "require_borrowers_drained"),
                patch.object(server.native_operation, "unit_observation") as observe,
                patch.object(server.native_operation, "drained") as drain,
                self.assertRaises(host.AdmissionError),
            ):
                server.all_contexts_drained(self.state)
            self.assertLess(server.time.monotonic() - started, 0.5)
            observe.assert_not_called()
            drain.assert_not_called()
            self.assertEqual(server.read_json(pending), selected)
        finally:
            child.communicate("release\n", timeout=5)
            self.assertEqual(child.returncode, 0)
        with self.assertRaises(server.DeadlineExpiredError):
            server.reconcile_registration(self.state, pending, deadline=deadline)
        self.assertEqual(server.read_json(pending), selected)

    def test_registration_recovery_retired_allocation_requires_retained_fresh_binding_and_drain(
        self,
    ) -> None:
        self.initialized()
        unit = server.worker_unit(self.state, 0)
        binding = {"unit": unit, "group": "/owned", "invocation": "b" * 32, "inode": 17}
        pending = self.state / "worker-admission-0.json"
        selected = {
            "nonce": "selected",
            "unit": unit,
            "boot": host.boot(),
            "unresolved": True,
            "allocation": str(self.allocation.directory / self.allocation.nonce),
            "predecessor": {"invocation": "a" * 32},
            "binding": binding,
            "handoff": str(self.root / "already-collected-handoff"),
        }

        @contextlib.contextmanager
        def retired(
            _directory: Path, **_kwargs: object
        ) -> Generator[dict[str, object], None, None]:
            yield {"owners": {}}

        for scenario in (
            "complete",
            "unknown",
            "replacement",
            "replacement-inode",
            "no-evidence",
        ):
            case = (
                {**selected, "binding": None} if scenario == "no-evidence" else selected
            )
            server.write_json(pending, case)
            observed = {"LoadState": "not-found"}
            if scenario == "replacement":
                observed = {
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "InvocationID": "c" * 32,
                    "ControlGroup": "/owned",
                }
            completion = server.native_operation.DrainObservation(
                binding,
                server.native_operation.DrainPhase.UNAVAILABLE
                if scenario == "unknown"
                else server.native_operation.DrainPhase.COMPLETE,
                "retired owner observation",
            )
            with (
                self.subTest(scenario=scenario),
                patch.object(host, "allocation_metadata", side_effect=retired),
                patch.object(
                    host,
                    "group_identity",
                    return_value=18 if scenario == "replacement-inode" else None,
                ),
                patch.object(
                    server.native_operation, "unit_observation", return_value=observed
                ),
                patch.object(
                    server.native_operation, "drained", return_value=completion
                ) as drain,
                patch.object(server, "systemctl") as control,
            ):
                deadline = server.time.monotonic() + 1
                if scenario == "complete":
                    server.reconcile_registration(
                        self.state, pending, deadline=deadline
                    )
                    self.assertFalse(pending.exists())
                    drain.assert_called_once()
                    self.assertEqual(drain.call_args.kwargs["deadline"], deadline)
                else:
                    with self.assertRaises(server.SupervisorError):
                        server.reconcile_registration(
                            self.state, pending, deadline=deadline
                        )
                    self.assertEqual(server.read_json(pending), case)
                control.assert_not_called()

    def test_recovery_persists_late_bound_owner_before_drain_and_allocation_retirement(
        self,
    ) -> None:
        self.initialized()
        unit = server.worker_unit(self.state, 0)
        binding = {"group": "/owned", "invocation": "b" * 32, "inode": 17}
        handoff = self.root / "late-bound-handoff"
        server.write_json(
            handoff,
            {
                "handoff": unit,
                "scope": {"unit": unit, "group": "/owned", "invocation": "b" * 32},
            },
        )
        pending = self.state / "worker-admission-0.json"
        server.write_json(
            pending,
            {
                "nonce": "selected",
                "unit": unit,
                "boot": host.boot(),
                "unresolved": True,
                "allocation": str(self.allocation.directory / self.allocation.nonce),
                "predecessor": {"invocation": "a" * 32},
                "binding": None,
                "handoff": str(handoff),
            },
        )
        retired = False

        @contextlib.contextmanager
        def ledger(
            _directory: Path, **_kwargs: object
        ) -> Generator[dict[str, object], None, None]:
            yield {
                "owners": {}
                if retired
                else {self.allocation.nonce: {"units": {unit: binding}}}
            }

        def observe(_unit: str, **_kwargs: object) -> dict[str, str]:
            self.assertEqual(
                server.read_json(pending)["binding"], {"unit": unit, **binding}
            )
            return {"LoadState": "not-found"}

        with (
            patch.object(host, "allocation_metadata", side_effect=ledger),
            patch.object(host, "group_identity", return_value=None),
            patch.object(
                server.native_operation, "unit_observation", side_effect=observe
            ),
            patch.object(
                server.native_operation,
                "drained",
                return_value=server.native_operation.DrainObservation(
                    binding,
                    server.native_operation.DrainPhase.UNAVAILABLE,
                    "transient observation",
                ),
            ) as drain,
        ):
            with self.assertRaises(server.SupervisorError):
                server.reconcile_registration(
                    self.state, pending, deadline=server.time.monotonic() + 1
                )
            retired = True
            handoff.unlink()
            drain.return_value = server.native_operation.DrainObservation(
                binding,
                server.native_operation.DrainPhase.COMPLETE,
                "retired exact owner empty",
            )
            server.reconcile_registration(
                self.state, pending, deadline=server.time.monotonic() + 1
            )
        self.assertFalse(pending.exists())

    def test_child_handoff_proof_survives_retirement_before_first_recovery_capture(
        self,
    ) -> None:
        self.initialized()
        operation = server.native_operation
        for enclosing_parent in (False, True):
            for primary in (False, True):
                with self.subTest(parent=enclosing_parent, primary=primary):
                    base = self.root / f"native-{enclosing_parent}-{primary}"
                    unit = (
                        server.primary_unit(self.state)
                        if primary
                        else server.worker_unit(self.state, 0)
                    )
                    pending = self.state / (
                        "primary-admission.json"
                        if primary
                        else "worker-admission-0.json"
                    )
                    nonce = "selected-registration"
                    predecessor: dict[str, object] = {"invocation": "a" * 32}
                    selected = {
                        "nonce": nonce,
                        "unit": unit,
                        "boot": host.boot(),
                        "unresolved": True,
                        "allocation": str(
                            self.allocation.directory / self.allocation.nonce
                        ),
                        "predecessor": predecessor,
                        "binding": None,
                    }
                    server.write_json(pending, selected)
                    with (
                        patch.dict(os.environ, {"PSE_NATIVE_CACHE": str(base)}),
                        patch.object(operation, "scope_owner", return_value=None),
                        operation.Operation(base)
                        if enclosing_parent
                        else contextlib.nullcontext(),
                    ):
                        handoff = operation.prepare_handoff(
                            unit, reservation=(pending, nonce)
                        )
                    assert handoff is not None
                    selected["handoff"] = str(handoff)
                    server.write_json(pending, selected)
                    retired = [False]

                    @contextlib.contextmanager
                    def ledger(
                        _directory: Path,
                        *,
                        retired: list[bool] = retired,
                        unit: str = unit,
                        **_kwargs: object,
                    ) -> Generator[dict[str, object], None, None]:
                        yield {
                            "owners": {}
                            if retired[0]
                            else {self.allocation.nonce: {"units": {unit: {}}}}
                        }

                    # Initial cleanup cannot capture any fresh ledger binding.
                    with (
                        patch.object(host, "allocation_metadata", side_effect=ledger),
                        self.assertRaisesRegex(server.SupervisorError, "fresh"),
                    ):
                        server.stop_registered_owner(
                            unit,
                            self.allocation,
                            predecessor=predecessor,
                            handoff=handoff,
                        )
                    self.assertIsNone(server.read_json(pending)["binding"])
                    # The actual child publishes to the existing handoff before
                    # payload completion; allocation retirement precedes first retry.
                    scope = {
                        "unit": unit,
                        "group": "/late-owned",
                        "invocation": "b" * 32,
                    }
                    with (
                        patch.object(operation, "scope_owner", return_value=scope),
                        patch.object(host, "group_identity", return_value=17),
                    ):
                        operation.bind_handoff(handoff)
                    retired[0] = True
                    self.assertIsNone(server.read_json(pending)["binding"])
                    with (
                        patch.object(host, "allocation_metadata", side_effect=ledger),
                        patch.object(host, "group_identity", return_value=None),
                        patch.object(operation, "populated", return_value=False),
                        patch.object(
                            operation,
                            "unit_observation",
                            return_value={"LoadState": "not-found"},
                        ),
                        patch.object(server, "systemctl") as control,
                    ):
                        self.assertEqual(operation.pinned_generations(base), set())
                        self.assertTrue(
                            handoff.exists(),
                            "settled proof remains owned by the matching reservation",
                        )
                        proof = server.read_json(handoff)
                        for invalid in (
                            {
                                **proof,
                                "registration": {
                                    "path": str(pending),
                                    "nonce": "predecessor",
                                },
                            },
                            {
                                key: value
                                for key, value in proof.items()
                                if key != "scope_inode"
                            },
                        ):
                            server.write_json(handoff, invalid)
                            with self.assertRaises(server.SupervisorError):
                                server.reconcile_registration(
                                    self.state,
                                    pending,
                                    deadline=server.time.monotonic() + 1,
                                )
                            self.assertEqual(server.read_json(pending), selected)
                        server.write_json(handoff, proof)
                        server.reconcile_registration(
                            self.state, pending, deadline=server.time.monotonic() + 1
                        )
                        self.assertFalse(pending.exists())
                        self.assertEqual(operation.pinned_generations(base), set())
                        self.assertFalse(
                            handoff.exists(),
                            "proof retires after its registration owner settles",
                        )
                    control.assert_not_called()

    def test_held_parent_handoff_lock_expires_without_dispatch_and_restores_signals(
        self,
    ) -> None:
        self.reference_fixture()
        operation = server.native_operation
        parent = self.root / "parent-operation.json"
        operation.write_json(
            parent,
            {
                "version": operation.VERSION,
                "scope": None,
                "admissions": {},
                "generations": [],
            },
        )
        program = "import fcntl,sys; f=open(sys.argv[1], 'a'); fcntl.flock(f, fcntl.LOCK_EX); print('held',flush=True); sys.stdin.readline()"
        child = subprocess.Popen(
            [sys.executable, "-c", program, str(parent.with_suffix(".lock"))],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
        )
        original_mask = server.signal.pthread_sigmask(server.signal.SIG_BLOCK, set())
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        try:
            assert child.stdout is not None
            self.assertEqual(child.stdout.readline().strip(), "held")
            parent_identity = parent.with_suffix(".lock").stat()
            real_flock = fcntl.flock
            for role in ("worker", "primary"):
                with self.subTest(role=role):
                    started = server.time.monotonic()
                    deadline = started + 0.2
                    clock = MagicMock(return_value=started)
                    blocked: list[bool] = []

                    def observed_flock(
                        lock: int | TextIO,
                        flags: int,
                        *,
                        clock: MagicMock = clock,
                        blocked: list[bool] = blocked,
                        deadline: float = deadline,
                    ) -> None:
                        try:
                            real_flock(lock, flags)
                        except BlockingIOError:
                            identity = os.fstat(
                                lock if isinstance(lock, int) else lock.fileno()
                            )
                            if (identity.st_dev, identity.st_ino) == (
                                parent_identity.st_dev,
                                parent_identity.st_ino,
                            ):
                                # Advance only after the real exact parent lock
                                # refuses acquisition, independent of host load.
                                blocked.append(True)
                                clock.return_value = deadline
                            raise

                    with (
                        patch.object(server.time, "monotonic", clock),
                        patch.object(
                            operation.fcntl, "flock", side_effect=observed_flock
                        ),
                        patch.object(
                            operation, "owner_record", return_value=parent
                        ) as discover,
                        patch.object(server, "start"),
                        patch.object(server, "ensure_execution_placement"),
                        patch.object(
                            server, "worker_observation", return_value=inactive
                        ),
                        patch.object(
                            server, "primary_observation", return_value=inactive
                        ),
                        patch.object(server, "primary_ready", return_value=False),
                        patch.object(server.subprocess, "Popen") as dispatch_worker,
                        patch.object(server.subprocess, "run") as dispatch_primary,
                        self.assertRaises(server.DeadlineExpiredError),
                    ):
                        if role == "worker":
                            server.worker(
                                self.state, ["/never-dispatch"], deadline=deadline
                            )
                        else:
                            with patch.object(
                                server._STARTUP,  # noqa: SLF001 -- exercise the original inherited admission clock
                                "deadline",
                                deadline,
                                create=True,
                            ):
                                server.ensure_primary(self.state)
                    self.assertEqual(blocked, [True])
                    self.assertEqual(clock.return_value, deadline)
                    discover.assert_called_once_with(deadline=deadline)
                    dispatch_worker.assert_not_called()
                    dispatch_primary.assert_not_called()
                    self.assertEqual(
                        server.signal.pthread_sigmask(server.signal.SIG_BLOCK, set()),
                        original_mask,
                    )
                    self.assertFalse(list(self.root.glob("handoff-*.json")))
                    self.assertFalse((self.state / "worker-admission-0.json").exists())
                    self.assertFalse((self.state / "primary-admission.json").exists())
        finally:
            child.communicate("release\n", timeout=5)
            self.assertEqual(child.returncode, 0)
        self.assertFalse(
            list(self.root.glob("handoff-*.json")),
            "release after expiry cannot publish a late handoff",
        )

    def test_held_host_ledger_expires_in_inherit_and_register_before_role_dispatch(
        self,
    ) -> None:
        self.reference_fixture()
        directory = self.allocation.directory
        directory.mkdir(mode=0o700)
        program = "import fcntl,os,sys; f=open(sys.argv[1], 'a'); os.chmod(sys.argv[1], 0o600); fcntl.flock(f, fcntl.LOCK_EX); print('held',flush=True); sys.stdin.readline()"
        child = subprocess.Popen(
            [sys.executable, "-c", program, str(directory / ".lock")],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
        )
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        try:
            assert child.stdout is not None
            self.assertEqual(child.stdout.readline().strip(), "held")
            for role in ("worker", "primary"):
                for boundary in ("inherit", "register"):
                    with self.subTest(role=role, boundary=boundary):
                        self.allocation.register.side_effect = (
                            None
                            if boundary == "inherit"
                            else lambda unit, *, deadline: host.Allocation.register(
                                self.allocation, unit, deadline=deadline
                            )
                        )
                        started = server.time.monotonic()
                        # Leave scheduling margin for reaching the real held-ledger
                        # boundary; that same finite deadline must still expire there.
                        deadline = started + 1.0
                        with (
                            patch.dict(
                                os.environ,
                                {host.MARKER: str(directory / self.allocation.nonce)},
                            ),
                            patch.object(
                                host,
                                "inherit",
                                side_effect=self.host_inherit
                                if boundary == "inherit"
                                else None,
                                return_value=self.allocation,
                            ) as inherit,
                            patch.object(
                                host,
                                "readonly_snapshot",
                                side_effect=self.host_snapshot,
                            ),
                            patch.object(
                                server.native_operation,
                                "prepare_handoff",
                                return_value=None,
                            ),
                            patch.object(server, "start"),
                            patch.object(server, "ensure_execution_placement"),
                            patch.object(
                                server, "worker_observation", return_value=inactive
                            ),
                            patch.object(
                                server, "primary_observation", return_value=inactive
                            ),
                            patch.object(server, "primary_ready", return_value=False),
                            patch.object(server.subprocess, "Popen") as dispatch_worker,
                            patch.object(server.subprocess, "run") as dispatch_primary,
                            self.assertRaises(server.DeadlineExpiredError),
                        ):
                            if role == "worker":
                                server.worker(
                                    self.state, ["/never-dispatch"], deadline=deadline
                                )
                            else:
                                with patch.object(
                                    server._STARTUP,  # noqa: SLF001 -- exercise the original inherited admission clock
                                    "deadline",
                                    deadline,
                                    create=True,
                                ):
                                    server.ensure_primary(self.state)
                        self.assertLess(server.time.monotonic() - started, 2.0)
                        self.assertEqual(inherit.call_args.kwargs["deadline"], deadline)
                        dispatch_worker.assert_not_called()
                        dispatch_primary.assert_not_called()
                        self.assertFalse(
                            (self.state / "worker-admission-0.json").exists()
                        )
                        self.assertFalse(
                            (self.state / "primary-admission.json").exists()
                        )
        finally:
            child.communicate("release\n", timeout=5)
            self.assertEqual(child.returncode, 0)

    def test_cli_and_direct_script_share_reciprocal_lifecycle_module(self) -> None:
        program = """
import argparse
import runpy
import sys
from pathlib import Path
from unittest.mock import patch

def selected(*_args, **_kwargs):
    from scripts import surreal_server
    from scripts import host_admission
    active = sys.modules['__main__']
    assert surreal_server is active, 'reciprocal supervisor import split lifecycle authority'
    active._LIFECYCLE.state = Path('/exact-owned-maintenance')
    active._STARTUP.deadline = 123.0  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
    assert surreal_server._LIFECYCLE.state == Path('/exact-owned-maintenance')
    assert surreal_server._STARTUP.deadline == 123.0  # noqa: SLF001 -- exercise the original owner failure/consistency boundary
    active.dispatch = lambda _args: 0
    return argparse.Namespace()

with patch.object(argparse.ArgumentParser, 'parse_args', side_effect=selected):
    if sys.argv[1] == 'module':
        runpy.run_module('scripts.surreal_server', run_name='__main__', alter_sys=True)
    else:
        runpy.run_path(sys.argv[2], run_name='__main__')
"""
        for route in ("module", "direct-script"):
            with self.subTest(route=route):
                result = subprocess.run(
                    [sys.executable, "-c", program, route, str(server.SCRIPT)],
                    cwd=server.SCRIPT.parents[1],
                    capture_output=True,
                    text=True,
                    timeout=15,
                    check=False,
                )
                self.assertEqual(result.returncode, 0, result.stderr)

    def test_cli_surfaces_safe_cleanup_notes_preserving_primary_error(self) -> None:
        for failure in (
            server.DeadlineExpiredError("original admission expired"),
            subprocess.TimeoutExpired("secret-bearing-command", 0),
        ):
            note = "Primary cleanup unresolved; selected=owned.service; admission reservation retained: TimeoutExpired: cleanup clock expired"
            failure.add_note(note)
            failure.add_note("untrusted validator secret must remain redacted")
            output = io.StringIO()
            with (
                self.subTest(error=type(failure).__name__),
                patch.object(server, "dispatch", side_effect=failure),
                patch.object(server.sys, "stderr", output),
            ):
                self.assertEqual(server.main(["status", "--state", str(self.state)]), 1)
            self.assertIn(note, output.getvalue())
            self.assertNotIn("secret", output.getvalue())
            self.assertIn(
                "original admission expired"
                if isinstance(failure, server.SupervisorError)
                else "TimeoutExpired: supervisor operation failed",
                output.getvalue(),
            )

    def test_worker_requires_systemd_and_refuses_mismatched_cap(self) -> None:
        self.initialized()
        with (
            patch.object(
                server, "systemctl", side_effect=server.SupervisorError("required")
            ),
            patch.object(server.subprocess, "Popen") as spawn,
            self.assertRaises(server.SupervisorError),
        ):
            server.worker(self.state, ["/worker"])
        spawn.assert_not_called()
        child = MagicMock()
        child.poll.return_value = None
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        uncapped = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "",
            "MemoryMax": "infinity",
        }
        with (
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.READY),
            ),
            patch.object(
                server,
                "worker_observation",
                side_effect=[inactive, inactive, inactive, uncapped],
            ),
            patch.object(server.subprocess, "Popen", return_value=child),
            self.assertRaisesRegex(server.SupervisorError, "cap differs"),
        ):
            server.worker(self.state, ["/worker"])

    def test_lexical_state_aliases_share_the_same_worker_slot_names(self) -> None:
        self.initialized()
        alias = self.state / ".." / self.state.name
        self.assertEqual(
            server.worker_unit(alias, 0), server.worker_unit(self.state, 0)
        )
        self.assertEqual(server.unit_name(alias), server.unit_name(self.state))
        self.assertEqual(server.checked_directory(alias), self.state)

    def test_only_empty_kernel_group_allows_orphan_scope_collection(self) -> None:
        self.initialized()
        empty = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "/owned-fixture",
        }
        inactive = {
            "LoadState": "not-found",
            "ActiveState": "inactive",
            "ControlGroup": "",
        }
        with (
            patch.object(server, "worker_observation", side_effect=[empty, inactive]),
            patch.object(server, "group_populated", return_value=False),
            patch.object(server, "systemctl") as manager,
        ):
            self.assertEqual(server.settled_worker(self.state, 0), inactive)
        manager.assert_called_once_with("stop", server.worker_unit(self.state, 0))
        with (
            patch.object(server, "worker_observation", return_value=empty),
            patch.object(server, "group_populated", return_value=True),
            patch.object(server, "systemctl") as manager,
        ):
            self.assertEqual(server.settled_worker(self.state, 0), empty)
        manager.assert_not_called()

    def test_caller_drain_ack_does_not_bypass_populated_managed_scope(self) -> None:
        config = self.initialized()
        observation = {
            "LoadState": "loaded",
            "ActiveState": "inactive",
            "ControlGroup": "/fixture",
        }
        with (
            patch.object(server, "worker_observation", return_value=observation),
            patch.object(server, "group_populated", return_value=True),
            patch.object(server, "active", return_value=False),
            self.assertRaisesRegex(server.SupervisorError, "remain active"),
        ):
            server.backup(self.state, config, self.root / "blocked-backup")
        self.assertFalse((self.root / "blocked-backup").exists())
        self.assertFalse(server.config_for(self.state)["accepting_writes"])

    def test_inspect_backup_uses_restore_verifier_and_never_creates_state(self) -> None:
        backup = self.backup_fixture()
        before = {
            str(p.relative_to(backup)): p.read_bytes()
            for p in backup.rglob("*")
            if p.is_file()
        }
        result = server.inspect_backup(backup, server.SUBSTRATE_INTERPRETATION)
        self.assertTrue(result["complete"])
        self.assertEqual(result["backup_version"], 4)
        self.assertIsNone(result["physical_reclaimed_bytes"])
        self.assertEqual(
            result["application_inventory"], self.application_inventory_fixture()
        )
        self.assertEqual(
            before,
            {
                str(p.relative_to(backup)): p.read_bytes()
                for p in backup.rglob("*")
                if p.is_file()
            },
        )

    def test_backup_requires_application_inventory_and_refuses_old_format(self) -> None:
        backup = self.backup_fixture()
        manifest = server.read_json(backup / "backup.json")
        for alteration in ("missing_owner", "missing_inventory", "old_format"):
            with self.subTest(alteration=alteration):
                value = json.loads(json.dumps(manifest))
                if alteration == "missing_owner":
                    del value["application_inventory"]["tables"][
                        "canonical_research_references"
                    ]
                elif alteration == "missing_inventory":
                    del value["application_inventory"]
                else:
                    value["backup_version"] = 3
                server.write_json(backup / "backup.json", value)
                inspection = server.inspect_backup(
                    backup, server.SUBSTRATE_INTERPRETATION
                )
                self.assertFalse(inspection["complete"])
                self.assertFalse(inspection["restorable"])
                destination = self.root / alteration
                with self.assertRaises(server.SupervisorError):
                    server.restore(backup, destination, server.SUBSTRATE_INTERPRETATION)
                self.assertFalse(destination.exists())
        server.write_json(backup / "backup.json", manifest)

    def test_application_restore_mismatch_keeps_admission_closed(self) -> None:
        backup = self.backup_fixture()
        manifest = server.read_json(backup / "backup.json")
        application_inventory = server.object_mapping(manifest["application_inventory"])
        tables = server.object_mapping(application_inventory["tables"])
        tables["canonical_project_imports"] = [
            {
                "key": "pending-import",
                "sha256": "a" * 64,
                "references": "{complete:false,project:original}",
            }
        ]
        application_inventory["tables"] = tables
        manifest["application_inventory"] = application_inventory
        server.write_json(backup / "backup.json", manifest)
        restored = self.root / "catalogue-mismatch"
        with (
            self.native_maintenance(),
            self.assertRaisesRegex(server.SupervisorError, "catalogue differs"),
        ):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        self.assertFalse(server.config_for(restored)["accepting_writes"])
        self.assertEqual(server.read_json(backup / "backup.json"), manifest)

    def test_readonly_payload_verification_keeps_descriptors_outside_backup(
        self,
    ) -> None:
        config = self.initialized()
        verifier = {"binary": "/controlled/native-verifier", "sha256": "a" * 64}
        config["payload_verifier"] = verifier
        inventory = [
            {
                "database": config["database"],
                "payload_incarnation": config["payload_incarnation"],
                "instance": {
                    "key": str(server.uuid.uuid4()),
                    "state": "sealed",
                    "digest": "original",
                    "encoding": "opaque-bytes-v1",
                    "batch_count": "0",
                    "schema": "",
                    "interpretation": "original",
                    "payload_bytes": "3",
                },
                "files": {},
            }
        ]
        before = set(self.state.rglob("*"))

        def native(
            arguments: list[str], **_options: object
        ) -> subprocess.CompletedProcess[str]:
            descriptor = Path(arguments[2])
            self.assertNotEqual(descriptor.parent, self.state)
            self.assertTrue(descriptor.is_file())
            return subprocess.CompletedProcess(
                arguments, 0, '{"verified_payload_files":1}', ""
            )

        with (
            patch.object(server, "payload_verifier", return_value=verifier),
            patch.object(server.subprocess, "run", side_effect=native),
        ):
            server.verify_payload_inventory(
                self.state, config, inventory, read_only_source=True
            )
        self.assertEqual(set(self.state.rglob("*")), before)

    def test_inspect_backup_refuses_native_semantic_payload_failure(self) -> None:
        backup = self.backup_fixture()
        with patch.object(
            server,
            "verify_payload_inventory",
            side_effect=server.SupervisorError("canonical payload meaning differs"),
        ):
            inspection = server.inspect_backup(backup, server.SUBSTRATE_INTERPRETATION)
        self.assertFalse(inspection["complete"])
        self.assertFalse(inspection["restorable"])
        self.assertIn("canonical payload meaning differs", str(inspection["refusal"]))

    def test_inspect_backup_reports_incomplete_members_without_effects(self) -> None:
        backup = self.backup_fixture()
        (backup / "database/fixture").unlink()
        inspection = server.inspect_backup(backup, server.SUBSTRATE_INTERPRETATION)
        self.assertFalse(inspection["complete"])
        self.assertIn("database/fixture", str(inspection["refusal"]))
        self.assertFalse((backup / "database/fixture").exists())

    def test_incomplete_application_capture_never_publishes_complete_manifest(
        self,
    ) -> None:
        config = self.initialized()
        destination = self.root / "interrupted-application"
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=server.SupervisorError("missing original reference"),
            ),
            self.assertRaisesRegex(
                server.SupervisorError, "missing original reference"
            ),
        ):
            server.backup(self.state, config, destination)
        self.assertFalse((destination / "backup.json").exists())

    def test_application_inventory_query_projects_declared_original_references(
        self,
    ) -> None:
        body = server.application_inventory_query(
            "canonical_projects", 'original"cursor'
        )
        self.assertIn('WHERE key>"original\\"cursor" ORDER BY key LIMIT 64', body)
        self.assertIn(
            "references:<string>{project:$row.project,draft:$row.draft,", body
        )
        self.assertIn("retiring:$row.retiring}", body)
        self.assertIn(
            "sha256:crypto::sha256(<string>object::remove($row,['id']))", body
        )
        self.assertNotIn("object::pick", body)
        with self.assertRaisesRegex(server.SupervisorError, "Unknown application"):
            server.application_inventory_query("unowned_table", "")

    def test_application_inventory_maintenance_preserves_transaction_acknowledgement(
        self,
    ) -> None:
        config = self.initialized()
        credentials = server.read_json(self.state / "credentials.json")
        authority = self.application_inventory_fixture()["issuing_store"]
        row = {
            "key": "original-stage",
            "sha256": "a" * 64,
            "references": "{closed:false}",
        }
        queries: list[str] = []

        def acknowledged(
            arguments: list[str], **options: object
        ) -> subprocess.CompletedProcess[str]:
            query = options["input"]
            assert isinstance(query, str)
            queries.append(query)
            nonce = query.split("maintenance: '")[1].split("'")[0]
            # RETURN inside an assigned block exits before the owner nonce. Only the
            # maintenance transport's final acknowledgement may return a result.
            self.assertEqual(query.count("RETURN "), 1)
            self.assertIn("LET $value=", query)
            value = (
                authority
                if "canonical_interpretations:current" in query
                else (
                    [row]
                    if "FROM canonical_stages " in query and 'key>""' in query
                    else []
                )
            )
            return subprocess.CompletedProcess(
                arguments,
                0,
                json.dumps(
                    [
                        None,
                        {"maintenance": nonce, "value": value},
                        None,
                    ]
                ),
                "",
            )

        with (
            patch.object(
                server, "maintenance_endpoint", return_value="http://127.0.0.1:18080"
            ),
            patch.object(server.subprocess, "run", side_effect=acknowledged),
        ):
            inventory = server.capture_application_inventory(
                self.state, config, credentials
            )
        self.assertEqual(inventory["issuing_store"], authority)
        self.assertEqual(
            server.object_mapping(inventory["tables"])["canonical_stages"], [row]
        )
        self.assertEqual(len(queries), len(server.APPLICATION_BACKUP_TABLES) + 2)
        self.assertFalse((self.state / "maintenance-cli-failure.json").exists())

    def test_application_inventory_pages_preserve_unsettled_original_references(
        self,
    ) -> None:
        config = self.initialized()
        key = "original-unsettled-stage"
        row = {
            "key": key,
            "sha256": "a" * 64,
            "references": "{activated:false,closed:false,problem:original}",
        }
        calls: list[str] = []

        def query(_state: Path, _config: dict[str, object], body: str) -> object:
            calls.append(body)
            if "canonical_interpretations:current" in body:
                return self.application_inventory_fixture()["issuing_store"]
            if "FROM canonical_stages " in body and 'key>""' in body:
                return [row]
            return []

        with patch.object(server, "payload_query", side_effect=query):
            inventory = server.capture_application_inventory(self.state, config)
        self.assertEqual(
            server.object_mapping(inventory["tables"])["canonical_stages"], [row]
        )
        self.assertTrue(any('key>"original-unsettled-stage"' in body for body in calls))
        self.assertFalse(any("UPDATE" in body or "DELETE" in body for body in calls))

    def test_restore_is_gated_and_preserves_exact_fixture(self) -> None:
        backup = self.backup_fixture()
        self.assertFalse((backup / ".resource-owner").exists())
        restored = self.root / "restored"
        prior = server.config_for(backup)
        prior_credentials = server.read_json(backup / "credentials.json")
        with self.native_maintenance() as calls:
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        config = server.config_for(restored)
        self.assertFalse(config["accepting_writes"])
        self.assertEqual(config["admission"], "validation_required")
        self.assertEqual(
            (restored / "database/fixture").read_bytes(),
            b"acknowledged fixed operation identity",
        )
        self.assertEqual(config["credentials_file"], str(restored / "credentials.json"))
        self.assertNotEqual(
            (config["namespace"], config["database"]),
            (prior["namespace"], prior["database"]),
        )
        self.assertNotEqual(config["instance_id"], prior["instance_id"])
        self.assertNotEqual(
            server.read_json(restored / "credentials.json"), prior_credentials
        )
        self.assertEqual(
            server.read_json(backup / "credentials.json"), prior_credentials
        )
        actions = [body for _, _, body in calls]
        self.assertLess(actions.index("export"), actions.index("import"))
        disposals = [call for call in calls if call[2].startswith("REMOVE DATABASE ")]
        self.assertEqual(len(disposals), 1)
        self.assertEqual(
            disposals[0][2],
            "REMOVE DATABASE "
            + server.sql_identifier(prior["database"])
            + "; LET $value=true;",
        )
        self.assertGreater(actions.index(disposals[0][2]), actions.index("import"))
        self.assertEqual(disposals[0][:2], (prior["namespace"], "None"))
        self.assertFalse(config.get("derived_rebuild_pending"))
        self.assertEqual(server.test_resources.service_resources(restored), {})
        with self.assertRaises(server.SupervisorError):
            server.start(restored, config)

    def test_restore_establishes_destination_registry_instead_of_backup_authority(
        self,
    ) -> None:
        backup = self.backup_fixture()
        source_binding = server.read_json(self.state / ".resource-owner/registry.json")
        restored = self.root / "restored"
        registry = self.root / "destination-resources"
        with (
            patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(registry)}),
            self.native_maintenance(),
        ):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
            self.assertEqual(server.test_resources.service_resources(restored), {})
        destination_binding = server.read_json(
            restored / ".resource-owner/registry.json"
        )
        self.assertEqual(destination_binding["registry"], str(registry))
        self.assertNotEqual(destination_binding, source_binding)
        self.assertEqual(
            server.read_json(self.state / ".resource-owner/registry.json"),
            source_binding,
        )

    def test_backup_requires_source_registry_coverage_before_freezing_payloads(
        self,
    ) -> None:
        config = self.initialized()
        (self.state / ".resource-owner/registry.json").unlink()
        destination = self.root / "unpublished-backup"
        with (
            patch.object(server, "freeze_payloads") as freeze,
            self.assertRaisesRegex(
                server.test_resources.ResourceError, "coverage is unestablished"
            ),
        ):
            server.backup(self.state, config, destination)
        freeze.assert_not_called()
        self.assertFalse(destination.exists())

    def test_restore_recovery_requires_original_destination_registry_before_copy(
        self,
    ) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        with (
            patch.object(
                server,
                "recover_maintenance",
                side_effect=OSError("interrupted publication"),
            ),
            self.assertRaisesRegex(OSError, "interrupted publication"),
        ):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        original = (restored / "config.json").read_bytes()
        binding_path = restored / ".resource-owner/registry.json"
        binding = binding_path.read_bytes()
        for changed in (False, True):
            with self.subTest(changed_registry=changed):
                if changed:
                    binding_path.write_bytes(binding)
                    binding_path.chmod(0o600)
                    selection = patch.dict(
                        os.environ,
                        {
                            "PSE_TEST_RESOURCE_STATE": str(
                                self.root / "another-registry"
                            )
                        },
                    )
                else:
                    binding_path.unlink()
                    selection = contextlib.nullcontext()
                with (
                    selection,
                    patch.object(server, "complete_backup_copy") as copy,
                    self.assertRaisesRegex(
                        server.test_resources.ResourceError, "coverage"
                    ),
                ):
                    server.recover_maintenance(restored, [])
                copy.assert_not_called()
                self.assertEqual((restored / "config.json").read_bytes(), original)
                self.assertFalse((restored / "database").exists())

    def test_interrupted_backup_copy_publishes_fresh_closed_identity_before_old_bytes(
        self,
    ) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        old = server.config_for(backup)
        original_copy = server.shutil.copy2

        def interrupt(source: Path, target: Path) -> None:
            config = server.config_for(restored)
            self.assertEqual(
                server.object_mapping(config["derived_rebuild_pending"])["phase"],
                "backup-copy",
            )
            self.assertNotEqual(
                (config["namespace"], config["database"]),
                (old["namespace"], old["database"]),
            )
            self.assertFalse(config["accepting_writes"])
            self.assertNotEqual(Path(source).name, "config.json")
            Path(target).write_bytes(b"partial known file")
            raise OSError("interrupted file copy")

        with (
            self.native_maintenance(),
            patch.object(server.shutil, "copy2", side_effect=interrupt),
            self.assertRaisesRegex(OSError, "interrupted"),
        ):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        config = server.config_for(restored)
        for operation in (
            lambda: server.start(restored, config),
            lambda: server.open_context_admission(restored),
            lambda: server.validate(
                restored, config, server.SUBSTRATE_INTERPRETATION, ["check"]
            ),
        ):
            with self.assertRaises(server.SupervisorError):
                operation()
        with (
            self.native_maintenance(),
            patch.object(server.shutil, "copy2", wraps=original_copy),
        ):
            server.recover_maintenance(restored, [])
        self.assertFalse(server.config_for(restored).get("derived_rebuild_pending"))
        self.assertEqual(
            (restored / "database/fixture").read_bytes(),
            (backup / "database/fixture").read_bytes(),
        )
        self.assertEqual(server.config_for(backup), old)

    def test_backup_copy_recovery_refuses_unknown_files_or_changed_preserved_backup(
        self,
    ) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        with (
            patch.object(
                server,
                "recover_maintenance",
                side_effect=server.SupervisorError("interrupted"),
            ),
            self.assertRaises(server.SupervisorError),
        ):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        unknown = restored / "unknown-owner"
        unknown.write_text("preserve me")
        with (
            self.native_maintenance(),
            patch.object(server.shutil, "copy2") as copy,
            self.assertRaisesRegex(
                server.SupervisorError, "Unknown interrupted backup"
            ),
        ):
            server.recover_maintenance(restored, [])
        copy.assert_not_called()
        self.assertEqual(unknown.read_text(), "preserve me")
        unknown.unlink()
        (backup / "database/fixture").write_bytes(b"changed preserved source")
        with (
            self.native_maintenance(),
            patch.object(server.shutil, "copy2") as copy,
            self.assertRaisesRegex(server.SupervisorError, "content failed validation"),
        ):
            server.recover_maintenance(restored, [])
        copy.assert_not_called()
        self.assertEqual(
            server.object_mapping(
                server.config_for(restored)["derived_rebuild_pending"]
            )["phase"],
            "backup-copy",
        )

    def test_restore_destination_reservation_excludes_concurrent_empty_contender(
        self,
    ) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        publishing = threading.Event()
        release = threading.Event()
        original_write = server.write_json
        owner = []

        def write(path: Path, value: dict[str, object]) -> None:
            if path == restored / "config.json":
                owner.append(
                    server.read_json(restored / "lifecycle-owner.json")["nonce"]
                )
                publishing.set()
                if not release.wait(10):
                    raise RuntimeError("publication barrier not released")
            original_write(path, value)

        def recover(state: Path, _initializer: list[str]) -> dict[str, object]:
            self.assertEqual(
                server.read_json(state / "lifecycle-owner.json")["nonce"], owner[0]
            )
            self.assertEqual(server._LIFECYCLE.state, restored)  # noqa: SLF001 -- assert shared reservation
            return {"same_owner": True}

        with (
            patch.object(server, "write_json", side_effect=write),
            patch.object(server, "recover_maintenance", side_effect=recover),
            ThreadPoolExecutor(max_workers=2) as workers,
        ):
            first = workers.submit(
                server.restore, backup, restored, server.SUBSTRATE_INTERPRETATION
            )
            try:
                self.assertTrue(publishing.wait(10))
                second = workers.submit(
                    server.restore, backup, restored, server.SUBSTRATE_INTERPRETATION
                )
                with self.assertRaisesRegex(
                    server.SupervisorError, "Another live owner"
                ):
                    second.result(timeout=10)
                self.assertFalse((restored / "config.json").exists())
            finally:
                release.set()
            self.assertEqual(first.result(timeout=10), {"same_owner": True})
        self.assertEqual(len(owner), 1)

    def test_backup_copy_preserves_atomic_metadata_scratch_without_consuming_it(
        self,
    ) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        with (
            patch.object(
                server,
                "recover_maintenance",
                side_effect=server.SupervisorError("interrupted"),
            ),
            self.assertRaises(server.SupervisorError),
        ):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        scratch = restored / ".config.json-a1b2c3d4"
        scratch.write_bytes(b'{"partially written metadata')
        scratch.chmod(0o600)
        lifecycle_scratch = restored / ".lifecycle-owner.json-a1b2c3d4"
        lifecycle_scratch.write_bytes(b"partial owner bytes")
        lifecycle_scratch.chmod(0o600)
        with self.native_maintenance():
            server.recover_maintenance(restored, [])
        self.assertEqual(scratch.read_bytes(), b'{"partially written metadata')
        self.assertEqual(lifecycle_scratch.read_bytes(), b"partial owner bytes")
        self.assertFalse(server.config_for(restored).get("derived_rebuild_pending"))

    def test_restore_reentry_preserves_prepublication_metadata_scratch(self) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        original_write = server.write_json
        scratch = restored / ".config.json-a1b2c3d4"

        def interrupt(path: Path, value: dict[str, object]) -> None:
            if path == restored / "config.json":
                scratch.write_bytes(b'{"partial before atomic replace')
                scratch.chmod(0o600)
                raise OSError("interrupted initial publication")
            original_write(path, value)

        with (
            patch.object(server, "write_json", side_effect=interrupt),
            self.assertRaisesRegex(OSError, "initial publication"),
        ):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        self.assertFalse((restored / "config.json").exists())
        self.assertFalse((restored / "database").exists())
        binding = (restored / ".resource-owner/registry.json").read_bytes()
        with self.native_maintenance():
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        self.assertEqual(
            (restored / ".resource-owner/registry.json").read_bytes(), binding
        )
        self.assertEqual(scratch.read_bytes(), b'{"partial before atomic replace')
        self.assertFalse(server.config_for(restored).get("derived_rebuild_pending"))

    def test_backup_corruption_is_refused_before_state_creation(self) -> None:
        backup = self.backup_fixture()
        (backup / "database/fixture").write_bytes(b"changed")
        restored = self.root / "restored"
        with self.assertRaises(server.SupervisorError):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        self.assertFalse(restored.exists())

    def test_unknown_interpretation_is_refused(self) -> None:
        backup = self.backup_fixture()
        with self.assertRaises(server.SupervisorError):
            server.restore(backup, self.root / "restored", "unknown")

    def test_failed_semantic_validation_retains_write_gate(self) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        with self.native_maintenance():
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        config = server.config_for(restored)
        with (
            patch.object(server, "start"),
            patch.object(server, "stop"),
            patch.object(server, "payload_query", return_value=True),
            patch.object(server, "all_contexts_drained"),
            patch.object(
                server,
                "reap_payload_owners",
                return_value={"recovered": 0, "live": 0, "unknown": 0},
            ),
            patch.object(
                server,
                "validate_interpretation",
                side_effect=server.SupervisorError("wrong metadata"),
            ),
            self.assertRaises(server.SupervisorError),
        ):
            server.validate(
                restored, config, server.SUBSTRATE_INTERPRETATION, ["unused"]
            )
        self.assertFalse(server.config_for(restored)["accepting_writes"])
        self.assertEqual(
            server.config_for(restored)["admission"], "validation_required"
        )

    def test_unsupported_restore_refuses_before_destination_creation(self) -> None:
        config = self.initialized()
        config.update(
            interpretation="pse.substrate.old",
            schema_interpretation="pse.substrate.old",
        )
        server.write_json(self.state / "config.json", config)
        backup = self.root / "backup"
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
        ):
            server.backup(self.state, config, backup)
        with self.assertRaisesRegex(server.SupervisorError, "explicit rebuild"):
            server.restore(backup, self.root / "restored", "pse.substrate.old")
        self.assertFalse((self.root / "restored").exists())
        self.assertEqual(
            server.config_for(backup)["interpretation"], "pse.substrate.old"
        )

    def test_restore_refuses_shared_context_before_destination_creation(self) -> None:
        backup = self.backup_fixture()
        with (
            patch.object(
                server, "managed_contexts", return_value=[backup, self.root / "shared"]
            ),
            self.assertRaisesRegex(server.SupervisorError, "shared"),
        ):
            server.restore(
                backup, self.root / "restored", server.SUBSTRATE_INTERPRETATION
            )
        self.assertFalse((self.root / "restored").exists())

    def test_cutover_requires_private_closed_stopped_drained_owner(self) -> None:
        config = self.initialized()
        config.update(accepting_writes=False, admission="quiesced")
        for changed in ({"accepting_writes": True}, {"admission": "open"}):
            with (
                self.subTest(changed=changed),
                self.assertRaisesRegex(server.SupervisorError, "closed admission"),
            ):
                server.private_offline_state(self.state, {**config, **changed})
        with (
            patch.object(
                server,
                "managed_contexts",
                return_value=[self.state, self.root / "shared"],
            ),
            self.assertRaisesRegex(server.SupervisorError, "shared"),
        ):
            server.private_offline_state(self.state, config)
        with (
            patch.object(server, "active", return_value=True),
            self.assertRaisesRegex(server.SupervisorError, "stopped"),
        ):
            server.private_offline_state(self.state, config)
        with (
            patch.object(
                server,
                "all_contexts_drained",
                side_effect=server.SupervisorError("live borrower"),
            ),
            patch.object(server, "stop") as stop,
            self.assertRaisesRegex(server.SupervisorError, "live borrower"),
        ):
            server.private_offline_state(self.state, config)
        stop.assert_not_called()

    def test_rebuild_mints_authority_and_disposes_only_after_current_validation(
        self,
    ) -> None:
        prior = self.initialized()
        prior.update(accepting_writes=False, admission="quiesced")
        server.write_json(self.state / "config.json", prior)
        source = self.root / "authored"
        source.mkdir()
        (source / "input").write_text("external authority")
        events: list[str] = []
        frozen = [{"files": {}, "context": "x" * 2048} for _ in range(49)]

        def rotate(
            _state: Path,
            old: dict[str, object],
            _before: dict[str, object],
            after: dict[str, object],
        ) -> None:
            self.assertEqual(
                (
                    server.object_mapping(old["derived_rebuild_pending"])["namespace"],
                    server.object_mapping(old["derived_rebuild_pending"])["database"],
                ),
                (prior["namespace"], prior["database"]),
            )
            config = server.config_for(self.state)
            self.assertTrue(config["derived_rebuild_pending"])
            self.assertFalse(config["accepting_writes"])
            self.assertNotEqual(config["database"], prior["database"])
            server.write_json(self.state / "credentials.json", after)
            events.append("accounts")

        def initialize(
            _arguments: list[str], environment: dict[str, str], **options: object
        ) -> int:
            config = server.config_for(self.state)
            self.assertFalse(config["accepting_writes"])
            self.assertEqual(config["admission"], "validation_required")
            self.assertLess((self.state / "config.json").stat().st_size, 64 * 1024)
            pending = server.object_mapping(config["derived_rebuild_pending"])
            self.assertEqual(
                server.maintenance_payloads(self.state, pending)["payload_inventory"],
                frozen,
            )
            self.assertGreater(
                server.integer(
                    server.object_mapping(pending["payload_retirement"])["bytes"]
                ),
                64 * 1024,
            )
            self.assertNotIn("PSE_PRESERVED_INPUTS", environment)
            self.assertIsInstance(options["stdout"], int)
            self.assertGreaterEqual(options["stdout"], 0)
            self.assertEqual(options["stderr"], options["stdout"])
            diagnostic = Path(f"/proc/self/fd/{options['stdout']}").readlink()
            self.assertEqual(diagnostic.parent, self.state)
            self.assertTrue(diagnostic.name.startswith("canonical-initializer-"))
            self.assertEqual(stat.S_IMODE(diagnostic.stat().st_mode), 0o600)
            self.assertNotIn("deadline", options)
            self.assertTrue((self.state / "lifecycle-owner.json").exists())
            self.assertEqual(
                environment["PSE_CANONICAL_INITIALIZER"],
                server.read_json(self.state / "lifecycle-owner.json")["nonce"],
            )
            self.assertEqual(
                server.object_mapping(config["derived_rebuild_pending"])["initializer"],
                server.read_json(self.state / "lifecycle-owner.json"),
            )
            events.append("initialize")
            return 0

        def discard(
            _state: Path,
            selected: dict[str, object],
            old: dict[str, object],
            _credentials: dict[str, object],
        ) -> None:
            self.assertEqual(
                (old["namespace"], old["database"]),
                (prior["namespace"], prior["database"]),
            )
            self.assertNotEqual(selected["database"], old["database"])
            self.assertFalse(server.config_for(self.state)["accepting_writes"])
            self.assertEqual(events[-2:], ["validate", "stop"])
            events.append("dispose")

        with (
            patch.object(server, "freeze_payloads", return_value=frozen),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "cleanup_retired_payloads"),
            patch.object(server, "rotate_database_accounts", side_effect=rotate),
            patch.object(
                server,
                "start",
                side_effect=lambda *_args, **_kw: events.append("start"),
            ),
            patch.object(
                server, "stop", side_effect=lambda *_args: events.append("stop")
            ),
            patch.object(server, "run_file_effect", side_effect=initialize),
            patch.object(
                server,
                "validate_interpretation",
                side_effect=lambda *_args: events.append("validate"),
            ),
            patch.object(server, "database_present", return_value=True),
            patch.object(server, "discard_database", side_effect=discard),
        ):
            server.rebuild(self.state, ["current-initializer"])
        self.assertEqual(
            events,
            ["stop", "accounts", "start", "initialize", "validate", "stop", "dispose"],
        )
        self.assertFalse((self.state / ".maintenance-listener.json").exists())
        config = server.config_for(self.state)
        self.assertFalse(config["accepting_writes"])
        self.assertFalse(config.get("derived_rebuild_pending"))
        self.assertEqual(config["admission"], "quiesced")
        self.assertEqual(config["interpretation"], server.SUBSTRATE_INTERPRETATION)
        self.assertEqual((source / "input").read_text(), "external authority")
        self.assertEqual(list(self.state.glob(".maintenance-payloads-*.json")), [])

    def test_rebuild_current_marker_failure_retains_old_database(
        self,
    ) -> None:
        config = self.initialized()
        config.update(accepting_writes=False, admission="quiesced")
        server.write_json(self.state / "config.json", config)
        source = self.root / "authored"
        source.mkdir()
        (source / "input").write_text("external authority")

        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "rotate_database_accounts"),
            patch.object(server, "start"),
            patch.object(server, "stop") as stop,
            patch.object(
                server,
                "run_file_effect",
                return_value=0,
            ),
            patch.object(
                server,
                "validate_interpretation",
                side_effect=server.SupervisorError("current marker differs"),
            ),
            patch.object(server, "discard_database") as discard,
            self.assertRaisesRegex(server.SupervisorError, "current marker differs"),
        ):
            server.rebuild(self.state, ["initializer"])
        self.assertEqual(stop.call_count, 2)
        self.assertFalse((self.state / ".maintenance-listener.json").exists())
        discard.assert_not_called()
        config = server.config_for(self.state)
        self.assertTrue(config["derived_rebuild_pending"])
        self.assertFalse(config["accepting_writes"])
        for operation in (
            lambda: server.start(self.state, config),
            lambda: server.open_context_admission(self.state),
            lambda: server.validate(
                self.state, config, server.SUBSTRATE_INTERPRETATION, ["check"]
            ),
            lambda: server.backup(self.state, config, self.root / "backup"),
        ):
            with self.assertRaises(server.SupervisorError):
                operation()

    def test_initializer_failure_never_disposes_old_database(
        self,
    ) -> None:
        config = self.initialized()
        config.update(accepting_writes=False, admission="quiesced")
        server.write_json(self.state / "config.json", config)
        source = self.root / "authored"
        source.mkdir()
        (source / "input").write_text("external authority")
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "rotate_database_accounts"),
            patch.object(server, "start"),
            patch.object(server, "stop") as stop,
            patch.object(
                server,
                "run_file_effect",
                return_value=1,
            ),
            patch.object(server, "validate_interpretation") as validate,
            patch.object(server, "discard_database") as discard,
            self.assertRaisesRegex(server.SupervisorError, "initializer failed"),
        ):
            server.rebuild(self.state, ["initializer"])
        self.assertEqual(stop.call_count, 2)
        self.assertFalse((self.state / ".maintenance-listener.json").exists())
        validate.assert_not_called()
        discard.assert_not_called()
        config = server.config_for(self.state)
        self.assertTrue(config["derived_rebuild_pending"])
        self.assertFalse(config["accepting_writes"])
        self.assertEqual((source / "input").read_text(), "external authority")

    def test_maintenance_domain_context_refuses_dead_generation_before_rpc(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "cleanup")
        owner = {
            "nonce": "actual-generation",
            "pid": os.getpid(),
            "start": "stale-start",
        }
        server.write_json(self.state / "lifecycle-owner.json", owner)
        before = (self.state / "config.json").read_bytes()
        with (
            patch.object(server, "reservation_live", return_value=False) as live,
            patch.object(server, "maintenance_domain_call") as domain,
            self.assertRaisesRegex(server.SupervisorError, "actual live generation"),
        ):
            server.maintenance_domain_context(
                self.state, config, {}, "rotate", "source", "source"
            )
        live.assert_called_once_with(owner)
        domain.assert_not_called()
        self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_maintenance_domain_context_requires_exact_persisted_phase_and_intent(
        self,
    ) -> None:
        original, _before, _after = self.maintenance_fixture("rebuild", "cleanup")
        for nonce in (None, "a" * 32):
            for changed in ({"phase": "disposing"}, {"kind": "current-restore"}):
                with self.subTest(nonce=nonce, changed=changed):
                    config = json.loads(json.dumps(original))
                    server.write_json(self.state / "config.json", config)
                    pending = server.object_mapping(config["derived_rebuild_pending"])
                    if nonce is not None:
                        pending["domain_nonce"] = nonce
                        server.write_json(self.state / "config.json", config)
                    persisted = json.loads(json.dumps(config))
                    server.object_mapping(persisted["derived_rebuild_pending"]).update(
                        changed
                    )
                    server.write_json(self.state / "config.json", persisted)
                    before = (self.state / "config.json").read_bytes()
                    with (
                        server.lifecycle_reservation(self.state),
                        patch.object(server, "maintenance_domain_call") as domain,
                        self.assertRaisesRegex(
                            server.SupervisorError, "phase.*changed|intent changed"
                        ),
                    ):
                        server.maintenance_domain_context(
                            self.state, config, {}, "rotate", "source", "source"
                        )
                    domain.assert_not_called()
                    self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_maintenance_domain_context_retains_nonce_across_lost_ack_and_generation_retry(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "cleanup")
        current: dict[str, object] | None = None
        claims: list[list[object]] = []
        lost_ack = True

        def domain(
            _state: Path,
            _config: dict[str, object],
            _credentials: dict[str, object] | None,
            function: str,
            arguments: list[object],
        ) -> object:
            nonlocal current, lost_ack
            if function == "current":
                self.assertEqual(arguments, [])
                return dict(current) if current else None
            self.assertEqual(function, "claim")
            self.assertEqual(arguments[5], current)
            pending = server.object_mapping(
                server.config_for(self.state)["derived_rebuild_pending"]
            )
            self.assertTrue(
                str(arguments[0]).startswith(
                    str(pending["domain_nonce"]) + ":rotate:source:source:"
                )
            )
            current = dict(
                zip(("operation", "generation", "phase"), arguments[:3], strict=True)
            )
            claims.append(arguments)
            if lost_ack:
                lost_ack = False
                raise server.SupervisorError("lost original acknowledgment")
            return dict(current)

        with patch.object(server, "maintenance_domain_call", side_effect=domain):
            with (
                server.lifecycle_reservation(self.state),
                self.assertRaisesRegex(
                    server.SupervisorError, "lost original acknowledgment"
                ),
            ):
                server.maintenance_domain_context(
                    self.state, config, {}, "rotate", "source", "source"
                )
            retained_nonce = server.object_mapping(
                server.config_for(self.state)["derived_rebuild_pending"]
            )["domain_nonce"]
            resumed = server.config_for(self.state)
            with server.lifecycle_reservation(self.state):
                generation = server.read_json(self.state / "lifecycle-owner.json")[
                    "nonce"
                ]
                context = server.maintenance_domain_context(
                    self.state, resumed, {}, "rotate", "source", "source"
                )
        self.assertEqual(len(claims), 2)
        self.assertEqual(claims[0][0], claims[1][0])
        self.assertNotEqual(claims[0][1], claims[1][1])
        self.assertEqual(context, (claims[1][0], generation, "cleanup"))
        self.assertEqual(
            server.object_mapping(
                server.config_for(self.state)["derived_rebuild_pending"]
            )["domain_nonce"],
            retained_nonce,
        )

    def test_maintenance_domain_context_refuses_nonexact_claim_acknowledgment(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "cleanup")
        for fault in ("operation", "generation", "phase", "extra", "nonobject"):

            def domain(
                _state: Path,
                _config: dict[str, object],
                _credentials: dict[str, object] | None,
                function: str,
                arguments: list[object],
                *,
                fault: str = fault,
            ) -> object:
                if function == "current":
                    return None
                self.assertEqual(function, "claim")
                acknowledgment = dict(
                    zip(
                        ("operation", "generation", "phase"), arguments[:3], strict=True
                    )
                )
                if fault == "nonobject":
                    return True
                acknowledgment["unknown" if fault == "extra" else fault] = "different"
                return acknowledgment

            with (
                self.subTest(fault=fault),
                server.lifecycle_reservation(self.state),
                patch.object(server, "maintenance_domain_call", side_effect=domain),
                self.assertRaisesRegex(
                    server.SupervisorError, "generation was not acknowledged"
                ),
            ):
                server.maintenance_domain_context(
                    self.state, config, {}, "rotate", "source", "source"
                )
        self.assertFalse(server.config_for(self.state)["accepting_writes"])

    def test_maintenance_domain_context_rejects_malformed_current_before_claim(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "cleanup")
        for previous in (
            True,
            {},
            {"operation": "old", "generation": "old"},
            {"operation": "old", "generation": "", "phase": "cleanup"},
            {
                "operation": "old",
                "generation": "old",
                "phase": "cleanup",
                "unknown": "value",
            },
        ):
            with (
                self.subTest(previous=previous),
                server.lifecycle_reservation(self.state),
                patch.object(
                    server, "maintenance_domain_call", return_value=previous
                ) as domain,
                self.assertRaisesRegex(
                    server.SupervisorError, "Malformed current maintenance generation"
                ),
            ):
                server.maintenance_domain_context(
                    self.state, config, {}, "rotate", "source", "source"
                )
            domain.assert_called_once_with(self.state, config, {}, "current", [])
        self.assertFalse(server.config_for(self.state)["accepting_writes"])

    def test_maintenance_domain_validation_refuses_pending_intent_missing_from_snapshot(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "cleanup")
        stale = {
            key: value
            for key, value in config.items()
            if key != "derived_rebuild_pending"
        }
        retained = (self.state / "config.json").read_bytes()
        with (
            server.lifecycle_reservation(self.state),
            patch.object(server, "maintenance_domain_call") as domain,
            self.assertRaisesRegex(server.SupervisorError, "phase changed"),
        ):
            server.maintenance_domain_context(
                self.state, stale, None, "validation", "source", "source"
            )
        domain.assert_not_called()
        self.assertEqual((self.state / "config.json").read_bytes(), retained)

    def test_generated_maintenance_adapter_is_closed_and_preserves_sole_ack(
        self,
    ) -> None:
        config = self.initialized()
        arguments: list[object] = [
            'quoted"source',
            "generation",
            "cleanup",
            "rotate",
            "",
        ]
        with (
            patch.object(server, "payload_expiry", return_value=123456),
            patch.object(
                server,
                "maintenance_query",
                return_value={"value": {"count": 0, "after": "", "complete": True}},
            ) as query,
        ):
            value = server.maintenance_domain_call(
                self.state, config, {}, "page", arguments
            )
        self.assertEqual(value, {"count": 0, "after": "", "complete": True})
        self.assertEqual(
            query.call_args.args[3],
            "LET $domain=fn::pse_maintenance_v1::page(123456,"
            + ",".join(json.dumps(argument) for argument in arguments)
            + "); LET $value=IF $domain=NONE {NULL} ELSE {$domain};",
        )
        with (
            patch.object(server, "maintenance_query") as query,
            patch.object(server, "payload_query") as payload,
            self.assertRaisesRegex(
                server.SupervisorError, "Unknown generated maintenance operation"
            ),
        ):
            server.maintenance_domain_call(
                self.state, config, {}, "unowned_mutation", []
            )
        query.assert_not_called()
        payload.assert_not_called()

    def test_generated_claim_preserves_absent_or_exact_original_predecessor(
        self,
    ) -> None:
        config = self.initialized()
        expected = {
            "operation": "operation",
            "generation": "generation",
            "phase": "copy",
        }
        for previous in (None, expected):
            with (
                self.subTest(previous=previous),
                patch.object(server, "payload_expiry", return_value=123456),
                patch.object(
                    server, "maintenance_query", return_value={"value": expected}
                ) as query,
            ):
                acknowledged = server.maintenance_domain_call(
                    self.state,
                    config,
                    {},
                    "claim",
                    ["operation", "generation", "copy", "NONE", "target", previous],
                )
            self.assertEqual(acknowledged, expected)
            predecessor = "NONE" if previous is None else json.dumps(previous)
            self.assertEqual(
                query.call_args.args[3],
                'LET $domain=fn::pse_maintenance_v1::claim(123456,"operation",'
                '"generation","copy","NONE","target",'
                + predecessor
                + "); LET $value=IF $domain=NONE {NULL} ELSE {$domain};",
            )

    def test_generated_domain_absence_is_explicit_and_missing_ack_refuses(self) -> None:
        config = self.initialized()
        with patch.object(
            server, "payload_query", return_value={"value": None}
        ) as query:
            self.assertIsNone(
                server.maintenance_domain_call(self.state, config, None, "current", [])
            )
        self.assertIn(
            "LET $domain=fn::pse_maintenance_v1::current(", query.call_args.args[2]
        )
        self.assertTrue(
            query.call_args.args[2].endswith(
                " RETURN {value:IF $domain=NONE {NULL} ELSE {$domain}};"
            )
        )
        for malformed in (None, {}, {"value": None, "extra": True}, [None]):
            with (
                self.subTest(malformed=malformed),
                patch.object(server, "payload_query", return_value=malformed),
                self.assertRaisesRegex(server.SupervisorError, "acknowledgment"),
            ):
                server.maintenance_domain_call(self.state, config, None, "current", [])
        with (
            patch.object(server, "maintenance_query", return_value={}),
            self.assertRaisesRegex(server.SupervisorError, "acknowledgment"),
        ):
            server.maintenance_domain_call(self.state, config, {}, "current", [])

    def test_validation_payload_reads_uses_exact_closed_profile_and_generated_ack(
        self,
    ) -> None:
        config = self.initialized()
        config.update(
            admission="validation_required",
            accepting_writes=False,
            payload_frozen=True,
            interpretation=server.SUBSTRATE_INTERPRETATION,
            schema_interpretation=server.SUBSTRATE_INTERPRETATION,
        )
        server.write_json(self.state / "config.json", config)
        context = ("operation", "actual-generation", "validation")
        with (
            patch.object(
                server, "maintenance_domain_context", return_value=context
            ) as claim,
            patch.object(
                server, "maintenance_domain_call", return_value=True
            ) as domain,
        ):
            server.validation_payload_reads(
                self.state, config, server.SUBSTRATE_INTERPRETATION
            )
        incarnation = config["payload_incarnation"]
        claim.assert_called_once_with(
            self.state, config, None, "validation", str(incarnation), str(incarnation)
        )
        domain.assert_called_once_with(
            self.state,
            config,
            None,
            "validation_reads",
            [*context, server.SUBSTRATE_INTERPRETATION],
        )
        for acknowledgment in (False, None, 1, "true", {"value": True}):
            with (
                self.subTest(acknowledgment=acknowledgment),
                patch.object(
                    server, "maintenance_domain_context", return_value=context
                ),
                patch.object(
                    server, "maintenance_domain_call", return_value=acknowledgment
                ),
                self.assertRaisesRegex(server.SupervisorError, "not acknowledged"),
            ):
                server.validation_payload_reads(
                    self.state, config, server.SUBSTRATE_INTERPRETATION
                )
        for changes in (
            {"accepting_writes": True},
            {"admission": "quiesced"},
            {"payload_frozen": False},
            {"derived_rebuild_pending": {"phase": "copy"}},
            {"surrealkit_rollout_pending": {"phase": "pending"}},
            {"interpretation": "another", "schema_interpretation": "another"},
            {"namespace": "replacement"},
            {"database": "replacement"},
            {"payload_incarnation": "11111111-1111-4111-8111-111111111111"},
            {"instance_id": "replacement"},
        ):
            with self.subTest(changes=changes):
                server.write_json(self.state / "config.json", {**config, **changes})
                with (
                    patch.object(server, "maintenance_domain_context") as claim,
                    patch.object(server, "maintenance_domain_call") as domain,
                    self.assertRaisesRegex(
                        server.SupervisorError, "exact closed restored profile"
                    ),
                ):
                    server.validation_payload_reads(
                        self.state, config, server.SUBSTRATE_INTERPRETATION
                    )
                claim.assert_not_called()
                domain.assert_not_called()
        server.write_json(self.state / "config.json", config)
        self.assertFalse(server.config_for(self.state)["accepting_writes"])

    def test_validation_payload_read_lifetime_recloses_and_drains_after_uncertain_admission_or_validator_failure(
        self,
    ) -> None:
        config = self.initialized()
        config.update(
            admission="validation_required",
            accepting_writes=False,
            payload_frozen=True,
            interpretation=server.SUBSTRATE_INTERPRETATION,
            schema_interpretation=server.SUBSTRATE_INTERPRETATION,
        )
        server.write_json(self.state / "config.json", config)
        for failure in ("admission", "validator"):
            events: list[str] = []

            def domain(
                *_arguments: object, failure: str = failure, events: list[str] = events
            ) -> bool:
                events.append("read-admission")
                if failure == "admission":
                    raise server.SupervisorError("uncertain generated acknowledgment")
                return True

            def validator(
                *_arguments: object, events: list[str] = events, **_options: object
            ) -> int:
                events.append("validator")
                raise subprocess.TimeoutExpired(["original-validator"], 120)

            def freeze(
                _state: Path,
                _config: dict[str, object],
                body: str,
                *,
                events: list[str] = events,
            ) -> bool:
                events.append("freeze")
                self.assertTrue(body.startswith("RETURN fn::pse_files_v1::freeze("))
                return True

            with (
                self.subTest(failure=failure),
                patch.object(server, "start"),
                patch.object(server, "validate_interpretation"),
                patch.object(
                    server,
                    "maintenance_domain_context",
                    return_value=("operation", "generation", "validation"),
                ),
                patch.object(server, "maintenance_domain_call", side_effect=domain),
                patch.object(server, "run_file_effect", side_effect=validator) as run,
                patch.object(server, "payload_query", side_effect=freeze),
                patch.object(
                    server,
                    "all_contexts_drained",
                    side_effect=lambda *_arguments, events=events: events.append(
                        "drain"
                    ),
                ),
                patch.object(
                    server,
                    "reap_payload_owners",
                    side_effect=lambda *_arguments, events=events, **_options: (
                        events.append("reap")
                    ),
                ) as reap,
                patch.object(
                    server,
                    "stop",
                    side_effect=lambda *_arguments, events=events: events.append(
                        "stop"
                    ),
                ),
                self.assertRaises((server.SupervisorError, subprocess.TimeoutExpired)),
            ):
                server.validate(
                    self.state,
                    config,
                    server.SUBSTRATE_INTERPRETATION,
                    ["original-validator"],
                )
            self.assertEqual(
                events,
                [
                    "drain",
                    "read-admission",
                    *(["validator"] if failure == "validator" else []),
                    "freeze",
                    "drain",
                    "reap",
                    "stop",
                ],
            )
            self.assertEqual(run.call_count, int(failure == "validator"))
            reap.assert_called_once_with(self.state, config, require_all=True)
            retained = server.config_for(self.state)
            self.assertEqual(retained["admission"], "validation_required")
            self.assertFalse(retained["accepting_writes"])
            self.assertTrue(retained["payload_frozen"])

    def test_validation_payload_closure_requires_ack_before_draining_or_reaping(
        self,
    ) -> None:
        config = self.initialized()
        for acknowledgment in (False, None, 1, "true"):
            with (
                self.subTest(acknowledgment=acknowledgment),
                patch.object(server, "payload_query", return_value=acknowledgment),
                patch.object(server, "all_contexts_drained") as drain,
                patch.object(server, "reap_payload_owners") as reap,
                self.assertRaisesRegex(
                    server.SupervisorError, "closure was not acknowledged"
                ),
            ):
                server.close_validation_payload_reads(self.state, config)
            drain.assert_not_called()
            reap.assert_not_called()

    def test_payload_rehome_refuses_incomplete_ack_and_keeps_original_backup(
        self,
    ) -> None:
        backup = self.backup_fixture()
        selected = server.fresh_database_identity(server.config_for(self.state))
        selected["derived_rebuild_pending"] = {
            "backup": str(backup),
            "backup_digest": server.file_digest(backup / "backup.json"),
            "kind": "current-restore",
            "phase": "copy",
        }
        server.write_json(self.state / "config.json", selected)
        original = (backup / "backup.json").read_bytes()
        context = ("original-operation", "actual-generation", "copy")
        for acknowledgment in (False, None, 1, "true"):
            with (
                self.subTest(acknowledgment=acknowledgment),
                patch.object(
                    server, "maintenance_domain_context", return_value=context
                ),
                patch.object(
                    server,
                    "verify_payload_inventory",
                    wraps=server.verify_payload_inventory,
                ) as verify,
                patch.object(
                    server,
                    "maintenance_domain_call",
                    side_effect=[
                        {"count": 0, "after": "", "complete": True},
                        acknowledgment,
                    ],
                ) as domain,
                server.lifecycle_reservation(self.state),
                self.assertRaisesRegex(
                    server.SupervisorError, "incarnation was not acknowledged"
                ),
            ):
                server.rehome_restored_payloads(self.state, selected, {})
            verify.assert_called_once()
            self.assertEqual(
                domain.call_args_list[0].args[-2:], ("page", [*context, "rehome", ""])
            )
            self.assertEqual(
                domain.call_args_list[1].args[-2:], ("rehome_complete", list(context))
            )
            self.assertEqual((backup / "backup.json").read_bytes(), original)
            self.assertEqual(
                server.object_mapping(
                    server.config_for(self.state)["derived_rebuild_pending"]
                )["phase"],
                "copy",
            )
            self.assertFalse(server.config_for(self.state)["accepting_writes"])

    def test_bounded_authority_rotation_preserves_admitted_analyses_and_borrowers(
        self,
    ) -> None:
        config = self.initialized()
        config["derived_rebuild_pending"] = {
            "payload_incarnation": config["payload_incarnation"]
        }
        context = ("original-operation", "actual-generation", "cleanup")
        with (
            patch.object(server, "maintenance_domain_context", return_value=context),
            patch.object(
                server,
                "maintenance_domain_call",
                side_effect=[
                    {"count": 1, "after": "retention:source", "complete": False},
                    {"count": 0, "after": "retention:source", "complete": True},
                ],
            ) as query,
        ):
            server.rotate_source_authorities(self.state, config, {})
        self.assertEqual(query.call_count, 2)
        self.assertEqual(
            query.call_args_list[0].args[-2:], ("page", [*context, "rotate", ""])
        )
        self.assertEqual(
            query.call_args_list[1].args[-2:],
            ("page", [*context, "rotate", "retention:source"]),
        )

    def test_authority_rotation_refuses_live_protection_and_invalid_page_acknowledgment(
        self,
    ) -> None:
        config = self.initialized()
        config["derived_rebuild_pending"] = {
            "payload_incarnation": config["payload_incarnation"]
        }
        with (
            patch.object(
                server,
                "maintenance_domain_context",
                return_value=("operation", "generation", "cleanup"),
            ),
            patch.object(
                server,
                "maintenance_domain_call",
                side_effect=server.SupervisorError("live protections"),
            ) as query,
            self.assertRaisesRegex(server.SupervisorError, "live protections"),
        ):
            server.rotate_source_authorities(self.state, config, {})
        self.assertEqual(query.call_count, 1)
        for value in (
            {"count": -1, "after": "retention:source", "complete": False},
            {"count": 65, "after": "retention:source", "complete": False},
            {"count": True, "after": "retention:source", "complete": False},
            {"count": "0", "after": "retention:source", "complete": True},
            {"count": 1, "after": "", "complete": False},
            {"count": 0, "after": "", "complete": False},
            {"count": 0, "after": "", "complete": True, "unknown": True},
        ):
            with self.subTest(value=value), self.assertRaises(server.SupervisorError):
                server.maintenance_domain_page(value, "")
        self.assertEqual(
            server.maintenance_domain_page(
                {"count": 0, "after": "retention:source", "complete": True},
                "retention:source",
            ),
            (0, "retention:source", True),
        )

    def test_payload_rehome_mutations_keep_sole_acknowledgment_across_same_intent_retry(
        self,
    ) -> None:
        backup = self.backup_fixture()
        original = server.config_for(self.state)
        selected = server.fresh_database_identity(original)
        selected["derived_rebuild_pending"] = {
            "backup": str(backup),
            "backup_digest": server.file_digest(backup / "backup.json"),
            "kind": "current-restore",
            "phase": "copy",
        }
        server.write_json(self.state / "config.json", selected)
        bodies: list[str] = []
        page = 0
        key = "11111111-1111-4111-8111-111111111111"

        def sql(
            _state: Path,
            _config: dict[str, object],
            _credentials: dict[str, object],
            body: str,
        ) -> tuple[str, subprocess.CompletedProcess[str]]:
            nonlocal page
            bodies.append(body)
            value: object = True
            if "fn::pse_maintenance_v1::page(" in body:
                value = (
                    {"count": 1, "after": key, "complete": False}
                    if page % 2 == 0
                    else {"count": 0, "after": key, "complete": True}
                )
                page += 1
            # Pinned UPDATE defaults RETURN AFTER. Intermediate mutation rows
            # would violate the unchanged strict one-ACK production decoder.
            extra = [
                {"id": "unexpected-mutation-result"}
                for statement in body.split(";")
                if "UPDATE ONLY" in statement and "RETURN NONE" not in statement
            ]
            nonce = "exact-maintenance-output-contract"
            return nonce, subprocess.CompletedProcess(
                [],
                0,
                json.dumps(
                    [None, *extra, {"maintenance": nonce, "value": value}, None]
                ),
                "",
            )

        with (
            patch.object(server, "maintenance_sql", side_effect=sql),
            patch.object(
                server,
                "maintenance_domain_context",
                return_value=("operation", "generation", "copy"),
            ),
            server.lifecycle_reservation(self.state),
        ):
            for _retry in range(2):
                server.rehome_restored_payloads(self.state, selected, {})
        self.assertTrue(any("fn::pse_maintenance_v1::page(" in body for body in bodies))
        self.assertTrue(
            any("fn::pse_maintenance_v1::rehome_complete(" in body for body in bodies)
        )
        self.assertFalse(any("UPDATE" in body or "DELETE" in body for body in bodies))
        self.assertEqual(
            server.object_mapping(
                server.config_for(self.state)["derived_rebuild_pending"]
            )["phase"],
            "copy",
        )
        self.assertEqual(
            server.config_for(self.state)["payload_incarnation"],
            selected["payload_incarnation"],
        )
        self.assertTrue(server.config_for(self.state)["payload_frozen"])

    def test_maintenance_query_requires_exact_ack_and_keeps_secrets_out_of_arguments(
        self,
    ) -> None:
        config = self.initialized()
        credentials = server.read_json(self.state / "credentials.json")

        def acknowledged(
            arguments: list[str], **options: object
        ) -> subprocess.CompletedProcess[str]:
            self.assertIn("http://127.0.0.1:18080", arguments)
            self.assertNotIn(credentials["password"], arguments)
            environment = server.object_mapping(options["env"])
            self.assertEqual(environment["SURREAL_PASS"], credentials["password"])
            self.assertNotIn("SURREAL_INJECTED", environment)
            self.assertTrue("SURREAL_GRPC_MAX_MESSAGE_SIZE" in environment)
            query = options["input"]
            assert isinstance(query, str)
            nonce = query.split("maintenance: '")[1].split("'")[0]
            self.assertTrue(query.startswith("BEGIN; "))
            self.assertTrue(query.endswith("COMMIT;\n"))
            self.assertIn("--log=none", arguments)
            prompt = f"{config['namespace']}/{config['database']}> "
            return subprocess.CompletedProcess(
                arguments,
                0,
                prompt
                + json.dumps([None, {"maintenance": nonce, "value": True}, None])
                + "\n\n"
                + prompt,
                "",
            )

        with (
            patch.dict(os.environ, {"SURREAL_INJECTED": "unowned"}),
            patch.object(
                server, "maintenance_endpoint", return_value="http://127.0.0.1:18080"
            ),
            patch.object(server.subprocess, "run", side_effect=acknowledged),
        ):
            self.assertTrue(
                server.maintenance_query(
                    self.state, config, credentials, "LET $value=true;"
                )["value"]
            )

        def unbound_statement_output(
            arguments: list[str], **options: object
        ) -> subprocess.CompletedProcess[str]:
            query = options["input"]
            assert isinstance(query, str)
            nonce = query.split("maintenance: '")[1].split("'")[0]
            return subprocess.CompletedProcess(
                arguments,
                0,
                json.dumps([None, [], {"maintenance": nonce, "value": True}, None]),
                "",
            )

        # Even a matching acknowledgment cannot authorize extra statement results.
        with (
            patch.object(
                server, "maintenance_endpoint", return_value="http://127.0.0.1:18080"
            ),
            patch.object(
                server.subprocess, "run", side_effect=unbound_statement_output
            ),
            self.assertRaisesRegex(server.SupervisorError, "acknowledgment"),
        ):
            server.maintenance_query(
                self.state, config, credentials, "LET $value=true;"
            )
        for output in (
            '[null,"query failed"]',
            "[null]",
            "[]",
            "{}",
            '[{"maintenance":"wrong"}]',
            '[{"maintenance":"wrong"},true]',
            "not-json",
            "other/database> []",
            "INFO startup\npse/canonical> []",
        ):
            with (
                self.subTest(output=output),
                patch.object(
                    server,
                    "maintenance_endpoint",
                    return_value="http://127.0.0.1:18080",
                ),
                patch.object(
                    server.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess([], 0, output, ""),
                ),
                self.assertRaisesRegex(server.SupervisorError, "acknowledgment"),
            ):
                server.maintenance_query(
                    self.state, config, credentials, "LET $value=true;"
                )

    def test_maintenance_catalog_requires_exact_two_outputs_and_scope_selection(
        self,
    ) -> None:
        config = self.initialized()
        credentials = server.read_json(self.state / "credentials.json")

        def result(
            arguments: list[str], scope: str, fault: str, options: dict[str, object]
        ) -> subprocess.CompletedProcess[str]:
            query = options["input"]
            assert isinstance(query, str)
            self.assertIn(f"INFO FOR {scope}; LET $value=true;", query)
            self.assertNotIn("=INFO FOR", query)
            nonce = query.split("maintenance: '")[1].split("'")[0]
            for flag, field, selected in (
                ("--namespace", "namespace", scope != "ROOT"),
                ("--database", "database", scope == "DB"),
            ):
                if selected:
                    self.assertEqual(
                        arguments[arguments.index(flag) + 1], config[field]
                    )
                else:
                    self.assertNotIn(flag, arguments)
            catalog: object = self.empty_catalog(scope)
            ack: dict[str, object] = {"maintenance": nonce, "value": True}
            if fault == "wrong-nonce":
                ack["maintenance"] = "wrong"
            elif fault == "extra-ack-field":
                ack["unexpected"] = True
            elif fault == "missing-ack-value":
                del ack["value"]
            elif fault == "false-ack":
                ack["value"] = False
            elif fault == "missing-field":
                catalog = {"tables": {}}
            elif fault == "unknown-field":
                catalog = {**self.empty_catalog(scope), "future": {}}
            elif fault == "nonobject-field":
                catalog = {**self.empty_catalog(scope), "tables": []}
            elif fault == "null-catalog":
                catalog = None
            outputs = [None, catalog, ack, None]
            if fault == "extra-result":
                outputs.insert(2, [])
            return subprocess.CompletedProcess(arguments, 0, json.dumps(outputs), "")

        for scope in ("ROOT", "NS", "DB"):

            def acknowledge(
                arguments: list[str], selected_scope: str = scope, **options: object
            ) -> subprocess.CompletedProcess[str]:
                return result(arguments, selected_scope, "", options)

            with (
                self.subTest(scope=scope),
                patch.object(
                    server,
                    "maintenance_endpoint",
                    return_value="http://127.0.0.1:18080",
                ),
                patch.object(server.subprocess, "run", side_effect=acknowledge),
            ):
                self.assertEqual(
                    server.maintenance_catalog(self.state, config, credentials, scope),
                    self.empty_catalog(scope),
                )
        for fault in (
            "extra-result",
            "wrong-nonce",
            "extra-ack-field",
            "missing-ack-value",
            "false-ack",
            "missing-field",
            "unknown-field",
            "nonobject-field",
            "null-catalog",
        ):

            def malformed(
                arguments: list[str], selected_fault: str = fault, **options: object
            ) -> subprocess.CompletedProcess[str]:
                return result(arguments, "DB", selected_fault, options)

            with (
                self.subTest(fault=fault),
                patch.object(
                    server,
                    "maintenance_endpoint",
                    return_value="http://127.0.0.1:18080",
                ),
                patch.object(server.subprocess, "run", side_effect=malformed),
                self.assertRaisesRegex(server.SupervisorError, "acknowledgment"),
            ):
                server.maintenance_catalog(self.state, config, credentials, "DB")
        for scope, selected in (
            ("UNKNOWN", config),
            ("NS", {**config, "namespace": None}),
            ("DB", {**config, "namespace": None}),
            ("DB", {**config, "database": None}),
        ):
            with (
                self.subTest(scope=scope, selected=selected),
                patch.object(server.subprocess, "run") as native,
                self.assertRaises(server.SupervisorError),
            ):
                server.maintenance_catalog(self.state, selected, credentials, scope)
            native.assert_not_called()

    def test_maintenance_failure_redacts_before_bounding_and_overwrites_owned_slot(
        self,
    ) -> None:
        config = self.initialized()
        credentials = server.read_json(self.state / "credentials.json")
        active_password = "active-admin-0123456789-abcdefghijklmnopqrstuv"  # noqa: S105 -- Fake fixture credential exercises boundary redaction.
        active_selection = "active-selection-ABCDEFGHIJ-0123456789abcdefghij"
        candidate_password = "candidate-admin-klmnopqrst-ABCDEFGHIJ0123456789"  # noqa: S105 -- Fake fixture credential exercises boundary redaction.
        candidate_selection = "candidate-selection-uvwxyz0123-KLMNOPQRSTabcdef"
        credentials.update(
            password=active_password, selection_password=active_selection
        )
        candidate = server.fresh_credentials()
        candidate.update(
            password=candidate_password, selection_password=candidate_selection
        )
        server.write_json(self.state / ".maintenance-credentials.json", candidate)
        secrets = (
            active_password,
            active_selection,
            candidate_password,
            candidate_selection,
        )
        limit = 128

        def boundary_output(secret: str, marker: str) -> str:
            split = len(secret) // 2
            padding = limit - (len(secret) - split) - len(marker)
            self.assertGreaterEqual(padding, 0)
            raw = "discarded-prefix:" + secret + "~" * padding + marker
            # Truncating first would retain a plaintext suffix that cannot be redacted.
            self.assertTrue(raw[-limit:].startswith(secret[split:]))
            return raw

        slot = self.state / "maintenance-cli-failure.json"
        before = set(self.state.iterdir())
        first_record: dict[str, object] | None = None
        for label, password, selection, code in (
            ("first", active_password, active_selection, 31),
            ("second", candidate_password, candidate_selection, 32),
        ):
            stdout_marker = f":{label}-stdout"
            stderr_marker = f":{label}-stderr"
            with (
                self.subTest(failure=label),
                patch.object(server, "MESSAGE_BYTES", limit),
                patch.object(
                    server,
                    "maintenance_endpoint",
                    return_value="http://127.0.0.1:18080",
                ),
                patch.object(
                    server.subprocess,
                    "run",
                    return_value=subprocess.CompletedProcess(
                        [],
                        code,
                        boundary_output(password, stdout_marker),
                        boundary_output(selection, stderr_marker),
                    ),
                ),
                self.assertRaisesRegex(
                    server.SupervisorError, "acknowledgment"
                ) as failure,
            ):
                server.maintenance_query(
                    self.state, config, credentials, "LET $value=true;"
                )
            record = server.read_json(slot)
            self.assertEqual(record["owner"], server.OWNER)
            self.assertEqual(record["kind"], "maintenance-cli-failure-v1")
            self.assertEqual(record["returncode"], code)
            for field, marker in (("stdout", stdout_marker), ("stderr", stderr_marker)):
                retained = record[field]
                assert isinstance(retained, str)
                self.assertLessEqual(len(retained), limit)
                self.assertTrue(retained.endswith(marker))
                self.assertIn("[REDACTED]", retained)
                for secret in secrets:
                    self.assertNotIn(secret, retained)
                    self.assertNotIn(secret[: len(secret) // 2], retained)
                    self.assertNotIn(secret[len(secret) // 2 :], retained)
                    self.assertNotIn(secret, str(failure.exception))
            self.assertEqual(slot.stat().st_mode & 0o777, 0o600)
            self.assertEqual(set(self.state.iterdir()) - before, {slot})
            if first_record is None:
                first_record = record
            else:
                self.assertNotEqual(record, first_record)
                self.assertNotIn("first-stdout", str(record["stdout"]))
                self.assertNotIn("first-stderr", str(record["stderr"]))

        # An unrelated producer's file never becomes an overwritable diagnostic slot.
        server.write_json(
            slot, {"owner": "unknown", "kind": "maintenance-cli-failure-v1"}
        )
        unknown = slot.read_bytes()
        with (
            patch.object(
                server, "maintenance_endpoint", return_value="http://127.0.0.1:18080"
            ),
            patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 1, "failed", "failed"),
            ),
            self.assertRaisesRegex(
                server.SupervisorError, "Unknown maintenance diagnostic"
            ),
        ):
            server.maintenance_query(
                self.state, config, credentials, "LET $value=true;"
            )
        self.assertEqual(slot.read_bytes(), unknown)
        self.assertEqual(set(self.state.iterdir()) - before, {slot})

    def test_maintenance_failure_refuses_nonregular_and_oversized_slot_before_read(
        self,
    ) -> None:
        self.initialized()
        credentials = server.read_json(self.state / "credentials.json")
        slot = self.state / "maintenance-cli-failure.json"
        for kind in ("fifo", "oversized"):
            if kind == "fifo":
                os.mkfifo(slot, 0o600)
            else:
                slot.write_bytes(b"unknown retained producer")
                slot.chmod(0o600)
                with slot.open("r+b") as retained:
                    retained.truncate(24 * 128 + 4096)
            before = slot.stat()
            with (
                self.subTest(kind=kind),
                patch.object(server, "MESSAGE_BYTES", 128),
                patch.object(
                    server,
                    "read_json",
                    side_effect=AssertionError("unknown slot was read"),
                ) as read,
                self.assertRaisesRegex(
                    server.SupervisorError, "Unknown maintenance diagnostic"
                ),
            ):
                server.maintenance_failure(
                    self.state,
                    credentials,
                    subprocess.CompletedProcess([], 1, "failed", "failed"),
                )
            read.assert_not_called()
            after = slot.stat()
            self.assertEqual(
                (after.st_dev, after.st_ino, after.st_size, after.st_mode),
                (before.st_dev, before.st_ino, before.st_size, before.st_mode),
            )
            if kind == "fifo":
                self.assertTrue(slot.is_fifo())
            else:
                with slot.open("rb") as retained:
                    self.assertEqual(retained.read(25), b"unknown retained producer")
            slot.unlink()

    def test_current_restore_retains_readable_active_analysis_and_future_artifact_borrowers(
        self,
    ) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        retained: dict[str, list[dict[str, object]]] = {
            "canonical_analyses": [
                {
                    "key": "admitted-analysis",
                    "active": True,
                    "retiring": False,
                    "primary_authority": "original-source-authority",
                    "configuration_hex": "00ff01",
                }
            ],
            "canonical_analysis_nodes": [
                {
                    "key": "analysis-node",
                    "analysis": "admitted-analysis",
                    "payload_hex": "8000000000000000",
                }
            ],
            "canonical_analysis_edges": [
                {
                    "key": "analysis-edge",
                    "analysis": "admitted-analysis",
                    "source": "analysis-node",
                    "target": "analysis-node",
                }
            ],
            "canonical_analysis_inputs": [
                {
                    "key": "future-input",
                    "analysis": "admitted-analysis",
                    "artifact": "independent-artifact",
                }
            ],
            "canonical_roots": [
                {
                    "key": "analysis-root",
                    "owner_kind": "analysis",
                    "owner": "admitted-analysis",
                    "problem": "original-problem",
                }
            ],
            "canonical_file_borrowers": [
                {
                    "key": "future-file-borrow",
                    "owner_kind": "analysis",
                    "owner": "admitted-analysis",
                    "instance": "independent-artifact-file",
                    "released": False,
                }
            ],
        }
        with self.native_maintenance(retained_science=retained):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
            selected = server.config_for(restored)
            credentials = server.read_json(restored / "credentials.json")
            for table, rows in retained.items():
                with self.subTest(table=table):
                    read = server.maintenance_query(
                        restored,
                        selected,
                        credentials,
                        "LET $value=SELECT * FROM " + table + ";",  # noqa: S608
                        # Table names come from this explicit fixture inventory.
                    )
                    self.assertEqual(read["value"], rows)
            self.assertEqual(selected["admission"], "validation_required")
            self.assertFalse(selected["accepting_writes"])

    def test_restore_copy_failure_retains_pending_identity_and_original_backup(
        self,
    ) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        with (
            self.native_maintenance(),
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "rotate_source_authorities"),
            patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 1),
            ),
            patch.object(server, "discard_database") as discard,
            self.assertRaisesRegex(server.SupervisorError, "copy failed"),
        ):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        discard.assert_not_called()
        self.assertTrue(server.config_for(restored)["derived_rebuild_pending"])
        self.assertFalse(server.config_for(restored)["accepting_writes"])
        self.assertEqual(
            (backup / "database/fixture").read_bytes(),
            b"acknowledged fixed operation identity",
        )

    def test_restore_authority_rotation_failure_is_durably_fenced_before_catalog_disposal(
        self,
    ) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"

        def refuse(
            state: Path, _config: dict[str, object], _credentials: dict[str, object]
        ) -> None:
            pending = server.config_for(state)
            self.assertTrue(pending["derived_rebuild_pending"])
            self.assertFalse(pending["accepting_writes"])
            with self.assertRaises(server.SupervisorError):
                server.open_context_admission(state)
            raise server.SupervisorError("live protection")

        with (
            self.native_maintenance(),
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "rotate_source_authorities", side_effect=refuse),
            patch.object(server, "discard_database") as discard,
            self.assertRaisesRegex(server.SupervisorError, "live protection"),
        ):
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        discard.assert_not_called()
        self.assertTrue(server.config_for(restored)["derived_rebuild_pending"])

    def test_account_rotation_uncertain_ack_retains_private_recovery_credentials(
        self,
    ) -> None:
        config = self.initialized()
        old = server.read_json(self.state / "credentials.json")
        new = server.fresh_credentials()
        with (
            patch.object(server, "require_private_catalog"),
            patch.object(
                server,
                "maintenance_query",
                side_effect=server.SupervisorError("uncertain"),
            ),
            self.assertRaisesRegex(server.SupervisorError, "uncertain"),
        ):
            server.rotate_database_accounts(self.state, config, old, new)
        self.assertEqual(server.read_json(self.state / "credentials.json"), old)
        self.assertEqual(
            server.read_json(self.state / ".maintenance-credentials.json"), new
        )
        self.assertEqual(
            (self.state / ".maintenance-credentials.json").stat().st_mode & 0o777, 0o600
        )

    def maintenance_fixture(
        self, kind: str, phase: str
    ) -> tuple[dict[str, object], dict[str, object], dict[str, object]]:
        old = self.initialized()
        source_backup = None
        if kind == "current-restore":
            old.update(
                interpretation=server.SUBSTRATE_INTERPRETATION,
                schema_interpretation=server.SUBSTRATE_INTERPRETATION,
            )
            server.write_json(self.state / "config.json", old)
            source_backup = self.root / "maintenance-backup"
            with (
                patch.object(server, "freeze_payloads", return_value=[]),
                patch.object(
                    server,
                    "capture_application_inventory",
                    side_effect=self.application_inventory_fixture,
                ),
            ):
                server.backup(self.state, old, source_backup)
        before = server.read_json(self.state / "credentials.json")
        after = server.fresh_credentials()
        config = server.fresh_database_identity(old)
        config.update(
            interpretation=server.SUBSTRATE_INTERPRETATION,
            schema_interpretation=server.SUBSTRATE_INTERPRETATION,
        )
        pending = {
            "namespace": old["namespace"],
            "database": old["database"],
            "kind": kind,
            "phase": phase,
            "target_namespace": config["namespace"],
            "target_database": config["database"],
            "previous_username": before["username"],
            "next_username": after["username"],
        }
        if source_backup is not None:
            pending.update(
                backup=str(source_backup),
                backup_digest=server.file_digest(source_backup / "backup.json"),
                payload_incarnation=old["payload_incarnation"],
            )
        config["derived_rebuild_pending"] = pending
        server.write_json(self.state / "config.json", config)
        server.write_json(
            self.state / "credentials.json",
            after if phase in {"initializing", "validated", "disposing"} else before,
        )
        return config, before, after

    def test_maintenance_payload_evidence_survives_target_remint_and_publish_failure(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        payloads: dict[str, object] = {
            "payload_inventory": [{"files": {}, "context": "x" * 2048}] * 49,
            "payload_targets": [],
            "retired_payloads": [],
        }
        with server.lifecycle_reservation(self.state):
            server.write_maintenance_payloads(self.state, config, payloads)
            pending = server.object_mapping(config["derived_rebuild_pending"])
            first = self.state / str(
                server.object_mapping(pending["payload_retirement"])["path"]
            )
            self.assertGreater(first.stat().st_size, 64 * 1024)
            self.assertLess((self.state / "config.json").stat().st_size, 64 * 1024)
            selected = server.fresh_database_identity(config)
            selected["derived_rebuild_pending"] = {
                **pending,
                "initializer": {"nonce": "replacement-owner"},
                "target_namespace": selected["namespace"],
                "target_database": selected["database"],
            }
            server.write_maintenance_payloads(self.state, selected, payloads)
            self.assertFalse(first.exists())
            pending = server.object_mapping(selected["derived_rebuild_pending"])
            self.assertEqual(server.maintenance_payloads(self.state, pending), payloads)
            second = self.state / str(
                server.object_mapping(pending["payload_retirement"])["path"]
            )
            write = server.write_json

            def fail_config(path: Path, value: Mapping[str, object]) -> None:
                if path == self.state / "config.json":
                    raise OSError("config publication interrupted")
                write(path, value)

            with (
                patch.object(server, "write_json", side_effect=fail_config),
                self.assertRaisesRegex(OSError, "publication interrupted"),
            ):
                server.write_maintenance_payloads(self.state, selected, payloads)
            current = server.object_mapping(
                server.config_for(self.state)["derived_rebuild_pending"]
            )
            self.assertTrue(second.exists())
            self.assertEqual(server.maintenance_payloads(self.state, current), payloads)
            orphans = set(self.state.glob(".maintenance-payloads-*.json")) - {second}
            self.assertEqual(len(orphans), 1)
            # Continue from the durable pointer, not the failed in-memory publication.
            selected = server.config_for(self.state)
            server.write_maintenance_payloads(self.state, selected, payloads)
            pending = server.object_mapping(selected["derived_rebuild_pending"])
            final = self.state / str(
                server.object_mapping(pending["payload_retirement"])["path"]
            )
            self.assertFalse(second.exists())
            with self.assertRaisesRegex(server.SupervisorError, "remains referenced"):
                server.retire_maintenance_payloads(self.state, pending)
            for changed in ({"sha256": "different"}, {"bytes": 1}):
                altered = {
                    **pending,
                    "payload_retirement": {
                        **server.object_mapping(pending["payload_retirement"]),
                        **changed,
                    },
                }
                server.write_json(
                    self.state / "config.json",
                    {**selected, "derived_rebuild_pending": altered},
                )
                with self.assertRaisesRegex(
                    server.SupervisorError, "remains referenced"
                ):
                    server.retire_maintenance_payloads(self.state, pending)
                self.assertTrue(final.exists())
            selected.pop("derived_rebuild_pending")
            server.write_json(self.state / "config.json", selected)
            server.retire_maintenance_payloads(self.state, pending)
            self.assertFalse(final.exists())
            self.assertTrue(all(path.exists() for path in orphans))

    def test_maintenance_payload_evidence_refuses_changed_source_bytes_and_links(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        payloads: dict[str, object] = {
            "payload_inventory": [],
            "payload_targets": [],
            "retired_payloads": [],
        }
        with server.lifecycle_reservation(self.state):
            server.write_maintenance_payloads(self.state, config, payloads)
        pending = server.object_mapping(config["derived_rebuild_pending"])
        reference = server.object_mapping(pending["payload_retirement"])
        path = self.state / str(reference["path"])
        raw = path.read_bytes()
        with self.assertRaisesRegex(server.SupervisorError, "source differs"):
            server.maintenance_payloads(
                self.state, {**pending, "database": "different-source"}
            )
        path.write_bytes(raw + b" ")
        with (
            patch.object(server, "database_present") as present,
            self.assertRaisesRegex(server.SupervisorError, "evidence changed"),
        ):
            server.cleanup_retired_payloads(self.state, config, {})
        present.assert_not_called()
        path.unlink()
        other = self.state / "unowned-payload-record.json"
        server.write_json(other, {"retained": True})
        path.symlink_to(other)
        with self.assertRaises(OSError):
            server.maintenance_payloads(self.state, pending)
        self.assertTrue(other.exists())

    def test_maintenance_listener_reuses_live_owner_and_drains_before_release(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(
                server, "publish_generation", return_value=config["service_supervisor"]
            ),
            patch.object(server, "start") as start,
            patch.object(server, "stop") as stop,
            patch.object(server, "active", return_value=False),
            patch.object(server, "listener_ready", return_value=True),
            server.lifecycle_reservation(self.state),
        ):
            endpoint = server.maintenance_endpoint(self.state)
            self.assertEqual(endpoint, "http://127.0.0.1:18080")
            self.assertEqual(server.maintenance_endpoint(self.state), endpoint)
            record = server.read_json(self.state / ".maintenance-listener.json")
            self.assertEqual(
                record["owner"], server.read_json(self.state / "lifecycle-owner.json")
            )
            start.assert_called_once()
            self.assertFalse(server.config_for(self.state)["accepting_writes"])
            stop.assert_not_called()
        stop.assert_called_once()
        self.assertFalse((self.state / ".maintenance-listener.json").exists())
        self.assertFalse((self.state / "lifecycle-owner.json").exists())

    def test_maintenance_owner_loss_terminates_child_without_retiring_phase(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        child = MagicMock()
        child.poll.return_value = None
        with patch.object(
            server,
            "require_maintenance_owner",
            side_effect=server.SupervisorError("owner died"),
        ):
            server.watch_maintenance_owner(self.state, child)
        child.terminate.assert_called_once()
        self.assertEqual(
            server.config_for(self.state)["derived_rebuild_pending"],
            config["derived_rebuild_pending"],
        )
        self.assertFalse(server.config_for(self.state)["accepting_writes"])

    def test_retained_zombie_owner_closes_child_and_allows_recovery_takeover(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        with subprocess.Popen(
            [sys.executable, "-c", "import sys; sys.stdin.buffer.read(1)"],
            stdin=subprocess.PIPE,
        ) as owner_process:
            owner: dict[str, object] = {
                "nonce": "retained-owner",
                "pid": owner_process.pid,
                "start": server.native_operation.start_identity(owner_process.pid),
                "boot": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
            }
            server.write_json(self.state / "lifecycle-owner.json", owner)
            server.write_json(
                self.state / ".maintenance-listener.json",
                {
                    "owner": owner,
                    "instance_id": config["instance_id"],
                    "server": config["server"],
                    "resources": config["resources"],
                    "supervisor": config["service_supervisor"],
                    "credentials_sha256": server.file_digest(
                        self.state / "credentials.json"
                    ),
                },
            )
            self.assertTrue(server.reservation_live(owner))
            self.assertFalse(server.reservation_live({**owner, "start": "changed"}))
            self.assertFalse(server.reservation_live({**owner, "boot": "changed"}))
            assert owner_process.stdin is not None
            owner_process.stdin.close()
            os.waitid(os.P_PID, owner_process.pid, os.WEXITED | os.WNOWAIT)
            self.assertEqual(
                Path(f"/proc/{owner_process.pid}/stat")
                .read_text()
                .rsplit(")", 1)[1]
                .split()[0],
                "Z",
            )
            self.assertEqual(
                server.native_operation.start_identity(owner_process.pid),
                owner["start"],
            )
            self.assertFalse(server.reservation_live(owner))
            child = MagicMock()
            child.poll.return_value = None
            server.watch_maintenance_owner(self.state, child)
            child.terminate.assert_called_once()
            with (
                patch.object(server, "active", return_value=False),
                patch.object(server, "stop") as stop,
                server.lifecycle_reservation(self.state),
            ):
                replacement = server.read_json(self.state / "lifecycle-owner.json")
                self.assertNotEqual(replacement["nonce"], owner["nonce"])
                self.assertEqual(replacement["pid"], os.getpid())
                server.drain_maintenance_listener(self.state)
                stop.assert_called_once()
                self.assertEqual(
                    server.config_for(self.state)["derived_rebuild_pending"],
                    config["derived_rebuild_pending"],
                )
                self.assertFalse(server.config_for(self.state)["accepting_writes"])
            self.assertFalse((self.state / ".maintenance-listener.json").exists())
            self.assertFalse((self.state / "lifecycle-owner.json").exists())

    def test_maintenance_failed_drain_preserves_owner_phase_and_listener(self) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(
                server, "publish_generation", return_value=config["service_supervisor"]
            ),
            patch.object(server, "start"),
            patch.object(server, "active", return_value=False),
            patch.object(
                server, "stop", side_effect=server.SupervisorError("undrained")
            ),
            self.assertRaisesRegex(server.SupervisorError, "undrained"),
            server.lifecycle_reservation(self.state),
        ):
            server.maintenance_endpoint(self.state)
        self.assertTrue((self.state / ".maintenance-listener.json").exists())
        self.assertTrue((self.state / "lifecycle-owner.json").exists())
        retained = server.config_for(self.state)
        self.assertEqual(
            server.object_mapping(retained["derived_rebuild_pending"])["phase"],
            "accounts",
        )
        self.assertFalse(retained["accepting_writes"])

    def test_maintenance_owner_refuses_changed_live_authority_and_profile(self) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(
                server, "publish_generation", return_value=config["service_supervisor"]
            ),
            patch.object(server, "start"),
            patch.object(server, "stop"),
            patch.object(server, "active", return_value=False),
            server.lifecycle_reservation(self.state),
        ):
            server.maintenance_endpoint(self.state)
            selected = server.config_for(self.state)
            for changes in (
                {"accepting_writes": True},
                {"instance_id": "other"},
                {"resources": {}},
                {"service_supervisor": {}},
            ):
                with (
                    self.subTest(changes=changes),
                    self.assertRaisesRegex(server.SupervisorError, "exact live"),
                ):
                    server.require_maintenance_owner(
                        self.state, {**selected, **changes}
                    )
            with (
                patch.object(server, "reservation_live", return_value=False),
                self.assertRaisesRegex(server.SupervisorError, "exact live"),
            ):
                server.require_maintenance_owner(self.state, selected)

    def bind_maintenance_listener(self, config: dict[str, object]) -> None:
        server.write_json(
            self.state / ".maintenance-listener.json",
            {
                "owner": server.read_json(self.state / "lifecycle-owner.json"),
                "instance_id": config["instance_id"],
                "server": config["server"],
                "resources": config["resources"],
                "supervisor": config["service_supervisor"],
                "credentials_sha256": server.file_digest(
                    self.state / "credentials.json"
                ),
            },
        )

    def test_exact_terminal_maintenance_launch_resets_only_its_owned_failure_window(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        absent = readiness_fixture(server.Readiness.ACTIONABLE_ABSENT)
        ready = server.ServiceObservation(
            server.Readiness.MAINTENANCE,
            self.state,
            config["instance_id"],
            server.unit_name(self.state),
            "owned maintenance ready",
            listener_owned=True,
        )
        with (
            server.lifecycle_reservation(self.state),
            patch.object(
                server, "service_readiness", side_effect=[absent, absent, ready]
            ),
            patch.object(server, "all_contexts_drained"),
            patch.object(server, "service_allocation", return_value=self.allocation),
            patch.object(server, "materialize_service"),
            patch.object(
                server,
                "systemctl",
                return_value=subprocess.CompletedProcess([], 0, "", ""),
            ) as manager,
        ):
            self.bind_maintenance_listener(config)
            server._start(  # noqa: SLF001 -- exercise internal maintenance startup
                self.state, config, validation=True, deadline=time.monotonic() + 30
            )
        reset = call("reset-failed", server.unit_name(self.state), check=False)
        start = call("start", server.unit_name(self.state), check=False)
        self.assertEqual(manager.call_args_list.count(reset), 1)
        self.assertLess(
            manager.call_args_list.index(reset), manager.call_args_list.index(start)
        )

    def test_ordinary_new_launch_retains_its_crash_failure_window(self) -> None:
        config = self.initialized()
        with (
            server.lifecycle_reservation(self.state),
            patch.object(
                server,
                "service_readiness",
                side_effect=[
                    readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
                    readiness_fixture(server.Readiness.READY),
                ],
            ),
            patch.object(server, "service_allocation", return_value=self.allocation),
            patch.object(server, "materialize_service"),
            patch.object(server, "establish_protocol_readiness"),
            patch.object(
                server,
                "systemctl",
                return_value=subprocess.CompletedProcess([], 0, "", ""),
            ) as manager,
        ):
            server._start(  # noqa: SLF001 -- exercise internal maintenance startup
                self.state, config, validation=False, deadline=time.monotonic() + 30
            )
        self.assertFalse(
            any(
                arguments.args[0] == "reset-failed"
                for arguments in manager.call_args_list
            )
        )

    def test_maintenance_reset_refuses_nonowned_nondrained_and_nonterminal_service(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        with server.lifecycle_reservation(self.state):
            self.bind_maintenance_listener(config)
            marker = server.read_json(self.state / ".maintenance-listener.json")
            server.write_json(
                self.state / ".maintenance-listener.json",
                {**marker, "instance_id": "another-owner"},
            )
            with (
                patch.object(server, "systemctl") as manager,
                self.assertRaisesRegex(
                    server.SupervisorError, "exact live closed owner"
                ),
            ):
                server.reset_maintenance_failure_window(
                    self.state, config, time.monotonic() + 30
                )
            manager.assert_not_called()
            server.write_json(self.state / ".maintenance-listener.json", marker)
            with (
                patch.object(
                    server,
                    "all_contexts_drained",
                    side_effect=server.SupervisorError("managed descendants remain"),
                ),
                patch.object(server, "systemctl") as manager,
                self.assertRaisesRegex(server.SupervisorError, "descendants"),
            ):
                server.reset_maintenance_failure_window(
                    self.state, config, time.monotonic() + 30
                )
            manager.assert_not_called()
            with (
                patch.object(server, "all_contexts_drained"),
                patch.object(
                    server,
                    "service_readiness",
                    return_value=readiness_fixture(server.Readiness.STOPPING),
                ),
                patch.object(server, "systemctl") as manager,
                self.assertRaisesRegex(server.SupervisorError, "drained terminal"),
            ):
                server.reset_maintenance_failure_window(
                    self.state, config, time.monotonic() + 30
                )
            manager.assert_not_called()

    def test_maintenance_reset_manager_refusal_never_launches_service(self) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        with (
            server.lifecycle_reservation(self.state),
            patch.object(server, "all_contexts_drained"),
            patch.object(
                server,
                "service_readiness",
                return_value=readiness_fixture(server.Readiness.ACTIONABLE_ABSENT),
            ),
            patch.object(
                server,
                "systemctl",
                return_value=subprocess.CompletedProcess([], 1, "", ""),
            ) as manager,
            self.assertRaisesRegex(server.SupervisorError, "reset was refused"),
        ):
            self.bind_maintenance_listener(config)
            server.reset_maintenance_failure_window(
                self.state, config, time.monotonic() + 30
            )
        self.assertEqual(
            manager.call_args_list[0],
            call("reset-failed", server.unit_name(self.state), check=False),
        )
        self.assertFalse(
            any(arguments.args[0] == "start" for arguments in manager.call_args_list)
        )

    def test_collected_initial_maintenance_reset_refusal_reauthenticates_exact_absence(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        fields = {
            "LoadState": "loaded",
            "ActiveState": "inactive",
            "ControlGroup": "",
            "InvocationID": "",
        }
        absent = server.ServiceObservation(
            server.Readiness.ACTIONABLE_ABSENT,
            self.state,
            config["instance_id"],
            server.unit_name(self.state),
            "never-started unit",
            manager=fields,
        )

        def manager(
            *arguments: str, **_kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess(
                [],
                1 if arguments[0] == "reset-failed" else 0,
                "\n".join(key + "=" + value for key, value in fields.items()),
                "",
            )

        deadline = time.monotonic() + 30
        with (
            server.lifecycle_reservation(self.state),
            patch.object(server, "all_contexts_drained") as drain,
            patch.object(server, "service_readiness", return_value=absent) as readiness,
            patch.object(
                server,
                "require_maintenance_owner",
                wraps=server.require_maintenance_owner,
            ) as authority,
            patch.object(server, "systemctl", side_effect=manager) as operations,
        ):
            self.bind_maintenance_listener(config)
            server.reset_maintenance_failure_window(self.state, config, deadline)
            self.assertEqual(drain.call_count, 2)
            self.assertEqual(authority.call_count, 3)
            self.assertTrue(
                all(
                    arguments.kwargs["deadline"] == deadline
                    for arguments in readiness.call_args_list
                )
            )
            self.assertFalse(
                any(
                    arguments.args[0] == "start"
                    for arguments in operations.call_args_list
                )
            )

    def test_maintenance_reset_absence_fallback_refuses_unknown_or_surviving_lifetime(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        for case in (
            "manager-error",
            "active",
            "group",
            "invocation",
            "missing-invocation",
            "foreign-owner",
            "changed-config",
            "changed-readiness",
        ):
            fields = {
                "LoadState": "loaded",
                "ActiveState": "inactive",
                "ControlGroup": "",
                "InvocationID": "",
            }
            if case == "active":
                fields["ActiveState"] = "active"
            elif case == "group":
                fields["ControlGroup"] = "/surviving-group"
            elif case == "invocation":
                fields["InvocationID"] = "a" * 32
            elif case == "missing-invocation":
                fields.pop("InvocationID")
            absent = server.ServiceObservation(
                server.Readiness.ACTIONABLE_ABSENT,
                self.state,
                config["instance_id"],
                server.unit_name(self.state),
                "controlled absence",
                manager=fields,
            )

            def manager(
                *arguments: str,
                case: str = case,
                fields: dict[str, str] = fields,
                **_kwargs: object,
            ) -> subprocess.CompletedProcess[str]:
                if arguments[0] == "reset-failed":
                    if case == "foreign-owner":
                        marker = server.read_json(
                            self.state / ".maintenance-listener.json"
                        )
                        server.write_json(
                            self.state / ".maintenance-listener.json",
                            {**marker, "instance_id": "other"},
                        )
                    elif case == "changed-config":
                        server.write_json(
                            self.state / "config.json",
                            {**config, "resident": not config.get("resident", False)},
                        )
                    return subprocess.CompletedProcess([], 1, "", "")
                return subprocess.CompletedProcess(
                    [],
                    1 if case == "manager-error" else 0,
                    "\n".join(key + "=" + value for key, value in fields.items()),
                    "",
                )

            with (
                self.subTest(case=case),
                server.lifecycle_reservation(self.state),
                patch.object(server, "all_contexts_drained"),
                patch.object(
                    server,
                    "service_readiness",
                    side_effect=[absent, readiness_fixture(server.Readiness.STARTING)]
                    if case == "changed-readiness"
                    else None,
                    return_value=absent,
                ),
                patch.object(server, "systemctl", side_effect=manager) as operations,
            ):
                server.write_json(self.state / "config.json", config)
                self.bind_maintenance_listener(config)
                with self.assertRaises(server.SupervisorError):
                    server.reset_maintenance_failure_window(
                        self.state, config, time.monotonic() + 30
                    )
                self.assertFalse(
                    any(
                        arguments.args[0] == "start"
                        for arguments in operations.call_args_list
                    )
                )
                # Restore fixture metadata before the enclosing normal drain.
                server.write_json(self.state / "config.json", config)
                self.bind_maintenance_listener(config)

    def test_maintenance_launch_has_no_restart_bootstrap_or_readiness_mutation(
        self,
    ) -> None:
        config, before, _after = self.maintenance_fixture("rebuild", "accounts")
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(
                server, "publish_generation", return_value=config["service_supervisor"]
            ),
            patch.object(server, "start"),
            patch.object(server, "stop"),
            patch.object(server, "active", return_value=False),
            server.lifecycle_reservation(self.state),
        ):
            server.maintenance_endpoint(self.state)
            selected = server.config_for(self.state)
            selected["resident"] = True
            with (
                patch.object(server, "recovery_qualified", return_value=True),
                patch.object(
                    server,
                    "storage_placement",
                    return_value={
                        "slice": "pse.slice",
                        "memory_bytes": server.GIB,
                        "cpu_threads": 2,
                    },
                ),
                patch.dict(
                    os.environ,
                    {
                        "XDG_CONFIG_HOME": str(self.root / "manager"),
                        "SURREAL_USER": "injected",
                        "SURREAL_PASS": "injected",
                    },
                ),
            ):
                server.materialize_service(self.state, selected, self.allocation)
                unit = self.root / "manager/systemd/user" / server.unit_name(self.state)
                self.assertIn("Restart=no\n", unit.read_text())
                environment = server.server_environment(
                    selected, before, bootstrap=False
                )
                self.assertNotIn("SURREAL_USER", environment)
                self.assertNotIn("SURREAL_PASS", environment)
                self.assertEqual(
                    environment["SURREAL_GRPC_MAX_MESSAGE_SIZE"],
                    str(server.MESSAGE_BYTES),
                )
                self.assertTrue("SURREAL_ROCKSDB_BLOCK_CACHE_SIZE" not in environment)
            with (
                patch.object(
                    server,
                    "service_readiness",
                    return_value=readiness_fixture(server.Readiness.STARTING),
                ),
                patch.object(server, "listener_ready", return_value=True),
                patch.object(server, "protocol_ready") as readiness,
                patch.object(server, "establish_protocol_readiness") as provision,
            ):
                server._start(  # noqa: SLF001 -- exercise already-active maintenance startup
                    self.state,
                    selected,
                    validation=True,
                    deadline=server.time.monotonic() + 10,
                )
                readiness.assert_not_called()
                provision.assert_not_called()
                with self.assertRaisesRegex(
                    server.SupervisorError, "ordinary startup|requires validate"
                ):
                    server._start(  # noqa: SLF001 -- ordinary startup must stay closed
                        self.state,
                        selected,
                        validation=False,
                        deadline=server.time.monotonic() + 10,
                    )

    def test_maintenance_replacement_invocation_is_preserved_without_stop(self) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(
                server, "publish_generation", return_value=config["service_supervisor"]
            ),
            patch.object(server, "start"),
            patch.object(server, "stop") as stop,
            patch.object(server, "active", return_value=False),
            server.lifecycle_reservation(self.state),
        ):
            server.maintenance_endpoint(self.state)
            for state in ("active", "activating", "deactivating", "inactive"):
                with (
                    self.subTest(state=state),
                    patch.object(
                        server,
                        "storage_launch",
                        return_value={
                            "binding": {
                                "invocation": "expected",
                                "group": "/owned",
                                "inode": [1, 2],
                            }
                        },
                    ),
                    patch.object(
                        server,
                        "systemctl",
                        return_value=subprocess.CompletedProcess(
                            [],
                            0,
                            f"LoadState=loaded\nActiveState={state}\nControlGroup=/owned\nInvocationID=replacement\n",
                            "",
                        ),
                    ),
                    self.assertRaisesRegex(
                        server.SupervisorError, "invocation changed"
                    ),
                ):
                    server.drain_maintenance_listener(self.state)
                stop.assert_not_called()
                self.assertTrue((self.state / ".maintenance-listener.json").exists())

    def test_fresh_database_retires_exact_drained_launch_and_retains_owned_unit(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        config["unit_materialized"] = True
        server.write_json(self.state / "config.json", config)
        path = self.state / "service-launch.json"
        server.write_json(path, {"generation": config["instance_id"]})
        with (
            patch.object(server, "active", return_value=False),
            patch.object(
                server,
                "storage_launch",
                return_value={"generation": config["instance_id"]},
            ) as observed,
            server.lifecycle_reservation(self.state),
        ):
            selected = server.fresh_owned_database_identity(self.state, config)
            observed.assert_called_once_with(self.state, config)
            self.assertFalse(path.exists())
            self.assertTrue(selected["unit_materialized"])
            self.assertNotEqual(selected["instance_id"], config["instance_id"])
            self.assertNotEqual(selected["namespace"], config["namespace"])
            server.write_json(path, {"generation": "unknown"})
            with (
                patch.object(
                    server,
                    "storage_launch",
                    side_effect=server.SupervisorError("unknown receipt"),
                ),
                self.assertRaisesRegex(server.SupervisorError, "unknown receipt"),
            ):
                server.fresh_owned_database_identity(self.state, config)
            self.assertTrue(path.exists())

    def test_database_disposal_selects_only_namespace_and_requires_confirmed_absence(
        self,
    ) -> None:
        config, credentials, _after = self.maintenance_fixture(
            "current-restore", "disposing"
        )
        pending = server.object_mapping(config["derived_rebuild_pending"])
        old = {
            **config,
            "namespace": pending["namespace"],
            "database": pending["database"],
        }
        retained = (self.state / "config.json").read_bytes()
        body = (
            "REMOVE DATABASE "
            + server.sql_identifier(old["database"])
            + "; LET $value=true;"
        )

        def acknowledged(
            arguments: list[str], **options: object
        ) -> subprocess.CompletedProcess[str]:
            self.assertNotIn("--database", arguments)
            self.assertEqual(
                arguments[arguments.index("--namespace") + 1], old["namespace"]
            )
            query = options["input"]
            assert isinstance(query, str)
            self.assertTrue(query.startswith("BEGIN; " + body + " RETURN "))
            self.assertNotIn(server.sql_identifier(config["database"]), query)
            nonce = query.split("maintenance: '")[1].split("'")[0]
            return subprocess.CompletedProcess(
                arguments,
                0,
                json.dumps([None, {"maintenance": nonce, "value": True}, None]),
                "",
            )

        for remains_present in (True, False):
            root = {
                **self.empty_catalog("ROOT"),
                "namespaces": {str(old["namespace"]): {}},
            }
            namespace = {
                **self.empty_catalog("NS"),
                "databases": {str(old["database"]): {}} if remains_present else {},
            }
            with (
                self.subTest(remains_present=remains_present),
                patch.object(
                    server,
                    "maintenance_endpoint",
                    return_value="http://127.0.0.1:18080",
                ),
                patch.object(
                    server.subprocess, "run", side_effect=acknowledged
                ) as native,
                patch.object(
                    server, "maintenance_catalog", side_effect=[root, namespace]
                ) as catalog,
            ):
                if remains_present:
                    with self.assertRaisesRegex(
                        server.SupervisorError, "remains addressable"
                    ):
                        server.discard_database(self.state, config, old, credentials)
                else:
                    server.discard_database(self.state, config, old, credentials)
            native.assert_called_once()
            self.assertEqual(
                [call.args[3] for call in catalog.call_args_list], ["ROOT", "NS"]
            )
            self.assertTrue(all(call.args[1] == old for call in catalog.call_args_list))
            self.assertEqual((self.state / "config.json").read_bytes(), retained)
            self.assertFalse(server.config_for(self.state)["accepting_writes"])

    def test_database_presence_never_selects_missing_namespace_or_database(
        self,
    ) -> None:
        config = self.initialized()
        root = self.empty_catalog("ROOT")
        namespace = self.empty_catalog("NS")
        with patch.object(server, "maintenance_catalog", return_value=root) as catalog:
            self.assertFalse(server.database_present(self.state, config, {}))
            catalog.assert_called_once_with(self.state, config, {}, "ROOT")
        root["namespaces"] = {str(config["namespace"]): {}}
        for present in (False, True):
            namespace["databases"] = {str(config["database"]): {}} if present else {}
            with (
                self.subTest(database_present=present),
                patch.object(
                    server, "maintenance_catalog", side_effect=[root, namespace]
                ) as catalog,
            ):
                self.assertEqual(
                    server.database_present(self.state, config, {}), present
                )
            self.assertEqual(
                [call.args[3] for call in catalog.call_args_list], ["ROOT", "NS"]
            )
            self.assertEqual(
                catalog.call_args.args[1]["namespace"], config["namespace"]
            )

    def test_recovery_observes_unknown_disposal_before_finishing_without_replay(
        self,
    ) -> None:
        config, _before, after = self.maintenance_fixture("rebuild", "validated")
        with (
            patch.object(server, "require_private_catalog"),
            patch.object(server, "database_present", return_value=True),
            patch.object(
                server,
                "discard_database",
                side_effect=server.SupervisorError("uncertain drop"),
            ),
            self.assertRaisesRegex(server.SupervisorError, "uncertain drop"),
        ):
            server.finish_maintenance(self.state, config, after)
        self.assertEqual(
            server.object_mapping(
                server.config_for(self.state)["derived_rebuild_pending"]
            )["phase"],
            "disposing",
        )
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "recover_credentials", return_value=(after, True)),
            patch.object(server, "current_marker"),
            patch.object(server, "database_present", return_value=False),
            patch.object(server, "discard_database") as discard,
            patch.object(server, "start") as start,
        ):
            server.recover_maintenance(self.state, [])
        discard.assert_not_called()
        start.assert_not_called()
        recovered = server.config_for(self.state)
        self.assertFalse(recovered.get("derived_rebuild_pending"))
        self.assertFalse(recovered["accepting_writes"])
        self.assertEqual(recovered["admission"], "quiesced")

    def test_recovery_refuses_unknown_phase_and_missing_initializer(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "initializing")
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "recover_credentials") as credentials,
            self.assertRaisesRegex(
                server.SupervisorError, "explicit current initializer"
            ),
        ):
            server.recover_maintenance(self.state, [])
        credentials.assert_not_called()
        server.object_mapping(config["derived_rebuild_pending"])["phase"] = "unknown"
        server.write_json(self.state / "config.json", config)
        with (
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            self.assertRaisesRegex(server.SupervisorError, "Unknown maintenance phase"),
        ):
            server.recover_maintenance(self.state, ["initializer"])

    def test_recovery_restarts_unacknowledged_initializer_only_in_fresh_target(
        self,
    ) -> None:
        config, _before, after = self.maintenance_fixture("rebuild", "initializing")
        prior_target = (config["namespace"], config["database"])
        source = (
            server.object_mapping(config["derived_rebuild_pending"])["namespace"],
            server.object_mapping(config["derived_rebuild_pending"])["database"],
        )
        events = []

        def drop(
            _state: Path,
            _selected: dict[str, object],
            old: dict[str, object],
            _credentials: dict[str, object],
        ) -> None:
            events.append(("drop", old["namespace"], old["database"]))

        def initialize(
            state: Path,
            selected: dict[str, object],
            command: list[str],
        ) -> None:
            self.assertEqual(command, ["current-initializer"])
            self.assertNotEqual(
                (selected["namespace"], selected["database"]), prior_target
            )
            self.assertFalse(selected["accepting_writes"])
            events.append(("initialize", selected["namespace"], selected["database"]))
            server.maintenance_phase(state, selected, "validated")

        with (
            self.native_maintenance(),
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "recover_credentials", return_value=(after, True)),
            patch.object(server, "database_present", return_value=True),
            patch.object(server, "discard_database", side_effect=drop),
            patch.object(server, "initialize_rebuild", side_effect=initialize),
        ):
            server.recover_maintenance(self.state, ["current-initializer"])
        self.assertEqual(events[0], ("drop", *prior_target))
        self.assertEqual(events[1][0], "initialize")
        self.assertEqual(events[2], ("drop", *source))

    def test_restore_recovery_rechecks_source_and_remints_before_fresh_copy(
        self,
    ) -> None:
        config, before, _after = self.maintenance_fixture("current-restore", "copy")
        partial = (config["namespace"], config["database"])
        old = (
            server.object_mapping(config["derived_rebuild_pending"])["namespace"],
            server.object_mapping(config["derived_rebuild_pending"])["database"],
        )
        events = []

        def copied(
            _state: Path,
            source: dict[str, object],
            target: dict[str, object],
            _credentials: dict[str, object],
        ) -> None:
            self.assertEqual((source["namespace"], source["database"]), old)
            self.assertNotEqual((target["namespace"], target["database"]), partial)
            self.assertEqual(
                server.object_mapping(target["derived_rebuild_pending"])["phase"],
                "copy",
            )
            # Native copy/start may refresh configuration, replacing nested owners.
            target.update(server.config_for(_state))
            events.append("copy")

        with (
            self.native_maintenance(),
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "recover_credentials", return_value=(before, False)),
            patch.object(server, "database_present", return_value=True),
            patch.object(
                server,
                "current_marker",
                side_effect=lambda *_args: events.append("marker"),
            ),
            patch.object(
                server,
                "rotate_source_authorities",
                side_effect=lambda *_args: events.append("remint"),
            ),
            patch.object(
                server,
                "discard_database",
                side_effect=lambda *_args: events.append("drop"),
            ),
            patch.object(server, "copy_current_database", side_effect=copied),
            patch.object(
                server,
                "rotate_database_accounts",
                side_effect=lambda *_args: events.append("accounts"),
            ),
        ):
            server.recover_maintenance(self.state, [])
        self.assertEqual(
            events, ["marker", "remint", "drop", "copy", "accounts", "marker", "drop"]
        )
        recovered = server.config_for(self.state)
        self.assertFalse(recovered.get("derived_rebuild_pending"))
        self.assertEqual(recovered["admission"], "validation_required")
        self.assertEqual(list(self.state.glob(".maintenance-payloads-*.json")), [])

    def test_restore_account_recovery_never_reimports_acknowledged_copy(self) -> None:
        _config, _before, after = self.maintenance_fixture(
            "current-restore", "accounts"
        )
        with (
            self.native_maintenance(),
            patch.object(server, "freeze_payloads", return_value=[]),
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.application_inventory_fixture,
            ),
            patch.object(server, "private_offline_state"),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "recover_credentials", return_value=(after, True)),
            patch.object(server, "current_marker"),
            patch.object(server, "database_present", return_value=True),
            patch.object(server, "discard_database"),
            patch.object(server, "copy_current_database") as copy,
            patch.object(server, "rotate_source_authorities") as cleanup,
        ):
            server.recover_maintenance(self.state, [])
        copy.assert_not_called()
        cleanup.assert_not_called()

    def test_private_catalog_allows_only_exact_owners_and_known_empty_databases(
        self,
    ) -> None:
        config, _before, after = self.maintenance_fixture("current-restore", "accounts")
        old = server.object_mapping(config["derived_rebuild_pending"])
        empty = self.empty_catalog("DB")
        calls: list[tuple[str, str, str]] = []
        namespaces = {
            str(config["namespace"]): {str(config["database"]): {}},
            str(old["namespace"]): {str(old["database"]): {}},
            "empty_namespace": {},
            "other": {"empty": {}},
        }

        def catalog(
            _state: Path,
            selected: dict[str, object],
            _credentials: dict[str, object],
            scope: str,
        ) -> dict[str, object]:
            calls.append(
                (scope, str(selected.get("namespace")), str(selected.get("database")))
            )
            if scope == "ROOT":
                return {
                    **self.empty_catalog(scope),
                    "namespaces": {name: {} for name in namespaces},
                }
            self.assertIn(str(selected["namespace"]), namespaces)
            if scope == "NS":
                return {
                    **self.empty_catalog(scope),
                    "databases": namespaces[str(selected["namespace"])],
                }
            self.assertIn(
                str(selected["database"]), namespaces[str(selected["namespace"])]
            )
            return empty

        with patch.object(server, "maintenance_catalog", side_effect=catalog):
            server.require_private_catalog(self.state, config, after)
        self.assertEqual(
            [call for call in calls if call[0] == "DB"], [("DB", "other", "empty")]
        )
        with (
            patch.object(
                server,
                "maintenance_catalog",
                side_effect=[
                    {**self.empty_catalog("ROOT"), "namespaces": {"other": {}}},
                    {**self.empty_catalog("NS"), "databases": {"shared": {}}},
                    {**empty, "tables": {"unknown": "definition"}},
                ],
            ),
            self.assertRaisesRegex(server.SupervisorError, "unknown nonempty database"),
        ):
            server.require_private_catalog(self.state, config, after)

    def test_catalog_refusal_precedes_global_account_mutation(self) -> None:
        config = self.initialized()
        old = server.read_json(self.state / "credentials.json")
        with (
            patch.object(
                server,
                "maintenance_catalog",
                side_effect=server.SupervisorError(
                    "Unknown physical namespace catalog"
                ),
            ) as catalog,
            patch.object(server, "maintenance_query") as mutate,
            self.assertRaises(server.SupervisorError),
        ):
            server.rotate_database_accounts(
                self.state, config, old, server.fresh_credentials()
            )
        catalog.assert_called_once_with(self.state, config, old, "ROOT")
        mutate.assert_not_called()
        self.assertEqual(server.read_json(self.state / "credentials.json"), old)

    def test_unknown_export_file_is_preserved_without_native_copy(self) -> None:
        config, before, _after = self.maintenance_fixture("current-restore", "copy")
        path = self.state / ".maintenance-export.surql"
        path.write_text("unknown producer")
        with (
            patch.object(server.subprocess, "run") as native,
            self.assertRaisesRegex(
                server.SupervisorError, "Unknown maintenance export"
            ),
        ):
            server.copy_current_database(self.state, config, config, before)
        native.assert_not_called()
        self.assertEqual(path.read_text(), "unknown producer")

    def test_export_creation_interruption_recovers_only_exact_reserved_empty_file(
        self,
    ) -> None:
        config, before, _after = self.maintenance_fixture("current-restore", "copy")
        original_write = server.write_json

        def interrupt(path: Path, value: dict[str, object]) -> None:
            pending = server.object_mapping(value.get("derived_rebuild_pending", {}))
            if path == self.state / "config.json" and "export_identity" in pending:
                export_path = pending["export_path"]
                assert isinstance(export_path, str)
                self.assertTrue((self.state / export_path).exists())
                raise OSError("interrupted before inode receipt")
            original_write(path, value)

        with (
            patch.object(server, "write_json", side_effect=interrupt),
            patch.object(server.subprocess, "run") as native,
            self.assertRaisesRegex(OSError, "before inode receipt"),
        ):
            server.copy_current_database(self.state, config, config, before)
        native.assert_not_called()
        recovered = server.config_for(self.state)
        pending = server.object_mapping(recovered["derived_rebuild_pending"])
        self.assertNotIn("export_identity", pending)
        export_path = pending["export_path"]
        assert isinstance(export_path, str)
        self.assertEqual((self.state / export_path).read_bytes(), b"")
        with self.native_maintenance():
            server.copy_current_database(self.state, recovered, recovered, before)
        self.assertFalse(list(self.state.glob(".maintenance-export*.surql")))
        self.assertNotIn(
            "export_path",
            server.object_mapping(
                server.config_for(self.state)["derived_rebuild_pending"]
            ),
        )

    def test_unbound_reserved_nonempty_export_remains_unknown_and_preserved(
        self,
    ) -> None:
        config, before, _after = self.maintenance_fixture("current-restore", "copy")
        nonce = "a" * 32
        name = ".maintenance-export-" + nonce + ".surql"
        server.object_mapping(config["derived_rebuild_pending"]).update(
            export_nonce=nonce, export_path=name
        )
        server.write_json(self.state / "config.json", config)
        path = self.state / name
        path.write_bytes(b"unknown nonempty producer")
        path.chmod(0o600)
        with (
            patch.object(server.subprocess, "run") as native,
            self.assertRaisesRegex(
                server.SupervisorError, "Unknown maintenance export"
            ),
        ):
            server.copy_current_database(self.state, config, config, before)
        native.assert_not_called()
        self.assertEqual(path.read_bytes(), b"unknown nonempty producer")

    def test_account_recovery_observes_committed_atomic_change_without_repeating_it(
        self,
    ) -> None:
        config, _before, after = self.maintenance_fixture("rebuild", "accounts")
        server.write_json(self.state / ".maintenance-credentials.json", after)
        with (
            patch.object(
                server,
                "maintenance_catalog",
                side_effect=[
                    server.SupervisorError("old denied"),
                    {
                        **self.empty_catalog("ROOT"),
                        "users": {str(after["username"]): {}},
                    },
                ],
            ),
            patch.object(server, "rotate_database_accounts") as rotate,
        ):
            credentials, committed = server.recover_credentials(self.state, config)
        self.assertEqual(credentials, after)
        self.assertTrue(committed)
        self.assertEqual(server.read_json(self.state / "credentials.json"), after)
        self.assertFalse((self.state / ".maintenance-credentials.json").exists())
        rotate.assert_not_called()

    def test_account_recovery_retries_only_observed_uncommitted_change_and_refuses_unknown(
        self,
    ) -> None:
        config, before, after = self.maintenance_fixture("rebuild", "accounts")
        server.write_json(self.state / ".maintenance-credentials.json", after)
        with (
            patch.object(
                server,
                "maintenance_catalog",
                side_effect=[
                    {
                        **self.empty_catalog("ROOT"),
                        "users": {str(before["username"]): {}},
                    },
                    server.SupervisorError("candidate denied"),
                ],
            ),
            patch.object(server, "rotate_database_accounts") as rotate,
        ):
            self.assertEqual(
                server.recover_credentials(self.state, config), (after, True)
            )
        rotate.assert_called_once_with(self.state, config, before, after)
        for users in ({str(before["username"]): {}, str(after["username"]): {}}, {}):
            with (
                patch.object(
                    server,
                    "maintenance_catalog",
                    return_value={**self.empty_catalog("ROOT"), "users": users},
                ),
                patch.object(server, "rotate_database_accounts") as rotate,
                self.assertRaisesRegex(server.SupervisorError, "uncertain"),
            ):
                server.recover_credentials(self.state, config)
            rotate.assert_not_called()


if __name__ == "__main__":
    unittest.main()


class StandaloneSupervisorTests(unittest.TestCase):
    def test_primary_handoff_resolves_shared_owners_before_placement(self) -> None:
        # Native callers execute the recorded file from their own working directory.
        # Stop at the environment owner so this import control launches no service.
        program = (
            "import importlib.util, sys\n"
            "from pathlib import Path\n"
            "spec = importlib.util.spec_from_file_location('supervisor', sys.argv[1])\n"
            "module = importlib.util.module_from_spec(spec)\n"
            "sys.modules[spec.name] = module\n"
            "spec.loader.exec_module(module)\n"
            "def observed_owner(*args):\n"
            "    raise RuntimeError('worker environment owner reached')\n"
            "module.worker_environment = observed_owner\n"
            "try:\n"
            "    module.primary_environment(Path(sys.argv[2]), {}, {})\n"
            "except RuntimeError as error:\n"
            "    assert str(error) == 'worker environment owner reached'\n"
            "    print(error)\n"
            "else:\n"
            "    raise AssertionError('handoff did not reach its environment owner')\n"
        )
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(
                [sys.executable, "-I", "-c", program, str(server.SCRIPT), directory],
                cwd=directory,
                capture_output=True,
                text=True,
                check=False,
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("worker environment owner reached", result.stdout)

    def test_placement_resolves_shared_owner_when_run_as_a_script(self) -> None:
        # `just surreal` and worker launches run the file as a script, from any cwd,
        # where `scripts` is not importable until the supervisor selects its root.
        program = (
            "import importlib.util, sys\n"
            "spec = importlib.util.spec_from_file_location('supervisor', sys.argv[1])\n"
            "module = importlib.util.module_from_spec(spec)\n"
            "sys.modules[spec.name] = module\n"
            "spec.loader.exec_module(module)\n"
            "print(module.placement_slice())\n"
        )
        script = Path(server.__file__).resolve()
        with tempfile.TemporaryDirectory() as directory:
            result = subprocess.run(
                [sys.executable, "-I", "-c", program, str(script)],
                cwd=directory,
                capture_output=True,
                text=True,
                check=False,
                env={**os.environ, "PSE_SLICE": "pse.slice"},
            )
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertIn("--slice=pse.slice", result.stdout)

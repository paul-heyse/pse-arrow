# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Scoped controls for owned SurrealDB state and lifecycle boundaries."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import contextlib
import errno
import json
import os
import shutil
import subprocess
import sys
import tempfile
import threading
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import MagicMock, patch

from scripts import host_admission as host
from scripts import surreal_server as server

if TYPE_CHECKING:
    from collections.abc import Generator


class SurrealSupervisorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.state = self.root / "state"
        # Supervisor unit controls borrow a declared owner without touching the
        # live machine ledger. Host admission itself has its own owner tests.
        self.allocation = MagicMock(spec=host.Allocation)
        self.allocation.directory = self.root / "admission"
        self.allocation.nonce = "a" * 32
        self.allocation.profile = host.select("reference")
        self.allocation.environment.return_value = {}
        for target, name, value in (
            (host, "inherit", self.allocation),
            (host, "enforce_allocation", None),
            (host, "enforce_parent", None),
        ):
            owned = patch.object(target, name, return_value=value)
            owned.start()
            self.addCleanup(owned.stop)

        def manager_result(
            *arguments: str, check: bool = True
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

    def initialized(self) -> dict[str, object]:
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
                    "version": "3.3.0",
                    "binary": str(binary),
                    "binary_sha256": server.file_digest(binary),
                    "archive_sha256": "abc",
                },
            ),
            patch.object(server, "active", return_value=False),
        ):
            server.setup(args)
        (self.state / "database").mkdir()
        (self.state / "database" / "fixture").write_bytes(
            b"acknowledged fixed operation identity"
        )
        return server.config_for(self.state)

    def test_joint_budget_rejects_overallocation(self) -> None:
        with self.assertRaises(server.SupervisorError):
            server.resources(2 * server.GIB, server.GIB, 2, server.GIB)
        allocation = server.resources(4 * server.GIB, 2 * server.GIB, 2, server.GIB)
        self.assertLess(
            server.integer(allocation["rocksdb_block_cache_bytes"]),
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
            patch.object(server, "ready", return_value=True),
            patch.object(server.subprocess, "run") as launch,
        ):
            self.assertTrue(server.ensure_primary(self.state)["ready"])
            launch.assert_not_called()

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
        with (
            patch.object(server, "ensure_execution_placement"),
            patch.object(server, "primary_observation", return_value=inactive),
            patch.object(
                server, "primary_ready", side_effect=[False, True]
            ) as readiness,
            patch.object(server, "settled_worker", return_value=inactive),
            patch.object(server, "ready", return_value=True),
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
            receipt = server.ensure_primary(self.state, database="isolated-study")
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
        def metadata(_directory: Path) -> Generator[dict[str, object], None, None]:
            yield ledger

        def manager(*arguments: str) -> subprocess.CompletedProcess[str]:
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
        with (
            self.observer_control_fixture() as (state, _calls, _drained),
            patch.object(server, "ensure_execution_placement"),
            patch.object(server.native_operation, "prepare_handoff", return_value=None),
            patch.object(server, "systemd_environment", return_value={}),
            patch.object(server, "observer_scope_command", return_value=["probe"]),
            patch.object(server.subprocess, "call", return_value=0) as launch,
        ):
            self.assertEqual(
                server.observer(self.state, ["probe"], "exclusive-observer"), 0
            )
            self.assertEqual(state["memory"], 144 * server.GIB)
        launch.assert_called_once()
        self.allocation.release.assert_not_called()
        self.assertFalse((self.state / "observer-launch.json").exists())

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
            patch.object(server.subprocess, "call", return_value=0),
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
                }
                original = json.loads(json.dumps(record))
                ledger: dict[str, object] = {"owners": {self.allocation.nonce: record}}

                @contextlib.contextmanager
                def metadata(
                    _directory: Path,
                    captured: dict[str, object] = ledger,
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
                self.assertEqual(record, original)

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
            patch.object(server.subprocess, "call") as launch,
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
            patch.object(server, "primary_observation", return_value=inactive),
            patch.object(
                server, "primary_ready", side_effect=[False, True]
            ) as readiness,
            patch.object(server, "settled_worker", return_value=inactive),
            patch.object(server, "ready", return_value=True),
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
        def lock(_state: Path) -> Generator[None, None, None]:
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
            patch.object(server, "listener_ready", return_value=True),
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
            environment["SURREAL_ROCKSDB_BLOCK_CACHE_SIZE"], str(4 * server.GIB)
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

    def test_release_metadata_refuses_nonofficial_origin_without_network(self) -> None:
        with patch.object(server.urllib.request, "urlopen") as opened:
            for origin in (
                "file:///etc/passwd",
                "http://api.github.com/",
                "https://api.github.com.evil/releases",
            ):
                with self.assertRaises(server.SupervisorError):
                    server.fetched_json(origin)
        opened.assert_not_called()

    def test_release_partial_and_corrupt_installations_refuse_same_path_overwrite(
        self,
    ) -> None:
        tag = "v3.3.0"
        filename = f"surreal-{tag}.linux-amd64.tgz"
        release = {
            "tag_name": tag,
            "assets": [
                {
                    "name": filename,
                    "digest": "sha256:" + "a" * 64,
                    "browser_download_url": f"https://github.com/surrealdb/surrealdb/releases/download/{tag}/{filename}",
                }
            ],
        }
        directory = self.root / "tools" / tag / "linux-amd64"
        directory.mkdir(parents=True)
        binary = directory / "surreal"
        binary.write_bytes(b"existing executable")
        with (
            patch.object(server.platform, "system", return_value="Linux"),
            patch.object(server.platform, "machine", return_value="x86_64"),
            patch.object(server, "fetched_json", return_value=release),
            patch.object(server.urllib.request, "urlopen") as download,
        ):
            with self.assertRaisesRegex(server.SupervisorError, "incomplete"):
                server.install(tag, self.root / "tools")
            server.write_json(
                directory / "release.json",
                {"archive_sha256": "a" * 64, "binary_sha256": "b" * 64},
            )
            with self.assertRaisesRegex(server.SupervisorError, "changed"):
                server.install(tag, self.root / "tools")
            self.assertEqual(binary.read_bytes(), b"existing executable")
            download.assert_not_called()

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

    def test_websocket_readiness_proof_binds_invocation_generation_and_credentials(
        self,
    ) -> None:
        config = self.initialized()
        observed = subprocess.CompletedProcess([], 0, "actual-invocation\n", "")
        with patch.object(server, "systemctl", return_value=observed):
            proof = {
                "schema": "native-ws-readiness-v1",
                "instance_id": config["instance_id"],
                "invocation": "actual-invocation",
                "binary_sha256": server.object_mapping(config["server"])[
                    "binary_sha256"
                ],
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
            ):
                with self.subTest(binding=key):
                    server.write_json(
                        self.state / "protocol-readiness.json",
                        {**proof, key: "different-admission"},
                    )
                    self.assertFalse(server.protocol_ready(self.state, config))

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
        self.assertEqual(environment["SURREAL_WEBSOCKET_MAX_MESSAGE_SIZE"], "4194304")
        self.assertEqual(
            environment["SURREAL_ROCKSDB_BLOCK_CACHE_SIZE"], str(512 * server.MIB)
        )
        self.assertEqual(environment["SURREAL_RUNTIME_WORKER_THREADS"], "4")

    def test_resource_override_outside_recorded_budget_is_refused(self) -> None:
        config = self.initialized()
        allocation = config["resources"]
        self.assertIsInstance(allocation, dict)
        if not isinstance(allocation, dict):
            self.fail("recorded resource allocation must be an object")
        allocation["rocksdb_block_cache_bytes"] = 100 * server.GIB
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

    def backup_fixture(self) -> Path:
        config = self.initialized()
        config.update(
            interpretation=server.SUBSTRATE_INTERPRETATION,
            schema_interpretation=server.SUBSTRATE_INTERPRETATION,
        )
        server.write_json(self.state / "config.json", config)
        destination = self.root / "backup"
        with patch.object(server, "active", return_value=False):
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
    def native_maintenance(self) -> Generator[list[tuple[str, str, str]], None, None]:
        """Model native acknowledgments; no service, embedded database or listener."""
        calls: list[tuple[str, str, str]] = []
        inventory: dict[Path, dict[str, set[str]]] = {}

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
            return inventory[state]

        def query(
            state: Path,
            config: dict[str, object],
            _credentials: dict[str, object],
            body: str,
        ) -> dict[str, object]:
            calls.append((str(config["namespace"]), str(config["database"]), body))
            self.assertFalse(server.config_for(state)["accepting_writes"])
            if "LET $value={count:" in body:
                return {"value": {"count": 0, "after": ""}}
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
            return {"value": 0 if "DELETE $rows.id" in body else True}

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
            self.assertIn(arguments[1], {"export", "import"})
            calls.append((arguments[6], arguments[8], arguments[1]))
            if arguments[1] == "export":
                Path(arguments[-1]).write_text(
                    "current authored records, excluding retired analyses"
                )
            else:
                self.assertTrue(Path(arguments[-1]).is_file())
                state = Path(str(options["cwd"]))
                known_databases(state).setdefault(arguments[6], set()).add(arguments[8])
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

        def observe(_state: Path, slot: int) -> dict[str, str]:
            if slot == 0:
                return busy
            return idle if spawn.call_count == 0 else busy

        with (
            patch.object(server, "ready", return_value=True),
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
        def lock(_state: Path) -> Generator[None, None, None]:
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
            patch.object(server, "ready", return_value=True),
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
            patch.object(server, "ready", return_value=True),
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

    def test_restore_is_gated_and_preserves_exact_fixture(self) -> None:
        backup = self.backup_fixture()
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
        with self.assertRaises(server.SupervisorError):
            server.start(restored, config)

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
        with self.native_maintenance():
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
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

    def test_explicit_inputs_preserve_bytes_hashes_and_owner_permissions(self) -> None:
        source = self.root / "authored"
        source.mkdir(mode=0o755)
        (source / "relations.json").write_bytes(b"authored relation authority")
        before = source.stat().st_mode
        preserved = server.preserve_inputs(source, self.root / "preserved", self.state)
        self.assertEqual(source.stat().st_mode, before)
        self.assertEqual(
            server.input_inventory(source), server.input_inventory(preserved)
        )
        self.assertEqual(
            server.read_json(preserved.parent / "inputs.json")["files"],
            server.input_inventory(source),
        )
        self.assertEqual(preserved.parent.stat().st_mode & 0o777, 0o700)

    def test_explicit_inputs_refuse_empty_symlink_alias_and_state_overlap(self) -> None:
        source = self.root / "authored"
        source.mkdir()
        with self.assertRaisesRegex(server.SupervisorError, "inventory"):
            server.preserve_inputs(source, self.root / "empty-copy", self.state)
        (source / "input").write_text("external authority")
        alias = self.root / "alias"
        alias.symlink_to(source)
        for selected, destination in (
            (alias, self.root / "copy"),
            (source, self.state),
            (source, self.root),
            (source, source / "copy"),
        ):
            with (
                self.subTest(selected=selected, destination=destination),
                self.assertRaises(server.SupervisorError),
            ):
                server.preserve_inputs(selected, destination, self.state)
        (source / "linked").symlink_to(source / "input")
        with self.assertRaisesRegex(server.SupervisorError, "real files"):
            server.preserve_inputs(source, self.root / "copy", self.state)

    def test_input_change_during_copy_refuses_before_native_work(self) -> None:
        source = self.root / "authored"
        source.mkdir()
        (source / "input").write_text("original")
        copy = shutil.copytree

        def changed(source_path: Path, destination: Path) -> Path:
            result = copy(source_path, destination)
            (source / "added").write_text("concurrent input")
            return result

        with (
            patch.object(server.shutil, "copytree", side_effect=changed),
            self.assertRaisesRegex(
                server.SupervisorError, "changed during preservation"
            ),
        ):
            server.preserve_inputs(source, self.root / "preserved", self.state)

    def test_rebuild_preserves_inputs_before_accounts_and_disposes_only_after_validation(
        self,
    ) -> None:
        prior = self.initialized()
        prior.update(accepting_writes=False, admission="quiesced")
        server.write_json(self.state / "config.json", prior)
        source = self.root / "authored"
        source.mkdir()
        (source / "input").write_text("external authority")
        events: list[str] = []

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
            self.assertEqual(
                (self.root / "preserved/inputs/input").read_text(), "external authority"
            )
            config = server.config_for(self.state)
            self.assertTrue(config["derived_rebuild_pending"])
            self.assertFalse(config["accepting_writes"])
            self.assertNotEqual(config["database"], prior["database"])
            server.write_json(self.state / "credentials.json", after)
            events.append("accounts")

        def initialize(
            _arguments: list[str], **options: object
        ) -> subprocess.CompletedProcess[str]:
            config = server.config_for(self.state)
            self.assertFalse(config["accepting_writes"])
            self.assertEqual(config["admission"], "validation_required")
            environment = options["env"]
            self.assertIsInstance(environment, dict)
            self.assertEqual(
                environment["PSE_PRESERVED_INPUTS"], str(self.root / "preserved/inputs")
            )
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
            return subprocess.CompletedProcess([], 0)

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
            patch.object(server, "private_offline_state"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "rotate_database_accounts", side_effect=rotate),
            patch.object(
                server,
                "start",
                side_effect=lambda *_args, **_kw: events.append("start"),
            ),
            patch.object(
                server, "stop", side_effect=lambda *_args: events.append("stop")
            ),
            patch.object(server.subprocess, "run", side_effect=initialize),
            patch.object(
                server,
                "validate_interpretation",
                side_effect=lambda *_args: events.append("validate"),
            ),
            patch.object(server, "database_present", return_value=True),
            patch.object(server, "discard_database", side_effect=discard),
        ):
            server.rebuild(
                self.state, source, self.root / "preserved", ["current-initializer"]
            )
        self.assertEqual(
            events, ["accounts", "start", "initialize", "validate", "stop", "dispose"]
        )
        config = server.config_for(self.state)
        self.assertFalse(config["accepting_writes"])
        self.assertFalse(config.get("derived_rebuild_pending"))
        self.assertEqual(config["admission"], "quiesced")
        self.assertEqual(config["interpretation"], server.SUBSTRATE_INTERPRETATION)
        self.assertEqual((source / "input").read_text(), "external authority")

    def test_rebuild_failure_or_preserved_input_mutation_retains_old_database(
        self,
    ) -> None:
        config = self.initialized()
        config.update(accepting_writes=False, admission="quiesced")
        server.write_json(self.state / "config.json", config)
        source = self.root / "authored"
        source.mkdir()
        (source / "input").write_text("external authority")

        def mutate(
            _arguments: list[str], **_options: object
        ) -> subprocess.CompletedProcess[str]:
            (self.root / "preserved/inputs/input").write_text("mutated")
            return subprocess.CompletedProcess([], 0)

        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "rotate_database_accounts"),
            patch.object(server, "start"),
            patch.object(server, "stop") as stop,
            patch.object(server.subprocess, "run", side_effect=mutate),
            patch.object(server, "validate_interpretation"),
            patch.object(server, "discard_database") as discard,
            self.assertRaisesRegex(server.SupervisorError, "inputs changed"),
        ):
            server.rebuild(self.state, source, self.root / "preserved", ["initializer"])
        stop.assert_called_once()
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

    def test_initializer_failure_retains_input_authority_and_never_disposes_old_database(
        self,
    ) -> None:
        config = self.initialized()
        config.update(accepting_writes=False, admission="quiesced")
        server.write_json(self.state / "config.json", config)
        source = self.root / "authored"
        source.mkdir()
        (source / "input").write_text("external authority")
        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "active", return_value=False),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "rotate_database_accounts"),
            patch.object(server, "start"),
            patch.object(server, "stop") as stop,
            patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 1),
            ),
            patch.object(server, "validate_interpretation") as validate,
            patch.object(server, "discard_database") as discard,
            self.assertRaisesRegex(server.SupervisorError, "initializer failed"),
        ):
            server.rebuild(self.state, source, self.root / "preserved", ["initializer"])
        stop.assert_called_once()
        validate.assert_not_called()
        discard.assert_not_called()
        config = server.config_for(self.state)
        self.assertTrue(config["derived_rebuild_pending"])
        self.assertFalse(config["accepting_writes"])
        self.assertEqual(
            (self.root / "preserved/inputs/input").read_text(), "external authority"
        )

    def test_bounded_analysis_cleanup_preserves_other_roots_and_guard_floor(
        self,
    ) -> None:
        config = self.initialized()
        bodies: list[str] = []
        edges = 0
        rotation = 0

        def query(
            _state: Path,
            _config: dict[str, object],
            _credentials: dict[str, object],
            body: str,
        ) -> dict[str, object]:
            nonlocal edges, rotation
            bodies.append(body)
            if "LET $value={count:" in body:
                rotation += 1
                return {
                    "value": {"count": 1, "after": "retention:source"}
                    if rotation == 1
                    else {"count": 0, "after": ""}
                }
            if "DELETE $rows.id" not in body:
                return {"value": True}
            if "FROM canonical_analysis_edges " in body:
                edges += 1
                return {"value": 64 if edges == 1 else 0}
            return {"value": 0}

        with patch.object(server, "maintenance_query", side_effect=query):
            server.remove_analysis_state(self.state, config, {})
        self.assertEqual(edges, 2)
        deletes = [body for body in bodies if "DELETE $rows.id" in body]
        self.assertTrue(all("LIMIT 64" in body for body in deletes))
        self.assertTrue(
            all("LET $deleted=DELETE $rows.id RETURN NONE;" in body for body in deletes)
        )
        self.assertIn("canonical_analysis_edges", deletes[0])
        self.assertIn("canonical_analysis_nodes", deletes[2])
        self.assertIn("'execution-run:'+$row.run", deletes[3])
        self.assertIn("owner_kind='analysis'", deletes[4])
        self.assertIn("'retention:'+$row.problem", deletes[4])
        self.assertIn("string::starts_with(key,'analysis:')", deletes[-1])
        self.assertIn("incarnation=<string>rand::uuid::v4()", bodies[-2])
        self.assertNotIn("analysis_creation_closed_through", " ".join(bodies))
        self.assertNotIn("DELETE canonical_revisions", " ".join(bodies))
        self.assertNotIn("DELETE canonical_protections", " ".join(bodies))

    def test_cleanup_refuses_live_protection_and_invalid_page_acknowledgment(
        self,
    ) -> None:
        config = self.initialized()
        with (
            patch.object(
                server,
                "maintenance_query",
                side_effect=server.SupervisorError("live protections"),
            ) as query,
            self.assertRaisesRegex(server.SupervisorError, "live protections"),
        ):
            server.remove_analysis_state(self.state, config, {})
        self.assertEqual(query.call_count, 1)
        for count in (-1, 65, True, "0"):
            with (
                self.subTest(count=count),
                patch.object(
                    server,
                    "maintenance_query",
                    side_effect=[{"value": True}, {"value": count}],
                ),
                self.assertRaisesRegex(server.SupervisorError, "bounded"),
            ):
                server.remove_analysis_state(self.state, config, {})

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
            self.assertIn("SURREAL_ROCKSDB_BLOCK_CACHE_SIZE", environment)
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

        # An unrelated producer's file never becomes an overwriteable diagnostic slot.
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

    def test_restore_copy_failure_retains_pending_identity_and_original_backup(
        self,
    ) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        with (
            self.native_maintenance(),
            patch.object(server, "private_offline_state"),
            patch.object(server, "remove_analysis_state"),
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

    def test_restore_cleanup_failure_is_durably_fenced_before_any_derived_deletion(
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
            patch.object(server, "private_offline_state"),
            patch.object(server, "remove_analysis_state", side_effect=refuse),
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
        if kind == "rebuild":
            source = self.root / "authored"
            source.mkdir()
            (source / "input").write_text("preserved authority")
            preserved = server.preserve_inputs(
                source, self.root / "preserved", self.state
            )
            pending.update(
                inputs=str(preserved),
                inputs_digest=server.file_digest(preserved.parent / "inputs.json"),
            )
        config["derived_rebuild_pending"] = pending
        server.write_json(self.state / "config.json", config)
        server.write_json(
            self.state / "credentials.json",
            after if phase in {"initializing", "validated", "disposing"} else before,
        )
        return config, before, after

    def test_maintenance_listener_reuses_live_owner_and_drains_before_release(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "accounts")
        with (
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

    def test_maintenance_launch_has_no_restart_bootstrap_or_readiness_mutation(
        self,
    ) -> None:
        config, before, _after = self.maintenance_fixture("rebuild", "accounts")
        with (
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
                    environment["SURREAL_ROCKSDB_BLOCK_CACHE_SIZE"],
                    str(
                        server.object_mapping(selected["resources"])[
                            "rocksdb_block_cache_bytes"
                        ]
                    ),
                )
            with (
                patch.object(server, "active", return_value=True),
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

    def test_recovery_refuses_unknown_phase_changed_inventory_and_missing_initializer(
        self,
    ) -> None:
        config, _before, _after = self.maintenance_fixture("rebuild", "initializing")
        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "recover_credentials") as credentials,
            self.assertRaisesRegex(
                server.SupervisorError, "explicit current initializer"
            ),
        ):
            server.recover_maintenance(self.state, [])
        credentials.assert_not_called()
        (self.root / "preserved/inputs/input").write_text("changed")
        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "recover_credentials") as credentials,
            self.assertRaisesRegex(server.SupervisorError, "inputs differ"),
        ):
            server.recover_maintenance(self.state, ["initializer"])
        credentials.assert_not_called()
        server.object_mapping(config["derived_rebuild_pending"])["phase"] = "unknown"
        server.write_json(self.state / "config.json", config)
        with (
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
            preserved: Path,
            _inventory: dict[str, object],
        ) -> None:
            self.assertEqual(command, ["current-initializer"])
            self.assertNotEqual(
                (selected["namespace"], selected["database"]), prior_target
            )
            self.assertEqual((preserved / "input").read_text(), "preserved authority")
            self.assertFalse(selected["accepting_writes"])
            events.append(("initialize", selected["namespace"], selected["database"]))
            server.maintenance_phase(state, selected, "validated")

        with (
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
            events.append("copy")

        with (
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
                "remove_analysis_state",
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

    def test_restore_account_recovery_never_reimports_acknowledged_copy(self) -> None:
        _config, _before, after = self.maintenance_fixture(
            "current-restore", "accounts"
        )
        with (
            patch.object(server, "private_offline_state"),
            patch.object(server, "require_private_catalog"),
            patch.object(server, "recover_credentials", return_value=(after, True)),
            patch.object(server, "current_marker"),
            patch.object(server, "database_present", return_value=True),
            patch.object(server, "discard_database"),
            patch.object(server, "copy_current_database") as copy,
            patch.object(server, "remove_analysis_state") as cleanup,
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

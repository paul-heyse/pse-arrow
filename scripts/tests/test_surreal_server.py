# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Scoped controls for owned SurrealDB state and lifecycle boundaries."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import MagicMock, patch

from scripts import surreal_server as server

if TYPE_CHECKING:
    from collections.abc import Generator


class SurrealSupervisorTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        self.state = self.root / "state"

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
            ]
        )
        with (
            patch.object(
                server,
                "install",
                return_value={
                    "version": "3.3.0",
                    "binary": "/unused",
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
        config["primary_receiver"] = server.primary_receiver(worker)
        server.write_json(self.state / "config.json", config)
        return config

    def test_changed_primary_executable_requires_readmission(self) -> None:
        config = self.reference_fixture()
        server.checked_primary(config)
        (self.root / "pse-worker").write_text("#!/bin/sh\nexit 1\n")
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
        other.write_text("#!/bin/sh\nexit 0\n")
        other.chmod(0o700)
        # Even identical bytes at another selected path are another receiving artifact.
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
        config["primary_receiver"] = server.primary_receiver(worker)
        server.write_json(self.state / "config.json", config)
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

    def test_primary_quiesce_drains_recorded_executable_after_atomic_rebuild(
        self,
    ) -> None:
        config, child, observation = self.live_primary_fixture()
        worker = self.root / "pse-worker"
        replacement = self.root / "rebuilt-worker"
        replacement.write_bytes(b"different next admission")
        replacement.replace(worker)
        self.assertTrue(
            str(Path(f"/proc/{child.pid}/exe").readlink()).endswith(" (deleted)")
        )
        args = server.parser().parse_args(["quiesce", "--state", str(self.state)])
        with (
            patch.object(server, "primary_observation", return_value=observation),
            patch.object(server, "group_populated", return_value=True),
            patch.object(server, "role_affinity_ready", return_value=True),
            patch.object(
                server, "effective_limits", return_value=(140 * server.GIB, 16)
            ),
            patch.dict(
                os.environ,
                {"PSE_WORKER_PRODUCER_RECEIPT": str(self.root / "removed-receipt")},
            ),
            patch.object(server.signal, "pidfd_send_signal") as signal_pid,
            patch.object(server, "active", return_value=False),
            patch("builtins.print"),
        ):
            with self.assertRaisesRegex(server.SupervisorError, "bytes changed"):
                server.primary_ready(self.state, config, observation)
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
        with (
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
        with (
            patch.object(server, "physical_cpus", return_value=list(range(16))),
            patch.object(server, "group_for_slice", return_value=self.root / "group"),
            patch.object(server, "effective_limits", return_value=(None, None)),
            patch.object(
                server, "host_memory", return_value=(159 * server.GIB, 150 * server.GIB)
            ),
            patch.object(server, "systemctl") as manager,
            self.assertRaisesRegex(server.SupervisorError, "Host physical memory"),
        ):
            server.ensure_execution_placement(self.state, server.reference_resources())
        manager.assert_not_called()

    def test_available_memory_is_observed_without_upfront_reservation(self) -> None:
        self.reference_fixture()
        group = self.root / "cgroups/profile"
        group.mkdir(parents=True)
        (group / "memory.current").write_text(str(2 * server.GIB))
        (group / "cpuset.cpus.effective").write_text("0-15")
        with (
            patch.object(server, "physical_cpus", return_value=list(range(16))),
            patch.object(server, "group_for_slice", return_value=group),
            patch.object(
                server,
                "effective_limits",
                side_effect=[(None, None), (160 * server.GIB, 16)],
            ),
            patch.object(
                server, "host_memory", return_value=(188 * server.GIB, 112 * server.GIB)
            ),
            patch.object(server, "systemctl") as manager,
        ):
            server.ensure_execution_placement(self.state, server.reference_resources())
        self.assertEqual(manager.call_count, 2)
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
        with (
            patch.object(server, "physical_cpus", return_value=list(range(16))),
            patch.object(server, "group_for_slice", return_value=group),
            patch.object(
                server,
                "effective_limits",
                side_effect=[(None, None), (160 * server.GIB, 16)],
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
                    "32768",
                    "--server-memory-mib",
                    "16384",
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
            server.resources(32 * server.GIB, 16 * server.GIB, 2, 8 * server.GIB),
        )
        before.pop("resources")
        after.pop("resources")
        self.assertEqual(after, before)
        self.assertEqual((self.state / "credentials.json").read_bytes(), credentials)
        self.assertEqual((self.state / "database/fixture").read_bytes(), fixture)
        self.assertEqual(
            units, [server.worker_unit(self.state, slot) for slot in range(2)]
        )

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

    def test_reconfigure_checks_and_write_hold_existing_lifecycle_lock(self) -> None:
        self.quiesced_fixture()
        locked = False
        original_write = server.write_json
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
            self.assertTrue(locked)
            original_drain(state, config)

        def write(path: Path, value: dict[str, object]) -> None:
            self.assertTrue(locked)
            original_write(path, value)

        with (
            patch.object(server, "state_lock", side_effect=lock),
            patch.object(server, "workers_drained", side_effect=drain),
            patch.object(server, "write_json", side_effect=write) as written,
        ):
            self.reconfigure("--memory-mib", "8192")
        written.assert_called_once()
        self.assertFalse(locked)

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
            patch.object(server, "ready", return_value=True),
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
        self.assertIn(
            f"--property=MemoryMax={16 * server.GIB}",
            started.call_args.args[0],
        )
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
        destination = self.root / "backup"
        with patch.object(server, "active", return_value=False):
            server.backup(self.state, config, destination)
        return destination

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
            patch.object(server, "worker_observation", side_effect=[inactive, active]),
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
                server, "worker_observation", side_effect=[inactive, uncapped]
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
        with patch.object(server, "active", return_value=False):
            server.restore(backup, restored, "pse.substrate.v1")
        config = server.config_for(restored)
        self.assertFalse(config["accepting_writes"])
        self.assertEqual(config["admission"], "validation_required")
        self.assertEqual(
            (restored / "database/fixture").read_bytes(),
            b"acknowledged fixed operation identity",
        )
        self.assertEqual(config["credentials_file"], str(restored / "credentials.json"))
        with self.assertRaises(server.SupervisorError):
            server.start(restored, config)

    def test_backup_corruption_is_refused_before_state_creation(self) -> None:
        backup = self.backup_fixture()
        (backup / "database/fixture").write_bytes(b"changed")
        restored = self.root / "restored"
        with self.assertRaises(server.SupervisorError):
            server.restore(backup, restored, "pse.substrate.v1")
        self.assertFalse(restored.exists())

    def test_unknown_interpretation_is_refused(self) -> None:
        backup = self.backup_fixture()
        with self.assertRaises(server.SupervisorError):
            server.restore(backup, self.root / "restored", "unknown")

    def test_failed_semantic_validation_retains_write_gate(self) -> None:
        backup = self.backup_fixture()
        restored = self.root / "restored"
        with patch.object(server, "active", return_value=False):
            server.restore(backup, restored, "pse.substrate.v1")
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
            server.validate(restored, config, "pse.substrate.v1", ["unused"])
        self.assertFalse(server.config_for(restored)["accepting_writes"])
        self.assertEqual(
            server.config_for(restored)["admission"], "validation_required"
        )


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

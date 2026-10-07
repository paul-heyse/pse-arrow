# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Scoped controls for owned SurrealDB state and lifecycle boundaries."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import json
import subprocess
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
            allocation["rocksdb_block_cache_bytes"],
            allocation["memory_threshold_bytes"],
        )
        self.assertLess(
            allocation["memory_threshold_bytes"], allocation["server_memory_bytes"]
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

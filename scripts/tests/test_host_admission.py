# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Shared capacity, explicit widening and conservative surviving-owner controls."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import contextlib
import fcntl
import os
import subprocess
import sys
import tempfile
import time
import unittest
from pathlib import Path
from typing import TYPE_CHECKING
from unittest.mock import patch

from scripts import host_admission as host
from scripts import surreal_server

if TYPE_CHECKING:
    from collections.abc import Generator


class HostAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name) / "admission"

    def test_explicit_stop_withdraws_only_selected_queued_resume(self) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        selected = self.directory / "selected"
        other = str(self.directory / "other")
        with host.allocation_metadata(self.directory) as ledger:
            ledger["parked_services"] = [str(selected), other]
            original = dict(ledger["owners"])
        host.withdraw_parked_service(selected, directory=self.directory)
        with host.allocation_metadata(self.directory) as ledger:
            self.assertEqual(ledger["parked_services"], [other])
            self.assertEqual(ledger["owners"], original)
            self.assertIn(owner.nonce, ledger["owners"])

    def test_parked_reference_service_survives_two_refusals_then_admitted_resume(
        self,
    ) -> None:
        service = self.directory / "service"
        self.directory.mkdir(mode=0o700)
        service.mkdir(mode=0o700, parents=True)
        config = {
            "owner": surreal_server.OWNER,
            "profile_version": 2,
            "instance_id": "parked-reference",
            "port": 18240,
            "endpoint": "ws://127.0.0.1:18240",
            "namespace": "pse",
            "database": "canonical",
            "interpretation": surreal_server.SUBSTRATE_INTERPRETATION,
            "schema_interpretation": surreal_server.SUBSTRATE_INTERPRETATION,
            "websocket_max_message_bytes": surreal_server.MESSAGE_BYTES,
            "max_message_bytes": surreal_server.MESSAGE_BYTES,
            "resources": surreal_server.reference_resources(),
            "unit_materialized": True,
            "resident": True,
            "parked": True,
            "admission": "quiesced",
            "accepting_writes": False,
        }
        surreal_server.write_json(service / "config.json", config)
        host.queue_parked_service(self.directory, service)
        admitted = False
        owner: host.Allocation | None = None

        def inherit(_environment: object) -> host.Allocation | None:
            nonlocal owner
            if not admitted:
                return None
            # Admit the later matching owner after reconcile takes its queue snapshot.
            # Registering it beforehand would correctly defer reconciliation entirely.
            owner = host.acquire(host.select("reference"), directory=self.directory)
            return owner

        def materialize(
            state: Path, _config: dict[str, object], allocation: host.Allocation
        ) -> None:
            self.assertIs(allocation, owner)
            self.assertTrue(surreal_server.config_for(state)["parked"])
            self.assertTrue((state / "service-launch.json").is_file())

        with (
            patch.object(host, "root_path", return_value=self.directory),
            patch.object(host, "inherit", side_effect=inherit) as inherited,
            patch.object(host, "enforce_parent"),
            patch.object(
                host,
                "memory_info",
                return_value={"MemAvailable": 1 << 40, "MemTotal": 1 << 40},
            ),
            patch.object(surreal_server, "active", return_value=False),
            patch.object(
                surreal_server,
                "systemctl",
                return_value=subprocess.CompletedProcess([], 0, "", ""),
            ) as manager,
            patch.object(
                surreal_server, "materialize_service", side_effect=materialize
            ),
            patch.object(surreal_server, "listener_ready", return_value=True),
            patch.object(surreal_server, "establish_protocol_readiness") as ready,
        ):
            self.assertEqual(
                surreal_server.object_mapping(config["resources"])[
                    "server_memory_bytes"
                ],
                16 * host.GIB,
            )
            for attempt in range(2):
                with self.subTest(attempt=attempt):
                    host.reconcile(self.directory)
                    current = surreal_server.config_for(service)
                    self.assertTrue(current["parked"])
                    self.assertFalse(current["accepting_writes"])
                    self.assertEqual(current["admission"], "quiesced")
                    with host.allocation_metadata(self.directory) as ledger:
                        self.assertEqual(ledger["parked_services"], [str(service)])
                        self.assertEqual(ledger["owners"], {})
            self.assertEqual(inherited.call_count, 2)
            self.assertNotIn("start", [call.args[0] for call in manager.call_args_list])
            admitted = True
            host.reconcile(self.directory)
        ready.assert_called_once()
        self.assertEqual(inherited.call_count, 3)
        self.assertIsNotNone(owner)
        current = surreal_server.config_for(service)
        self.assertFalse(current["parked"])
        self.assertTrue(current["accepting_writes"])
        self.assertEqual(current["admission"], "open")
        assert owner is not None
        self.assertTrue(owner.profile.exclusive)
        with host.allocation_metadata(self.directory) as ledger:
            self.assertEqual(ledger["parked_services"], [])
            record = ledger["owners"][owner.nonce]
            self.assertIn(surreal_server.unit_name(service), record["units"])
            self.assertEqual(record["borrowed_services"], [str(service)])

    def test_explicit_stop_between_snapshot_and_parking_cannot_queue_restart(
        self,
    ) -> None:
        service = self.directory / "service"
        self.directory.mkdir(mode=0o700)
        service.mkdir(parents=True, mode=0o700)
        unit = surreal_server.unit_name(service)
        config: dict[str, object] = {
            "resident": True,
            "unit_materialized": True,
            "parked": False,
        }
        observed = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "/owned-service",
            "InvocationID": "a" * 32,
        }
        owner = host.acquire(host.select("store-functional"), directory=self.directory)
        with host.allocation_metadata(self.directory) as ledger:
            ledger["owners"][owner.nonce].update(
                service=str(service),
                parkable=True,
                units={
                    unit: {
                        "group": "/owned-service",
                        "invocation": "a" * 32,
                        "inode": 123,
                    }
                },
            )
        interleaved = False
        parking_deadlines: list[float | None] = []

        def observation(_unit: str, deadline: float | None) -> dict[str, str]:
            parking_deadlines.append(deadline)
            return dict(observed)

        def stop(_state: Path, _config: dict[str, object], *, abrupt: bool) -> None:
            self.assertFalse(abrupt)
            observed.update(ActiveState="inactive", ControlGroup="")
            self.assertTrue(owner.release())

        @contextlib.contextmanager
        def lifecycle(_state: Path) -> Generator[None, None, None]:
            nonlocal interleaved
            if not interleaved:
                interleaved = True
                args = surreal_server.parser().parse_args(
                    ["stop", "--state", str(service), "--drained"]
                )
                self.assertEqual(surreal_server.dispatch(args), 0)
            yield

        with (
            patch.object(host, "root_path", return_value=self.directory),
            patch.object(
                host,
                "memory_info",
                return_value={"MemAvailable": 1 << 40, "MemTotal": 1 << 40},
            ),
            patch.object(host, "retire_empty_allocation"),
            patch.object(host, "group_identity", return_value=123),
            patch.object(host.operation, "populated", return_value=False),
            patch.object(
                host.operation, "unit_observation", side_effect=lambda _: dict(observed)
            ),
            patch.object(host, "control_unit_observation", side_effect=observation),
            patch.object(
                surreal_server, "lifecycle_reservation", side_effect=lifecycle
            ),
            patch.object(surreal_server, "service_directory", return_value=service),
            patch.object(
                surreal_server, "config_for", side_effect=lambda _: dict(config)
            ),
            patch.object(
                surreal_server,
                "write_json",
                side_effect=lambda _path, value: config.update(value),
            ),
            patch.object(surreal_server, "public_status", return_value={}),
            patch.object(surreal_server, "stop", side_effect=stop) as stopped,
            patch.object(surreal_server, "start") as restarted,
            patch("builtins.print"),
        ):
            selected = host.acquire(host.select("reference"), directory=self.directory)
            self.assertTrue(interleaved)
            self.assertEqual(parking_deadlines, [selected.deadline])
            self.assertTrue(selected.release())
            host.reconcile(self.directory)
        self.assertFalse(config["parked"])
        self.assertEqual(observed["ActiveState"], "inactive")
        stopped.assert_called_once()
        restarted.assert_not_called()
        with host.allocation_metadata(self.directory) as ledger:
            self.assertNotIn(str(service), ledger.get("parked_services", []))

    def test_readonly_snapshot_preserves_metadata_bytes_inodes_and_modes(self) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        paths = [
            self.directory,
            self.directory / ".lock",
            self.directory / "allocations.json",
        ]
        before = [
            (
                p.stat().st_ino,
                p.stat().st_mode,
                p.stat().st_mtime_ns,
                p.read_bytes() if p.is_file() else None,
            )
            for p in paths
        ]
        with patch.object(
            host, "reconcile", side_effect=AssertionError("reconciliation")
        ):
            snapshot = host.readonly_snapshot(
                self.directory, deadline=time.monotonic() + 1
            )
        snapshot["owners"][owner.nonce]["units"]["pse-detached.scope"] = {}
        after = [
            (
                p.stat().st_ino,
                p.stat().st_mode,
                p.stat().st_mtime_ns,
                p.read_bytes() if p.is_file() else None,
            )
            for p in paths
        ]
        self.assertEqual(before, after)
        self.assertNotIn(
            "pse-detached.scope",
            host.readonly_snapshot(self.directory, deadline=time.monotonic() + 1)[
                "owners"
            ][owner.nonce]["units"],
        )

    def test_snapshot_missing_unsafe_and_corrupt_state_never_repairs(self) -> None:
        with self.assertRaises(OSError):
            host.readonly_snapshot(self.directory, deadline=time.monotonic() + 1)
        self.assertFalse(self.directory.exists())
        host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        ledger = self.directory / "allocations.json"
        ledger.chmod(0o644)
        with self.assertRaisesRegex(host.AdmissionError, "Unsafe admission ledger"):
            host.readonly_snapshot(self.directory, deadline=time.monotonic() + 1)
        self.assertEqual(ledger.stat().st_mode & 0o777, 0o644)
        ledger.chmod(0o600)
        ledger.write_text('{"version":1,"owners":{"bad":{}}}')
        before = ledger.read_bytes()
        with self.assertRaises(host.AdmissionError):
            host.readonly_snapshot(self.directory, deadline=time.monotonic() + 1)
        self.assertEqual(ledger.read_bytes(), before)

    def test_snapshot_and_control_admission_locks_obey_absolute_deadline(self) -> None:
        host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        descriptor = os.open(self.directory / ".lock", os.O_RDWR)
        self.addCleanup(os.close, descriptor)
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        before = (self.directory / "allocations.json").read_bytes()
        for action in (host.readonly_snapshot, host.acquire_light_control):
            started = time.monotonic()
            with self.assertRaisesRegex(host.AdmissionError, "deadline"):
                action(directory=self.directory, deadline=started + 0.03)
            self.assertLess(time.monotonic() - started, 0.2)
        self.assertEqual((self.directory / "allocations.json").read_bytes(), before)

    def test_control_keeps_stale_owners_charged_and_never_reconciles_services(
        self,
    ) -> None:
        with (
            patch.object(host, "reconcile", side_effect=AssertionError("reconcile")),
            patch.object(
                host, "drain_borrowed_services", side_effect=AssertionError("drain")
            ),
            patch.object(
                surreal_server, "unpark_service", side_effect=AssertionError("unpark")
            ),
        ):
            owner = host.acquire_light_control(
                directory=self.directory, deadline=time.monotonic() + 1
            )
            with host.allocation_metadata(self.directory) as state:
                state["owners"][owner.nonce]["pid"] = -1
                state["owners"][owner.nonce]["start"] = "gone"
                state["owners"][owner.nonce]["service"] = "/missing/service"
            second = host.acquire_light_control(
                directory=self.directory, deadline=time.monotonic() + 1
            )
            self.assertTrue(
                host.release_light_control(second, deadline=time.monotonic() + 1)
            )
            self.assertIn(
                owner.nonce,
                host.readonly_snapshot(self.directory, deadline=time.monotonic() + 1)[
                    "owners"
                ],
            )

    def test_control_cleanup_retains_active_unknown_and_expired_ownership(self) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        owner.register("pse-control.scope", deadline=time.monotonic() + 1)
        for observation in (
            {"LoadState": "loaded", "ActiveState": "active", "ControlGroup": "/g"},
            ValueError("unavailable"),
        ):
            mocked = (
                patch.object(host, "control_unit_observation", side_effect=observation)
                if isinstance(observation, Exception)
                else patch.object(
                    host, "control_unit_observation", return_value=observation
                )
            )
            with mocked:
                self.assertFalse(
                    host.release_light_control(owner, deadline=time.monotonic() + 0.03)
                )
        self.assertFalse(
            host.release_light_control(owner, deadline=time.monotonic() - 1)
        )
        self.assertIn(
            owner.nonce,
            host.readonly_snapshot(self.directory, deadline=time.monotonic() + 1)[
                "owners"
            ],
        )
        with patch.object(
            host,
            "control_unit_observation",
            return_value={
                "LoadState": "not-found",
                "ActiveState": "inactive",
                "ControlGroup": "",
            },
        ):
            self.assertTrue(
                host.release_light_control(owner, deadline=time.monotonic() + 1)
            )

    def test_control_cleanup_waits_for_manager_drain_inside_original_clock(
        self,
    ) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        owner.register("pse-control.scope", deadline=time.monotonic() + 1)
        deadline = time.monotonic() + 0.1
        with (
            patch.object(host, "retire_light_control", return_value=True),
            patch.object(
                host,
                "control_unit_observation",
                side_effect=[
                    {
                        "LoadState": "loaded",
                        "ActiveState": "active",
                        "ControlGroup": "/g",
                    },
                    {
                        "LoadState": "not-found",
                        "ActiveState": "inactive",
                        "ControlGroup": "",
                    },
                ],
            ) as observe,
        ):
            self.assertTrue(host.release_light_control(owner, deadline=deadline))
        self.assertEqual(observe.call_count, 2)
        self.assertTrue(
            all(call.args[1] == deadline for call in observe.call_args_list)
        )

    def test_parent_configuration_lock_uses_control_deadline(self) -> None:
        host.protected(self.directory)
        path = self.directory / ".parent-update.lock"
        descriptor = os.open(path, os.O_CREAT | os.O_RDWR, 0o600)
        self.addCleanup(os.close, descriptor)
        fcntl.flock(descriptor, fcntl.LOCK_EX)
        started = time.monotonic()
        with (
            self.assertRaisesRegex(host.AdmissionError, "deadline"),
            host.parent_update(self.directory, deadline=started + 0.03),
        ):
            self.fail("lock acquired")
        self.assertLess(time.monotonic() - started, 0.2)

    def test_control_wait_and_cancellation_use_original_deadline(self) -> None:
        started = time.monotonic()
        # A descendant inherits stdout: pipe-based subprocess.run cleanup would
        # wait for it after killing the direct child. Regular-file capture cannot.
        program = "import subprocess,time,sys; subprocess.Popen([sys.executable,'-c','import time; time.sleep(.08)']); time.sleep(1)"
        with self.assertRaises(subprocess.TimeoutExpired):
            host.control_run(
                [sys.executable, "-c", program], env=os.environ, deadline=started + 0.04
            )
        self.assertLess(time.monotonic() - started, 0.2)

    def test_readonly_inheritance_releases_lock_before_unit_observation(self) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        group = host.operation.process_group(os.getpid())
        with host.allocation_metadata(self.directory) as state:
            state["owners"][owner.nonce]["units"]["pse-owned.scope"] = {
                "group": group,
                "invocation": "a" * 32,
            }

        def observe(_unit: str, _deadline: float | None) -> dict[str, str]:
            fd = os.open(self.directory / ".lock", os.O_RDWR)
            try:
                fcntl.flock(fd, fcntl.LOCK_EX | fcntl.LOCK_NB)
            finally:
                os.close(fd)
            return {"ControlGroup": group, "InvocationID": "a" * 32}

        before = (self.directory / "allocations.json").read_bytes()
        with patch.object(host, "control_unit_observation", side_effect=observe):
            inherited = host.inherit(owner.environment(), deadline=time.monotonic() + 1)
        assert inherited is not None
        self.assertEqual(inherited.nonce, owner.nonce)
        self.assertEqual((self.directory / "allocations.json").read_bytes(), before)

    def test_fixed_control_child_binds_only_its_actual_scope(self) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        owner.register(
            "pse-control-" + "b" * 32 + ".scope", deadline=time.monotonic() + 1
        )
        with (
            patch.object(
                host.operation, "process_group", return_value="/control/child"
            ),
            patch.object(
                host,
                "control_unit_observation",
                return_value={"ControlGroup": "/control", "InvocationID": "c" * 32},
            ),
            patch.object(host, "group_identity", return_value=91),
        ):
            verified = host.verify_light_control_child(
                owner.environment(), deadline=time.monotonic() + 1
            )
        self.assertEqual(verified.nonce, owner.nonce)
        record = host.readonly_snapshot(self.directory, deadline=time.monotonic() + 1)[
            "owners"
        ][owner.nonce]
        self.assertEqual(
            record["units"],
            {
                "pse-control-" + "b" * 32 + ".scope": {
                    "group": "/control",
                    "invocation": "c" * 32,
                    "inode": 91,
                }
            },
        )
        with (
            patch.object(host.operation, "process_group", return_value="/outside"),
            patch.object(
                host,
                "control_unit_observation",
                return_value={"ControlGroup": "/control", "InvocationID": "c" * 32},
            ),
            self.assertRaisesRegex(host.AdmissionError, "outside"),
        ):
            host.verify_light_control_child(
                owner.environment(), deadline=time.monotonic() + 1
            )

    def test_bounded_inheritance_and_control_fallback_reject_stale_bound_identity(
        self,
    ) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        unit = "pse-control-" + "b" * 32 + ".scope"
        owner.register(unit, deadline=time.monotonic() + 1)
        with host.allocation_metadata(self.directory) as state:
            record = state["owners"][owner.nonce]
            record["pid"] = os.getppid()
            record["start"] = host.operation.start_identity(os.getppid())
            record["units"][unit] = {
                "group": "/control",
                "invocation": "c" * 32,
                "inode": 91,
            }
        ledger = self.directory / "allocations.json"
        before = (ledger.read_bytes(), ledger.stat().st_ino)
        for invocation, inode in (("d" * 32, 91), ("c" * 32, 92)):
            with (
                self.subTest(invocation=invocation, inode=inode),
                patch.object(
                    host.operation, "process_group", return_value="/control/child"
                ),
                patch.object(
                    host,
                    "control_unit_observation",
                    return_value={
                        "ControlGroup": "/control",
                        "InvocationID": invocation,
                    },
                ),
                patch.object(host, "group_identity", return_value=inode),
            ):
                with self.assertRaisesRegex(host.AdmissionError, "actual cgroup"):
                    host.inherit(owner.environment(), deadline=time.monotonic() + 1)
                with self.assertRaisesRegex(host.AdmissionError, "outside"):
                    host.verify_light_control_child(
                        owner.environment(), deadline=time.monotonic() + 1
                    )
            self.assertEqual((ledger.read_bytes(), ledger.stat().st_ino), before)
        with (
            patch.object(
                host.operation, "process_group", return_value="/control/child"
            ),
            patch.object(
                host,
                "control_unit_observation",
                return_value={"ControlGroup": "/control", "InvocationID": "c" * 32},
            ),
            patch.object(host, "group_identity", return_value=91),
        ):
            inherited = host.inherit(owner.environment(), deadline=time.monotonic() + 1)
            assert inherited is not None
            self.assertEqual(inherited.nonce, owner.nonce)
            with self.assertRaisesRegex(host.AdmissionError, "already bound"):
                owner.bind(unit, deadline=time.monotonic() + 1)
        self.assertEqual((ledger.read_bytes(), ledger.stat().st_ino), before)

    def test_unbound_control_child_refuses_dead_launch_parent_without_writes(
        self,
    ) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        owner.register(
            "pse-control-" + "b" * 32 + ".scope", deadline=time.monotonic() + 1
        )
        with host.allocation_metadata(self.directory) as state:
            state["owners"][owner.nonce].update(pid=-1, start="gone")
        ledger = self.directory / "allocations.json"
        before = ledger.read_bytes()
        with self.assertRaisesRegex(host.AdmissionError, "no longer live"):
            host.verify_light_control_child(
                owner.environment(), deadline=time.monotonic() + 1
            )
        self.assertEqual(ledger.read_bytes(), before)

    def test_control_cleanup_uses_bound_empty_kernel_lifetime_before_gc(self) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        unit = "pse-control-" + "b" * 32 + ".scope"
        owner.register(unit, deadline=time.monotonic() + 1)
        with host.allocation_metadata(self.directory) as state:
            state["owners"][owner.nonce]["units"][unit] = {
                "group": "/control",
                "invocation": "c" * 32,
                "inode": 91,
            }
        with (
            patch.object(host, "group_identity", return_value=91),
            patch.object(host.operation, "populated", return_value=False),
            patch.object(
                host,
                "control_unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "ControlGroup": "/control",
                    "InvocationID": "c" * 32,
                },
            ),
            patch.object(host, "retire_light_control", return_value=True),
        ):
            self.assertTrue(
                host.release_light_control(owner, deadline=time.monotonic() + 1)
            )

    def test_control_retirement_stops_only_empty_matching_owned_units(self) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        unit = "pse-control-" + "b" * 32 + ".scope"
        parent = "/pse.slice/" + host.allocation_slice(owner)
        group = parent + "/" + unit
        owner.register(unit, deadline=time.monotonic() + 1)
        with host.allocation_metadata(self.directory) as state:
            state["owners"][owner.nonce]["units"][unit] = {
                "group": group,
                "invocation": "c" * 32,
                "inode": 91,
            }
        record = host.readonly_snapshot(self.directory, deadline=time.monotonic() + 1)[
            "owners"
        ][owner.nonce]

        def observe(name: str, _deadline: float) -> dict[str, str]:
            return {
                "LoadState": "loaded",
                "ActiveState": "active",
                "ControlGroup": parent if name.endswith(".slice") else group,
                "InvocationID": "c" * 32,
            }

        deadline = time.monotonic() + 1
        with (
            patch.object(host, "control_unit_observation", side_effect=observe),
            patch.object(
                host,
                "group_identity",
                side_effect=lambda selected: 92 if selected == parent else 91,
            ),
            patch.object(host.operation, "populated", return_value=False),
            patch.object(
                host,
                "control_run",
                return_value=subprocess.CompletedProcess([], 0, "", ""),
            ) as run,
        ):
            self.assertTrue(host.retire_light_control(owner, record, deadline=deadline))
        self.assertEqual(
            [call.args[0][-1] for call in run.call_args_list],
            [unit, host.allocation_slice(owner)],
        )
        self.assertTrue(
            all(call.kwargs["deadline"] == deadline for call in run.call_args_list)
        )
        for invalid in (
            "parent-populated",
            "wrong-invocation",
            "wrong-inode",
            "stop-failure",
        ):
            with (
                self.subTest(invalid=invalid),
                patch.object(
                    host,
                    "control_unit_observation",
                    side_effect=(
                        lambda name, clock: {
                            **observe(name, clock),
                            "InvocationID": "d" * 32,
                        }
                    )
                    if invalid == "wrong-invocation"
                    else observe,
                ),
                patch.object(
                    host,
                    "group_identity",
                    side_effect=lambda selected, selected_case=invalid: (
                        92
                        if selected == parent
                        else 93
                        if selected_case == "wrong-inode"
                        else 91
                    ),
                ),
                patch.object(
                    host.operation,
                    "populated",
                    side_effect=lambda selected, selected_case=invalid: (
                        selected_case == "parent-populated" and selected == parent
                    ),
                ),
                patch.object(
                    host,
                    "control_run",
                    return_value=subprocess.CompletedProcess(
                        [], 1 if invalid == "stop-failure" else 0, "", ""
                    ),
                ) as run,
            ):
                self.assertFalse(
                    host.retire_light_control(owner, record, deadline=deadline)
                )
                if invalid != "stop-failure":
                    run.assert_not_called()
                else:
                    self.assertFalse(
                        host.release_light_control(owner, deadline=deadline)
                    )
                    self.assertIn(
                        owner.nonce,
                        host.readonly_snapshot(self.directory, deadline=deadline)[
                            "owners"
                        ],
                    )

    def test_control_enforcement_calls_share_deadline(self) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        deadline = time.monotonic() + 1
        with (
            patch.object(
                host,
                "control_run",
                return_value=subprocess.CompletedProcess([], 0, "", ""),
            ) as run,
            patch("scripts.pse_env.limits", return_value=[]),
            patch.object(host, "root_path", return_value=self.directory),
        ):
            host.enforce_parent(owner.profile, {}, deadline=deadline)
        self.assertEqual(run.call_args.kwargs["deadline"], deadline)
        self.assertEqual(run.call_args.kwargs["maximum"], 5)

    def test_profile_widening_preserves_reference_and_role_sums(self) -> None:
        self.assertEqual(host.select("functional", "64G").name, "wide")
        self.assertEqual(host.select("functional", "128G").name, "exclusive")
        reference = host.select("reference")
        self.assertEqual(reference.memory, 140 * host.GIB)
        self.assertEqual(host.aggregate(reference), 160 * host.GIB)
        with self.assertRaisesRegex(host.AdmissionError, "unchanged"):
            host.select("reference", "64G")
        for name in ("functional", "wide", "timing", "exclusive"):
            profile = host.select(name)
            declared = host.settings(name)
            execution = host.execution(name)
            self.assertEqual(
                profile.memory,
                (
                    declared["receiver_gib"]
                    + declared["observer_gib"]
                    + declared["control_gib"]
                )
                * host.GIB,
            )
            self.assertLessEqual(
                execution["pool_memory_bytes"] + execution["process_headroom_bytes"],
                declared["receiver_gib"] * host.GIB,
            )
        for value in ("off", "infinity", "0", "999T"):
            with self.assertRaises(host.AdmissionError):
                host.select("functional", value)

    def test_two_independent_callers_share_slots_and_original_timeout(self) -> None:
        first = host.acquire(host.select("functional"), directory=self.directory)
        second = host.acquire(host.select("functional"), directory=self.directory)
        self.assertFalse(set(first.profile.cores).intersection(second.profile.cores))
        begun = time.monotonic()
        with self.assertRaisesRegex(host.AdmissionError, "slots occupied"):
            host.acquire(
                host.select("functional"),
                directory=self.directory,
                deadline=begun + 0.03,
            )
        self.assertLess(time.monotonic() - begun, 0.2)
        self.assertTrue(first.release())
        third = host.acquire(host.select("functional"), directory=self.directory)
        self.assertFalse(set(second.profile.cores).intersection(third.profile.cores))
        self.assertTrue(second.release())
        self.assertTrue(third.release())

    def test_original_deadline_and_actual_membership_required_for_reentry(self) -> None:
        owner = host.acquire(host.select("compile"), directory=self.directory)
        nested = host.inherit(owner.environment())
        assert nested is not None
        self.assertEqual(nested.deadline, owner.deadline)
        with (
            patch.object(host.operation, "start_identity", return_value="reused"),
            self.assertRaisesRegex(host.AdmissionError, "actual cgroup"),
        ):
            host.inherit(owner.environment())
        self.assertTrue(owner.release())

    def test_parent_death_and_pid_reuse_do_not_release_surviving_unit(self) -> None:
        owner = host.acquire(host.select("functional"), directory=self.directory)
        owner.register("pse-owned-test.scope")
        with host.allocation_metadata(self.directory) as state:
            record = state["owners"][owner.nonce]
            record["start"] = "previous-start"
            record["units"]["pse-owned-test.scope"] = {
                "group": "/owned",
                "invocation": "a" * 32,
            }
        with (
            patch.object(host.operation, "populated", return_value=True),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={"LoadState": "loaded", "InvocationID": "a" * 32},
            ),
        ):
            host.reconcile(self.directory)
            with host.allocation_metadata(self.directory) as state:
                self.assertIn(owner.nonce, state["owners"])
        with (
            patch.object(host.operation, "populated", return_value=False),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={"LoadState": "not-found"},
            ),
        ):
            host.reconcile(self.directory)
            with host.allocation_metadata(self.directory) as state:
                self.assertNotIn(owner.nonce, state["owners"])

    def test_unknown_unit_and_corrupt_ledger_refuse_capacity_recovery(self) -> None:
        owner = host.acquire(host.select("functional"), directory=self.directory)
        owner.register("pse-owned-test.scope")
        with patch.object(
            host.operation, "unit_observation", side_effect=ValueError("unknown")
        ):
            self.assertFalse(owner.release())
        (self.directory / "allocations.json").write_text('{"version":99}')
        with self.assertRaisesRegex(host.AdmissionError, "Corrupt"):
            host.reconcile(self.directory)

    def test_corrupt_host_owner_cannot_be_released_as_an_old_boot(self) -> None:
        owner = host.acquire(host.select("functional"), directory=self.directory)
        with host.metadata(self.directory) as ledger:
            ledger["owners"][owner.nonce].pop("boot")
        with self.assertRaisesRegex(host.AdmissionError, "Corrupt host allocation"):
            host.reconcile(self.directory)
        with host.metadata(self.directory) as ledger:
            self.assertIn(owner.nonce, ledger["owners"])
        self.assertFalse(host.drained({"units": {}, "released": True}))

    def test_persistent_unit_restart_does_not_charge_destroyed_generation(self) -> None:
        record = {
            "boot": host.boot(),
            "pid": -1,
            "start": "gone",
            "released": True,
            "units": {
                "owned.service": {
                    "group": "/owned",
                    "invocation": "a" * 32,
                    "inode": 123,
                }
            },
        }
        with (
            patch.object(host, "group_identity", return_value=456),
            patch.object(host.operation, "populated", return_value=True),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "InvocationID": "b" * 32,
                },
            ),
        ):
            self.assertTrue(host.drained(record))

    def test_resident_restart_retains_charge_until_the_service_is_stopped(self) -> None:
        record = {
            "boot": host.boot(),
            "pid": -1,
            "start": "gone",
            "service": "/owned/store",
            "units": {
                "owned.service": {
                    "group": "/owned",
                    "invocation": "a" * 32,
                    "inode": 123,
                }
            },
        }
        with patch.object(host, "group_identity", return_value=456):
            for status in ("activating", "active", "deactivating"):
                with patch.object(
                    host.operation,
                    "unit_observation",
                    return_value={"LoadState": "loaded", "ActiveState": status},
                ):
                    self.assertFalse(host.drained(record))
            with patch.object(
                host.operation,
                "unit_observation",
                return_value={"LoadState": "loaded", "ActiveState": "inactive"},
            ):
                self.assertTrue(host.drained(record))
        with (
            patch.object(host, "group_identity", return_value=123),
            patch.object(host.operation, "populated", return_value=False),
        ):
            for invocation in ("", "a" * 32):
                with patch.object(
                    host.operation,
                    "unit_observation",
                    return_value={
                        "LoadState": "loaded",
                        "ActiveState": "activating",
                        "InvocationID": invocation,
                    },
                ):
                    self.assertFalse(host.drained(record))
        record["units"]["owned.service"].pop("inode")
        with (
            patch.object(host.operation, "populated", return_value=False),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "inactive",
                    "InvocationID": "b" * 32,
                },
            ),
        ):
            self.assertTrue(host.drained(record))

    def test_timing_is_disjoint_and_exclusive_excludes_heavy(self) -> None:
        functional = host.acquire(host.select("wide"), directory=self.directory)
        timing = host.acquire(host.select("timing"), directory=self.directory)
        self.assertFalse(
            set(functional.profile.cores).intersection(timing.profile.cores)
        )
        with self.assertRaisesRegex(host.AdmissionError, "exclusive"):
            host.acquire(
                host.select("reference"),
                directory=self.directory,
                deadline=time.monotonic() + 0.02,
            )
        self.assertTrue(functional.release())
        self.assertTrue(timing.release())
        reference = host.acquire(host.select("reference"), directory=self.directory)
        light = host.acquire(host.select("light"), directory=self.directory)
        self.assertTrue(light.release())
        self.assertTrue(reference.release())

    def test_cpu_topology_matches_declared_physical_lanes(self) -> None:
        functional = set(host.cpu_set(host.select("wide").cores))
        timing = set(host.cpu_set(host.select("timing").cores))
        self.assertFalse(functional.intersection(timing))
        self.assertEqual(functional | timing, set(range(32)))

    def test_borrow_release_never_resumes_intentionally_stopped_resident(self) -> None:
        service = str(self.directory / "service")
        record = {"borrowed_services": [service], "units": {}}
        with (
            patch.object(host, "drained", return_value=True),
            patch.object(
                surreal_server,
                "lifecycle_reservation",
                side_effect=lambda _: contextlib.nullcontext(),
            ),
            patch.object(surreal_server, "config_for", return_value={"resident": True}),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={"ActiveState": "inactive"},
            ),
            patch.object(surreal_server, "workers_drained"),
            patch.object(surreal_server, "stop") as stop,
            patch.object(surreal_server, "park_service") as park,
        ):
            host.drain_borrowed_services(self.directory, record, explicit=True)
        stop.assert_called_once()
        park.assert_not_called()
        with host.allocation_metadata(self.directory) as ledger:
            self.assertNotIn(service, ledger.get("parked_services", []))

    def test_borrowed_partial_park_preserves_actual_resume_intent(self) -> None:
        service = self.directory / "service"
        config: dict[str, object] = {"resident": True, "parked": False}

        def park(_state: Path) -> None:
            config["parked"] = True
            raise surreal_server.SupervisorError("partial owned stop")

        with (
            patch.object(host, "drained", return_value=True),
            patch.object(
                surreal_server,
                "lifecycle_reservation",
                side_effect=lambda _: contextlib.nullcontext(),
            ),
            patch.object(
                surreal_server, "config_for", side_effect=lambda _: dict(config)
            ),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={"ActiveState": "active"},
            ),
            patch.object(surreal_server, "park_service", side_effect=park),
            self.assertRaisesRegex(
                surreal_server.SupervisorError, "partial owned stop"
            ),
        ):
            host.drain_borrowed_services(
                self.directory,
                {"borrowed_services": [str(service)], "units": {}},
                explicit=True,
            )
        with host.allocation_metadata(self.directory) as ledger:
            self.assertIn(str(service), ledger.get("parked_services", []))


if __name__ == "__main__":
    unittest.main()

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
import threading
import time
import unittest
from concurrent.futures import ThreadPoolExecutor
from pathlib import Path
from typing import TYPE_CHECKING, cast
from unittest.mock import patch

from scripts import host_admission as host
from scripts import surreal_server

if TYPE_CHECKING:
    from collections.abc import Generator


def readiness_fixture(
    phase: surreal_server.Readiness,
) -> surreal_server.ServiceObservation:
    return surreal_server.ServiceObservation(
        phase,
        Path("/fixture"),
        "fixture",
        "fixture.service",
        "controlled observation",
        manager={"ActiveState": "active"}
        if phase in {surreal_server.Readiness.READY, surreal_server.Readiness.STARTING}
        else {},
        listener_owned=phase
        in {surreal_server.Readiness.READY, surreal_server.Readiness.STARTING},
        protocol_ready=phase is surreal_server.Readiness.READY,
    )


class HostAdmissionTests(unittest.TestCase):
    def test_native_unit_aliases_select_the_existing_scientific_lane_before_nesting(
        self,
    ) -> None:
        for recipe in (
            "unit-package",
            "unit-libraries",
            "unit-native-selected",
            "unit-native-package",
            "unit-native-capability-package",
            "unit-native-contracts",
        ):
            with self.subTest(recipe=recipe):
                self.assertEqual(
                    host.classify(
                        ["just", recipe, "pse-runtime", "test(example)"], native=True
                    ),
                    "exclusive",
                )
        self.assertEqual(
            host.classify(
                ["just", "unit-native-capability-package", "--managed-primary-route"],
                native=True,
            ),
            "reference",
        )
        self.assertEqual(host.classify(["just", "py-unit"], native=True), "compile")

    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name) / "admission"

    def storage_process_fixture(
        self, *, borrowed: bool = False
    ) -> tuple[host.Allocation, Path, str]:
        with patch.object(
            host,
            "memory_info",
            return_value={"MemAvailable": 1 << 40, "MemTotal": 1 << 40},
        ):
            owner = host.acquire(
                host.select("store-functional"),
                directory=self.directory,
                deadline=time.monotonic() + 10,
            )
        service = self.directory / "service"
        service.mkdir()
        unit = surreal_server.unit_name(service)
        with host.allocation_metadata(self.directory) as ledger:
            record = ledger["owners"][owner.nonce]
            record.update(pid=-1, start="ended")
            if borrowed:
                record["borrowed_services"] = [str(service)]
            else:
                record["service"] = str(service)
            record["units"][unit] = {
                "group": "/owned-store",
                "inode": 123,
                "invocation": "a" * 32,
            }
        surreal_server.write_json(
            service / "service-launch.json",
            {"allocation": str(self.directory / owner.nonce), "generation": "original"},
        )
        surreal_server.write_json(
            service / "server-process.json",
            {
                "allocation": str(self.directory / owner.nonce),
                "instance_id": "original",
                "pid": 12345,
                "start": "child",
            },
        )
        return owner, service, unit

    def test_concurrent_acquisition_retains_exact_zombie_while_stop_waits(self) -> None:
        owner, service, _unit = self.storage_process_fixture()
        child = subprocess.Popen([sys.executable, "-c", "pass"])
        self.addCleanup(child.wait)
        os.waitid(os.P_PID, child.pid, os.WEXITED | os.WNOWAIT)
        process = surreal_server.read_json(service / "server-process.json")
        process.update(pid=child.pid, start=host.operation.start_identity(child.pid))
        surreal_server.write_json(service / "server-process.json", process)
        entered, resume = threading.Event(), threading.Event()
        config: dict[str, object] = {"admission": "quiesced", "accepting_writes": False}

        def wait(_duration: float) -> None:
            entered.set()
            if not resume.wait(5):
                raise AssertionError("stop wait was not released")

        def stop() -> None:
            surreal_server._STARTUP.deadline = time.monotonic() + 10  # noqa: SLF001 -- exercise the captured stop clock directly
            surreal_server._stop(service, config)  # noqa: SLF001 -- hold exact process drain independently of lifecycle admission

        def observation(
            *arguments: str, **_kwargs: object
        ) -> subprocess.CompletedProcess[str]:
            return subprocess.CompletedProcess(
                [],
                0,
                "" if "--property=ControlGroup" in arguments else "inactive\n",
                "",
            )

        with (
            patch.object(surreal_server, "systemctl", side_effect=observation),
            patch.object(surreal_server, "all_contexts_drained"),
            patch.object(surreal_server, "release_stopped_service") as released,
            patch.object(surreal_server.time, "sleep", side_effect=wait),
            patch.object(host, "group_identity", return_value=123),
            patch.object(host.operation, "populated", return_value=False),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "inactive",
                    "ControlGroup": "",
                },
            ),
            ThreadPoolExecutor(max_workers=1) as executor,
        ):
            pending = executor.submit(stop)
            try:
                self.assertTrue(entered.wait(5))
                observer = host.acquire(
                    host.select("light"),
                    directory=self.directory,
                    deadline=time.monotonic() + 5,
                )
                with host.allocation_metadata(self.directory) as ledger:
                    self.assertIn(owner.nonce, ledger["owners"])
                released.assert_not_called()
                child.wait()
            finally:
                resume.set()
            pending.result(timeout=5)
            released.assert_called_once_with(service, config)
            host.reconcile(self.directory, resume_parked=False)
            with host.allocation_metadata(self.directory) as ledger:
                self.assertNotIn(owner.nonce, ledger["owners"])
                self.assertIn(observer.nonce, ledger["owners"])

    def test_storage_reclaim_requires_matching_process_affiliation(self) -> None:
        owner, service, _unit = self.storage_process_fixture()
        with host.allocation_metadata(self.directory) as ledger:
            record = cast("host.AllocationRecord", dict(ledger["owners"][owner.nonce]))
        original = surreal_server.read_json(service / "server-process.json")
        for changes in (
            {"allocation": str(self.directory / ("b" * 32))},
            {"instance_id": "other"},
            {"start": None},
        ):
            with self.subTest(changes=changes):
                surreal_server.write_json(
                    service / "server-process.json", {**original, **changes}
                )
                self.assertFalse(
                    host.drained(record, allocation=self.directory / owner.nonce)
                )
        (service / "server-process.json").unlink()
        self.assertFalse(host.drained(record, allocation=self.directory / owner.nonce))
        (service / "server-process.json").write_text("not JSON")
        self.assertFalse(host.drained(record, allocation=self.directory / owner.nonce))
        surreal_server.write_json(service / "server-process.json", original)
        with patch.object(
            host.operation, "start_identity", side_effect=PermissionError("unavailable")
        ):
            self.assertFalse(
                host.drained(record, allocation=self.directory / owner.nonce)
            )
        with patch.object(
            host, "boot", return_value="00000000-0000-0000-0000-000000000000"
        ):
            self.assertTrue(
                host.drained(record, allocation=self.directory / owner.nonce)
            )

    def test_stale_service_descriptor_does_not_pin_different_allocation(self) -> None:
        owner, service, _unit = self.storage_process_fixture()
        launch = surreal_server.read_json(service / "service-launch.json")
        launch["allocation"] = str(self.directory / ("b" * 32))
        surreal_server.write_json(service / "service-launch.json", launch)
        with (
            patch.object(host, "group_identity", return_value=123),
            patch.object(host.operation, "populated", return_value=False),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={"LoadState": "loaded", "ActiveState": "inactive"},
            ),
        ):
            host.reconcile(self.directory, resume_parked=False)
        with host.allocation_metadata(self.directory) as ledger:
            self.assertNotIn(owner.nonce, ledger["owners"])

    def test_borrowed_storage_guard_only_applies_to_selected_service_unit(self) -> None:
        owner, _service, _unit = self.storage_process_fixture(borrowed=True)
        with host.allocation_metadata(self.directory) as ledger:
            record = cast("host.AllocationRecord", dict(ledger["owners"][owner.nonce]))
        with patch.object(host.operation, "start_identity", return_value="child"):
            self.assertFalse(
                host.drained(record, allocation=self.directory / owner.nonce)
            )
            self.assertFalse(host.drained(record))
        observer = {**record, "units": {}, "released": True}
        self.assertTrue(host.drained(observer))

    def test_unbound_not_found_storage_without_child_can_be_reclaimed(self) -> None:
        owner, service, unit = self.storage_process_fixture()
        (service / "server-process.json").unlink()
        with host.allocation_metadata(self.directory) as ledger:
            ledger["owners"][owner.nonce]["units"][unit] = {}
        with patch.object(
            host.operation, "unit_observation", return_value={"LoadState": "not-found"}
        ):
            host.reconcile(self.directory, resume_parked=False)
        with host.allocation_metadata(self.directory) as ledger:
            self.assertNotIn(owner.nonce, ledger["owners"])

    def test_failed_unbound_restart_retains_receipts_and_requires_complete_drain(
        self,
    ) -> None:
        owner, service, unit = self.storage_process_fixture()
        process = surreal_server.read_json(service / "server-process.json")
        process["allocation"] = str(self.directory / ("b" * 32))
        launch = surreal_server.read_json(service / "service-launch.json")
        launch.update(
            deadline=time.monotonic() + 10,
            supervisor_generation={},
            server_generation={},
            service_class="functional",
            server_memory_bytes=host.GIB,
        )
        config = {
            "instance_id": "original",
            "service_supervisor": {},
            "server": {},
            "resources": {"server_memory_bytes": host.GIB},
        }
        manager = {
            "LoadState": "loaded",
            "ActiveState": "failed",
            "ControlGroup": "",
            "InvocationID": "a" * 32,
        }
        absent = surreal_server.ServiceObservation(
            surreal_server.Readiness.ACTIONABLE_ABSENT,
            service,
            "original",
            unit,
            "terminal exact owner",
            manager=manager,
        )
        with host.allocation_metadata(self.directory) as ledger:
            record = dict(ledger["owners"][owner.nonce])
        record["units"] = {unit: {}}
        for case in (
            "drained",
            "wrong-generation",
            "bound",
            "bound-launch",
            "live",
            "reused",
            "unknown-process",
            "unknown-unit",
            "active-unit",
            "listener",
            "changed-invocation",
            "changed-selection",
            "foreign-ledger",
            "relative-affiliation",
            "malformed-affiliation",
            "alias-current",
            "alias-previous",
            "manager-timeout",
        ):
            with self.subTest(case=case):
                selected = {**record, "units": {unit: {}}}
                current_process = dict(process)
                current_launch = dict(launch)
                current_manager = dict(manager)
                current_config = dict(config)
                readiness = absent
                identity: object = FileNotFoundError()
                if case == "wrong-generation":
                    current_process["instance_id"] = "other"
                elif case == "bound":
                    selected["units"] = {
                        unit: {
                            "group": "/owned-store",
                            "inode": 123,
                            "invocation": "a" * 32,
                        }
                    }
                elif case == "bound-launch":
                    current_launch["binding"] = {"group": "/owned-store"}
                elif case in {"live", "reused"}:
                    identity = "child" if case == "live" else "replacement"
                elif case == "unknown-process":
                    identity = PermissionError("unknown")
                elif case == "unknown-unit":
                    current_manager["LoadState"] = "not-found"
                elif case == "active-unit":
                    current_manager["ActiveState"] = "active"
                elif case == "listener":
                    readiness = surreal_server.ServiceObservation(
                        surreal_server.Readiness.MISMATCH,
                        service,
                        "original",
                        unit,
                        "listener survives",
                        manager=manager,
                        listener_owned=True,
                    )
                elif case == "changed-invocation":
                    readiness = surreal_server.ServiceObservation(
                        surreal_server.Readiness.ACTIONABLE_ABSENT,
                        service,
                        "original",
                        unit,
                        "replacement",
                        manager={**manager, "InvocationID": "c" * 32},
                    )
                elif case == "changed-selection":
                    current_config["instance_id"] = "other"
                elif case == "foreign-ledger":
                    current_process["allocation"] = "/unknown-ledger/" + "b" * 32
                elif case == "relative-affiliation":
                    current_process["allocation"] = "relative/" + "b" * 32
                elif case == "malformed-affiliation":
                    current_process["allocation"] = str(self.directory / "unknown")
                elif case == "alias-current":
                    current_process["allocation"] = (
                        str(self.directory) + "/./" + owner.nonce
                    )
                elif case == "alias-previous":
                    current_process["allocation"] = (
                        str(self.directory) + "/./" + "b" * 32
                    )
                surreal_server.write_json(
                    service / "service-launch.json", current_launch
                )
                surreal_server.write_json(
                    service / "server-process.json", current_process
                )
                with (
                    patch.object(
                        surreal_server, "config_for", return_value=current_config
                    ),
                    patch.object(
                        surreal_server, "service_readiness", return_value=readiness
                    ),
                    patch.object(
                        host.operation,
                        "unit_observation",
                        return_value=current_manager,
                        side_effect=subprocess.TimeoutExpired("controlled-manager", 1)
                        if case == "manager-timeout"
                        else None,
                    ),
                    patch.object(
                        host.operation,
                        "start_identity",
                        side_effect=identity
                        if isinstance(identity, BaseException)
                        else None,
                        return_value=identity,
                    ),
                ):
                    self.assertEqual(
                        host.drained(selected, allocation=self.directory / owner.nonce),
                        case == "drained",
                    )
                self.assertEqual(
                    surreal_server.read_json(service / "server-process.json"),
                    current_process,
                )
                self.assertEqual(
                    surreal_server.read_json(service / "service-launch.json"),
                    current_launch,
                )

    def test_unbound_storage_observation_uses_original_drain_clock(self) -> None:
        owner, service, unit = self.storage_process_fixture()
        (service / "server-process.json").unlink()
        with host.allocation_metadata(self.directory) as ledger:
            record = dict(ledger["owners"][owner.nonce])
        record["units"] = {unit: {}}
        deadline = time.monotonic() + 1
        with patch.object(
            host.operation, "unit_observation", return_value={"LoadState": "not-found"}
        ) as observe:
            self.assertTrue(
                host.service_processes_drained(
                    record, self.directory / owner.nonce, deadline=deadline
                )
            )
        observe.assert_called_once_with(unit, deadline=deadline)

    def test_stale_borrowed_snapshot_cannot_park_resumed_allocation(self) -> None:
        original, service, unit = self.storage_process_fixture(borrowed=True)
        replacement: host.Allocation | None = None

        @contextlib.contextmanager
        def replaced_lifecycle(_state: Path) -> Generator[None, None, None]:
            nonlocal replacement
            # R1 took A's snapshot before acquiring lifecycle ownership. R2
            # has since drained A and admitted the same service under B.
            with host.allocation_metadata(self.directory) as ledger:
                del ledger["owners"][original.nonce]
            replacement = host.acquire(
                host.select("store-functional"),
                directory=self.directory,
                deadline=time.monotonic() + 5,
            )
            with host.allocation_metadata(self.directory) as ledger:
                ledger["owners"][replacement.nonce].update(
                    borrowed_services=[str(service)],
                    units={
                        unit: {
                            "group": "/replacement",
                            "inode": 456,
                            "invocation": "b" * 32,
                        }
                    },
                )
            surreal_server.write_json(
                service / "service-launch.json",
                {
                    "allocation": str(self.directory / replacement.nonce),
                    "generation": "replacement",
                    "binding": {
                        "group": "/replacement",
                        "inode": 456,
                        "invocation": "b" * 32,
                    },
                },
            )
            yield

        with (
            patch.object(
                host,
                "memory_info",
                return_value={"MemAvailable": 1 << 40, "MemTotal": 1 << 40},
            ),
            patch.object(
                surreal_server, "lifecycle_reservation", side_effect=replaced_lifecycle
            ),
            patch.object(host, "group_identity", return_value=456),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "ControlGroup": "/replacement",
                    "InvocationID": "b" * 32,
                },
            ),
            patch.object(surreal_server, "config_for") as config,
            patch.object(surreal_server, "workers_drained") as workers,
            patch.object(surreal_server, "stop") as stop,
            patch.object(surreal_server, "park_service") as park,
            patch.object(host, "queue_parked_service") as queue,
        ):
            host.reconcile(self.directory, resume_parked=False)
        self.assertIsNotNone(replacement)
        assert replacement is not None
        with host.allocation_metadata(self.directory) as ledger:
            self.assertNotIn(original.nonce, ledger["owners"])
            self.assertIn(replacement.nonce, ledger["owners"])
            self.assertEqual(
                ledger["owners"][replacement.nonce]["units"][unit]["inode"], 456
            )
            self.assertEqual(ledger.get("parked_services", []), [])
        for effect in (config, workers, stop, park, queue):
            effect.assert_not_called()

    def test_borrowed_drain_refuses_changed_or_unavailable_incarnation(self) -> None:
        owner, service, unit = self.storage_process_fixture(borrowed=True)
        with host.allocation_metadata(self.directory) as ledger:
            record = cast("host.AllocationRecord", dict(ledger["owners"][owner.nonce]))
        launch = surreal_server.read_json(service / "service-launch.json")
        launch["binding"] = record["units"][unit]
        surreal_server.write_json(service / "service-launch.json", launch)
        active = {
            "LoadState": "loaded",
            "ActiveState": "active",
            "ControlGroup": "/owned-store",
            "InvocationID": "a" * 32,
        }
        for observed, inode, generation in (
            ({**active, "InvocationID": "b" * 32}, 123, "original"),
            (active, 456, "original"),
            (active, 123, "replacement"),
            ({}, 123, "original"),
            (PermissionError("unavailable"), 123, "original"),
        ):
            with (
                self.subTest(observed=observed, inode=inode, generation=generation),
                patch.object(
                    surreal_server,
                    "lifecycle_reservation",
                    side_effect=lambda _: contextlib.nullcontext(),
                ),
                patch.object(host, "group_identity", return_value=inode),
                patch.object(
                    host.operation, "unit_observation", side_effect=[observed]
                ),
                patch.object(
                    surreal_server,
                    "config_for",
                    return_value={"instance_id": generation, "resident": True},
                ),
                patch.object(surreal_server, "workers_drained") as workers,
                patch.object(surreal_server, "stop") as stop,
                patch.object(surreal_server, "park_service") as park,
                patch.object(host, "queue_parked_service") as queue,
                self.assertRaises((host.AdmissionError, OSError)),
            ):
                host.drain_borrowed_services(
                    self.directory,
                    record,
                    allocation=self.directory / owner.nonce,
                    explicit=True,
                )
            for effect in (workers, stop, park, queue):
                effect.assert_not_called()

    def test_acquisition_defers_unrelated_queued_resume_under_original_clock(
        self,
    ) -> None:
        for name in ("light", "functional", "reference", "store-functional"):
            with self.subTest(profile=name):
                directory = self.directory / name
                parked = str(directory / "parked-service")
                with host.allocation_metadata(directory) as ledger:
                    ledger["parked_services"] = [parked]
                deadline = time.monotonic() + 1
                with (
                    patch.object(
                        host,
                        "memory_info",
                        return_value={"MemAvailable": 1 << 40, "MemTotal": 1 << 40},
                    ),
                    patch.object(
                        surreal_server,
                        "unpark_service",
                        side_effect=AssertionError(
                            "unrelated startup consumed admission"
                        ),
                    ) as resume,
                ):
                    owner = host.acquire(
                        host.select(name), directory=directory, deadline=deadline
                    )
                self.assertEqual(owner.deadline, deadline)
                resume.assert_not_called()
                with host.allocation_metadata(directory) as ledger:
                    self.assertIn(owner.nonce, ledger["owners"])
                    self.assertEqual(ledger["parked_services"], [parked])

    def test_acquisition_still_reconciles_only_drained_owner_lifetimes(self) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        unit = "pse-reconcile-fixture.scope"
        owner.register(unit)
        with host.allocation_metadata(self.directory) as ledger:
            ledger["owners"][owner.nonce].update(pid=-1, start="ended")
            ledger["owners"][owner.nonce]["units"][unit] = {
                "group": "/owned",
                "invocation": "a" * 32,
            }
            ledger["parked_services"] = ["/unrelated-parked-service"]
        with (
            patch.object(
                host.operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ControlGroup": "/owned",
                    "InvocationID": "a" * 32,
                },
            ),
            patch.object(host.operation, "populated", return_value=True),
            patch.object(
                surreal_server,
                "unpark_service",
                side_effect=AssertionError("unrelated resume"),
            ),
        ):
            host.acquire(
                host.select("light"),
                directory=self.directory,
                deadline=time.monotonic() + 1,
            )
        with host.allocation_metadata(self.directory) as ledger:
            self.assertIn(owner.nonce, ledger["owners"])
        with (
            patch.object(
                host.operation,
                "unit_observation",
                return_value={"LoadState": "not-found"},
            ),
            patch.object(host.operation, "populated", return_value=False),
            patch.object(
                surreal_server,
                "unpark_service",
                side_effect=AssertionError("unrelated resume"),
            ),
        ):
            host.acquire(
                host.select("light"),
                directory=self.directory,
                deadline=time.monotonic() + 1,
            )
        with host.allocation_metadata(self.directory) as ledger:
            self.assertNotIn(owner.nonce, ledger["owners"])
            self.assertEqual(ledger["parked_services"], ["/unrelated-parked-service"])

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

        def inherit(_environment: object, *, deadline: float) -> host.Allocation | None:
            nonlocal owner
            self.assertGreater(deadline, time.monotonic())
            if not admitted:
                return None
            # Admit the later matching owner after reconcile takes its queue snapshot.
            # Registering it beforehand would correctly defer reconciliation entirely.
            owner = host.acquire(
                host.select("reference"), directory=self.directory, deadline=deadline
            )
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
                "service_readiness",
                side_effect=lambda *_args, **_kwargs: readiness_fixture(
                    surreal_server.Readiness.STARTING
                    if any(call.args[0] == "start" for call in manager.call_args_list)
                    else surreal_server.Readiness.ACTIONABLE_ABSENT
                ),
            ),
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
        surreal_server.write_json(
            service / "service-launch.json",
            {
                "allocation": str(self.directory / owner.nonce),
                "generation": "stopped-fixture",
            },
        )
        surreal_server.write_json(
            service / "server-process.json",
            {
                "allocation": str(self.directory / owner.nonce),
                "instance_id": "stopped-fixture",
                "pid": os.getpid(),
                "start": str(int(host.operation.start_identity(os.getpid())) - 1),
            },
        )
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
        def lifecycle(_state: Path, **_kwargs: object) -> Generator[None, None, None]:
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
                owner.bind(unit, deadline=time.monotonic() + 1, fresh=True)
        self.assertEqual((ledger.read_bytes(), ledger.stat().st_ino), before)

    def test_authenticated_rebind_has_bounded_clock_separate_from_fixed_freshness(
        self,
    ) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        unit = "owned-storage.service"
        owner.register(unit)
        original = {"group": "/owned", "invocation": "a" * 32, "inode": 17}
        with host.allocation_metadata(self.directory) as ledger:
            ledger["owners"][owner.nonce]["units"][unit] = original
        deadline = time.monotonic() + 1
        with (
            patch.object(
                host,
                "control_unit_observation",
                return_value={"ControlGroup": "/owned", "InvocationID": "b" * 32},
            ) as observe,
            patch.object(host.operation, "process_group", return_value="/owned/child"),
            patch.object(host, "group_identity", return_value=18),
        ):
            with self.assertRaisesRegex(host.AdmissionError, "already bound"):
                owner.bind(unit, deadline=deadline, fresh=True)
            owner.bind(unit, deadline=deadline)
        self.assertEqual(observe.call_args.args, (unit, deadline))
        with host.allocation_metadata(self.directory) as ledger:
            self.assertEqual(
                ledger["owners"][owner.nonce]["units"][unit],
                {"group": "/owned", "invocation": "b" * 32, "inode": 18},
            )
        before = (self.directory / "allocations.json").read_bytes()
        with (
            patch.object(
                host,
                "control_unit_observation",
                return_value={"ControlGroup": "/owned", "InvocationID": "c" * 32},
            ),
            patch.object(host.operation, "process_group", return_value="/outside"),
            self.assertRaisesRegex(host.AdmissionError, "actual unit"),
        ):
            owner.bind(unit, deadline=time.monotonic() + 1)
        self.assertEqual((self.directory / "allocations.json").read_bytes(), before)

    def test_authenticated_rebind_held_ledger_expires_without_late_publication(
        self,
    ) -> None:
        owner = host.acquire_light_control(
            directory=self.directory, deadline=time.monotonic() + 1
        )
        unit = "owned-storage.service"
        owner.register(unit)
        with host.allocation_metadata(self.directory) as ledger:
            ledger["owners"][owner.nonce]["units"][unit] = {
                "group": "/owned",
                "invocation": "a" * 32,
                "inode": 17,
            }
        before = (self.directory / "allocations.json").read_bytes()
        program = "import fcntl,sys; f=open(sys.argv[1], 'a'); fcntl.flock(f, fcntl.LOCK_EX); print('held',flush=True); sys.stdin.readline()"
        child = subprocess.Popen(
            [sys.executable, "-c", program, str(self.directory / ".lock")],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            text=True,
        )
        try:
            assert child.stdout is not None
            self.assertEqual(child.stdout.readline().strip(), "held")
            started = time.monotonic()
            with (
                patch.object(
                    host,
                    "control_unit_observation",
                    return_value={"ControlGroup": "/owned", "InvocationID": "b" * 32},
                ),
                patch.object(
                    host.operation, "process_group", return_value="/owned/child"
                ),
                patch.object(host, "group_identity", return_value=18),
                self.assertRaises(host.AdmissionError),
            ):
                owner.bind(unit, deadline=started + 0.05)
            self.assertLess(time.monotonic() - started, 0.5)
        finally:
            child.communicate("release\n", timeout=5)
            self.assertEqual(child.returncode, 0)
        self.assertEqual((self.directory / "allocations.json").read_bytes(), before)

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

    def test_release_refreshes_returned_affiliation_and_preserves_other_work(
        self,
    ) -> None:
        for surviving in (False, True):
            with self.subTest(surviving=surviving):
                owner = host.acquire(
                    host.select("functional"), directory=self.directory
                )
                with host.allocation_metadata(self.directory) as ledger:
                    ledger["owners"][owner.nonce]["borrowed_services"] = [
                        "/owned/store"
                    ]

                def returned(
                    *_args: object,
                    owner: host.Allocation = owner,
                    surviving: bool = surviving,
                    **_kwargs: object,
                ) -> None:
                    with host.allocation_metadata(self.directory) as ledger:
                        record = ledger["owners"][owner.nonce]
                        record["borrowed_services"] = []
                        if surviving:
                            record["units"]["other.scope"] = {
                                "group": "/other",
                                "invocation": "a" * 32,
                                "inode": 17,
                            }

                def drained(record: dict[str, object], **_kwargs: object) -> bool:
                    self.assertEqual(record["borrowed_services"], [])
                    return not record["units"]

                with (
                    patch.object(host, "drain_borrowed_services", side_effect=returned),
                    patch.object(host, "drained", side_effect=drained),
                    patch.object(host, "retire_empty_allocation"),
                ):
                    self.assertEqual(owner.release(), not surviving)
                with host.allocation_metadata(self.directory) as ledger:
                    if surviving:
                        record = ledger["owners"][owner.nonce]
                        self.assertIn("other.scope", record["units"])
                        self.assertTrue(record["released"])
                    else:
                        self.assertNotIn(owner.nonce, ledger["owners"])
                if surviving:
                    with (
                        patch.object(host, "drained", return_value=True),
                        patch.object(host, "retire_empty_allocation"),
                    ):
                        self.assertTrue(owner.release())

    def test_release_reports_retention_when_other_work_registers_during_proof(
        self,
    ) -> None:
        owner = host.acquire(host.select("functional"), directory=self.directory)

        def register(_record: dict[str, object], **_kwargs: object) -> bool:
            owner.register("late.scope")
            return True

        with (
            patch.object(host, "drained", side_effect=register),
            patch.object(host, "retire_empty_allocation") as retire,
        ):
            self.assertFalse(owner.release())
        retire.assert_not_called()
        with host.allocation_metadata(self.directory) as ledger:
            self.assertIn("late.scope", ledger["owners"][owner.nonce]["units"])
        with (
            patch.object(host, "drained", return_value=True),
            patch.object(host, "retire_empty_allocation"),
        ):
            self.assertTrue(owner.release())

    def test_release_preserves_original_deadline_and_expiry_keeps_charge(self) -> None:
        owner = host.acquire(host.select("light"), directory=self.directory)
        expired = time.monotonic() - 1
        with (
            patch.object(host, "drain_borrowed_services") as borrowed,
            self.assertRaises(host.AdmissionError),
        ):
            owner.release(deadline=expired)
        borrowed.assert_not_called()
        with host.allocation_metadata(self.directory) as ledger:
            self.assertIn(owner.nonce, ledger["owners"])
        deadline = time.monotonic() + 5
        with (
            patch.object(host, "drain_borrowed_services") as borrowed,
            patch.object(host, "drained", return_value=True) as proof,
            patch.object(host, "retire_empty_allocation") as retire,
        ):
            self.assertTrue(owner.release(deadline=deadline))
        self.assertEqual(borrowed.call_args.kwargs["deadline"], deadline)
        self.assertEqual(proof.call_args.kwargs["deadline"], deadline)
        retire.assert_called_once_with(owner, deadline=deadline)

    def test_missing_terminal_ack_pins_empty_allocation_independently_of_scope_drain(
        self,
    ) -> None:
        owner = host.acquire(host.select("light"), directory=self.directory)
        with host.allocation_metadata(self.directory) as ledger:
            ledger["owners"][owner.nonce]["terminal_guards"] = {
                "a" * 32: {
                    "operation": "/missing/.operations/" + "b" * 32 + ".json",
                    "token": "c" * 32,
                    "scope": {},
                }
            }
        with patch.object(host, "retire_empty_allocation") as retire:
            self.assertFalse(owner.release(deadline=time.monotonic() + 5))
        retire.assert_not_called()
        with host.allocation_metadata(self.directory) as ledger:
            self.assertIn(owner.nonce, ledger["owners"])

    def test_verified_ack_is_durable_in_existing_guard_before_evidence_retirement(
        self,
    ) -> None:
        from scripts import native_operation as operation
        from scripts import resource_store
        from scripts import test_resources as resources

        owner = host.acquire(host.select("light"), directory=self.directory)
        scope = {
            "unit": "pse-native-" + "d" * 32 + ".scope",
            "group": operation.process_group(os.getpid()),
            "invocation": "e" * 32,
        }
        with host.allocation_metadata(self.directory) as ledger:
            ledger["owners"][owner.nonce]["units"][scope["unit"]] = {
                "group": scope["group"],
                "invocation": scope["invocation"],
                "inode": 17,
            }
        base = self.directory / "fixture"
        with (
            patch.dict(os.environ, {"PSE_TEST_RESOURCE_STATE": str(base / "registry")}),
            patch.object(operation, "scope_owner", return_value=scope),
            patch.object(host, "inherit", return_value=owner),
            patch.object(
                resource_store, "publish", wraps=resource_store.publish
            ) as publish,
            operation.Operation(base) as parent,
        ):
            path = resources.invocation("python", [])
            registration = operation.register_composer(path)
            with host.allocation_metadata(self.directory) as ledger:
                guard = dict(
                    ledger["owners"][owner.nonce]["terminal_guards"][path.stem]
                )
                snapshot = dict(ledger["owners"][owner.nonce])
            self.assertFalse(operation.terminal_guard_complete(path.stem, guard))
            resources.finalize(path, {}, complete=False, terminal_owner="runner")
            resource_store.publish(
                path.with_suffix(".terminal.json"), {"exit_code": None}
            )
            operation.acknowledge_composer(registration)
            with host.allocation_metadata(self.directory) as ledger:
                guard = dict(
                    ledger["owners"][owner.nonce]["terminal_guards"][path.stem]
                )
            self.assertEqual(guard["phase"], "acknowledged")
            self.assertGreaterEqual(
                sum(
                    call.args[0] == self.directory / "allocations.json"
                    for call in publish.call_args_list
                ),
                2,
            )
        path.with_suffix(".terminal.json").unlink()
        (path.parent.parent / "invocation-state" / path.name).unlink()
        parent.path.unlink()
        self.assertTrue(operation.terminal_guard_complete(path.stem, guard))
        snapshot.update(released=True, terminal_guards={path.stem: guard})
        with (
            patch.object(
                operation, "unit_observation", return_value={"LoadState": "not-found"}
            ),
            patch.object(host, "group_identity", return_value=None),
        ):
            self.assertTrue(host.drained(snapshot))
            for changes in (
                {"scope": {}},
                {"binding": {**guard["binding"], "invocation": "f" * 32}},
            ):
                corrupted = {
                    **snapshot,
                    "terminal_guards": {path.stem: {**guard, **changes}},
                }
                self.assertFalse(host.drained(corrupted))

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
        owner, state, unit = self.storage_process_fixture(borrowed=True)
        service = str(state)
        with host.allocation_metadata(self.directory) as ledger:
            record = cast("host.AllocationRecord", dict(ledger["owners"][owner.nonce]))
        launch = surreal_server.read_json(state / "service-launch.json")
        launch["binding"] = record["units"][unit]
        surreal_server.write_json(state / "service-launch.json", launch)
        with (
            patch.object(host, "drained", return_value=True),
            patch.object(
                surreal_server,
                "lifecycle_reservation",
                side_effect=lambda _: contextlib.nullcontext(),
            ),
            patch.object(
                surreal_server,
                "config_for",
                return_value={"resident": True, "instance_id": "original"},
            ),
            patch.object(host, "group_identity", return_value=123),
            patch.object(
                host.operation,
                "unit_observation",
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "inactive",
                    "ControlGroup": "",
                    "InvocationID": "a" * 32,
                },
            ),
            patch.object(surreal_server, "workers_drained"),
            patch.object(surreal_server, "stop") as stop,
            patch.object(surreal_server, "park_service") as park,
        ):
            host.drain_borrowed_services(
                self.directory,
                record,
                allocation=self.directory / owner.nonce,
                explicit=True,
            )
        stop.assert_called_once()
        park.assert_not_called()
        with host.allocation_metadata(self.directory) as ledger:
            self.assertNotIn(service, ledger.get("parked_services", []))

    def test_borrowed_partial_park_preserves_actual_resume_intent(self) -> None:
        owner, service, unit = self.storage_process_fixture(borrowed=True)
        with host.allocation_metadata(self.directory) as ledger:
            record = cast("host.AllocationRecord", dict(ledger["owners"][owner.nonce]))
        launch = surreal_server.read_json(service / "service-launch.json")
        launch["binding"] = record["units"][unit]
        surreal_server.write_json(service / "service-launch.json", launch)
        config: dict[str, object] = {
            "resident": True,
            "parked": False,
            "instance_id": "original",
        }

        def park(_state: Path, *, expected_binding: host.BoundUnit) -> None:
            self.assertEqual(expected_binding, record["units"][unit])
            config["parked"] = True
            raise surreal_server.SupervisorError("partial owned stop")

        with (
            patch.object(host, "drained", return_value=True),
            patch.object(host, "group_identity", return_value=123),
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
                return_value={
                    "LoadState": "loaded",
                    "ActiveState": "active",
                    "ControlGroup": "/owned-store",
                    "InvocationID": "a" * 32,
                },
            ),
            patch.object(surreal_server, "park_service", side_effect=park),
            self.assertRaisesRegex(
                surreal_server.SupervisorError, "partial owned stop"
            ),
        ):
            host.drain_borrowed_services(
                self.directory,
                record,
                allocation=self.directory / owner.nonce,
                explicit=True,
            )
        with host.allocation_metadata(self.directory) as ledger:
            self.assertIn(str(service), ledger.get("parked_services", []))


if __name__ == "__main__":
    unittest.main()

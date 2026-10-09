# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Shared capacity, explicit widening and conservative surviving-owner controls."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import contextlib
import tempfile
import time
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import host_admission as host
from scripts import surreal_server


class HostAdmissionTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.directory = Path(temporary.name) / "admission"

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


if __name__ == "__main__":
    unittest.main()

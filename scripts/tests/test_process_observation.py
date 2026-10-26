# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Shared observations retain identity races and unavailable owner authority."""

from __future__ import annotations

import os
import unittest
from unittest.mock import patch

from scripts import process_observation as observation
from scripts import service_borrowers


class ProcessObservationTests(unittest.TestCase):
    def test_current_process_binds_kernel_facts_and_supplied_manager_identity(self) -> None:
        pid = os.getpid()
        group = observation.process_group(pid)
        unit = group.rsplit("/", 1)[-1]
        manager = {"LoadState": "loaded", "ControlGroup": group, "InvocationID": "a" * 32}
        result = observation.observe_process(pid, unit=unit, manager=manager)
        self.assertEqual(result.availability, observation.Availability.PRESENT)
        self.assertEqual(result.start, observation.start_identity(pid))
        self.assertEqual(result.namespace, observation.pid_namespace(pid))
        self.assertEqual(result.executable, observation.executable(pid))
        self.assertEqual(result.invocation, manager["InvocationID"])
        self.assertEqual(result.uncertainty, {})

    def test_process_reuse_never_combines_old_and_new_owner_facts(self) -> None:
        original = ["S", *(["0"] * 18), "original"]
        replacement = ["S", *(["0"] * 18), "replacement"]
        with patch.object(observation, "process_stat", side_effect=[original, replacement]):
            result = observation.observe_process(os.getpid())
        self.assertEqual(result.availability, observation.Availability.CHANGED)
        self.assertIsNone(result.start)
        self.assertIsNone(result.executable)

    def test_unavailable_boot_is_not_process_absence(self) -> None:
        with patch.object(observation, "boot", side_effect=FileNotFoundError):
            result = observation.observe_process(os.getpid())
        self.assertEqual(result.availability, observation.Availability.UNAVAILABLE)
        with patch.object(observation, "process_stat", side_effect=FileNotFoundError):
            result = observation.observe_process(os.getpid())
        self.assertEqual(result.availability, observation.Availability.ABSENT)

    def test_optional_executable_failure_and_manager_mismatch_remain_explicit(self) -> None:
        manager = {"LoadState": "loaded", "ControlGroup": "/foreign.scope", "InvocationID": "a" * 32}
        with patch.object(observation, "executable", side_effect=PermissionError):
            result = observation.observe_process(os.getpid(), unit="foreign.scope", manager=manager)
        self.assertEqual(result.availability, observation.Availability.PRESENT)
        self.assertIsNone(result.executable)
        self.assertIsNone(result.invocation)
        self.assertEqual(set(result.uncertainty), {"executable", "invocation"})

    def test_unreadable_group_is_not_reported_empty(self) -> None:
        with patch.object(observation.Path, "stat", side_effect=PermissionError):
            with self.assertRaises(PermissionError):
                observation.populated("/unreadable.scope")

    def test_unavailable_borrower_identity_retains_the_obligation(self) -> None:
        record = {"pid": os.getpid(), "start": observation.start_identity(os.getpid()), "boot": observation.boot()}
        with patch.object(observation, "start_identity", side_effect=PermissionError):
            self.assertTrue(service_borrowers.borrower_alive(record))
        with patch.object(observation, "start_identity", side_effect=FileNotFoundError):
            self.assertFalse(service_borrowers.borrower_alive(record))


if __name__ == "__main__":
    unittest.main()

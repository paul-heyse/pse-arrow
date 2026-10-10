# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Live native ownership scales with unsettled operations, not completed history."""
# ruff: noqa: PT009 -- stdlib tooling controls

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import native_operation as operation


class NativePinsTests(unittest.TestCase):
    def test_completed_owners_disappear_and_live_unknown_pins_remain(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            records = base / ".operations"
            records.mkdir()
            for index in range(100):
                operation.write_json(
                    records / f"settled-{index}.json",
                    {
                        "version": operation.VERSION,
                        "scope": {"settled": True},
                        "admissions": {},
                        "generations": [f"/old/{index}"],
                    },
                )
            operation.write_json(
                records / "active.json",
                {
                    "version": operation.VERSION,
                    "scope": {"settled": False},
                    "admissions": {},
                    "generations": ["/live/one", "/live/two"],
                },
            )
            # Unscoped/unknown authentic drain remains protective.
            operation.write_json(
                records / "unknown.json",
                {
                    "version": operation.VERSION,
                    "scope": None,
                    "admissions": {},
                    "generations": ["/unknown"],
                },
            )
            with (
                patch.object(
                    operation,
                    "drained",
                    side_effect=lambda record: bool(
                        record.get("scope", {}) and record["scope"]["settled"]
                    ),
                ),
                patch.object(operation, "_record", wraps=operation._record) as read,  # noqa: SLF001 -- count cold ledger reads
            ):
                expected = {"/live/one", "/live/two", "/unknown"}
                self.assertEqual(operation.pinned_generations(base), expected)
                self.assertEqual(read.call_count, 102)
                read.reset_mock()
                self.assertEqual(operation.pinned_generations(base), expected)
                self.assertEqual(read.call_count, 2)
            self.assertEqual(len(list(records.glob("*.json"))), 2)
            self.assertFalse(list(records.glob("settled-*.lock")))

    def test_malformed_record_refuses_reclamation_without_deletion(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            base = Path(directory)
            records = base / ".operations"
            records.mkdir()
            bad = records / "unknown.json"
            bad.write_text('{"version": -1}')
            self.assertIsNone(operation.pinned_generations(base))
            self.assertTrue(bad.exists())

    def test_restart_gap_and_changed_generation_do_not_prove_drain(self) -> None:
        owner = {"unit": "fixture.service", "group": "/fixture", "invocation": "exact"}
        observed = {
            "LoadState": "loaded",
            "ControlGroup": "/fixture",
            "InvocationID": "exact",
            "ActiveState": "activating",
        }
        with (
            patch.object(operation, "populated", return_value=False),
            patch.object(operation, "unit_observation", return_value=observed),
        ):
            self.assertFalse(operation.drained({"scope": owner}))
            observed["ActiveState"] = "inactive"
            self.assertTrue(operation.drained({"scope": owner}))
            observed["InvocationID"] = "replacement"
            self.assertFalse(operation.drained({"scope": owner}))
        with (
            patch.object(operation, "populated", side_effect=[False, True]),
            patch.object(
                operation,
                "unit_observation",
                return_value={**observed, "InvocationID": "exact"},
            ),
        ):
            self.assertFalse(operation.drained({"scope": owner}))


if __name__ == "__main__":
    unittest.main()

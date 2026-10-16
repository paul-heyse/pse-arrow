# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact admitted-primary observation; no services are started or stopped."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import host_admission, native_operation
from scripts import surreal_server as server


class PrimaryObservationTests(unittest.TestCase):
    def setUp(self) -> None:
        self.temporary = tempfile.TemporaryDirectory()
        self.addCleanup(self.temporary.cleanup)
        self.state = Path(self.temporary.name)
        self.scope = {
            "unit": server.primary_unit(self.state),
            "group": "/owned-primary",
            "invocation": "a" * 32,
        }
        self.owner: dict[str, object] = {"scope": self.scope, "scope_inode": 41}
        self.observed = {
            "LoadState": "loaded",
            "ActiveState": "failed",
            "InvocationID": self.scope["invocation"],
            "ControlGroup": self.scope["group"],
            "MemoryMax": "1000",
        }

    def observation(
        self,
        *,
        populated: bool = False,
        inode: int = 41,
        manager: dict[str, str] | None = None,
    ) -> dict[str, object]:
        with (
            patch.object(server, "receiver_context", return_value=self.state),
            patch.object(server, "config_for", return_value={"database": "case"}),
            patch.object(host_admission, "group_identity", return_value=inode),
            patch.object(native_operation, "populated", return_value=populated),
            patch.object(
                native_operation,
                "unit_observation",
                return_value=manager or self.observed,
            ),
        ):
            return server.observe_primary(self.state, "case", self.owner)

    def test_exact_terminal_owner_and_empty_group_establish_drain(self) -> None:
        result = self.observation()
        self.assertEqual(result["phase"], "complete")
        self.assertEqual(result["primary_owner"], self.owner)
        self.assertEqual(result["canonical_database"], "case")

    def test_owned_descendants_and_active_unit_do_not_establish_drain(self) -> None:
        self.assertEqual(self.observation(populated=True)["phase"], "populated")
        self.assertEqual(
            self.observation(manager={**self.observed, "ActiveState": "active"})[
                "phase"
            ],
            "active",
        )

    def test_mismatched_invocation_is_unknown_even_with_empty_group(self) -> None:
        result = self.observation(manager={**self.observed, "InvocationID": "b" * 32})
        self.assertEqual(result["phase"], "mismatch")

    def test_manager_failure_cannot_authorize_release(self) -> None:
        with (
            patch.object(
                native_operation,
                "unit_observation",
                side_effect=ValueError("unavailable"),
            ),
            patch.object(server, "receiver_context", return_value=self.state),
            patch.object(server, "config_for", return_value={"database": "case"}),
            patch.object(host_admission, "group_identity", return_value=41),
            patch.object(native_operation, "populated", return_value=False),
        ):
            self.assertEqual(
                server.observe_primary(self.state, "case", self.owner)["phase"],
                "unavailable",
            )

    def test_replacement_group_does_not_become_the_original_owner(self) -> None:
        # The kernel identity ended. A new populated group at that path is a
        # distinct receiver, and must not keep the old dispatch joined forever.
        self.assertEqual(
            self.observation(populated=True, inode=42)["phase"], "complete"
        )

    def test_other_database_and_missing_invocation_refuse_before_observation(
        self,
    ) -> None:
        with (
            patch.object(server, "receiver_context", return_value=self.state),
            patch.object(server, "config_for", return_value={"database": "case"}),
            patch.object(native_operation, "drained") as drained,
        ):
            with self.assertRaises(server.SupervisorError):
                server.observe_primary(self.state, "other", self.owner)
            owner = {**self.owner, "scope": {**self.scope, "invocation": ""}}
            with self.assertRaises(server.SupervisorError):
                server.observe_primary(self.state, "case", owner)
            drained.assert_not_called()

    def test_dispatch_owner_pins_the_ready_invocation_and_inode(self) -> None:
        observed = {**self.observed, "ActiveState": "active"}
        with (
            patch.object(native_operation, "unit_observation", return_value=observed),
            patch.object(host_admission, "group_identity", return_value=41),
            patch.object(server, "primary_ready", return_value=True),
        ):
            self.assertEqual(
                server.primary_dispatch_owner(self.state, {}, "case", None), self.owner
            )

    def test_dispatch_owner_rejects_invocation_or_inode_change(self) -> None:
        observed = {**self.observed, "ActiveState": "active"}
        for replacement, inodes in (
            ({**observed, "InvocationID": "b" * 32}, [41]),
            (observed, [41, 42]),
        ):
            with (
                self.subTest(replacement=replacement, inodes=inodes),
                patch.object(
                    native_operation,
                    "unit_observation",
                    side_effect=[observed, replacement],
                ),
                patch.object(host_admission, "group_identity", side_effect=inodes),
                patch.object(server, "primary_ready", return_value=True),
                self.assertRaises(server.SupervisorError),
            ):
                server.primary_dispatch_owner(self.state, {}, "case", None)


if __name__ == "__main__":
    unittest.main()

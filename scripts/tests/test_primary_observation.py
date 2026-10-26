# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exact admitted-primary observation; no services are started or stopped."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import host_admission, operation_lifetime
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
        server.write_json(self.state / "primary-launch.json", {"nonce": "registered"})
        self.observed = {
            "LoadState": "loaded",
            "ActiveState": "failed",
            "InvocationID": self.scope["invocation"],
            "ControlGroup": self.scope["group"],
            "MemoryMax": "1000",
        }

    def diagnostics(self, marker: dict, database: str = "case") -> dict[str, object]:
        with patch.object(operation_lifetime, "start_identity", return_value="9"):
            return server.primary_diagnostics(self.state, database, marker)

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
            patch.object(operation_lifetime, "populated", return_value=populated),
            patch.object(
                operation_lifetime,
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

    def test_optional_diagnostic_handoff_never_grants_cleanup_or_requires_publication(
        self,
    ) -> None:
        marker: dict = {
            "ready": True,
            "canonical_database": "case",
            "pid": 42,
            "nonce": "registered",
            "diagnostics": {
                "identity": {
                    "process": "instance",
                    "locator": {"pid": 42, "start_ticks": 9},
                },
                "checkpoint": str(self.state / "pending-first-checkpoint.json"),
                "enabled": True,
                "writer_failure": "optional writer failed",
            },
        }
        server.write_json(self.state / "primary-receiver.json", marker)
        result = self.diagnostics(marker)
        self.assertTrue(result["available"])
        self.assertEqual(result["identity"], marker["diagnostics"]["identity"])
        self.assertEqual(result["writer_failure"], "optional writer failed")
        self.assertEqual(result["receiver_state"], str(self.state))
        self.assertNotIn("primary_owner", result)
        self.assertNotIn("scope", result)
        self.assertNotIn("scope_inode", result)
        self.assertFalse(Path(marker["diagnostics"]["checkpoint"]).exists())
        marker["diagnostics"]["enabled"] = False
        server.write_json(self.state / "primary-receiver.json", marker)
        self.assertFalse(self.diagnostics(marker)["available"])
        self.assertTrue(server.read_json(self.state / "primary-receiver.json")["ready"])

    def test_optional_handoff_rejects_wrong_database_and_start_identity(self) -> None:
        marker: dict = {
            "ready": True,
            "canonical_database": "case",
            "pid": 42,
            "nonce": "registered",
            "diagnostics": {
                "identity": {
                    "process": "instance",
                    "locator": {"pid": 42, "start_ticks": 9},
                },
                "checkpoint": "prefix.json",
                "enabled": True,
            },
        }
        server.write_json(self.state / "primary-receiver.json", marker)
        self.assertFalse(self.diagnostics(marker, "different")["available"])
        marker["diagnostics"]["identity"]["locator"]["start_ticks"] = None
        server.write_json(self.state / "primary-receiver.json", marker)
        self.assertFalse(self.diagnostics(marker)["available"])
        marker.pop("diagnostics")
        server.write_json(self.state / "primary-receiver.json", marker)
        self.assertFalse(self.diagnostics(marker)["available"])

    def test_owned_descendants_and_active_unit_do_not_establish_drain(self) -> None:
        self.assertEqual(self.observation(populated=True)["phase"], "populated")
        self.assertEqual(
            self.observation(manager={**self.observed, "ActiveState": "active"})[
                "phase"
            ],
            "active",
        )

    def handoff_marker(self) -> dict:
        return {
            "ready": True,
            "canonical_database": "case",
            "pid": 42,
            "nonce": "registered",
            "diagnostics": {
                "identity": {
                    "process": "instance",
                    "locator": {"pid": 42, "start_ticks": 9},
                },
                "checkpoint": "prefix.json",
                "enabled": True,
            },
        }

    def test_dispatch_handoff_captures_marker_before_owner_admission(self) -> None:
        marker = self.handoff_marker()
        server.write_json(self.state / "primary-receiver.json", marker)

        def admit(*_args: object) -> dict[str, object]:
            replacement = {**marker, "nonce": "replacement"}
            server.write_json(self.state / "primary-receiver.json", replacement)
            return self.owner

        with (
            patch.object(server, "primary_dispatch_owner", side_effect=admit) as owner,
            patch.object(server, "primary_ready") as ready,
        ):
            result = server.primary_dispatch_handoff(self.state, {}, "case", None)
        self.assertEqual(result["primary_owner"], self.owner)
        diagnostics = result["diagnostics"]
        assert isinstance(diagnostics, dict)
        self.assertFalse(diagnostics["available"])
        owner.assert_called_once_with(self.state, {}, "case", None)
        ready.assert_not_called()

    def test_missing_optional_marker_preserves_dispatch_owner(self) -> None:
        with patch.object(server, "primary_dispatch_owner", return_value=self.owner):
            result = server.primary_dispatch_handoff(self.state, {}, "case", None)
        self.assertEqual(result["primary_owner"], self.owner)
        diagnostics = result["diagnostics"]
        assert isinstance(diagnostics, dict)
        self.assertFalse(diagnostics["available"])

    def test_optional_copy_requires_launch_nonce_and_actual_start_ticks(self) -> None:
        marker = self.handoff_marker()
        server.write_json(self.state / "primary-receiver.json", marker)
        with patch.object(operation_lifetime, "start_identity", return_value="10"):
            self.assertFalse(
                server.primary_diagnostics(self.state, "case", marker)["available"]
            )
        server.write_json(self.state / "primary-launch.json", {"nonce": "replacement"})
        self.assertFalse(self.diagnostics(marker)["available"])
        (self.state / "primary-launch.json").unlink()
        self.assertFalse(self.diagnostics(marker)["available"])

    def test_optional_copy_rejects_any_marker_drift(self) -> None:
        marker = self.handoff_marker()
        for field, value in (("pid", 43), ("nonce", "replacement"), ("ready", False)):
            with self.subTest(field=field):
                server.write_json(
                    self.state / "primary-receiver.json", {**marker, field: value}
                )
                self.assertFalse(self.diagnostics(marker)["available"])
        server.write_json(self.state / "primary-receiver.json", marker)
        with patch.object(
            operation_lifetime,
            "start_identity",
            side_effect=FileNotFoundError("exited"),
        ):
            self.assertFalse(
                server.primary_diagnostics(self.state, "case", marker)["available"]
            )

    def test_mismatched_invocation_is_unknown_even_with_empty_group(self) -> None:
        result = self.observation(manager={**self.observed, "InvocationID": "b" * 32})
        self.assertEqual(result["phase"], "mismatch")

    def test_manager_failure_cannot_authorize_release(self) -> None:
        with (
            patch.object(
                operation_lifetime,
                "unit_observation",
                side_effect=ValueError("unavailable"),
            ),
            patch.object(server, "receiver_context", return_value=self.state),
            patch.object(server, "config_for", return_value={"database": "case"}),
            patch.object(host_admission, "group_identity", return_value=41),
            patch.object(operation_lifetime, "populated", return_value=False),
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
            patch.object(operation_lifetime, "drained") as drained,
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
            patch.object(operation_lifetime, "unit_observation", return_value=observed),
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
                    operation_lifetime,
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

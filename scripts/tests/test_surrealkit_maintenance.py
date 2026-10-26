# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Closed maintenance authority and unknown SQL retention controls."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import contextlib
import hashlib
import os
import subprocess
import sys
import tempfile
import threading
import time
import tomllib
import unittest
from pathlib import Path
from typing import TYPE_CHECKING, cast
from unittest.mock import patch

from scripts import operation_lifetime, surrealkit_source
from scripts import surreal_server as server
from scripts import surrealkit_maintenance as tool
from scripts.tests import surrealkit_rollout_fixture_check as probe

if TYPE_CHECKING:
    from collections.abc import Generator, Mapping
    from typing import BinaryIO


VERIFY_SAVED = tool.surrealkit.verify_saved


class MaintenanceTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.state = self.root / "state"
        self.state.mkdir()
        self.folder = self.root / "frozen"
        (self.folder / "rollouts/one/schema").mkdir(parents=True)
        data = b"DEFINE TABLE item SCHEMAFULL;"
        (self.folder / "rollouts/one/schema/item.surql").write_bytes(data)
        (self.folder / "rollouts/one.toml").write_text(
            'id = "one"\n[[steps]]\nid = "definition"\nphase = "start"\nkind = "apply_files"\n'
            '[[steps.files]]\npath = "schema/item.surql"\nhash = "'
            + hashlib.sha256(data).hexdigest()
            + '"\n'
        )
        self.config: dict[str, object] = {
            "instance_id": "selected-generation",
            "namespace": "canonical",
            "database": "target",
            "accepting_writes": False,
            "admission": "quiesced",
        }
        server.write_json(self.state / "config.json", self.config)
        self.owner = {
            "nonce": "owner",
            "pid": os.getpid(),
            "start": "start",
            "boot": "boot",
            "purpose": "maintenance",
        }
        server.write_json(self.state / "lifecycle-owner.json", self.owner)
        server.write_json(
            self.state / "credentials.json",
            {"username": "root-owner", "password": "secret"},
        )
        self.calls: list[str] = []
        self.native_status = 0
        self.native_drain = "complete"
        self.child_reason = "exited"
        self.binary = self.root / "beta"
        for replacement in (
            patch.object(server, "lifecycle_reservation", side_effect=self.reservation),
            patch.object(
                server,
                "config_for",
                side_effect=lambda state: server.read_json(state / "config.json"),
            ),
            patch.object(server, "service_directory", side_effect=lambda state: state),
            patch.object(server, "managed_contexts", side_effect=lambda state: [state]),
            patch.object(server, "all_contexts_drained"),
            patch.object(tool, "predecessor_marker", return_value=None),
            patch.object(server, "active", return_value=False),
            patch.object(server, "reservation_live", return_value=True),
            patch.object(server, "maintenance_endpoint", side_effect=self.endpoint),
            patch.object(
                server, "require_maintenance_owner", side_effect=self.require_owner
            ),
            patch.object(server, "drain_maintenance_listener", side_effect=self.drain),
            patch.object(
                tool.surrealkit,
                "verify",
                return_value={"version": "1.0.0-beta.6", "binary_sha256": "binary"},
            ),
            patch.object(
                tool.surrealkit,
                "verify_saved",
                return_value={"version": "1.0.0-beta.6", "binary_sha256": "binary"},
            ),
            patch.object(
                tool.subprocess,
                "run",
                return_value=subprocess.CompletedProcess(
                    [], 0, "1 rollout(s) are valid, in order: one\n", ""
                ),
            ),
            patch.object(
                tool.native_operation, "run_child_scope", side_effect=self.native
            ),
        ):
            replacement.start()
            self.addCleanup(replacement.stop)

    @contextlib.contextmanager
    def reservation(self, _state: Path) -> Generator[None]:
        self.calls.append("reserve")
        yield
        self.calls.append("release")

    def endpoint(self, state: Path) -> str:
        self.calls.append("listener")
        (state / ".maintenance-listener.json").write_text("{}")
        return "http://127.0.0.1:19000"

    def require_owner(self, _state: Path, config: dict[str, object]) -> None:
        self.calls.append("authority")
        self.assertFalse(config["accepting_writes"])

    def drain(self, state: Path) -> None:
        self.calls.append("drain")
        (state / ".maintenance-listener.json").unlink(missing_ok=True)

    def native(
        self,
        command: list[str],
        env: dict[str, str],
        *,
        cwd: Path,
        observation: dict[str, object],
    ) -> int:
        del cwd  # The child callback is deliberately independent of working directory.
        self.calls.append("child")
        attempt = Path(command[command.index("--attempt") + 1])
        snapshot = Path(command[command.index("--folder") + 1])
        self.assertEqual(tool.inputs(snapshot)[1], tool.inputs(self.folder)[1])
        self.assertEqual(env["SURREALDB_HOST"], "grpc://127.0.0.1:19000")
        self.assertEqual(env["SURREALDB_AUTH_LEVEL"], "root")
        self.assertNotIn("SURREAL_PASS", env)
        server.write_json(
            attempt / "child.json",
            {"reason": self.child_reason, "exit_code": self.native_status},
        )
        observation["owned_drain"] = self.native_drain
        return self.native_status

    def run_tool(
        self,
        *,
        rollout_id: str | None = None,
        pending_nonce: str | None = None,
        timeout: float = 120,
    ) -> dict[str, object]:
        return tool.execute(
            self.binary,
            self.state,
            self.folder,
            "up",
            rollout_id=rollout_id,
            pending_nonce=pending_nonce,
            timeout=timeout,
        )

    def current(self) -> dict[str, object]:
        return cast(
            "dict[str, object]",
            server.read_json(self.state / "config.json")[tool.PENDING],
        )

    def evidence(self, selected: Mapping[str, object]) -> dict[str, object]:
        return {
            "nonce": selected["nonce"],
            "input_sha256": selected["input_sha256"],
            "outcome": "settled",
            "operation": {
                "owner": "canonical-rollout-operation",
                "identity": "actual-operation-id",
            },
            "phase": "complete",
            "steps": [{"id": "definition", "outcome": "acknowledged"}],
        }

    def test_frozen_input_missing_or_falsified_refuses_before_listener(self) -> None:
        path = self.folder / "rollouts/one/schema/item.surql"
        for remove in [False, True]:
            if remove:
                path.unlink()
            else:
                path.write_text("changed SQL")
            with self.assertRaisesRegex(tool.MaintenanceError, "missing or falsified"):
                self.run_tool()
        self.assertNotIn("listener", self.calls)
        self.assertNotIn("child", self.calls)

    def test_external_paths_and_ambient_target_injection_refused(self) -> None:
        manifest = self.folder / "rollouts/one.toml"
        manifest.write_text(
            manifest.read_text().replace("schema/item.surql", "../outside.surql")
        )
        with self.assertRaisesRegex(tool.MaintenanceError, "External"):
            self.run_tool()
        (self.folder / "surrealkit.toml").write_text(
            '[target.other]\nhost="http://elsewhere"'
        )
        with self.assertRaisesRegex(tool.MaintenanceError, "Ambient"):
            self.run_tool()

    def test_open_admission_and_shared_context_refuse_without_child(self) -> None:
        self.config["accepting_writes"] = True
        server.write_json(self.state / "config.json", self.config)
        with self.assertRaisesRegex(tool.MaintenanceError, "exclusive closed"):
            self.run_tool()
        self.config["accepting_writes"] = False
        server.write_json(self.state / "config.json", self.config)
        with (
            patch.object(
                server, "managed_contexts", return_value=[self.state, self.root]
            ),
            self.assertRaises(tool.MaintenanceError),
        ):
            self.run_tool()
        self.assertNotIn("child", self.calls)

    def test_active_service_requires_explicit_stop(self) -> None:
        with (
            patch.object(server, "active", return_value=True),
            patch.object(server, "stop") as stop,
            self.assertRaisesRegex(tool.MaintenanceError, "explicitly"),
        ):
            self.run_tool()
        stop.assert_not_called()
        self.assertNotIn("listener", self.calls)

    def test_fork_lint_failure_preserves_prepared_identity_without_sql(self) -> None:
        with (
            patch.object(
                tool.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 1, "", "fork in chain"),
            ),
            self.assertRaisesRegex(tool.MaintenanceError, "lint failed"),
        ):
            self.run_tool()
        self.assertEqual(self.current()["sql_outcome"], "not_submitted")
        self.assertEqual(self.current()["phase"], "prepared")
        self.assertNotIn("listener", self.calls)
        self.assertNotIn("child", self.calls)

    def test_success_is_not_settlement_or_activation_and_binds_snapshot(self) -> None:
        with (
            patch.dict(
                os.environ, {"DATABASE_URL": "injected", "SURREAL_PASS": "injected"}
            ),
            patch.object(server, "open_context_admission") as activate,
        ):
            result = self.run_tool()
        activate.assert_not_called()
        self.assertEqual(result["phase"], "cli_succeeded")
        self.assertEqual(result["sql_outcome"], "unknown")
        self.assertEqual(result["owner"], self.owner)
        self.assertEqual(self.current()["nonce"], result["nonce"])
        self.assertFalse(
            server.read_json(self.state / "config.json")["accepting_writes"]
        )
        self.assertLess(self.calls.index("authority"), self.calls.index("child"))
        self.assertGreater(self.calls.index("drain"), self.calls.index("child"))
        self.assertEqual(
            cast("list[dict[str, object]]", result["attempts"])[0]["listener_drain"],
            "complete",
        )

    def test_private_bootstrap_failure_resumes_same_unsubmitted_nonce_after_drain(
        self,
    ) -> None:
        self.config.update(
            admission="validation_required",
            fresh_root_bootstrap={
                "instance_id": self.config["instance_id"],
                "credentials_sha256": server.file_digest(
                    self.state / "credentials.json"
                ),
                "nonce": "a" * 32,
            },
        )
        server.write_json(self.state / "config.json", self.config)
        attempts = 0

        def bootstrap(state: Path) -> str:
            nonlocal attempts
            attempts += 1
            selected = server.read_json(state / "config.json")
            self.assertIsNotNone(server.fresh_root_authority(state, selected))
            if attempts == 1:
                raise server.SupervisorError("private ROOT acknowledgment interrupted")
            selected.pop("fresh_root_bootstrap")
            server.write_json(state / "config.json", selected)
            return self.endpoint(state)

        with patch.object(server, "maintenance_endpoint", side_effect=bootstrap):
            with self.assertRaisesRegex(server.SupervisorError, "ROOT acknowledgment"):
                self.run_tool()
            pending = self.current()
            self.assertEqual(
                (pending["phase"], pending["sql_outcome"]),
                ("prepared", "not_submitted"),
            )
            self.assertNotIn("child", self.calls)
            self.owner = {**self.owner, "nonce": "b" * 32}
            server.write_json(self.state / "lifecycle-owner.json", self.owner)
            self.calls.clear()
            resumed = self.run_tool(pending_nonce=str(pending["nonce"]))
        self.assertEqual(resumed["nonce"], pending["nonce"])
        self.assertEqual(resumed["owner"], self.owner)
        self.assertLess(self.calls.index("drain"), self.calls.index("listener"))
        self.assertEqual(resumed["sql_outcome"], "unknown")
        with self.assertRaisesRegex(tool.MaintenanceError, "prepared, unsubmitted"):
            self.run_tool(pending_nonce=str(pending["nonce"]))

    def test_whole_chain_submission_binds_beta_order_instead_of_filename_order(
        self,
    ) -> None:
        for identity in ("a", "z"):
            (self.folder / "rollouts" / (identity + ".toml")).write_text(
                'id = "' + identity + '"\n'
            )
        output = "3 rollout(s) are valid, in order: z, one, a\n"
        with patch.object(
            tool.subprocess,
            "run",
            return_value=subprocess.CompletedProcess([], 0, output, ""),
        ):
            value = self.run_tool()
        self.assertEqual(value["rollouts"], ["z", "one", "a"])
        self.assertEqual(self.current()["rollouts"], ["z", "one", "a"])
        self.assertEqual(value["sql_outcome"], "unknown")
        self.assertEqual(
            cast(
                "dict[str, object]",
                cast("list[dict[str, object]]", value["attempts"])[0]["child"],
            )["owned_drain"],
            "complete",
        )

    def test_invalid_beta_order_refuses_before_listener_or_sql(self) -> None:
        for output in (
            "chain valid",  # No authenticated chain order.
            "1 rollout(s) are valid, in order: other\n",  # Wrong inventory.
            "2 rollout(s) are valid, in order: one, one\n",  # Duplicate.
            "2 rollout(s) are valid, in order: one\n",  # Wrong count.
            "1 rollout(s) are valid, in order: one, extra\n",  # Extra input.
        ):
            with self.subTest(output=output):
                server.write_json(self.state / "config.json", self.config)
                self.calls.clear()
                with (
                    patch.object(
                        tool.subprocess,
                        "run",
                        return_value=subprocess.CompletedProcess([], 0, output, ""),
                    ),
                    self.assertRaisesRegex(
                        tool.MaintenanceError, "Validated beta rollout order"
                    ),
                ):
                    self.run_tool()
                self.assertEqual(self.current()["phase"], "prepared")
                self.assertEqual(self.current()["sql_outcome"], "not_submitted")
                self.assertNotIn("listener", self.calls)
                self.assertNotIn("child", self.calls)

    def test_timeout_failure_or_unproved_child_drain_retains_unknown(self) -> None:
        for status, reason, drain in [
            (1, "exited", "complete"),
            (125, "timeout", "complete"),
            (0, "exited", "unresolved"),
        ]:
            with self.subTest(status=status, reason=reason, drain=drain):
                config = server.read_json(self.state / "config.json")
                config.pop(tool.PENDING, None)
                server.write_json(self.state / "config.json", config)
                self.native_status, self.child_reason, self.native_drain = (
                    status,
                    reason,
                    drain,
                )
                result = self.run_tool()
                self.assertEqual(result["phase"], "unknown")
                self.assertEqual(result["sql_outcome"], "unknown")
                self.assertEqual(
                    cast(
                        "dict[str, object]",
                        cast("list[dict[str, object]]", result["attempts"])[0][
                            "result"
                        ],
                    )["reason"],
                    reason,
                )
                self.assertFalse(
                    server.read_json(self.state / "config.json")["accepting_writes"]
                )

    def test_blind_up_and_unsettled_repair_refused_status_preserves_unknown_command(
        self,
    ) -> None:
        self.native_status = 125
        result = self.run_tool()
        nonce = str(result["nonce"])
        with self.assertRaisesRegex(tool.MaintenanceError, "blind replay"):
            self.run_tool()
        self.native_status = 0
        for mode, identity in [("status", None)]:
            result = tool.execute(
                self.binary,
                self.state,
                self.folder,
                mode,
                rollout_id=identity,
                pending_nonce=nonce,
            )
            self.assertEqual(result["phase"], "unknown")
            self.assertEqual(result["sql_outcome"], "unknown")
            self.assertEqual(result["command"], ["rollout", "up", "--complete"])
            self.assertEqual(
                cast("list[dict[str, object]]", result["attempts"])[-1]["phase"],
                "cli_succeeded",
            )
        with self.assertRaisesRegex(tool.MaintenanceError, "exact SQL settlement"):
            tool.execute(
                self.binary,
                self.state,
                self.folder,
                "repair",
                rollout_id="one",
                pending_nonce=nonce,
            )
        with self.assertRaisesRegex(tool.MaintenanceError, "pending rollout identity"):
            tool.execute(
                self.binary, self.state, self.folder, "status", pending_nonce="other"
            )
        with self.assertRaisesRegex(tool.MaintenanceError, "submitted rollout"):
            tool.execute(
                self.binary,
                self.state,
                self.folder,
                "repair",
                rollout_id="other",
                pending_nonce=nonce,
            )

    def test_recovery_cannot_substitute_tool_provenance(self) -> None:
        value = self.run_tool()
        self.calls.clear()
        with (
            patch.object(
                tool.surrealkit,
                "verify_saved",
                return_value={"binary_sha256": "replacement"},
            ),
            self.assertRaisesRegex(tool.MaintenanceError, "tool provenance"),
        ):
            tool.execute(
                self.binary,
                self.state,
                self.folder,
                "status",
                pending_nonce=str(value["nonce"]),
            )
        self.assertNotIn("listener", self.calls)
        self.assertEqual(self.current()["nonce"], value["nonce"])

    def test_exact_pending_recovery_uses_saved_immutable_tool_after_source_changes(
        self,
    ) -> None:
        value = self.run_tool()
        built = self.root / "built"
        built.write_bytes(b"immutable saved executable")
        declared = tool.surrealkit.selection()
        old = surrealkit_source.publish(
            self.root,
            declared,
            "a" * 64,
            built,
            {"source_build": True, "release_build_locked": True},
        )
        receipt = server.read_json(old.parent / "release.json")
        value["tool"] = receipt
        tool.update(self.state, value)

        def run(args: list[str], **_kwargs: object) -> subprocess.CompletedProcess[str]:
            output = (
                "surrealkit " + declared["version"] + "\n"
                if args[-1] == "--version"
                else "1 rollout(s) are valid, in order: one\n"
            )
            return subprocess.CompletedProcess(args, 0, output, "")

        with (
            patch.object(
                tool.surrealkit,
                "verify",
                side_effect=tool.surrealkit.ToolError("current inputs changed"),
            ) as ordinary,
            patch.object(
                tool.surrealkit, "verify_saved", side_effect=VERIFY_SAVED
            ) as saved,
            patch.object(tool.subprocess, "run", side_effect=run),
        ):
            recovered = tool.execute(
                old,
                self.state,
                self.folder,
                "status",
                pending_nonce=str(value["nonce"]),
            )
            self.assertEqual(recovered["tool"], receipt)
            ordinary.assert_not_called()
            saved.assert_called_once_with(old, receipt)
            other = surrealkit_source.publish(
                self.root,
                declared,
                "b" * 64,
                built,
                {"source_build": True, "release_build_locked": True},
            )
            with self.assertRaisesRegex(tool.surrealkit.ToolError, "submitted rollout"):
                tool.execute(
                    other,
                    self.state,
                    self.folder,
                    "status",
                    pending_nonce=str(value["nonce"]),
                )
            saved.reset_mock()
            with self.assertRaisesRegex(
                tool.MaintenanceError, "pending rollout identity"
            ):
                tool.execute(
                    old, self.state, self.folder, "status", pending_nonce="arbitrary"
                )
            saved.assert_not_called()
            self.assertEqual(self.current()["command"], ["rollout", "up", "--complete"])

    def test_recovery_cannot_substitute_frozen_chain(self) -> None:
        result = self.run_tool()
        (self.folder / "schema").mkdir()
        (self.folder / "schema/new.surql").write_text("DEFINE TABLE other;")
        with self.assertRaisesRegex(tool.MaintenanceError, "input differs"):
            tool.execute(
                self.binary,
                self.state,
                self.folder,
                "status",
                pending_nonce=str(result["nonce"]),
            )

    def test_listener_drain_failure_preserves_pending_even_on_cli_zero(self) -> None:
        with (
            patch.object(
                server,
                "drain_maintenance_listener",
                side_effect=server.SupervisorError("invocation changed"),
            ),
            self.assertRaisesRegex(server.SupervisorError, "invocation changed"),
        ):
            self.run_tool()
        self.assertEqual(
            cast("list[dict[str, object]]", self.current()["attempts"])[0][
                "listener_drain"
            ],
            "unresolved",
        )
        self.assertFalse(
            server.read_json(self.state / "config.json")["accepting_writes"]
        )

    def test_unresolved_predecessor_cannot_reconnect_via_recovery_or_settlement(
        self,
    ) -> None:
        self.native_drain = "unresolved"
        value = self.run_tool()
        self.calls.clear()
        for action in [
            lambda: tool.execute(
                self.binary,
                self.state,
                self.folder,
                "status",
                pending_nonce=str(value["nonce"]),
            ),
            lambda: tool.settle(
                self.state, str(value["nonce"]), lambda *_: self.evidence(value)
            ),
        ]:
            with self.assertRaisesRegex(tool.MaintenanceError, "exact drain recovery"):
                action()
        self.assertNotIn("listener", self.calls)
        self.assertNotIn("child", self.calls)
        self.assertEqual(self.current()["nonce"], value["nonce"])

    def test_recovery_observes_exact_predecessor_native_drain_before_listener(
        self,
    ) -> None:
        self.native_drain = "unresolved"
        value = self.run_tool()
        attempt = Path(
            cast(
                "str",
                cast("list[dict[str, object]]", value["attempts"])[0]["directory"],
            )
        )
        native_record = {
            "scope": {
                "unit": "original",
                "group": "/original",
                "invocation": "original-invocation",
            },
            "scope_inode": 123,
        }
        server.write_json(
            attempt / "child.json",
            {"reason": "timeout", "native_record": native_record},
        )
        self.calls.clear()
        unresolved = tool.operation_lifetime.DrainObservation(
            native_record["scope"],
            tool.operation_lifetime.DrainPhase.POPULATED,
            "still live",
        )
        with (
            patch.object(operation_lifetime, "drained", return_value=unresolved),
            self.assertRaisesRegex(tool.MaintenanceError, "still live"),
        ):
            tool.execute(
                self.binary,
                self.state,
                self.folder,
                "status",
                pending_nonce=str(value["nonce"]),
            )
        self.assertNotIn("listener", self.calls)
        self.native_drain = "complete"
        complete = tool.operation_lifetime.DrainObservation(
            native_record["scope"],
            tool.operation_lifetime.DrainPhase.COMPLETE,
            "owned scope drained",
        )
        with patch.object(
            operation_lifetime, "drained", return_value=complete
        ) as observed:
            result = tool.execute(
                self.binary,
                self.state,
                self.folder,
                "status",
                pending_nonce=str(value["nonce"]),
            )
        observed.assert_called_once_with(native_record)
        self.assertEqual(
            cast(
                "dict[str, object]",
                cast("list[dict[str, object]]", result["attempts"])[0]["child"],
            )["owned_drain"],
            "complete",
        )
        self.assertEqual(
            result["phase"], "unknown", "native drain alone never settles submitted SQL"
        )

    def test_settlement_rejects_boolean_and_wrong_selection(self) -> None:
        value = self.run_tool()
        for proof in [True, {**self.evidence(value), "input_sha256": "wrong"}]:

            def invalid_evidence(
                _state: Path,
                _endpoint: str,
                _selected: Mapping[str, object],
                *,
                proof: object = proof,
            ) -> Mapping[str, object]:
                # Deliberately violate the callback's evidence contract to test refusal.
                return cast("Mapping[str, object]", proof)

            with self.assertRaisesRegex(tool.MaintenanceError, "structured operation"):
                tool.settle(self.state, str(value["nonce"]), invalid_evidence)
            self.assertEqual(self.current()["nonce"], value["nonce"])
        self.assertFalse((self.state / ".maintenance-listener.json").exists())

    def test_unknown_requires_actual_operation_evidence_then_drain_and_stays_closed(
        self,
    ) -> None:
        self.native_status = 125
        value = self.run_tool()

        def operation(
            state: Path, endpoint: str, selected: Mapping[str, object]
        ) -> dict[str, object]:
            self.assertEqual(state, self.state)
            self.assertEqual(endpoint, "http://127.0.0.1:19000")
            self.assertTrue((state / ".maintenance-listener.json").exists())
            self.assertEqual(selected["phase"], "unknown")
            # Mutation probes deliberately cast away the immutable callback surface.
            with self.assertRaises(TypeError):
                cast("dict[str, object]", selected)["input_sha256"] = "changed"
            with self.assertRaises(TypeError):
                cast("list[dict[str, object]]", selected["attempts"])[0]["phase"] = (
                    "changed"
                )
            return self.evidence(selected)

        result = tool.settle(self.state, str(value["nonce"]), operation)
        self.assertEqual(
            cast(
                "dict[str, object]",
                cast("dict[str, object]", result["settlement"])["operation"],
            )["identity"],
            "actual-operation-id",
        )
        self.assertEqual(self.current()["phase"], "settled")
        self.assertEqual(self.current()["sql_outcome"], "settled")
        self.assertFalse(
            server.read_json(self.state / "config.json")["accepting_writes"]
        )
        self.assertTrue(
            (self.state / ".surrealkit" / str(value["nonce"]) / "settled.json").exists()
        )
        self.assertFalse((self.state / ".maintenance-listener.json").exists())

    def test_complete_requires_settled_start_and_retains_one_nonce(self) -> None:
        value = tool.execute(
            self.binary, self.state, self.folder, "start", rollout_id="one"
        )
        nonce = cast("str", value["nonce"])
        with self.assertRaisesRegex(tool.MaintenanceError, "exact settled start"):
            tool.execute(
                self.binary,
                self.state,
                self.folder,
                "complete",
                rollout_id="one",
                pending_nonce=nonce,
            )
        proof = {**self.evidence(value), "phase": "start"}
        tool.settle(self.state, nonce, lambda *_: proof)
        result = tool.execute(
            self.binary,
            self.state,
            self.folder,
            "complete",
            rollout_id="one",
            pending_nonce=nonce,
        )
        self.assertEqual(result["nonce"], nonce)
        self.assertEqual(result["command"], ["rollout", "complete", "one"])
        self.assertEqual(
            result["phase_history"],
            [{"command": ["rollout", "start", "one"], "settlement": proof}],
        )
        self.assertEqual(result["sql_outcome"], "unknown")
        self.assertNotIn("settlement", result)
        self.assertIsNone(result["previous_marker"])

    def test_predecessor_is_captured_before_submission_and_never_recaptured(
        self,
    ) -> None:
        marker = {
            "interpretation": "v5",
            "schema_digest": "before",
            "issuing_store": "store",
        }
        with patch.object(tool, "predecessor_marker", return_value=marker) as capture:
            value = tool.execute(
                self.binary, self.state, self.folder, "start", rollout_id="one"
            )
            capture.assert_called_once()
        self.assertEqual(value["previous_marker"], marker)
        proof = {**self.evidence(value), "phase": "start"}
        tool.settle(self.state, cast("str", value["nonce"]), lambda *_: proof)
        with patch.object(tool, "predecessor_marker") as capture:
            result = tool.execute(
                self.binary,
                self.state,
                self.folder,
                "complete",
                rollout_id="one",
                pending_nonce=cast("str", value["nonce"]),
            )
            capture.assert_not_called()
        self.assertEqual(result["previous_marker"], marker)

    def test_settlement_rejects_wrong_phase_without_publishing_settled(self) -> None:
        value = tool.execute(
            self.binary, self.state, self.folder, "start", rollout_id="one"
        )
        with self.assertRaisesRegex(tool.MaintenanceError, "structured operation"):
            tool.settle(
                self.state, cast("str", value["nonce"]), lambda *_: self.evidence(value)
            )
        self.assertEqual(self.current()["sql_outcome"], "unknown")
        self.assertNotIn("settlement", self.current())

    def generated_fixture(
        self, status: str = "completed", *, acknowledged: bool = True
    ) -> tuple[
        dict[str, object], dict[str, object], dict[str, object], dict[str, object]
    ]:
        value = self.run_tool()
        step = {
            "id": "definition",
            "phase": "start",
            "kind": "apply_files",
            "checksum": "exact-step",
        }
        item: dict[str, object] = {
            "id": "one",
            "manifest_checksum": "exact-manifest",
            "source_schema_hash": "source",
            "target_schema_hash": "target",
            "source_sql": "source.surql",
            "target_sql": "target.surql",
            "steps": [step],
        }
        row: dict[str, object] = {
            name: item[name]
            for name in (
                "manifest_checksum",
                "source_schema_hash",
                "target_schema_hash",
            )
        }
        row.update(
            rollout_id="one",
            status=status,
            steps=[
                {
                    "step_id": "definition",
                    **{name: step[name] for name in ("phase", "kind", "checksum")},
                    "status": "completed" if acknowledged else "running",
                    "error": None,
                }
            ],
        )
        witness: dict[str, object] = {
            "scope": "generated-target",
            "previous_marker": None,
            "schema_digest": "current",
        }
        return value, item, row, witness

    def test_generated_settlement_joins_exact_ack_and_canonical_postconditions(
        self,
    ) -> None:
        value, item, row, witness = self.generated_fixture()
        with (
            patch.object(tool, "publication", return_value={"rollouts": [item]}),
            patch.object(server, "maintenance_query", return_value={"value": [row]}),
            patch.object(tool, "canonical_tool", return_value=witness) as canonical,
        ):
            result = tool.settle_generated(
                self.state, cast("str", value["nonce"]), self.folder, self.binary
            )
        self.assertEqual(result["phase"], "settled")
        self.assertEqual(
            cast(
                "list[dict[str, object]]",
                cast("dict[str, object]", result["settlement"])["steps"],
            )[0]["outcome"],
            "acknowledged",
        )
        self.assertEqual(
            cast("dict[str, object]", result["settlement"])["canonical"], witness
        )
        self.assertEqual(canonical.call_args.kwargs["expected_schemas"], ())

    def test_whole_chain_settlement_cannot_omit_initial_baseline(self) -> None:
        value, item, row, witness = self.generated_fixture()
        baseline = {**item, "id": "initial-baseline"}
        with (
            patch.object(
                tool, "publication", return_value={"rollouts": [baseline, item]}
            ),
            patch.object(
                server, "maintenance_query", return_value={"value": [row]}
            ) as query,
            patch.object(tool, "canonical_tool", return_value=witness) as canonical,
            self.assertRaisesRegex(tool.MaintenanceError, "Whole-chain submission"),
        ):
            tool.settle_generated(
                self.state, cast("str", value["nonce"]), self.folder, self.binary
            )
        query.assert_not_called()
        canonical.assert_not_called()
        self.assertEqual(self.current()["sql_outcome"], "unknown")
        value["rollouts"] = ["initial-baseline", "one"]
        tool.update(self.state, value)
        with (
            patch.object(
                tool, "publication", return_value={"rollouts": [baseline, item]}
            ),
            patch.object(server, "maintenance_query", return_value={"value": [row]}),
            patch.object(tool, "canonical_tool", return_value=witness) as canonical,
            self.assertRaisesRegex(
                tool.MaintenanceError, "Actual beta manifest identity"
            ),
        ):
            tool.settle_generated(
                self.state, cast("str", value["nonce"]), self.folder, self.binary
            )
        canonical.assert_not_called()
        self.assertEqual(self.current()["sql_outcome"], "unknown")

    def test_generated_unknown_sql_reconciles_but_metadata_must_complete_before_activation(
        self,
    ) -> None:
        value, item, row, witness = self.generated_fixture(
            "running_complete", acknowledged=False
        )
        with (
            patch.object(tool, "publication", return_value={"rollouts": [item]}),
            patch.object(server, "maintenance_query", return_value={"value": [row]}),
            patch.object(tool, "canonical_tool", return_value=witness),
        ):
            result = tool.settle_generated(
                self.state, cast("str", value["nonce"]), self.folder, self.binary
            )
        self.assertEqual(result["phase"], "sql_settled")
        self.assertEqual(
            cast(
                "list[dict[str, object]]",
                cast("dict[str, object]", result["settlement"])["steps"],
            )[0]["outcome"],
            "reconciled",
        )
        with self.assertRaisesRegex(tool.MaintenanceError, "final canonical"):
            tool.activate(self.state, cast("str", value["nonce"]), self.binary)
        repaired = tool.execute(
            self.binary,
            self.state,
            self.folder,
            "repair",
            rollout_id="one",
            pending_nonce=cast("str", value["nonce"]),
        )
        self.assertEqual(repaired["phase"], "sql_settled")
        row["status"] = "completed"
        with (
            patch.object(tool, "publication", return_value={"rollouts": [item]}),
            patch.object(server, "maintenance_query", return_value={"value": [row]}),
            patch.object(tool, "canonical_tool", return_value=witness),
        ):
            result = tool.settle_generated(
                self.state, cast("str", value["nonce"]), self.folder, self.binary
            )
        self.assertEqual(result["phase"], "settled")

    def test_generated_settlement_refuses_wrong_step_or_data_sql_before_canonical_tool(
        self,
    ) -> None:
        value, item, row, witness = self.generated_fixture()
        for change in ("checksum", "data"):
            with self.subTest(change=change):
                if change == "checksum":
                    cast("list[dict[str, object]]", row["steps"])[0]["checksum"] = (
                        "substituted"
                    )
                else:
                    cast("list[dict[str, object]]", row["steps"])[0]["checksum"] = cast(
                        "list[dict[str, object]]", item["steps"]
                    )[0]["checksum"]
                    cast("list[dict[str, object]]", row["steps"])[0]["kind"] = "run_sql"
                    cast("list[dict[str, object]]", item["steps"])[0]["kind"] = (
                        "run_sql"
                    )
                with (
                    patch.object(
                        tool, "publication", return_value={"rollouts": [item]}
                    ),
                    patch.object(
                        server, "maintenance_query", return_value={"value": [row]}
                    ),
                    patch.object(
                        tool, "canonical_tool", return_value=witness
                    ) as canonical,
                    self.assertRaises(tool.MaintenanceError),
                ):
                    tool.settle_generated(
                        self.state,
                        cast("str", value["nonce"]),
                        self.folder,
                        self.binary,
                    )
                canonical.assert_not_called()
                self.assertEqual(self.current()["sql_outcome"], "unknown")

    def test_canonical_child_wrong_authority_witness_keeps_pending_and_child_drain(
        self,
    ) -> None:
        value = self.run_tool()
        value["initializer"] = self.owner
        tool.update(self.state, value)

        def wrong_witness(
            command: list[str],
            _env: dict[str, str],
            *,
            cwd: Path,
            observation: dict[str, object],
        ) -> int:
            del cwd
            attempt = Path(command[command.index("--attempt") + 1])
            server.write_json(attempt / "child.json", {"reason": "exited"})
            server.write_json(
                attempt / "stdout",
                {
                    "namespace": "other",
                    "database": value["database"],
                    "instance_id": value["instance_id"],
                    "initializer_nonce": self.owner["nonce"],
                },
            )
            observation["owned_drain"] = "complete"
            return 0

        with (
            patch.object(
                tool.native_operation, "run_child_scope", side_effect=wrong_witness
            ),
            self.assertRaisesRegex(
                tool.MaintenanceError, "selected maintenance authority"
            ),
        ):
            tool.canonical_tool(
                self.state, self.endpoint(self.state), value, self.binary
            )
        self.assertEqual(self.current()["nonce"], value["nonce"])
        self.assertEqual(
            cast("list[dict[str, object]]", self.current()["canonical_attempts"])[-1][
                "phase"
            ],
            "unknown",
        )
        self.assertEqual(
            cast(
                "dict[str, object]",
                cast("list[dict[str, object]]", self.current()["canonical_attempts"])[
                    -1
                ]["child"],
            )["owned_drain"],
            "complete",
        )

    def test_activation_requires_final_canonical_settlement(self) -> None:
        value = self.run_tool()
        with (
            patch.object(tool, "canonical_tool") as canonical,
            self.assertRaisesRegex(tool.MaintenanceError, "final canonical"),
        ):
            tool.activate(self.state, cast("str", value["nonce"]), self.binary)
        canonical.assert_not_called()
        self.assertEqual(self.current()["nonce"], value["nonce"])

    def test_activation_failure_and_drain_failure_retain_exact_pending(self) -> None:
        value = self.run_tool()
        proof = {**self.evidence(value), "canonical": {"scope": "generated-target"}}
        tool.settle(self.state, cast("str", value["nonce"]), lambda *_: proof)
        with (
            patch.object(
                tool,
                "canonical_tool",
                side_effect=tool.MaintenanceError("lost activation ACK"),
            ),
            self.assertRaisesRegex(tool.MaintenanceError, "lost activation"),
        ):
            tool.activate(self.state, cast("str", value["nonce"]), self.binary)
        self.assertEqual(self.current()["nonce"], value["nonce"])
        with (
            patch.object(
                tool, "canonical_tool", return_value={"issuing_store": "store"}
            ),
            patch.object(
                server,
                "drain_maintenance_listener",
                side_effect=[None, server.SupervisorError("drain failed")],
            ),
            self.assertRaisesRegex(server.SupervisorError, "drain failed"),
        ):
            tool.activate(self.state, cast("str", value["nonce"]), self.binary)
        self.assertEqual(self.current()["nonce"], value["nonce"])
        self.assertEqual(self.current()["activation"], {"issuing_store": "store"})

    def test_activation_releases_pending_only_after_owned_drain_and_remains_closed(
        self,
    ) -> None:
        value = self.run_tool()
        proof = {**self.evidence(value), "canonical": {"scope": "generated-target"}}
        tool.settle(self.state, cast("str", value["nonce"]), lambda *_: proof)

        def activation(
            state: Path,
            _endpoint: str,
            pending: dict[str, object],
            _binary: Path,
            **options: object,
        ) -> dict[str, object]:
            self.assertEqual(pending["initializer"], self.owner)
            self.assertEqual(pending["settlement"], proof)
            self.assertTrue(options["activate"])
            self.assertIn(tool.PENDING, server.read_json(state / "config.json"))
            self.assertTrue((state / ".maintenance-listener.json").exists())
            return {"issuing_store": "same-store"}

        with patch.object(tool, "canonical_tool", side_effect=activation):
            result = tool.activate(self.state, cast("str", value["nonce"]), self.binary)
        self.assertEqual(result["activation"], {"issuing_store": "same-store"})
        self.assertNotIn(tool.PENDING, server.read_json(self.state / "config.json"))
        self.assertFalse((self.state / ".maintenance-listener.json").exists())
        self.assertFalse(
            server.read_json(self.state / "config.json")["accepting_writes"]
        )

    def test_settlement_drain_or_owner_loss_cannot_clear_pending(self) -> None:
        value = self.run_tool()
        with (
            patch.object(
                server,
                "drain_maintenance_listener",
                side_effect=[None, server.SupervisorError("drain failed")],
            ),
            self.assertRaisesRegex(server.SupervisorError, "drain failed"),
        ):
            tool.settle(
                self.state, str(value["nonce"]), lambda *_: self.evidence(value)
            )
        self.assertEqual(self.current()["nonce"], value["nonce"])
        with (
            patch.object(server, "reservation_live", return_value=False),
            self.assertRaisesRegex(tool.MaintenanceError, "owner changed"),
        ):
            tool.settle(
                self.state, str(value["nonce"]), lambda *_: self.evidence(value)
            )
        self.assertEqual(self.current()["nonce"], value["nonce"])


class OwnedChildTests(unittest.TestCase):
    def test_owner_loss_stops_actual_child_despite_continuing_beta_heartbeat(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            attempt = root / "attempt"
            attempt.mkdir()
            binary = root / "healthy-beta"
            heartbeat = root / "heartbeat"
            binary.write_text(
                "#!"
                + sys.executable
                + "\nimport signal,time\nfrom pathlib import Path\n"
                "signal.signal(signal.SIGTERM, signal.SIG_IGN)\n"
                "while True:\n Path("
                + repr(str(heartbeat))
                + ').write_text("beta ttl healthy")\n time.sleep(.01)\n'
            )
            binary.chmod(0o700)
            native_record = root / "native.json"
            server.write_json(
                native_record,
                {"scope": {"unit": "owned", "group": "/owned", "invocation": "exact"}},
            )
            owner = {"nonce": "original"}
            server.write_json(root / "lifecycle-owner.json", owner)
            server.write_json(root / ".maintenance-listener.json", {"owner": owner})
            server.write_json(
                attempt / "authority.json",
                {
                    "owner": owner,
                    "listener": {"owner": owner},
                    "endpoint": "http://127.0.0.1:19000",
                },
            )
            lost = threading.Event()

            def authority(*_: object) -> None:
                if lost.is_set():
                    raise server.SupervisorError("owner lost")

            def lose_owner() -> None:
                deadline = time.monotonic() + 5
                while not heartbeat.exists() and time.monotonic() < deadline:
                    time.sleep(0.01)
                lost.set()

            losing = threading.Thread(target=lose_owner)
            losing.start()
            with (
                patch.dict(os.environ, {"SURREALDB_HOST": "grpc://127.0.0.1:19000"}),
                patch.object(server, "reservation_live", return_value=True),
                patch.object(server, "config_for", return_value={"port": 19000}),
                patch.object(
                    server, "require_maintenance_owner", side_effect=authority
                ),
                patch.object(
                    operation_lifetime, "scope_owner", return_value={"unit": "owned"}
                ),
                patch.object(
                    tool.native_operation, "current", return_value=native_record
                ),
            ):
                status = tool.owned_child(binary, root, root, attempt, "up", None, 10)
            losing.join(timeout=1)
            self.assertFalse(losing.is_alive())
            self.assertEqual(heartbeat.read_text(), "beta ttl healthy")
            self.assertEqual(status, 125)
            result = server.read_json(attempt / "child.json")
            self.assertEqual(result["reason"], "owner_lost")
            self.assertLess(
                cast("int", result["exit_code"]),
                0,
                "TERM-ignoring child was reaped after owned escalation",
            )

    def test_valid_successor_cannot_adopt_original_child_even_after_concurrent_exit(
        self,
    ) -> None:
        for concurrent_exit in (False, True):
            with (
                self.subTest(concurrent_exit=concurrent_exit),
                tempfile.TemporaryDirectory() as temporary,
            ):
                root = Path(temporary)
                attempt = root / "attempt"
                attempt.mkdir()
                owner = {"nonce": "original"}
                successor: dict[str, object] = {"nonce": "valid-successor"}
                server.write_json(root / "lifecycle-owner.json", owner)
                server.write_json(root / ".maintenance-listener.json", {"owner": owner})
                server.write_json(
                    attempt / "authority.json",
                    {
                        "owner": owner,
                        "listener": {"owner": owner},
                        "endpoint": "http://127.0.0.1:19000",
                    },
                )
                native = root / "native.json"
                server.write_json(native, {"scope": {"unit": "owned"}})
                binary = root / "beta"
                binary.write_text(
                    "#!"
                    + sys.executable
                    + "\nimport time\ntime.sleep("
                    + ("0.05" if concurrent_exit else "20")
                    + ")\n"
                )
                binary.chmod(0o700)
                real_spawn = subprocess.Popen

                def replace(
                    *,
                    root: Path = root,
                    successor: dict[str, object] = successor,
                ) -> None:
                    server.write_json(root / "lifecycle-owner.json", successor)
                    server.write_json(
                        root / ".maintenance-listener.json", {"owner": successor}
                    )

                def spawn(
                    args: list[str],
                    *,
                    env: dict[str, str],
                    cwd: Path,
                    stdout: BinaryIO,
                    stderr: BinaryIO,
                    start_new_session: bool,
                    real_spawn: type[subprocess.Popen[bytes]] = real_spawn,
                    concurrent_exit: bool = concurrent_exit,
                ) -> subprocess.Popen[bytes]:
                    child = real_spawn(
                        args,
                        env=env,
                        cwd=cwd,
                        stdout=stdout,
                        stderr=stderr,
                        start_new_session=start_new_session,
                    )
                    real_wait = child.wait

                    def wait(
                        timeout: float | None = None,
                        *,
                        concurrent_exit: bool = concurrent_exit,
                    ) -> int:
                        if not concurrent_exit:
                            replace()
                        result = real_wait(timeout=timeout)
                        if concurrent_exit:
                            replace()
                        return result

                    child.wait = wait
                    return child

                with (
                    patch.dict(
                        os.environ, {"SURREALDB_HOST": "grpc://127.0.0.1:19000"}
                    ),
                    patch.object(server, "reservation_live", return_value=True),
                    patch.object(server, "config_for", return_value={"port": 19000}),
                    patch.object(
                        server, "require_maintenance_owner", return_value=None
                    ),
                    patch.object(
                        operation_lifetime,
                        "scope_owner",
                        return_value={"unit": "owned"},
                    ),
                    patch.object(tool.native_operation, "current", return_value=native),
                    patch.object(tool.subprocess, "Popen", side_effect=spawn),
                    patch.object(
                        os,
                        "killpg",
                        side_effect=AssertionError("numeric PGID signal refused"),
                    ),
                ):
                    status = tool.owned_child(
                        binary, root, root, attempt, "up", None, 5
                    )
                self.assertEqual(status, 125)
                self.assertEqual(
                    server.read_json(attempt / "child.json")["reason"], "owner_lost"
                )

    def test_direct_internal_entry_without_native_admission_refuses_spawn(self) -> None:
        with (
            patch.object(operation_lifetime, "scope_owner", return_value=None),
            patch.object(tool.subprocess, "Popen") as spawn,
            self.assertRaisesRegex(tool.MaintenanceError, "admitted native scope"),
        ):
            tool.owned_child(
                Path("binary"),
                Path("state"),
                Path("input"),
                Path("attempt"),
                "up",
                None,
                1,
            )
        spawn.assert_not_called()


class RevocationPreparationTests(unittest.TestCase):
    def test_metadata_readiness_retries_only_exact_absence_within_observer_clock(
        self,
    ) -> None:
        for table in ("__rollout", "__entity"):
            values: list[object] = [
                "The table '" + table + "' does not exist",
                [],
                {"tables": {}},
            ]
            self.assertEqual(probe.checked_catalog(values, bootstrap=True), [])
            with self.assertRaisesRegex(probe.tool.ToolError, "query refused"):
                probe.checked_catalog(values)
        for error in (
            "Parse error",
            "The table 'canonical_runs' does not exist",
            "Permission denied",
        ):
            with self.assertRaisesRegex(probe.tool.ToolError, "query refused"):
                probe.checked_catalog([error], bootstrap=True)
        acknowledged: list[object] = [
            [{"status": "running_start"}],
            [True],
            {"tables": {}},
        ]
        self.assertEqual(
            probe.checked_catalog(acknowledged, bootstrap=True), acknowledged
        )

    def test_wrong_controls_selection_refuses_before_input_or_service_effects(
        self,
    ) -> None:
        config = {
            "instance_id": "selected",
            "namespace": "pse",
            "database": "canonical",
        }
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            with (
                patch.object(probe.maintenance, "selected", return_value=config),
                patch.object(
                    probe.resources,
                    "resource_status",
                    return_value={"kind": "namespace"},
                ),
                patch.object(server, "lifecycle_reservation") as lifecycle,
                self.assertRaisesRegex(
                    probe.tool.ToolError, "registered disposable maintenance control"
                ),
            ):
                probe.owner_revocation(
                    root, "wrong", root / "input", root / "binary", root / "evidence"
                )
            lifecycle.assert_not_called()
            self.assertFalse((root / "evidence").exists())

    def test_barrier_retains_real_generated_frozen_bytes_and_records_exact_probe_authority(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            folder = root / "input"
            (folder / "schema").mkdir(parents=True)
            frozen = folder / "rollouts/one/schema/canonical.surql"
            frozen.parent.mkdir(parents=True)
            generated = (probe.tool.ROOT / probe.tool.GENERATED_SCHEMA).read_bytes()
            frozen.write_bytes(generated)
            (folder / "schema/canonical.surql").write_bytes(generated)
            manifest = folder / "rollouts/one.toml"
            manifest.write_text(
                'id="one"\nname="baseline"\nsource_schema_hash="source"\ntarget_schema_hash="target"\n[[steps]]\nid="apply"\nphase="start"\nkind="apply_files"\n[[steps.files]]\npath="schema/canonical.surql"\nhash="'
                + hashlib.sha256(generated).hexdigest()
                + '"\n'
            )
            config = {
                "instance_id": "selected",
                "namespace": "pse",
                "database": "canonical",
            }
            record: dict[str, object] = {
                "kind": "controls",
                "cleanup": "retained",
                "state": str(root),
                "namespace": "pse",
                "service_generation": "selected",
                "drained": True,
                "directory": str(root),
                "maintenance_selection": {"state": str(root), **config},
            }

            @contextlib.contextmanager
            def update(_resource: str) -> Generator[dict[str, object]]:
                yield record

            with (
                patch.object(probe.maintenance, "selected", return_value=config),
                patch.object(probe.resources, "resource_status", return_value=record),
                patch.object(probe.resources, "resource_update", side_effect=update),
                patch.object(probe.tool, "verify", return_value={"version": "beta6"}),
                patch.object(
                    probe.resources,
                    "borrow",
                    side_effect=RuntimeError(
                        "preparation stops before service effects"
                    ),
                ),
                self.assertRaisesRegex(RuntimeError, "preparation stops"),
            ):
                probe.owner_revocation(
                    root, "owned", folder, root / "binary", root / "evidence"
                )
            value = tomllib.loads(manifest.read_text())
            self.assertEqual(value["steps"][0]["sql"], probe.REVOCATION_SQL)
            self.assertEqual(value["steps"][0]["id"], "owner_revocation_barrier")
            self.assertEqual(frozen.read_bytes(), generated)
            self.assertEqual(
                cast("dict[str, object]", record["maintenance_revocation_probe"])[
                    "input_sha256"
                ],
                probe.maintenance.inputs(folder)[1],
            )
            self.assertEqual(value["source_schema_hash"], "source")
            self.assertEqual(value["target_schema_hash"], "target")


if __name__ == "__main__":
    unittest.main()

# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public supervisor backup/restore controls for the private sealed file tier."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import json
import os
import subprocess
import unittest
import uuid
from pathlib import Path
from unittest.mock import patch

from scripts import payload_files
from scripts import surreal_server as server
from scripts.tests import test_surreal_server as supervisor_controls


class PayloadLifecycleTests(unittest.TestCase):
    def setUp(self) -> None:
        self.fixture = supervisor_controls.SurrealSupervisorTests(methodName="runTest")
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root
        self.state = self.fixture.state
        self.config = self.fixture.initialized()
        self.config.update(interpretation=server.SUBSTRATE_INTERPRETATION, schema_interpretation=server.SUBSTRATE_INTERPRETATION)
        server.write_json(self.state / "config.json", self.config)
        self.key = str(uuid.uuid4())
        self.owner = {
            "key": str(uuid.uuid4()), "issuing_store": "scientific-store",
            "storage_incarnation": self.config["payload_incarnation"],
            "boot": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
            "pid_namespace": os.readlink("/proc/self/ns/pid"), "pid": str(os.getpid()),
            "start": server.native_operation.start_identity(os.getpid()), "drained": False,
        }
        self.instance = {
            "key": self.key, "intent": str(uuid.uuid4()), "owner": self.owner["key"],
            "issuing_store": "scientific-store", "storage_incarnation": self.config["payload_incarnation"],
            "digest": "a" * 64, "payload_bytes": "13", "encoding": "opaque-json-v1",
            "batch_count": "0", "schema": "", "interpretation": server.SUBSTRATE_INTERPRETATION,
            "state": "sealed",
        }
        self.final, self.staging = payload_files.instance_paths(self.state, self.config, self.key)
        self.final.parent.mkdir(parents=True, exist_ok=True)
        self.final.write_bytes(b'{"seed":true}')
        self.instance["payload_bytes"] = str(self.final.stat().st_size)
        self.events: list[str] = []
        self.instances = [self.instance]
        self.owners = [self.owner]

    def query(self, _state: Path, _config: dict[str, object], body: str) -> object:
        self.events.append(body)
        if "LET $rows=SELECT" in body:
            rows = self.owners if "canonical_file_owners" in body else self.instances
            after = json.loads(body.split("key>", 1)[1].split(" ORDER BY", 1)[0])
            return [dict(row) for row in sorted(rows, key=lambda row: row["key"]) if row["key"] > after][:64]
        if "$marker=SELECT" in body:
            return {"payload_incarnation": self.config["payload_incarnation"], "interpretation": self.config["interpretation"]}
        if "drain_owner" in body:
            expected = json.loads(body.split(",", 1)[1].removesuffix(");"))
            for owner in self.owners:
                if owner["key"] == expected["key"]:
                    owner["drained"] = True
        return True

    def backup(self, destination: Path) -> dict[str, object]:
        with (
            patch.object(server, "payload_query", side_effect=self.query),
            patch.object(server, "start", side_effect=lambda *_args, **_kwargs: self.events.append("start-closed")),
            patch.object(server, "stop", side_effect=lambda *_args: self.events.append("stop")),
            patch.object(server, "all_contexts_drained", side_effect=lambda *_args: self.events.append("managed-drain")),
        ):
            return server.backup(self.state, self.config, destination)

    def verifier(self) -> Path:
        path = self.root / "verifier"
        path.write_text('#!/bin/sh\necho \'{"verified_payload_files":1}\'\n')
        path.chmod(0o700)
        self.config["payload_verifier"] = server.payload_verifier(path)
        server.write_json(self.state / "config.json", self.config)
        return path

    def test_setup_selects_fresh_incarnation_and_open_refuses_missing_identity(self) -> None:
        self.assertEqual(str(uuid.UUID(self.config["payload_incarnation"])), self.config["payload_incarnation"])
        old = dict(self.config)
        old.pop("payload_incarnation")
        server.write_json(self.state / "config.json", old)
        with self.assertRaisesRegex(server.SupervisorError, "explicitly regenerate"):
            server.config_for(self.state)
        self.assertNotIn("payload_incarnation", server.read_json(self.state / "config.json"))
        self.assertNotEqual(server.fresh_database_identity(self.config)["payload_incarnation"], self.config["payload_incarnation"])

    def test_live_independent_owner_refuses_before_database_stop_or_completion(self) -> None:
        destination = self.root / "backup"
        with self.assertRaisesRegex(server.SupervisorError, "live independent"):
            self.backup(destination)
        self.assertNotIn("stop", self.events)
        self.assertFalse(destination.exists())
        self.assertFalse(server.config_for(self.state)["accepting_writes"])
        self.assertTrue(any("freeze" in event for event in self.events))
        self.assertLess(next(i for i, event in enumerate(self.events) if "freeze" in event), max(i for i, event in enumerate(self.events) if event == "managed-drain"))

    def test_dead_owner_is_exactly_drained_and_backup_preserves_sealed_and_staging(self) -> None:
        self.owner["boot"] = str(uuid.uuid4())
        self.staging.parent.mkdir(parents=True)
        self.staging.write_bytes(b"unresolved stage")
        self.verifier()
        destination = self.root / "backup"
        self.backup(destination)
        manifest, _original = server.validated_restore_backup(destination, server.SUBSTRATE_INTERPRETATION)
        self.assertTrue(self.owner["drained"])
        self.assertTrue(any("drain_owner" in event for event in self.events))
        self.assertLess(next(i for i, event in enumerate(self.events) if "canonical_file_instances" in event), self.events.index("stop"))
        self.assertEqual((destination / self.final.relative_to(self.state)).read_bytes(), self.final.read_bytes())
        self.assertEqual((destination / self.staging.relative_to(self.state)).read_bytes(), self.staging.read_bytes())
        self.assertEqual(manifest["payload_inventory"][0]["instance"]["state"], "sealed")

    def test_missing_sealed_file_refuses_completion(self) -> None:
        self.owner["drained"] = True
        self.final.unlink()
        with self.assertRaisesRegex(server.SupervisorError, "sealed file is missing"):
            self.backup(self.root / "backup")
        self.assertNotIn("stop", self.events)

    def test_pending_absence_remains_unresolved_without_inventing_file_completion(self) -> None:
        self.owner["drained"] = True
        self.final.unlink()
        self.instance.update(state="pending", digest="", payload_bytes="0")
        destination = self.root / "backup"
        self.backup(destination)
        manifest, _original = server.validated_restore_backup(destination, server.SUBSTRATE_INTERPRETATION)
        entry = manifest["payload_inventory"][0]
        self.assertEqual(entry["instance"]["state"], "pending")
        self.assertEqual(entry["files"], {})

    def test_pending_partial_stage_is_preserved_without_native_complete_file_verification(self) -> None:
        self.owner["drained"] = True
        self.final.unlink()
        self.staging.parent.mkdir(parents=True)
        self.staging.write_bytes(b"partial native file")
        self.instance.update(state="pending", digest="", payload_bytes="0")
        destination = self.root / "backup"
        self.backup(destination)
        manifest, _original = server.validated_restore_backup(destination, server.SUBSTRATE_INTERPRETATION)
        self.assertEqual(manifest["payload_inventory"][0]["instance"]["state"], "pending")
        self.assertEqual((destination / self.staging.relative_to(self.state)).read_bytes(), b"partial native file")

    def test_sealed_backup_requires_native_verifier_and_preserves_incomplete_destination(self) -> None:
        self.owner["drained"] = True
        with self.assertRaisesRegex(server.SupervisorError, "payload-verifier"):
            self.backup(self.root / "backup")
        self.assertNotIn("stop", self.events)
        verifier = self.verifier()
        verifier.write_text("#!/bin/sh\nexit 1\n")
        self.config["payload_verifier"] = server.payload_verifier(verifier)
        server.write_json(self.state / "config.json", self.config)
        with self.assertRaisesRegex(server.SupervisorError, "canonical payload verification failed"):
            self.backup(self.root / "backup")
        self.assertFalse((self.root / "backup/backup.json").exists())
        self.assertEqual(list(self.state.glob(".payload-verification-*")), [])

    def test_payload_corruption_or_omission_refuses_restore_before_target_creation(self) -> None:
        self.owner["drained"] = True
        self.verifier()
        backup = self.root / "backup"
        self.backup(backup)
        copied = backup / self.final.relative_to(self.state)
        copied.write_bytes(b"corrupt")
        with self.assertRaises(server.SupervisorError):
            server.restore(backup, self.root / "restored", server.SUBSTRATE_INTERPRETATION)
        self.assertFalse((self.root / "restored").exists())

    def test_restore_uses_fresh_physical_root_and_preserves_descriptor_bytes(self) -> None:
        self.owner["drained"] = True
        self.verifier()
        backup = self.root / "backup"
        self.backup(backup)
        original_descriptor = json.dumps(self.instance, sort_keys=True)
        restored = self.root / "restored"
        with self.fixture.native_maintenance():
            result = server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        selected = server.config_for(restored)
        final, _pending = payload_files.instance_paths(restored, selected, self.key)
        self.assertEqual(final.read_bytes(), self.final.read_bytes())
        self.assertNotEqual(selected["payload_incarnation"], self.config["payload_incarnation"])
        self.assertNotEqual(selected["database"], self.config["database"])
        self.assertEqual(json.dumps(self.instance, sort_keys=True), original_descriptor)
        self.assertEqual(result["admission"], "validation_required")
        self.assertFalse(selected["accepting_writes"])
        self.assertTrue(selected["payload_frozen"])
        self.assertFalse((restored / self.final.relative_to(self.state)).exists())
        self.assertEqual(server.read_json(backup / "credentials.json"), server.read_json(self.state / "credentials.json"))

    def test_symlink_or_unknown_payload_refuses_inventory(self) -> None:
        self.owner["drained"] = True
        unknown = self.final.parent / "unregistered.sealed"
        unknown.write_bytes(b"preserve unknown")
        with self.assertRaisesRegex(server.SupervisorError, "Unknown payload"):
            self.backup(self.root / "backup")
        self.assertEqual(unknown.read_bytes(), b"preserve unknown")
        unknown.unlink()
        self.final.unlink()
        self.final.symlink_to(self.state / "database/fixture")
        with self.assertRaises(server.SupervisorError):
            self.backup(self.root / "backup")

    def test_foreign_namespace_or_unobservable_owner_remains_protected(self) -> None:
        self.owner["pid_namespace"] = "pid:[unknown]"
        with self.assertRaisesRegex(server.SupervisorError, "unobservable"):
            self.backup(self.root / "backup")
        self.assertFalse(self.owner["drained"])
        self.assertNotIn("stop", self.events)

    def test_dead_managed_leader_with_live_descendants_refuses_backup(self) -> None:
        self.owner.update(start="0", invocation="managed-invocation", unit="pse-owned.scope", group="/pse-owned.scope", group_inode="99")
        with (
            patch.object(server.host_admission, "group_identity", return_value=99),
            patch.object(server, "systemctl", return_value=subprocess.CompletedProcess([], 0, "ControlGroup=/pse-owned.scope\nInvocationID=managed-invocation\n", "")),
            patch.object(server, "group_populated", return_value=True),
            self.assertRaisesRegex(server.SupervisorError, "descendants"),
        ):
            self.backup(self.root / "backup")
        self.assertFalse(self.owner["drained"])
        self.assertNotIn("stop", self.events)

    def test_managed_scope_absence_is_refused_and_exact_removed_group_is_drained(self) -> None:
        self.owner.update(start="0", invocation="managed-invocation")
        with self.assertRaisesRegex(server.SupervisorError, "scope identity"):
            self.backup(self.root / "backup")
        self.owner.update(unit="pse-owned.scope", group="/pse-owned.scope", group_inode="99")
        self.verifier()
        with patch.object(server.host_admission, "group_identity", return_value=None):
            self.backup(self.root / "backup")
        self.assertTrue(self.owner["drained"])

    def test_drained_owner_from_prior_incarnation_remains_historical_after_restore(self) -> None:
        self.owner.update(drained=True, storage_incarnation=str(uuid.uuid4()))
        self.verifier()
        self.backup(self.root / "backup")
        self.assertTrue((self.root / "backup/backup.json").is_file())

    def test_public_owner_recovery_retains_live_unknown_and_reports_exact_death(self) -> None:
        dead = {**self.owner, "key": str(uuid.uuid4()), "boot": str(uuid.uuid4())}
        unknown = {**self.owner, "key": str(uuid.uuid4()), "pid_namespace": "pid:[foreign]"}
        self.owners.extend([dead, unknown])
        args = server.parser().parse_args(["recover-file-owners", "--state", str(self.state)])
        with (
            patch.object(server, "owns_listener", return_value=True),
            patch.object(server, "listener_ready", return_value=True),
            patch.object(server, "payload_query", side_effect=self.query),
            patch("builtins.print") as output,
        ):
            self.assertEqual(server.dispatch(args), 0)
        self.assertEqual(json.loads(output.call_args.args[0]), {"recovered": 1, "live": 1, "unknown": 1})
        self.assertTrue(dead["drained"])
        self.assertFalse(self.owner["drained"])
        self.assertFalse(unknown["drained"])
        self.assertTrue(server.config_for(self.state)["accepting_writes"])
        self.assertFalse(any("freeze" in event or "thaw" in event for event in self.events))

    def test_owner_recovery_refuses_stale_catalog_or_unowned_listener_without_reaping(self) -> None:
        with (
            patch.object(server, "owns_listener", return_value=True),
            patch.object(server, "listener_ready", return_value=True),
            patch.object(server, "payload_query", return_value={"payload_incarnation": str(uuid.uuid4()), "interpretation": server.SUBSTRATE_INTERPRETATION}) as query,
            self.assertRaisesRegex(server.SupervisorError, "current selected"),
        ):
            server.recover_file_owners(self.state)
        self.assertEqual(query.call_count, 1)
        with (
            patch.object(server, "owns_listener", return_value=False),
            patch.object(server, "payload_query") as query,
            self.assertRaisesRegex(server.SupervisorError, "active owned catalog"),
        ):
            server.recover_file_owners(self.state)
        query.assert_not_called()

    def test_partial_initializer_target_cannot_retire_live_file_owner(self) -> None:
        self.config["derived_rebuild_pending"] = {"kind": "rebuild", "phase": "initializing"}
        server.write_json(self.state / "config.json", self.config)
        with (
            patch.object(server, "maintenance_catalog", return_value={"tables": {name: {} for name in ("canonical_file_admission", "canonical_file_owners", "canonical_file_instances")}}),
            patch.object(server, "payload_query", side_effect=self.query),
            patch.object(server, "start"),
            patch.object(server, "all_contexts_drained"),
            self.assertRaisesRegex(server.SupervisorError, "live independent"),
        ):
            server.retire_partial_payload_target(self.state, self.config, {})
        self.assertNotIn("retired_payloads", self.config["derived_rebuild_pending"])
        self.assertTrue(self.final.is_file())

    def test_partial_initializer_retirement_retains_exact_drained_inventory(self) -> None:
        self.owner["boot"] = str(uuid.uuid4())
        self.config["derived_rebuild_pending"] = {"kind": "rebuild", "phase": "initializing"}
        server.write_json(self.state / "config.json", self.config)
        with (
            patch.object(server, "maintenance_catalog", return_value={"tables": {name: {} for name in ("canonical_file_admission", "canonical_file_owners", "canonical_file_instances")}}),
            patch.object(server, "payload_query", side_effect=self.query),
            patch.object(server, "start"),
            patch.object(server, "all_contexts_drained"),
        ):
            server.retire_partial_payload_target(self.state, self.config, {})
        retired = server.config_for(self.state)["derived_rebuild_pending"]["retired_payloads"][0]
        self.assertEqual(retired["owners"], [self.owner])
        self.assertEqual(retired["inventory"][0]["instance"], self.instance)
        self.assertTrue(self.final.is_file())

    def test_replaced_pid_is_death_but_exact_live_pid_is_not(self) -> None:
        self.assertFalse(payload_files.owner_dead(self.owner))
        replaced = {**self.owner, "start": "0"}
        self.assertTrue(payload_files.owner_dead(replaced))


if __name__ == "__main__":
    unittest.main()

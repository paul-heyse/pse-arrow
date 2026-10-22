# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public supervisor backup/restore controls for the private sealed file tier."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import fcntl
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
    def persist_maintenance_payloads(self, config: dict[str, object]) -> None:
        pending = server.object_mapping(config["derived_rebuild_pending"])
        payloads = {
            name: pending.pop(name, [])
            for name in ("payload_inventory", "payload_targets", "retired_payloads")
        }
        with server.lifecycle_reservation(self.state):
            server.write_maintenance_payloads(self.state, config, payloads)

    def setUp(self) -> None:
        self.fixture = supervisor_controls.SurrealSupervisorTests(methodName="runTest")
        self.fixture.setUp()
        self.addCleanup(self.fixture.doCleanups)
        self.root = self.fixture.root
        self.state = self.fixture.state
        self.config = self.fixture.initialized()
        self.config.update(
            interpretation=server.SUBSTRATE_INTERPRETATION,
            schema_interpretation=server.SUBSTRATE_INTERPRETATION,
        )
        server.write_json(self.state / "config.json", self.config)
        self.key = str(uuid.uuid4())
        self.owner = {
            "key": str(uuid.uuid4()),
            "issuing_store": "scientific-store",
            "storage_incarnation": self.config["payload_incarnation"],
            "boot": Path("/proc/sys/kernel/random/boot_id").read_text().strip(),
            "pid_namespace": str(Path("/proc/self/ns/pid").readlink()),
            "pid": str(os.getpid()),
            "start": server.native_operation.start_identity(os.getpid()),
            "drained": False,
        }
        self.instance = {
            "key": self.key,
            "intent": str(uuid.uuid4()),
            "owner": self.owner["key"],
            "issuing_store": "scientific-store",
            "storage_incarnation": self.config["payload_incarnation"],
            "digest": "a" * 64,
            "payload_bytes": "13",
            "encoding": "opaque-json-v1",
            "batch_count": "0",
            "schema": "",
            "interpretation": server.SUBSTRATE_INTERPRETATION,
            "state": "sealed",
        }
        self.final, self.staging = payload_files.instance_paths(
            self.state, self.config, self.key
        )
        self.final.parent.mkdir(parents=True, exist_ok=True)
        self.final.write_bytes(b'{"seed":true}')
        self.instance["payload_bytes"] = str(self.final.stat().st_size)
        self.events: list[str] = []
        self.instances = [self.instance]
        self.owners = [self.owner]
        self.domain: dict[str, object] | None = None

    def query(self, _state: Path, _config: dict[str, object], body: str) -> object:
        self.events.append(body)
        if "fn::pse_maintenance_v1::" in body:
            function = body.split("fn::pse_maintenance_v1::", 1)[1].split("(", 1)[0]
            arguments = json.loads("[" + body.split("(", 1)[1].split(");", 1)[0] + "]")
            self.assertGreater(arguments.pop(0), server.time.time_ns() // 1000)

            def acknowledged(value: object) -> object:
                return {"value": value} if "RETURN {value:" in body else value

            if function == "current":
                self.assertEqual(arguments, [])
                return acknowledged(self.domain)
            generation = server.read_json(self.state / "lifecycle-owner.json")["nonce"]
            if function == "claim":
                operation, actual_generation, phase, source, target, previous = (
                    arguments
                )
                self.assertEqual(
                    operation.split(":")[1:],
                    [
                        "validation",
                        source,
                        target,
                        self.config["namespace"],
                        self.config["database"],
                    ],
                )
                self.assertEqual(actual_generation, generation)
                self.assertEqual(phase, "validation")
                self.assertEqual(source, self.config["payload_incarnation"])
                self.assertEqual(target, source)
                self.assertEqual(previous, self.domain)
                self.domain = dict(
                    zip(
                        ("operation", "generation", "phase"), arguments[:3], strict=True
                    )
                )
                return acknowledged(self.domain)
            self.assertEqual(function, "validation_reads")
            self.assertIsNotNone(self.domain)
            self.assertEqual(arguments[:3], list(self.domain.values()))
            self.assertEqual(arguments[3], server.SUBSTRATE_INTERPRETATION)
            return acknowledged(True)
        if "LET $rows=SELECT" in body:
            rows = self.owners if "canonical_file_owners" in body else self.instances
            after = json.loads(body.split("key>", 1)[1].split(" ORDER BY", 1)[0])
            return [
                dict(row)
                for row in sorted(rows, key=lambda row: row["key"])
                if row["key"] > after
            ][:64]
        if "$marker=SELECT" in body:
            return {
                "payload_incarnation": self.config["payload_incarnation"],
                "interpretation": self.config["interpretation"],
                "study_module_sha256": server.study_module(self.config)["sha256"],
            }
        if "drain_owner" in body:
            expected = json.loads(
                body.split(",", 1)[1].removesuffix(");").replace("<decimal>", "")
            )
            for owner in self.owners:
                if owner["key"] == expected["key"]:
                    owner["drained"] = True
        return True

    def backup(self, destination: Path) -> dict[str, object]:
        with (
            patch.object(server, "payload_query", side_effect=self.query),
            patch.object(
                server,
                "start",
                side_effect=lambda *_args, **_kwargs: self.events.append(
                    "start-closed"
                ),
            ),
            patch.object(
                server, "stop", side_effect=lambda *_args: self.events.append("stop")
            ),
            patch.object(
                server,
                "all_contexts_drained",
                side_effect=lambda *_args: self.events.append("managed-drain"),
            ),
        ):
            return server.backup(self.state, self.config, destination)

    def verifier(self) -> Path:
        path = self.root / "verifier"
        path.write_text("#!/bin/sh\necho '{\"verified_payload_files\":1}'\n")
        path.chmod(0o700)
        self.config["payload_verifier"] = server.payload_verifier(path)
        server.write_json(self.state / "config.json", self.config)
        return path

    def validation_profile(self) -> None:
        self.config.update(
            accepting_writes=False, admission="validation_required", payload_frozen=True
        )
        server.write_json(self.state / "config.json", self.config)
        self.owners = []

    def validator_owner(self, *, live: bool = False) -> None:
        owner = dict(self.owner)
        if not live:
            owner["boot"] = str(uuid.uuid4())
        self.owners = [owner]
        self.events.append("validator")
        selected = server.config_for(self.state)
        self.assertFalse(selected["accepting_writes"])
        self.assertEqual(selected["admission"], "validation_required")
        self.assertTrue(selected["payload_frozen"])

    def validation_controls(self) -> None:
        self.enterContext(server.lifecycle_reservation(self.state))
        for replacement in (
            patch.object(server, "payload_query", side_effect=self.query),
            patch.object(
                server,
                "start",
                side_effect=lambda *_args, **_kwargs: self.events.append(
                    "start-closed"
                ),
            ),
            patch.object(
                server, "stop", side_effect=lambda *_args: self.events.append("stop")
            ),
            patch.object(
                server,
                "all_contexts_drained",
                side_effect=lambda *_args: self.events.append("managed-drain"),
            ),
            patch.object(server, "validate_interpretation"),
            patch.object(server, "public_status", return_value={}),
        ):
            replacement.start()
            self.addCleanup(replacement.stop)

    def test_gated_validation_admits_file_reads_without_writers_or_gc_then_recloses_and_drains(
        self,
    ) -> None:
        self.validation_profile()
        self.validation_controls()

        def validator(*_args: object, **_kwargs: object) -> int:
            self.validator_owner()
            self.assertEqual(_kwargs["stdout"], subprocess.DEVNULL)
            self.assertEqual(_kwargs["stderr"], subprocess.DEVNULL)
            self.assertIsInstance(_kwargs["deadline"], float)
            return 0

        with patch.object(server, "run_file_effect", side_effect=validator):
            server.validate(
                self.state, self.config, server.SUBSTRATE_INTERPRETATION, ["validator"]
            )
        opened = next(
            i
            for i, event in enumerate(self.events)
            if "fn::pse_maintenance_v1::validation_reads(" in event
        )
        frozen = next(
            i
            for i, event in enumerate(self.events)
            if "fn::pse_files_v1::freeze(" in event
        )
        drained = next(
            i for i, event in enumerate(self.events) if "drain_owner(" in event
        )
        self.assertLess(opened, self.events.index("validator"))
        self.assertLess(self.events.index("validator"), frozen)
        self.assertLess(frozen, drained)
        self.assertLess(drained, self.events.index("stop"))
        self.assertIn(server.SUBSTRATE_INTERPRETATION, self.events[opened])
        self.assertIn(str(self.config["payload_incarnation"]), self.events[opened])
        self.assertIn(
            str(server.read_json(self.state / "lifecycle-owner.json")["nonce"]),
            self.events[opened],
        )
        self.assertTrue(self.owners[0]["drained"])
        selected = server.config_for(self.state)
        self.assertFalse(selected["accepting_writes"])
        self.assertTrue(selected["payload_frozen"])
        self.assertEqual(selected["admission"], "quiesced")

    def test_validation_read_mutation_requires_one_exact_generated_ack(self) -> None:
        self.validation_profile()

        def administrative(
            state: Path, query: str, _deadline: float, **_kwargs: object
        ) -> list[dict[str, object]]:
            self.assertTrue(query.startswith("BEGIN; LET $domain="))
            self.assertTrue(query.endswith(" COMMIT;"))
            self.assertEqual(_kwargs["namespace"], self.config["namespace"])
            self.assertEqual(_kwargs["database"], self.config["database"])
            body = query.removeprefix("BEGIN; ").removesuffix(" COMMIT;")
            value = self.query(state, self.config, body)
            return [{"status": "OK", "result": value}]

        with (
            server.lifecycle_reservation(self.state),
            patch.object(server, "administrative_query", side_effect=administrative),
        ):
            server.validation_payload_reads(
                self.state, self.config, server.SUBSTRATE_INTERPRETATION
            )
        selected = server.config_for(self.state)
        self.assertFalse(selected["accepting_writes"])
        self.assertTrue(selected["payload_frozen"])
        self.assertEqual(selected["admission"], "validation_required")

    def test_domain_missing_or_multiple_administrative_results_cannot_admit_reads(
        self,
    ) -> None:
        self.validation_profile()
        before = (self.state / "config.json").read_bytes()
        for results in (
            [{"status": "OK", "result": None}],
            [
                {"status": "OK", "result": {"value": None}},
                {"status": "OK", "result": True},
            ],
        ):
            with (
                self.subTest(results=results),
                server.lifecycle_reservation(self.state),
                patch.object(
                    server, "administrative_query", return_value=results
                ) as query,
                self.assertRaisesRegex(
                    server.SupervisorError, "exactly one acknowledgment"
                ),
            ):
                server.validation_payload_reads(
                    self.state, self.config, server.SUBSTRATE_INTERPRETATION
                )
            query.assert_called_once()
            self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_failed_or_timed_out_validation_recloses_and_authenticates_drain(
        self,
    ) -> None:
        self.validation_profile()
        self.validation_controls()
        for timed_out in (False, True):
            self.validation_profile()
            self.events.clear()

            def validator(
                *_args: object, timed_out: bool = timed_out, **_kwargs: object
            ) -> int:
                self.validator_owner()
                if timed_out:
                    raise subprocess.TimeoutExpired("validator", 120)
                return 1

            expected = (
                subprocess.TimeoutExpired if timed_out else server.SupervisorError
            )
            with (
                self.subTest(timed_out=timed_out),
                patch.object(server, "run_file_effect", side_effect=validator),
                self.assertRaises(expected),
            ):
                server.validate(
                    self.state,
                    self.config,
                    server.SUBSTRATE_INTERPRETATION,
                    ["validator"],
                )
            frozen = next(
                i
                for i, event in enumerate(self.events)
                if "fn::pse_files_v1::freeze(" in event
            )
            drained = next(
                i for i, event in enumerate(self.events) if "drain_owner(" in event
            )
            self.assertLess(frozen, drained)
            self.assertLess(drained, self.events.index("stop"))
            self.assertTrue(self.owners[0]["drained"])
            self.assertEqual(
                server.config_for(self.state)["admission"], "validation_required"
            )

    def test_validator_exit_does_not_bypass_live_unknown_or_managed_descendant_owner(
        self,
    ) -> None:
        self.validation_profile()
        self.validation_controls()
        for case in ("live", "unknown", "managed-descendant"):
            self.validation_profile()
            self.events.clear()

            def validator(*_args: object, case: str = case, **_kwargs: object) -> int:
                self.validator_owner(live=case != "managed-descendant")
                if case == "unknown":
                    self.owners[0]["pid_namespace"] = "pid:[unknown]"
                elif case == "managed-descendant":
                    self.owners[0].update(
                        boot=self.owner["boot"],
                        pid="2147483647",
                        start="original-dead-parent",
                        invocation="owned-managed-invocation",
                    )
                return 0

            with (
                self.subTest(case=case),
                patch.object(server, "run_file_effect", side_effect=validator),
                patch.object(server, "managed_file_owner_drained", return_value=False),
                self.assertRaises(server.SupervisorError),
            ):
                server.validate(
                    self.state,
                    self.config,
                    server.SUBSTRATE_INTERPRETATION,
                    ["validator"],
                )
            self.assertTrue(
                any("fn::pse_files_v1::freeze(" in event for event in self.events)
            )
            self.assertNotIn("stop", self.events)
            self.assertFalse(self.owners[0]["drained"])
            self.assertEqual(
                server.config_for(self.state)["admission"], "validation_required"
            )

    def test_validation_refuses_writer_permission_or_missing_freeze_before_start(
        self,
    ) -> None:
        self.validation_profile()
        for field, value in (("accepting_writes", True), ("payload_frozen", False)):
            selected = {**self.config, field: value}
            with (
                self.subTest(field=field),
                patch.object(server, "start") as start,
                self.assertRaises(server.SupervisorError),
            ):
                server.validate(
                    self.state, selected, server.SUBSTRATE_INTERPRETATION, ["validator"]
                )
            start.assert_not_called()

    def test_unacknowledged_validation_read_opening_recloses_without_dispatch(
        self,
    ) -> None:
        self.validation_profile()
        self.validation_controls()

        def query(state: Path, config: dict[str, object], body: str) -> object:
            value = self.query(state, config, body)
            return (
                {"value": False}
                if "fn::pse_maintenance_v1::validation_reads(" in body
                else value
            )

        with (
            patch.object(server, "payload_query", side_effect=query),
            patch.object(server, "run_file_effect") as child,
            self.assertRaisesRegex(server.SupervisorError, "not acknowledged"),
        ):
            server.validate(
                self.state, self.config, server.SUBSTRATE_INTERPRETATION, ["validator"]
            )
        child.assert_not_called()
        self.assertTrue(
            any("fn::pse_files_v1::freeze(" in event for event in self.events)
        )
        self.assertIn("stop", self.events)
        self.assertEqual(
            server.config_for(self.state)["admission"], "validation_required"
        )

    def test_unacknowledged_validation_read_closure_prevents_stop_or_success(
        self,
    ) -> None:
        self.validation_profile()
        self.validation_controls()

        def query(state: Path, config: dict[str, object], body: str) -> object:
            value = self.query(state, config, body)
            return False if "fn::pse_files_v1::freeze(" in body else value

        with (
            patch.object(server, "payload_query", side_effect=query),
            patch.object(
                server,
                "run_file_effect",
                return_value=0,
            ),
            self.assertRaisesRegex(
                server.SupervisorError, "closure was not acknowledged"
            ),
        ):
            server.validate(
                self.state, self.config, server.SUBSTRATE_INTERPRETATION, ["validator"]
            )
        self.assertNotIn("stop", self.events)
        self.assertFalse(any("canonical_file_owners" in event for event in self.events))
        self.assertEqual(
            server.config_for(self.state)["admission"], "validation_required"
        )

    def test_setup_selects_fresh_incarnation_and_open_refuses_missing_identity(
        self,
    ) -> None:
        incarnation = self.config["payload_incarnation"]
        assert isinstance(incarnation, str)
        self.assertEqual(
            str(uuid.UUID(incarnation)),
            incarnation,
        )
        old = dict(self.config)
        old.pop("payload_incarnation")
        server.write_json(self.state / "config.json", old)
        with self.assertRaisesRegex(server.SupervisorError, "explicitly regenerate"):
            server.config_for(self.state)
        self.assertNotIn(
            "payload_incarnation", server.read_json(self.state / "config.json")
        )
        self.assertNotEqual(
            server.fresh_database_identity(self.config)["payload_incarnation"],
            self.config["payload_incarnation"],
        )

    def test_live_independent_owner_refuses_before_database_stop_or_completion(
        self,
    ) -> None:
        destination = self.root / "backup"
        with self.assertRaisesRegex(server.SupervisorError, "live independent"):
            self.backup(destination)
        self.assertNotIn("stop", self.events)
        self.assertFalse(destination.exists())
        self.assertFalse(server.config_for(self.state)["accepting_writes"])
        self.assertTrue(any("freeze" in event for event in self.events))
        self.assertLess(
            next(i for i, event in enumerate(self.events) if "freeze" in event),
            max(i for i, event in enumerate(self.events) if event == "managed-drain"),
        )

    def test_dead_owner_is_exactly_drained_and_backup_reconciles_exact_staging_alias(
        self,
    ) -> None:
        self.owner["boot"] = str(uuid.uuid4())
        self.staging.parent.mkdir(parents=True)
        self.final.chmod(0o600)
        os.link(self.final, self.staging)
        locks = payload_files.root(self.state, self.config) / ".locks"
        locks.mkdir()
        lock = locks / (self.key + ".lock")
        lock.write_bytes(b"")
        lock.chmod(0o600)
        self.verifier()
        destination = self.root / "backup"
        self.backup(destination)
        _manifest, _original, inventory = server.validated_restore_backup(
            destination, server.SUBSTRATE_INTERPRETATION
        )
        self.assertTrue(self.owner["drained"])
        self.assertTrue(any("drain_owner" in event for event in self.events))
        self.assertLess(
            next(
                i
                for i, event in enumerate(self.events)
                if "canonical_file_instances" in event
            ),
            self.events.index("stop"),
        )
        self.assertEqual(
            (destination / self.final.relative_to(self.state)).read_bytes(),
            self.final.read_bytes(),
        )
        self.assertFalse(self.staging.exists())
        self.assertFalse((destination / self.staging.relative_to(self.state)).exists())
        self.assertEqual(self.final.stat().st_nlink, 1)
        self.assertEqual(
            server.object_mapping(inventory[0]["instance"])["state"], "sealed"
        )

    def test_pending_publication_alias_is_reconciled_without_fabricating_acknowledgment(
        self,
    ) -> None:
        self.owner["boot"] = str(uuid.uuid4())
        self.instance.update(
            state="pending",
            digest="",
            payload_bytes="0",
            encoding="",
            batch_count="0",
            schema="",
        )
        self.final.chmod(0o600)
        self.staging.parent.mkdir()
        os.link(self.final, self.staging)
        locks = payload_files.root(self.state, self.config) / ".locks"
        locks.mkdir()
        lock = locks / (self.key + ".lock")
        lock.write_bytes(b"")
        lock.chmod(0o600)
        lock_inode = lock.stat().st_ino
        backup = self.root / "backup"
        self.backup(backup)
        _manifest, _original, inventory = server.validated_restore_backup(
            backup, server.SUBSTRATE_INTERPRETATION
        )
        self.assertEqual(server.object_mapping(inventory[0]["instance"]), self.instance)
        self.assertEqual(self.instance["state"], "pending")
        self.assertEqual(self.instance["digest"], "")
        self.assertEqual(self.instance["payload_bytes"], "0")
        self.assertFalse(self.staging.exists())
        self.assertEqual(self.final.stat().st_nlink, 1)
        self.assertEqual(lock.stat().st_ino, lock_inode)
        restored = self.root / "restored"
        with self.fixture.native_maintenance():
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        selected = server.config_for(restored)
        final, pending = payload_files.instance_paths(restored, selected, self.key)
        self.assertEqual(final.read_bytes(), self.final.read_bytes())
        self.assertFalse(pending.exists())
        self.assertEqual(
            server.object_mapping(inventory[0]["instance"])["state"], "pending"
        )

    def test_frozen_alias_reconciliation_refuses_unknown_aliases_and_held_locks(
        self,
    ) -> None:
        self.owner["drained"] = True
        self.final.chmod(0o600)
        self.staging.parent.mkdir()
        locks = payload_files.root(self.state, self.config) / ".locks"
        locks.mkdir()
        lock = locks / (self.key + ".lock")
        external = self.root / "external-alias"
        for case in (
            "different-inode",
            "external-alias",
            "symlink",
            "nonprivate",
            "missing-lock",
            "held-lock",
        ):
            with self.subTest(case=case):
                self.final.chmod(0o600)
                os.link(self.final, self.staging)
                if case != "missing-lock":
                    lock.write_bytes(b"")
                    lock.chmod(0o600)
                if case == "different-inode":
                    self.staging.unlink()
                    self.staging.write_bytes(b"different stage")
                    self.staging.chmod(0o600)
                elif case == "external-alias":
                    os.link(self.final, external)
                elif case == "symlink":
                    self.staging.unlink()
                    self.staging.symlink_to(self.final)
                elif case == "nonprivate":
                    self.final.chmod(0o644)
                if case == "held-lock":
                    with lock.open("r+b") as holder:
                        fcntl.flock(holder.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
                        with self.assertRaisesRegex(
                            server.SupervisorError,
                            "staging alias reconciliation refused",
                        ):
                            self.backup(self.root / ("backup-" + case))
                else:
                    with self.assertRaisesRegex(
                        server.SupervisorError, "staging alias reconciliation refused"
                    ):
                        self.backup(self.root / ("backup-" + case))
                self.assertTrue(self.final.exists())
                self.assertTrue(self.staging.exists() or self.staging.is_symlink())
                self.staging.unlink()
                if external.exists():
                    external.unlink()
                if lock.exists():
                    lock.unlink()

    def test_missing_sealed_file_refuses_completion(self) -> None:
        self.owner["drained"] = True
        self.final.unlink()
        with self.assertRaisesRegex(server.SupervisorError, "sealed file is missing"):
            self.backup(self.root / "backup")
        self.assertNotIn("stop", self.events)

    def test_collector_crash_after_unlink_preserves_tombstone_through_backup_restore(
        self,
    ) -> None:
        self.owner["boot"] = str(uuid.uuid4())
        self.instance["state"] = "collecting"
        self.final.unlink()
        backup = self.root / "backup"
        self.backup(backup)
        _manifest, _original, inventory = server.validated_restore_backup(
            backup, server.SUBSTRATE_INTERPRETATION
        )
        entry = inventory[0]
        self.assertEqual(
            server.object_mapping(entry["instance"])["state"], "collecting"
        )
        self.assertEqual(entry["files"], {})
        self.assertTrue(self.owner["drained"])
        restored = self.root / "restored"
        with self.fixture.native_maintenance() as calls:
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        selected = server.config_for(restored)
        final, staging = payload_files.instance_paths(restored, selected, self.key)
        self.assertFalse(final.exists())
        self.assertFalse(staging.exists())
        self.assertEqual(
            server.object_mapping(
                server.object_inventory(
                    server.read_json(backup / "backup.json")["payload_inventory"]
                )[0]["instance"]
            )["state"],
            "collecting",
        )
        self.assertTrue(selected["payload_frozen"])
        self.assertFalse(selected["accepting_writes"])
        self.assertFalse(
            any("SET state=" in query for _namespace, _database, query in calls)
        )

    def test_collecting_bytes_still_present_are_verified_and_copied(self) -> None:
        self.owner["drained"] = True
        self.instance["state"] = "collecting"
        with self.assertRaisesRegex(server.SupervisorError, "payload-verifier"):
            self.backup(self.root / "backup")
        self.verifier()
        backup = self.root / "backup"
        self.backup(backup)
        _manifest, _original, inventory = server.validated_restore_backup(
            backup, server.SUBSTRATE_INTERPRETATION
        )
        self.assertEqual(
            server.object_mapping(inventory[0]["instance"])["state"], "collecting"
        )
        self.assertEqual(
            (backup / self.final.relative_to(self.state)).read_bytes(),
            self.final.read_bytes(),
        )

    def test_abandoned_unsealed_collector_crash_preserves_unknown_bytes_through_restore(
        self,
    ) -> None:
        self.owner["boot"] = str(uuid.uuid4())
        # Reservation -> authentic abandonment -> collecting changes state only.
        # The collector dies before unlink, without any scientific seal ACK.
        self.instance.update(
            state="collecting",
            digest="",
            payload_bytes="0",
            encoding="",
            batch_count="0",
            schema="",
        )
        original = dict(self.instance)
        expected = self.final.read_bytes()
        backup = self.root / "backup"
        with patch.object(
            server,
            "payload_verifier",
            side_effect=AssertionError(
                "unsealed bytes must not fabricate canonical verification"
            ),
        ):
            self.backup(backup)
            _manifest, _original, inventory = server.validated_restore_backup(
                backup, server.SUBSTRATE_INTERPRETATION
            )
            self.assertEqual(server.object_mapping(inventory[0]["instance"]), original)
            self.assertEqual(
                (backup / self.final.relative_to(self.state)).read_bytes(), expected
            )
            self.assertTrue(self.owner["drained"])
            restored = self.root / "restored"
            with self.fixture.native_maintenance() as calls:
                server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        selected = server.config_for(restored)
        final, staging = payload_files.instance_paths(restored, selected, self.key)
        self.assertEqual(final.read_bytes(), expected)
        self.assertFalse(staging.exists())
        self.assertTrue(selected["payload_frozen"])
        self.assertFalse(selected["accepting_writes"])
        self.assertEqual(
            server.object_mapping(inventory[0]["instance"])["state"], "collecting"
        )
        self.assertFalse(
            any("SET state=" in query for _namespace, _database, query in calls)
        )

    def test_collecting_partial_descriptor_cannot_claim_unsealed_exemption(
        self,
    ) -> None:
        self.owner["drained"] = True
        self.instance.update(
            state="collecting",
            digest="",
            payload_bytes="0",
            encoding="",
            batch_count="0",
            schema="",
        )
        for field, value in (
            ("digest", "a" * 64),
            ("encoding", "opaque-json-v1"),
            ("batch_count", "1"),
            ("schema", "nonempty"),
        ):
            with self.subTest(field=field):
                self.instance[field] = value
                with self.assertRaisesRegex(server.SupervisorError, "extent differs"):
                    self.backup(self.root / ("backup-" + field))
                self.instance[field] = "0" if field == "batch_count" else ""

    def test_pending_absence_remains_unresolved_without_inventing_file_completion(
        self,
    ) -> None:
        self.owner["drained"] = True
        self.final.unlink()
        self.instance.update(state="pending", digest="", payload_bytes="0")
        destination = self.root / "backup"
        self.backup(destination)
        _manifest, _original, inventory = server.validated_restore_backup(
            destination, server.SUBSTRATE_INTERPRETATION
        )
        entry = inventory[0]
        self.assertEqual(server.object_mapping(entry["instance"])["state"], "pending")
        self.assertEqual(entry["files"], {})

    def test_pending_partial_stage_is_preserved_without_native_complete_file_verification(
        self,
    ) -> None:
        self.owner["drained"] = True
        self.final.unlink()
        self.staging.parent.mkdir(parents=True)
        self.staging.write_bytes(b"partial native file")
        self.instance.update(state="pending", digest="", payload_bytes="0")
        destination = self.root / "backup"
        self.backup(destination)
        _manifest, _original, inventory = server.validated_restore_backup(
            destination, server.SUBSTRATE_INTERPRETATION
        )
        self.assertEqual(
            server.object_mapping(inventory[0]["instance"])["state"], "pending"
        )
        self.assertEqual(
            (destination / self.staging.relative_to(self.state)).read_bytes(),
            b"partial native file",
        )

    def test_sealed_backup_requires_native_verifier_and_preserves_incomplete_destination(
        self,
    ) -> None:
        self.owner["drained"] = True
        with self.assertRaisesRegex(server.SupervisorError, "payload-verifier"):
            self.backup(self.root / "backup")
        self.assertNotIn("stop", self.events)
        verifier = self.verifier()
        verifier.write_text("#!/bin/sh\nexit 1\n")
        self.config["payload_verifier"] = server.payload_verifier(verifier)
        server.write_json(self.state / "config.json", self.config)
        with self.assertRaisesRegex(
            server.SupervisorError, "canonical payload verification failed"
        ):
            self.backup(self.root / "backup")
        self.assertFalse((self.root / "backup/backup.json").exists())
        self.assertEqual(list(self.state.glob(".payload-verification-*")), [])

    def test_payload_corruption_or_omission_refuses_restore_before_target_creation(
        self,
    ) -> None:
        self.owner["drained"] = True
        self.verifier()
        backup = self.root / "backup"
        self.backup(backup)
        copied = backup / self.final.relative_to(self.state)
        copied.write_bytes(b"corrupt")
        with self.assertRaises(server.SupervisorError):
            server.restore(
                backup, self.root / "restored", server.SUBSTRATE_INTERPRETATION
            )
        self.assertFalse((self.root / "restored").exists())

    def test_restore_uses_fresh_physical_root_and_preserves_descriptor_bytes(
        self,
    ) -> None:
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
        self.assertNotEqual(
            selected["payload_incarnation"], self.config["payload_incarnation"]
        )
        self.assertNotEqual(selected["database"], self.config["database"])
        self.assertEqual(json.dumps(self.instance, sort_keys=True), original_descriptor)
        self.assertEqual(result["admission"], "validation_required")
        self.assertFalse(selected["accepting_writes"])
        self.assertTrue(selected["payload_frozen"])
        self.assertFalse((restored / self.final.relative_to(self.state)).exists())
        self.assertEqual(
            server.read_json(backup / "credentials.json"),
            server.read_json(self.state / "credentials.json"),
        )

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
        self.owner.update(
            start="0",
            invocation="managed-invocation",
            unit="pse-owned.scope",
            group="/pse-owned.scope",
            group_inode="99",
        )
        with (
            patch.object(server.host_admission, "group_identity", return_value=99),
            patch.object(
                server,
                "systemctl",
                return_value=subprocess.CompletedProcess(
                    [],
                    0,
                    "ControlGroup=/pse-owned.scope\nInvocationID=managed-invocation\n",
                    "",
                ),
            ),
            patch.object(server, "group_populated", return_value=True),
            self.assertRaisesRegex(server.SupervisorError, "descendants"),
        ):
            self.backup(self.root / "backup")
        self.assertFalse(self.owner["drained"])
        self.assertNotIn("stop", self.events)

    def test_managed_scope_absence_is_refused_and_exact_removed_group_is_drained(
        self,
    ) -> None:
        self.owner.update(start="0", invocation="managed-invocation")
        with self.assertRaisesRegex(server.SupervisorError, "scope identity"):
            self.backup(self.root / "backup")
        self.owner.update(
            unit="pse-owned.scope", group="/pse-owned.scope", group_inode="99"
        )
        self.verifier()
        with patch.object(server.host_admission, "group_identity", return_value=None):
            self.backup(self.root / "backup")
        self.assertTrue(self.owner["drained"])

    def test_drained_owner_from_prior_incarnation_remains_historical_after_restore(
        self,
    ) -> None:
        self.owner.update(drained=True, storage_incarnation=str(uuid.uuid4()))
        self.verifier()
        self.backup(self.root / "backup")
        self.assertTrue((self.root / "backup/backup.json").is_file())

    def test_public_owner_recovery_retains_live_unknown_and_reports_exact_death(
        self,
    ) -> None:
        dead = {**self.owner, "key": str(uuid.uuid4()), "boot": str(uuid.uuid4())}
        unknown = {
            **self.owner,
            "key": str(uuid.uuid4()),
            "pid_namespace": "pid:[foreign]",
        }
        self.owners.extend([dead, unknown])
        args = server.parser().parse_args(
            ["recover-file-owners", "--state", str(self.state)]
        )
        with (
            patch.object(server, "owns_listener", return_value=True),
            patch.object(server, "listener_ready", return_value=True),
            patch.object(server, "payload_query", side_effect=self.query),
            patch("builtins.print") as output,
        ):
            self.assertEqual(server.dispatch(args), 0)
        self.assertEqual(
            json.loads(output.call_args.args[0]),
            {"recovered": 1, "live": 1, "unknown": 1},
        )
        self.assertTrue(dead["drained"])
        self.assertFalse(self.owner["drained"])
        self.assertFalse(unknown["drained"])
        self.assertTrue(server.config_for(self.state)["accepting_writes"])
        self.assertFalse(
            any("freeze" in event or "thaw" in event for event in self.events)
        )

    def test_native_owner_projection_null_optionals_recover_as_exact_absent_fields(
        self,
    ) -> None:
        expected = {**self.owner, "boot": str(uuid.uuid4())}
        projected = {
            **expected,
            "group": None,
            "group_inode": None,
            "invocation": None,
            "unit": None,
        }
        acknowledgments = []

        def query(_state: Path, _config: dict[str, object], body: str) -> object:
            if "LET $rows=SELECT" in body:
                after = json.loads(body.split("key>", 1)[1].split(" ORDER BY", 1)[0])
                return [dict(projected)] if not after else []
            self.assertIn("drain_owner", body)
            self.assertIn('"pid":<decimal>', body)
            original = json.loads(
                body.split(",", 1)[1].removesuffix(");").replace("<decimal>", "")
            )
            acknowledgments.append(original)
            return True

        with patch.object(server, "payload_query", side_effect=query):
            counts = server.reap_payload_owners(
                self.state, self.config, require_all=False
            )
        self.assertEqual(counts, {"recovered": 1, "live": 0, "unknown": 0})
        self.assertEqual(acknowledgments, [expected])
        self.assertEqual(projected["group_inode"], None)

    def test_owner_projection_normalizes_only_declared_optional_null_fields(
        self,
    ) -> None:
        projected = {
            **self.owner,
            "group_inode": None,
            "unknown_field": None,
            "issuing_store": None,
        }
        with patch.object(server, "payload_query", side_effect=[[projected], []]):
            pages = list(
                server.payload_pages(self.state, self.config, "canonical_file_owners")
            )
        self.assertNotIn("group_inode", pages[0][0])
        self.assertIn("unknown_field", pages[0][0])
        self.assertIsNone(pages[0][0]["unknown_field"])
        self.assertIn("issuing_store", pages[0][0])
        self.assertIsNone(pages[0][0]["issuing_store"])

    def test_owner_recovery_refuses_stale_catalog_or_unowned_listener_without_reaping(
        self,
    ) -> None:
        with (
            patch.object(server, "owns_listener", return_value=True),
            patch.object(server, "listener_ready", return_value=True),
            patch.object(
                server,
                "payload_query",
                return_value={
                    "payload_incarnation": str(uuid.uuid4()),
                    "interpretation": server.SUBSTRATE_INTERPRETATION,
                },
            ) as query,
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
        self.config["derived_rebuild_pending"] = {
            "kind": "rebuild",
            "phase": "initializing",
        }
        server.write_json(self.state / "config.json", self.config)
        with (
            patch.object(
                server,
                "maintenance_catalog",
                return_value={
                    "tables": {
                        name: {}
                        for name in (
                            "canonical_file_admission",
                            "canonical_file_owners",
                            "canonical_file_instances",
                        )
                    }
                },
            ),
            patch.object(server, "payload_query", side_effect=self.query),
            patch.object(server, "start"),
            patch.object(server, "all_contexts_drained"),
            self.assertRaisesRegex(server.SupervisorError, "live independent"),
        ):
            server.retire_partial_payload_target(self.state, self.config, {})
        self.assertNotIn("retired_payloads", self.config["derived_rebuild_pending"])
        self.assertTrue(self.final.is_file())

    def test_partial_initializer_retirement_retains_exact_drained_inventory(
        self,
    ) -> None:
        self.owner["boot"] = str(uuid.uuid4())
        self.config["derived_rebuild_pending"] = {
            "kind": "rebuild",
            "phase": "initializing",
        }
        server.write_json(self.state / "config.json", self.config)
        with (
            patch.object(
                server,
                "maintenance_catalog",
                return_value={
                    "tables": {
                        name: {}
                        for name in (
                            "canonical_file_admission",
                            "canonical_file_owners",
                            "canonical_file_instances",
                        )
                    }
                },
            ),
            patch.object(server, "payload_query", side_effect=self.query),
            patch.object(server, "start"),
            patch.object(server, "all_contexts_drained"),
            server.lifecycle_reservation(self.state),
        ):
            server.retire_partial_payload_target(self.state, self.config, {})
        pending = server.object_mapping(
            server.config_for(self.state)["derived_rebuild_pending"]
        )
        retired = server.object_inventory(
            server.maintenance_payloads(self.state, pending)["retired_payloads"]
        )[0]
        self.assertEqual(retired["owners"], [self.owner])
        self.assertEqual(
            server.object_inventory(retired["inventory"])[0]["instance"], self.instance
        )
        self.assertTrue(self.final.is_file())

    def test_catalog_instance_locks_are_preserved_live_and_excluded_from_backup(
        self,
    ) -> None:
        self.owner["drained"] = True
        collected = {**self.instance, "key": str(uuid.uuid4()), "state": "collected"}
        self.instances.append(collected)
        locks = payload_files.root(self.state, self.config) / ".locks"
        locks.mkdir()
        identities = {}
        for instance in self.instances:
            lock = locks / (instance["key"] + ".lock")
            lock.write_bytes(b"")
            lock.chmod(0o600)
            identities[lock] = lock.stat().st_ino
        self.verifier()
        backup = self.root / "backup"
        self.backup(backup)
        _manifest, _original, inventory = server.validated_restore_backup(
            backup, server.SUBSTRATE_INTERPRETATION
        )
        self.assertFalse(
            any(
                "/.locks/" in name for name in server.object_mapping(_manifest["files"])
            )
        )
        self.assertEqual({lock: lock.stat().st_ino for lock in identities}, identities)
        self.assertEqual(
            {server.object_mapping(entry["instance"])["state"] for entry in inventory},
            {"sealed", "collected"},
        )
        restored = self.root / "restored"
        with self.fixture.native_maintenance():
            server.restore(backup, restored, server.SUBSTRATE_INTERPRETATION)
        selected = server.config_for(restored)
        self.assertNotEqual(
            selected["payload_incarnation"], self.config["payload_incarnation"]
        )
        self.assertFalse((payload_files.root(restored, selected) / ".locks").exists())
        self.assertEqual({lock: lock.stat().st_ino for lock in identities}, identities)

    def test_unknown_malformed_and_nonprivate_instance_locks_refuse_backup(
        self,
    ) -> None:
        self.owner["drained"] = True
        locks = payload_files.root(self.state, self.config) / ".locks"
        locks.mkdir()
        for case in (
            "unknown",
            "malformed",
            "contents",
            "permissions",
            "symlink",
            "directory",
            "fifo",
        ):
            with self.subTest(case=case):
                name = (str(uuid.uuid4()) if case == "unknown" else self.key) + ".lock"
                if case == "malformed":
                    name = "malformed.lock"
                lock = locks / name
                if case == "symlink":
                    lock.symlink_to(self.final)
                elif case == "directory":
                    lock.mkdir()
                elif case == "fifo":
                    os.mkfifo(lock, 0o600)
                else:
                    lock.write_bytes(b"unexpected bytes" if case == "contents" else b"")
                    lock.chmod(0o644 if case == "permissions" else 0o600)
                with self.assertRaisesRegex(
                    server.SupervisorError, "synchronization lock inventory"
                ):
                    self.backup(self.root / ("backup-" + case))
                self.assertTrue(lock.exists() or lock.is_symlink())
                self.assertTrue(self.final.is_file())
                if case == "directory":
                    lock.rmdir()
                else:
                    lock.unlink()

    def test_still_held_instance_lock_refuses_backup_after_catalog_owner_drain(
        self,
    ) -> None:
        self.owner["drained"] = True
        locks = payload_files.root(self.state, self.config) / ".locks"
        locks.mkdir()
        lock = locks / (self.key + ".lock")
        lock.write_bytes(b"")
        lock.chmod(0o600)
        with lock.open("r+b") as holder:
            fcntl.flock(holder.fileno(), fcntl.LOCK_EX | fcntl.LOCK_NB)
            with self.assertRaisesRegex(
                server.SupervisorError, "synchronization lock inventory"
            ):
                self.backup(self.root / "backup")
        self.assertTrue(lock.exists())
        self.assertTrue(self.final.exists())

    def test_whole_old_root_cleanup_removes_only_known_catalog_locks(self) -> None:
        self.owner["drained"] = True
        locks = payload_files.root(self.state, self.config) / ".locks"
        locks.mkdir()
        known = locks / (self.key + ".lock")
        known.write_bytes(b"")
        known.chmod(0o600)
        self.verifier()
        backup = self.root / "backup"
        self.backup(backup)
        _manifest, _original, inventory = server.validated_restore_backup(
            backup, server.SUBSTRATE_INTERPRETATION
        )
        selected = server.fresh_database_identity(self.config)
        selected["derived_rebuild_pending"] = {
            "kind": "rebuild",
            "phase": "disposing",
            "namespace": self.config["namespace"],
            "database": self.config["database"],
            "payload_incarnation": self.config["payload_incarnation"],
            "payload_inventory": inventory,
        }
        self.persist_maintenance_payloads(selected)
        unknown = locks / (str(uuid.uuid4()) + ".lock")
        unknown.write_bytes(b"")
        unknown.chmod(0o600)
        with (
            patch.object(server, "database_present", return_value=True),
            patch.object(server, "maintenance_query", return_value={"value": True}),
            self.assertRaisesRegex(
                server.SupervisorError, "synchronization locks retained"
            ),
        ):
            server.cleanup_retired_payloads(self.state, selected, {})
        self.assertTrue(known.exists())
        self.assertTrue(unknown.exists())
        self.assertTrue(self.final.exists())
        unknown.unlink()
        with (
            patch.object(server, "database_present", return_value=True),
            patch.object(server, "maintenance_query", return_value={"value": True}),
        ):
            server.cleanup_retired_payloads(self.state, selected, {})
        self.assertFalse(payload_files.root(self.state, self.config).exists())
        self.assertTrue(
            server.object_mapping(
                server.config_for(self.state)["derived_rebuild_pending"]
            )["payload_cleanup_done"]
        )

    def test_restore_rejects_malformed_last_inventory_entry_before_receiver_effects(
        self,
    ) -> None:
        self.owner["drained"] = True
        self.verifier()
        backup = self.root / "backup"
        self.backup(backup)
        manifest = server.read_json(backup / "backup.json")
        inventory = server.object_inventory(manifest["payload_inventory"])
        receiver = self.root / "malformed-receiver"
        invalid_values: tuple[object, ...] = (None, {}, "invalid")
        for invalid in invalid_values:
            manifest["payload_inventory"] = [*inventory, invalid]
            server.write_json(backup / "backup.json", manifest)
            with (
                self.subTest(invalid=invalid),
                patch.object(server, "lifecycle_reservation") as reservation,
                patch.object(server.shutil, "copy2") as copy,
                self.assertRaises(server.SupervisorError),
            ):
                server.restore(backup, receiver, server.SUBSTRATE_INTERPRETATION)
            reservation.assert_not_called()
            copy.assert_not_called()
            self.assertFalse(receiver.exists())
            self.assertEqual(
                (backup / self.final.relative_to(self.state)).read_bytes(),
                self.final.read_bytes(),
            )

    def test_cleanup_preflights_every_placement_before_any_deletion(self) -> None:
        digest, extent = payload_files.checked_file(self.final)
        entry: dict[str, object] = {
            "context": ".",
            "database": self.config["database"],
            "payload_incarnation": self.config["payload_incarnation"],
            "instance": dict(self.instance),
            "files": {
                str(self.final.relative_to(self.state)): {
                    "sha256": digest,
                    "bytes": extent,
                }
            },
        }
        for case in (
            "target-member",
            "retired-member",
            "inventory-member",
            "instance-object",
            "files-object",
            "path-prefix",
            "unknown-file",
            "symlink",
            "held-lock",
        ):
            selected = server.fresh_database_identity(self.config)
            retired = server.fresh_database_identity(self.config)
            retired_root = payload_files.root(self.state, retired)
            retired_root.mkdir(parents=True)
            pending: dict[str, object] = {
                "kind": "rebuild",
                "namespace": self.config["namespace"],
                "database": self.config["database"],
                "payload_incarnation": self.config["payload_incarnation"],
                "payload_inventory": [entry],
            }
            retirement: dict[str, object] = {
                "payload_incarnation": retired["payload_incarnation"],
                "database": retired["database"],
                "owners": [{"drained": True}],
                "inventory": [entry],
            }
            pending["retired_payloads"] = [retirement]
            if case == "target-member":
                pending["payload_targets"] = [retired, None]
            elif case == "retired-member":
                pending["retired_payloads"] = [retirement, None]
            elif case == "inventory-member":
                retirement["inventory"] = [entry, None]
            elif case == "instance-object":
                retirement["inventory"] = [entry, {"instance": None, "files": {}}]
            elif case == "files-object":
                retirement["inventory"] = [entry, {**entry, "files": None}]
            elif case == "path-prefix":
                retirement["inventory"] = [
                    entry,
                    {**entry, "files": {"wrong/" + self.final.name: {}}},
                ]
            elif case == "unknown-file":
                (retired_root / "unknown").write_bytes(b"retained")
            elif case == "symlink":
                (retired_root / "unknown").symlink_to(self.final)
            selected["derived_rebuild_pending"] = pending
            self.persist_maintenance_payloads(selected)
            before = (self.state / "config.json").read_bytes()
            if case == "held-lock":
                locks = retired_root / ".locks"
                locks.mkdir()
                lock = locks / (self.key + ".lock")
                lock.write_bytes(b"")
                lock.chmod(0o600)
                descriptor = os.open(lock, os.O_RDONLY)
                fcntl.flock(descriptor, fcntl.LOCK_EX | fcntl.LOCK_NB)
            else:
                descriptor = None
            try:
                with (
                    self.subTest(case=case),
                    patch.object(server, "database_present", return_value=True),
                    patch.object(
                        server, "maintenance_query", return_value={"value": True}
                    ),
                    self.assertRaisesRegex(
                        server.SupervisorError,
                        {
                            "target-member": "metadata object",
                            "retired-member": "metadata object",
                            "inventory-member": "metadata object",
                            "instance-object": "metadata object",
                            "files-object": "metadata object",
                            "path-prefix": "invalid path",
                            "unknown-file": "Unknown retired payload content",
                            "symlink": "must not traverse symlinks",
                            "held-lock": "synchronization locks retained",
                        }[case],
                    ),
                ):
                    server.cleanup_retired_payloads(self.state, selected, {})
            finally:
                if descriptor is not None:
                    os.close(descriptor)
            self.assertEqual(self.final.read_bytes(), b'{"seed":true}')
            self.assertTrue(retired_root.exists())
            self.assertEqual((self.state / "config.json").read_bytes(), before)

    def test_replaced_pid_is_death_but_exact_live_pid_is_not(self) -> None:
        self.assertFalse(payload_files.owner_dead(self.owner))
        replaced = {**self.owner, "start": "0"}
        self.assertTrue(payload_files.owner_dead(replaced))


if __name__ == "__main__":
    unittest.main()

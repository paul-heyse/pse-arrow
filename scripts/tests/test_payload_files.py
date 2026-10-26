# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Public supervisor backup/restore controls for the private sealed file tier."""
# ruff: noqa: PT009, PT027

from __future__ import annotations

import io
import json
import os
import subprocess
import tempfile
import unittest
import uuid
from pathlib import Path
from unittest.mock import patch

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
        directory = (
            self.state
            / "payloads"
            / str(self.config["payload_incarnation"])
            / str(self.config["database"])
        )
        self.final = directory / "instances" / (self.key + ".sealed")
        self.staging = directory / ".pending" / (self.key + ".pending")
        self.final.parent.mkdir(parents=True, exist_ok=True)
        self.final.write_bytes(b'{"seed":true}')
        self.instance["payload_bytes"] = str(self.final.stat().st_size)
        self.events: list[str] = []
        self.instances = [self.instance]
        self.owners = [self.owner]
        self.domain: dict[str, object] | None = None
        # These supervisor controls supply factual native responses. Native
        # offline tests independently exercise actual bytes, inodes and locks.
        self.live_boot = self.owner["boot"]
        self.live_namespace = self.owner["pid_namespace"]
        self.live_start = self.owner["start"]
        self.enterContext(
            patch.object(
                server, "native_payload_operation", side_effect=self.native_facts
            )
        )

    def native_facts(
        self,
        state: Path,
        _config: object,
        operation: str,
        inventory: object = None,
        **options: object,
    ) -> list[dict[str, object]]:
        if operation == "owner-observations":
            observations = []
            for owner in server.object_inventory(inventory):
                if owner["boot"] != self.live_boot:
                    observations.append({"key": owner["key"], "dead": True})
                elif owner["pid_namespace"] != self.live_namespace:
                    observations.append(
                        {"key": owner["key"], "refusal": "unobservable fixture"}
                    )
                else:
                    observations.append(
                        {"key": owner["key"], "dead": owner["start"] != self.live_start}
                    )
            return observations
        if operation == "layout":
            placement = server.object_mapping(options["placement"])
            return [
                {
                    "root": str(
                        state
                        / "payloads"
                        / str(placement["payload_incarnation"])
                        / str(placement["database"])
                    )
                }
            ]
        return server.object_inventory(inventory or [])

    def query(self, _state: Path, _config: dict[str, object], body: str) -> object:
        self.events.append(body)
        if "fn::pse_maintenance_v1::" in body:
            function = body.split("fn::pse_maintenance_v1::", 1)[1].split("(", 1)[0]
            arguments_source = body.split("(", 1)[1].split(");", 1)[0]
            # Only claim's final option<object> uses Surreal NONE. Quoted
            # source strings remain original JSON bytes.
            if function == "claim" and arguments_source.endswith(",NONE"):
                arguments_source = arguments_source.removesuffix(",NONE") + ",null"
            arguments = json.loads("[" + arguments_source + "]")
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
            # This unit fixture has file rows, but no application database.
            # Application closure capture is exercised by supervisor controls.
            patch.object(
                server,
                "capture_application_inventory",
                side_effect=self.fixture.application_inventory_fixture,
            ),
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
        # The native operation is supplied above; this is an explicit fixture artifact.
        path = self.root / "verifier"
        path.write_text("fixture-native-artifact")
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


class NativePayloadProtocolTests(unittest.TestCase):
    def test_native_entry_is_bounded_before_dispatch(self) -> None:
        config = {"payload_verifier": {"binary": "/fixture/helper", "sha256": "a" * 64}}
        with (
            patch.object(
                server, "payload_verifier", return_value=config["payload_verifier"]
            ),
            patch.object(server.subprocess, "run") as dispatch,
            self.assertRaisesRegex(server.SupervisorError, "bounded envelope"),
        ):
            server.native_payload_operation(
                Path("/fixture/state"),
                config,
                "inventory",
                [{"instance": {"large": "x" * (64 * 1024)}}],
            )
        dispatch.assert_not_called()

    def test_native_reply_bound_and_malformed_reply_refuse(self) -> None:
        config = {"payload_verifier": {"binary": "/fixture/helper", "sha256": "a" * 64}}
        for response in (b"x" * (64 * 1024 + 1), b"not json\n"):

            def dispatch(
                arguments: list[str], *, reply: bytes = response, **options: object
            ) -> subprocess.CompletedProcess[bytes]:
                output = options["stdout"]
                assert isinstance(output, io.BufferedIOBase)
                output.write(reply)
                return subprocess.CompletedProcess(arguments, 0, b"", b"")

            with (
                self.subTest(bytes=len(response)),
                patch.object(
                    server, "payload_verifier", return_value=config["payload_verifier"]
                ),
                patch.object(server.subprocess, "run", side_effect=dispatch),
                self.assertRaises(server.SupervisorError),
            ):
                server.native_payload_operation(
                    Path("/fixture/state"), config, "verify", []
                )

    def test_success_without_the_exact_inventory_acknowledgment_refuses(self) -> None:
        config = {"payload_verifier": {"binary": "/fixture/helper", "sha256": "a" * 64}}
        with (
            patch.object(
                server, "payload_verifier", return_value=config["payload_verifier"]
            ),
            patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0, b"", b""),
            ),
            self.assertRaisesRegex(
                server.SupervisorError, "omitted an inventory acknowledgment"
            ),
        ):
            server.native_payload_operation(
                Path("/fixture/state"), config, "verify", [{"original": "inventory"}]
            )

    def test_default_already_built_helper_is_witnessed_for_published_supervisor(
        self,
    ) -> None:
        config: dict[str, object] = {}
        binary = (
            Path(server.__file__).resolve().parents[1]
            / "target/debug/pse-canonical-recovery"
        )
        receipt = {"binary": str(binary), "sha256": "b" * 64}
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(server, "payload_verifier", return_value=receipt),
            patch.object(
                server.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0, b"", b""),
            ),
        ):
            server.native_payload_operation(Path(directory), config, "verify", [])
        self.assertEqual(config["payload_verifier"], receipt)


if __name__ == "__main__":
    unittest.main()

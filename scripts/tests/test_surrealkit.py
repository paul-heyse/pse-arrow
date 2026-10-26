# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Namespace ownership and restricted beta dispatch controls."""

# ruff: noqa: PT009, PT027
from __future__ import annotations


import contextlib
import json
import tempfile
import unittest
from pathlib import Path
from typing import cast
from unittest.mock import patch

from scripts import service_borrowers
from scripts import surreal_server as server
from scripts import surrealkit as tool
from scripts import test_resources as resources
from scripts.tests.test_surrealism_module import fixture_module


class NamespaceTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.root = Path(temporary.name)
        self.state = self.root / "state"
        self.state.mkdir()
        self.config: dict[str, object] = {
            "instance_id": "fixture-generation",
            "namespace": "pse",
            "admission": "open",
            "accepting_writes": True,
        }
        for replacement in (
            patch.object(service_borrowers, "registry", return_value=self.root / "registry"),
            patch.object(server, "config_for", return_value=self.config),
            patch.object(resources.host, "inherit", return_value=None),
            patch.dict("os.environ", {resources.MARKER: ""}),
        ):
            replacement.start()
            self.addCleanup(replacement.stop)

    def register(self, digit: str = "a") -> str:
        return resources.register(
            {
                "state": str(self.state),
                "kind": "namespace",
                "namespace": "canonical_test_ns_" + digit * 32,
            }
        )

    def test_pending_rollout_blocks_unrelated_recovery_before_effects(self) -> None:
        self.config["surrealkit_rollout_pending"] = {"nonce": "unresolved"}
        with (
            patch.object(
                server, "lifecycle_reservation", return_value=contextlib.nullcontext()
            ),
            patch.object(server, "close_context_admission") as close,
            patch.object(server, "start") as start,
            self.assertRaisesRegex(server.SupervisorError, "operation settlement"),
        ):
            server.qualify_recovery(self.state)
        close.assert_not_called()
        start.assert_not_called()

    def test_pending_rollout_blocks_admission_open(self) -> None:
        self.config["surrealkit_rollout_pending"] = {"nonce": "unresolved"}
        with (
            patch.object(server, "managed_contexts", return_value=[self.state]),
            patch.object(server, "write_json") as publish,
            self.assertRaisesRegex(
                server.SupervisorError, "maintenance remains closed"
            ),
        ):
            server.open_context_admission(self.state)
        publish.assert_not_called()

    def test_enclosing_identity_survives_and_cannot_be_replaced(self) -> None:
        first, second = self.register(), self.register("b")
        self.assertNotEqual(first, second)
        record = resources.resource_status(first)
        self.assertEqual(record["service_namespace"], "pse")
        self.assertEqual(record["namespace"], "canonical_test_ns_" + "a" * 32)
        self.assertTrue(record["pin"])
        with (
            self.assertRaisesRegex(resources.ResourceError, "replace logical identity"),
            resources.resource_update(first) as update,
        ):
            update["namespace"] = "canonical_test_ns_" + "c" * 32
        self.assertEqual(
            resources.resource_status(first)["namespace"], record["namespace"]
        )

    def test_registration_refusal_precedes_namespace_creation(self) -> None:
        self.config["admission"] = "quiescing"
        with (
            patch.object(resources, "namespace_sql") as sql,
            self.assertRaises(server.SupervisorError),
        ):
            tool.provision(self.state)
        sql.assert_not_called()
        self.assertEqual(resources.resource_status(), {})

    def test_unproved_creation_remains_registered_and_pinned(self) -> None:
        with (
            patch.object(
                resources, "namespace_sql", return_value=[None, None, "error"]
            ),
            self.assertRaisesRegex(tool.ToolError, "unproved"),
        ):
            tool.provision(self.state)
        records = list(resources.resource_status().values())
        self.assertEqual(len(records), 1)
        self.assertEqual(records[0]["namespace_creation"], "submitted")
        self.assertTrue(records[0]["pin"])
        self.assertFalse(records[0]["drained"])

    def test_inventory_and_checked_removal_only_touch_owned_namespace(self) -> None:
        resource, other = self.register(), self.register("b")
        record = resources.resource_status(resource)
        namespace = record["namespace"]
        with patch.object(
            resources,
            "namespace_sql",
            side_effect=[
                [{"namespaces": {namespace: "definition"}}],
                [{"databases": {"suite": "definition", "parity": "definition"}}],
                [None, None],
                [None],
                [{"namespaces": {}}],
            ],
        ) as sql:
            resources.remove_owned(record)
        queries = "\n".join(str(call.args[1]) for call in sql.call_args_list)
        self.assertIn("REMOVE DATABASE IF EXISTS `suite`", queries)
        self.assertNotIn(resources.resource_status(other)["namespace"], queries)
        self.assertNotIn("namespace", sql.call_args_list[3].kwargs)

    def test_cleanup_ack_error_retains_charge_and_can_resume(self) -> None:
        resource = self.register()
        resources.record_drain(resource)
        resources.pin(resource, False)
        namespace = resources.resource_status(resource)["namespace"]
        with (
            patch.object(
                resources,
                "namespace_sql",
                side_effect=[
                    [{"namespaces": {namespace: ""}}],
                    [{"databases": {}}],
                    ["error"],
                ],
            ),
            self.assertRaisesRegex(resources.ResourceError, "unproved"),
        ):
            resources.reclaim(resource)
        record = resources.resource_status(resource)
        self.assertTrue(record["pin"])
        self.assertEqual(record["cleanup"], "retained")
        resources.pin(resource, False)
        with patch.object(
            resources, "namespace_sql", return_value=[{"namespaces": {}}]
        ):
            self.assertEqual(resources.reclaim(resource), "removed")

    def test_namespace_recreation_cannot_be_marked_removed(self) -> None:
        resource = self.register()
        resources.record_drain(resource)
        resources.pin(resource, False)
        namespace = resources.resource_status(resource)["namespace"]
        with (
            patch.object(
                resources,
                "namespace_sql",
                side_effect=[
                    [{"namespaces": {namespace: ""}}],
                    [{"databases": {}}],
                    [None],
                    [{"namespaces": {namespace: ""}}],
                ],
            ),
            self.assertRaisesRegex(resources.ResourceError, "absence unproved"),
        ):
            resources.reclaim(resource)
        self.assertTrue(resources.resource_status(resource)["pin"])
        self.assertEqual(resources.resource_status(resource)["cleanup"], "retained")

    def test_malformed_inventory_does_not_issue_removal(self) -> None:
        record = resources.resource_status(self.register())
        with (
            patch.object(
                resources,
                "namespace_sql",
                side_effect=[
                    [{"namespaces": {record["namespace"]: ""}}],
                    [{"databases": []}],
                ],
            ) as sql,
            self.assertRaisesRegex(resources.ResourceError, "malformed"),
        ):
            resources.remove_owned(record)
        self.assertEqual(sql.call_count, 2)

    def test_long_borrow_releases_with_a_fresh_metadata_clock(self) -> None:
        resource = self.register()
        clock = resources.resource_store.clock
        with (
            patch.object(resources.resource_store, "clock", wraps=clock) as observed,
            resources.borrow(resource),
        ):
            self.assertTrue(resources.resource_status(resource)["borrows"])
        self.assertFalse(resources.resource_status(resource)["borrows"])
        # Admission and release independently request the ordinary clock when no
        # caller-wide cancellation clock was supplied.
        self.assertGreaterEqual(
            sum(call.args == (None,) for call in observed.call_args_list), 2
        )

    def test_tool_borrow_excludes_service_maintenance_after_creation_settles(
        self,
    ) -> None:
        resource = self.register()
        resources.record_drain(resource)
        with patch.object(server, "service_directory", return_value=self.state):
            server.require_borrowers_drained(self.state)
            with (
                resources.borrow(resource),
                self.assertRaisesRegex(server.SupervisorError, "protected borrowers"),
            ):
                server.require_borrowers_drained(self.state)
            server.require_borrowers_drained(self.state)

    def test_unknown_prior_activity_cannot_be_reused(self) -> None:
        resource = self.register()
        with resources.resource_update(resource) as record:
            record["namespace_creation"] = "acknowledged"
        with self.assertRaisesRegex(tool.ToolError, "settlement"):
            tool.fixture(resource)

    def test_lifecycle_reservation_wins_before_child_submission(self) -> None:
        resource = self.register()
        with (
            patch.object(tool, "verify"),
            patch.object(
                tool, "fixture", return_value=({"state": str(self.state)}, {}, {})
            ),
            patch.object(tool, "validate_input"),
            patch.object(
                server,
                "context_admission",
                side_effect=server.SupervisorError("closed owner"),
            ),
            patch.object(tool.native_operation, "run_child_scope") as child,
            self.assertRaisesRegex(server.SupervisorError, "closed owner"),
        ):
            tool.execute(Path("binary"), resource, self.root, "sync")
        child.assert_not_called()

    def test_failed_cli_keeps_unknown_server_activity_protected(self) -> None:
        resource = self.register()
        resources.record_drain(resource)

        def failed(*_args: object, **kwargs: object) -> int:
            cast("dict[str, object]", kwargs["observation"]).update(
                owned_drain="complete"
            )
            return 1

        with (
            patch.object(tool, "verify"),
            patch.object(
                tool,
                "fixture",
                return_value=(resources.resource_status(resource), {}, {}),
            ),
            patch.object(tool, "validate_input"),
            patch.object(tool, "environment", return_value={}),
            patch.object(tool.native_operation, "run_child_scope", side_effect=failed),
        ):
            self.assertEqual(
                tool.execute(Path("binary"), resource, self.root, "sync"), 1
            )
        record = resources.resource_status(resource)
        self.assertFalse(record["drained"])
        self.assertTrue(record["creation_borrower"])
        self.assertFalse(record["borrows"])
        self.assertTrue(record["pin"])


class DispatchTests(unittest.TestCase):
    def test_tool_sessions_use_the_finite_server_global_capacity(self) -> None:
        config: dict[str, object] = {
            "instance_id": "2" * 32,
            "resources": server.resources(
                4 * server.GIB, 2 * server.GIB, 2, server.GIB
            ),
        }
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        config["telemetry_state"] = str(Path(temporary.name) / "telemetry")
        config["server"] = {"binary_sha256": "1" * 64}
        config["study_module"] = fixture_module(Path(temporary.name) / "modules")
        allocation = server.object_mapping(config["resources"])
        allocation["grpc_max_attached_sessions"] = 37
        credentials: dict[str, object] = {"username": "user", "password": "secret"}
        for fixture in (False, True):
            config["tooling_fixture"] = fixture
            env = server.server_environment(config, credentials)
            self.assertEqual(env["SURREAL_GRPC_MAX_ATTACHED_SESSIONS"], "37")
            self.assertNotIn("SURREAL_WEBSOCKET_MAX_ATTACHED_SESSIONS", env)

    def test_environment_has_only_exact_namespace_credentials(self) -> None:
        with patch.dict(
            "os.environ",
            {
                "DATABASE_URL": "evil",
                "SURREALDB_PASSWORD": "root",
                "SURREAL_PASS": "root",
            },
        ):
            env = tool.environment(
                {"endpoint": "grpc://127.0.0.1:1"},
                {"namespace": "owned"},
                {"username": "tool", "password": "ns-secret"},
                Path("/owned/input"),
            )
        self.assertNotIn("DATABASE_URL", env)
        self.assertNotIn("SURREAL_PASS", env)
        self.assertEqual(env["SURREALDB_PASSWORD"], "ns-secret")
        self.assertEqual(env["SURREALDB_AUTH_LEVEL"], "namespace")

    def test_explicit_actor_and_ambient_configuration_refused_before_child(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            folder = root / "input"
            (folder / "schema").mkdir(parents=True)
            schema = folder / "schema/canonical.surql"
            schema.write_bytes((tool.ROOT / tool.GENERATED_SCHEMA).read_bytes())
            provenance = {
                "schema_sha256": tool.digest(schema),
                "producer": str(tool.GENERATED_SCHEMA),
                "files": {"canonical.surql": tool.digest(schema)},
            }
            (folder / "input.json").write_text(json.dumps(provenance))
            record = {"directory": str(root), "tool_inputs": {folder.name: provenance}}
            (folder / "tests/suites").mkdir(parents=True)
            (folder / "tests/suites/canonical.toml").write_bytes(
                (
                    tool.ROOT / "scripts/tests/fixtures/surrealkit/canonical.toml"
                ).read_bytes()
            )
            for name, data in (
                (".env.local", ""),
                ("suite.toml", '[actors.root]\nkind="root"\n'),
                ("config.toml", '[defaults]\nbase_url="https://unrelated.invalid"\n'),
                ("http.toml", '[[cases]]\nkind="api_request"\n'),
                ("parity.toml", "[rollouts]\nparity=false\n"),
                ("schema.toml", 'schema_from="sync"\n'),
                ("negative.toml", '[[cases]]\nkind="sql_expect"\nallow=false\n'),
                ("file.toml", '[[fixtures]]\nfile="/outside.sql"\n'),
            ):
                path = folder / name
                path.write_text(data)
                with self.subTest(name=name), self.assertRaises(tool.ToolError):
                    tool.validate_input(folder, record)
                path.unlink()
            tool.validate_input(folder, record)
            (folder / "tests/suites/canonical.toml").write_text(
                '[[cases]]\nkind="sql_expect"\nallow=true\nsql="DEFINE EVENT queued ON fixture WHEN true THEN (RETURN true) ASYNC;"\n'
            )
            with self.assertRaisesRegex(tool.ToolError, "qualified declarative"):
                tool.validate_input(folder, record)
            (folder / "tests/suites/canonical.toml").write_bytes(
                (
                    tool.ROOT / "scripts/tests/fixtures/surrealkit/canonical.toml"
                ).read_bytes()
            )
            # Mutating an editable sibling manifest cannot replace its owner.
            schema.write_text("DEFINE TABLE alternate;")
            manifest = dict(
                provenance,
                schema_sha256=tool.digest(schema),
                files={"canonical.surql": tool.digest(schema)},
            )
            (folder / "input.json").write_text(json.dumps(manifest))
            with self.assertRaisesRegex(tool.ToolError, "provenance"):
                tool.validate_input(folder, record)
            schema.write_bytes((tool.ROOT / tool.GENERATED_SCHEMA).read_bytes())
            (folder / "input.json").write_text(json.dumps(provenance))
            (folder / "schema/extra.surql").write_text("DEFINE TABLE extra;")
            with self.assertRaisesRegex(tool.ToolError, "file set"):
                tool.validate_input(folder, record)


if __name__ == "__main__":
    unittest.main()

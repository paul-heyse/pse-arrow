# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Immutable publication controls; actual beta CLI fixture is separate."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import hashlib
import json
import tempfile
import unittest
from pathlib import Path
from typing import cast
from unittest.mock import patch

from scripts import surreal_rollout_producer as producer


class StepContractTests(unittest.TestCase):
    def test_beta_entity_scope_defaults_to_null_in_fixed_field_order(self) -> None:
        step: dict[str, object] = {
            "phase": "complete",
            "entities": [{"name": "sample", "kind": "table"}],
            "id": "remove",
            "kind": "remove_entities",
        }
        normalized = b'{"id":"remove","phase":"complete","kind":"remove_entities","entities":[{"kind":"table","scope":null,"name":"sample"}]}'
        self.assertEqual(
            producer.beta6_step_checksum(step), hashlib.sha256(normalized).hexdigest()
        )

    def test_frozen_file_order_and_unicode_are_preserved(self) -> None:
        step: dict[str, object] = {
            "id": "apply",
            "phase": "start",
            "kind": "apply_files",
            "files": [{"hash": "abc", "path": "schema/\u03b1.surql"}],
        }
        normalized = '{"id":"apply","phase":"start","kind":"apply_files","files":[{"path":"schema/\u03b1.surql","hash":"abc"}]}'.encode()
        self.assertEqual(
            producer.beta6_step_checksum(step), hashlib.sha256(normalized).hexdigest()
        )

    def test_live_files_and_arbitrary_data_sql_are_refused(self) -> None:
        invalid_steps: tuple[dict[str, object], ...] = (
            {
                "id": "data",
                "phase": "start",
                "kind": "run_sql",
                "sql": "DELETE sample;",
            },
            {
                "id": "live",
                "phase": "start",
                "kind": "apply_files",
                "files": ["schema/canonical.surql"],
            },
        )
        for step in invalid_steps:
            with self.subTest(step=step), self.assertRaises(producer.ProducerError):
                producer.beta6_step_checksum(step)

    def test_schema_removal_start_keeps_source_until_completion(self) -> None:
        spec: dict[str, object] = {
            "steps": [
                {"id": "apply", "phase": "start", "kind": "apply_files", "files": []},
                {
                    "id": "remove",
                    "phase": "complete",
                    "kind": "remove_entities",
                    "entities": [],
                },
            ]
        }
        steps = producer.step_bindings(spec, "source.surql", "target.surql")
        self.assertEqual(
            cast("dict[str, object]", steps[0]["postcondition"])["sql"],
            ["source.surql", "target.surql"],
        )
        self.assertEqual(
            cast("dict[str, object]", steps[1]["postcondition"])["sql"],
            ["target.surql"],
        )


class FreshPublicationTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.folder = Path(temporary.name) / "fresh"
        self.binary = Path(temporary.name) / "tool"
        for replacement in (
            patch.object(
                producer.surrealkit, "verify", return_value={"version": producer.BETA}
            ),
            patch.object(producer, "beta", side_effect=self.native_plan),
        ):
            replacement.start()
            self.addCleanup(replacement.stop)

    def native_plan(self, _binary: Path, folder: Path, *args: str) -> str:
        if args[0] == "lint":
            if len(args) == 1:
                return "valid chain"
            raw = (folder / "rollouts" / (args[1] + ".toml")).read_bytes()
            return "checksum " + hashlib.sha256(raw).hexdigest()
        schema = (folder / "schema/canonical.surql").read_bytes()
        digest = hashlib.sha256(schema).hexdigest()
        identity = "schema_" + digest[:16]
        previous = json.loads((folder / "snapshots/schema_snapshot.json").read_text())
        owned = folder / "rollouts" / identity
        (owned / "schema").mkdir(parents=True)
        (owned / "schema/canonical.surql").write_bytes(schema)
        source = previous.get("native_hash", "empty")
        (folder / "rollouts" / (identity + ".toml")).write_text(
            f'id="{identity}"\nname="{identity}"\nsource_schema_hash="{source}"\n'
            f'target_schema_hash="{digest}"\ncompatibility="phased"\n'
            '[[steps]]\nid="apply"\nphase="start"\nkind="apply_files"\n'
            f'[[steps.files]]\npath="schema/canonical.surql"\nhash="{digest}"\n'
        )
        (folder / "snapshots/schema_snapshot.json").write_text(
            json.dumps({"version": 1, "files": [], "native_hash": digest})
        )
        (folder / "snapshots/catalog_snapshot.json").write_text(
            json.dumps({"version": 2, "entities": []})
        )
        return "planned"

    def publish(self, schema: bytes, identity: str) -> dict[str, object]:
        return producer.publish(
            self.binary,
            self.folder,
            schema,
            identity,
            hashlib.sha256(schema).hexdigest(),
        )

    def test_fresh_baseline_needs_no_historical_predecessor(self) -> None:
        head = self.publish(b"DEFINE TABLE current SCHEMAFULL;", "current")
        self.assertIsNone(head["predecessor"])
        self.assertEqual((self.folder / str(head["source_sql"])).read_bytes(), b"")
        self.assertNotIn("cutover", head)
        rollouts = producer.load(self.folder)["rollouts"]
        assert isinstance(rollouts, list), rollouts
        self.assertEqual(len(rollouts), 1)

    def test_identical_publication_is_idempotent(self) -> None:
        head = self.publish(b"DEFINE TABLE current SCHEMAFULL;", "current")
        again = self.publish(b"DEFINE TABLE current SCHEMAFULL;", "current")
        self.assertEqual(again["id"], head["id"])
        self.assertEqual(again["schema_digest"], head["schema_digest"])
        self.assertEqual(len(list((self.folder / "rollouts").glob("*.toml"))), 1)

    def test_forward_publication_keeps_frozen_predecessor_bytes(self) -> None:
        first = self.publish(b"DEFINE TABLE first SCHEMAFULL;", "first")
        original = {
            p.relative_to(self.folder): p.read_bytes()
            for p in (self.folder / "rollouts").rglob("*")
            if p.is_file()
        }
        second = self.publish(b"DEFINE TABLE second SCHEMAFULL;", "second")
        self.assertEqual(second["predecessor"], first["id"])
        for path, raw in original.items():
            self.assertEqual((self.folder / path).read_bytes(), raw)

    def test_modified_frozen_sql_refuses_before_appending(self) -> None:
        head = self.publish(b"DEFINE TABLE current SCHEMAFULL;", "current")
        (self.folder / str(head["target_sql"])).write_bytes(b"changed")
        with self.assertRaises(producer.ProducerError):
            self.publish(b"DEFINE TABLE next SCHEMAFULL;", "next")
        self.assertEqual(len(list((self.folder / "rollouts").glob("*.toml"))), 1)

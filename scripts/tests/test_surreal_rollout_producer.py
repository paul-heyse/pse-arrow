# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Immutable publication controls; actual beta CLI fixture is separate."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import hashlib
import json
import shutil
import tempfile
import tomllib
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


class DesignCutoverTests(unittest.TestCase):
    def setUp(self) -> None:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        self.folder = Path(temporary.name) / "publication"
        self.binary = Path(temporary.name) / "beta"
        self.old = b"DEFINE TABLE sample SCHEMAFULL; DEFINE FIELD payload ON sample TYPE bytes;"
        self.new = b"DEFINE TABLE sample SCHEMAFULL; DEFINE FIELD file_instance ON sample TYPE uuid;"
        self.calls: list[tuple[str, ...]] = []
        for mock in (
            patch.object(producer.surrealkit, "verify", return_value={"version": producer.BETA}),
            patch.object(producer, "beta", side_effect=self.beta),
            patch.object(producer, "declaration", return_value=(self.new, "pse.substrate.v6", "6" * 64)),
        ):
            mock.start()
            self.addCleanup(mock.stop)

    def beta(self, _binary: Path, folder: Path, *args: str) -> str:
        self.calls.append(args)
        if args[0] == "lint":
            if len(args) > 1:
                return "checksum " + producer.sha((folder / "rollouts" / (args[1] + ".toml")).read_bytes())
            return "valid chain"
        data = (folder / "schema/canonical.surql").read_bytes()
        old_snapshot = json.loads((folder / "snapshots/schema_snapshot.json").read_text())
        old_hash = old_snapshot.get("native_hash", "empty-native-hash")
        identity = "native_v5" if data == self.old else "native_v6"
        native_hash = "native-" + identity
        owned = folder / "rollouts" / identity
        (owned / "schema").mkdir(parents=True)
        (owned / "schema/canonical.surql").write_bytes(data)
        (owned / "snapshots").mkdir()
        for name in ("schema_snapshot.json", "catalog_snapshot.json"):
            shutil.copyfile(folder / "snapshots" / name, owned / "snapshots" / name)
        field = "payload" if data == self.old else "file_instance"
        entities = [{"kind": "field", "scope": "sample", "name": field, "statement_hash": field}]
        producer.write_json(folder / "snapshots/schema_snapshot.json", {"version": 1, "files": [], "native_hash": native_hash})
        producer.write_json(folder / "snapshots/catalog_snapshot.json", {"version": 2, "entities": entities})
        spec = {"id": identity, "name": identity, "source_schema_hash": old_hash,
                "target_schema_hash": native_hash, "compatibility": "phased", "steps": [
                    {"id": "apply_expand_schema", "phase": "start", "kind": "apply_files",
                     "files": [{"path": "schema/canonical.surql", "hash": producer.sha(data)}]}
                ]}
        (folder / "rollouts" / (identity + ".toml")).write_text(producer.manual_manifest(spec))
        return "planned"

    def test_explicit_cutover_keeps_historical_bytes_and_native_final_identity(self) -> None:
        old = producer.publish(self.binary, self.folder, self.old, "pse.substrate.v5", "5" * 64)
        releases = cast("list[dict[str, object]]", producer.load(self.folder)["releases"])
        immutable = {name: (self.folder / name).read_bytes() for name in cast("dict[str, str]", releases[0]["files"])}
        head = producer.publish(self.binary, self.folder, self.new, "pse.substrate.v6", "6" * 64, design_phase_v6_cutover=True)
        self.assertEqual(head["cutover"], producer.CUTOVER)
        self.assertEqual(head["source_schema_hash"], old["target_schema_hash"])
        self.assertEqual(head["target_schema_hash"], "native-native_v6")
        self.assertEqual(head["schema_digest"], "6" * 64)
        self.assertEqual((self.folder / cast("str", head["target_sql"])).read_bytes(), self.new)
        self.assertEqual((self.folder / "rollouts/native_v6/schema/canonical.surql").read_bytes(), self.new)
        for name, data in immutable.items():
            self.assertEqual((self.folder / name).read_bytes(), data)
        spec = tomllib.loads((self.folder / "rollouts/native_v6.toml").read_text())
        self.assertEqual([step["kind"] for step in spec["steps"]], ["apply_files", "remove_entities", "remove_entities"])
        self.assertEqual(spec["steps"][-1]["entities"], [{"kind": "field", "scope": "sample", "name": "payload"}])
        self.assertEqual(json.loads((self.folder / "rollouts/native_v6/snapshots/schema_snapshot.json").read_text())["native_hash"], old["target_schema_hash"])
        self.assertEqual(len(cast("list[object]", producer.load(self.folder)["releases"])), 2)
        self.assertEqual(self.calls[-2:], [("lint", "native_v6"), ("lint",)])

    def test_cutover_requires_v5_predecessor_and_exact_generated_v6(self) -> None:
        with self.assertRaisesRegex(producer.ProducerError, "v5 predecessor"):
            producer.publish(self.binary, self.folder, self.new, "pse.substrate.v6", "6" * 64, design_phase_v6_cutover=True)
        self.assertFalse(self.folder.exists())
        producer.publish(self.binary, self.folder, self.old, "pse.substrate.v5", "5" * 64)
        before = producer.inventory(self.folder)
        with self.assertRaisesRegex(producer.ProducerError, "exact generated v6"):
            producer.publish(self.binary, self.folder, self.new + b"-- changed", "pse.substrate.v6", "6" * 64, design_phase_v6_cutover=True)
        self.assertEqual(producer.inventory(self.folder), before)

    def test_modified_entities_restore_original_definition_without_data_sql(self) -> None:
        def entity(kind: str, name: str, digest: str) -> dict[str, str]:
            return {"kind": kind, "name": name, "statement_hash": digest}
        native = {"steps": [{"id": "apply", "phase": "start", "kind": "apply_files", "files": [{"path": "schema/canonical.surql", "hash": "abc"}]}]}
        old = {"version": 2, "entities": [entity("table", "sample", "old"), entity("user", "secret", "old")]}
        new = {"version": 2, "entities": [entity("table", "sample", "new"), entity("user", "secret", "new")]}
        steps = producer.cutover_steps(native, old, new)
        self.assertEqual(steps[1]["kind"], "restore_definitions")
        self.assertEqual(steps[1]["entities"], [{"kind": "table", "name": "sample"}])
        self.assertNotIn("run_sql", producer.manual_manifest({"id": "cutover", "name": "cutover", "source_schema_hash": "old", "target_schema_hash": "new", "compatibility": "phased", "steps": steps}))


if __name__ == "__main__":
    unittest.main()

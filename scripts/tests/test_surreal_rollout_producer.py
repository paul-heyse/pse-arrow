# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Immutable publication controls; actual beta CLI fixture is separate."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import hashlib
import unittest
from typing import cast

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


if __name__ == "__main__":
    unittest.main()

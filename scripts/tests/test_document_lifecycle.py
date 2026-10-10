# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Strict interpretation and derived multi-plan scope, without product imports."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import document_lifecycle as lifecycle
from scripts import document_metadata as metadata


class MetadataTests(unittest.TestCase):
    def test_yaml_semantics_and_source_bytes_survive(self) -> None:
        raw = b'---\r\ntitle: "A: title"\r\ndate: 2026-10-09\r\nstatus: in-progress\r\nnotes: [on, off]\r\n---\r\n\r\n# Exact body\r\n  trailing spaces  \r\n'
        document = metadata.read(raw)
        self.assertEqual(document.metadata["date"], "2026-10-09")
        self.assertEqual(document.metadata["notes"], ["on", "off"])
        self.assertEqual(document.prefix + document.body, raw)
        self.assertEqual(document.body, b"\r\n# Exact body\r\n  trailing spaces  \r\n")

    def test_ambiguous_unsafe_or_invalid_metadata_fails(self) -> None:
        cases = [
            "title: a\ntitle: b",
            "title: &x {nested: *x}",
            "base: &x {title: a}\nother: {<<: *x}",
            "? [a, b]\n: c",
            "title: !!python/object/apply:os.system [echo nope]",
            "doc_retention: delete",
            "doc_unknown: x",
            "doc_topics: topic",
            "title: unquoted: colon",
        ]
        for content in cases:
            with self.subTest(content=content), self.assertRaises(ValueError):
                metadata.read(f"---\n{content}\n---\n# Body\n")

    def test_shared_alias_and_quoted_merge_spelling_are_safe(self) -> None:
        document = metadata.read(
            '---\na: &a [one]\nb: *a\n"<<": literal\n---\n# Body\n'
        )
        self.assertEqual(document.metadata["b"], ["one"])
        self.assertEqual(document.metadata["<<"], "literal")

    def test_owner_bases_precedence_and_conflicts(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / "docs/plans/28-work.md"
            path.parent.mkdir(parents=True)
            path.write_text("# Owner\n")
            policy: dict = {
                "topics": ["work"],
                "defaults": {"doc_retention": "protected-unknown"},
                "collections": {"plans": {"doc_role": "plan"}},
                "bundles": [
                    {"root": "docs/plans", "defaults": {"doc_topics": ["work"]}}
                ],
            }
            site = {"collections": [{"root": "plans"}]}

            def interpret(text: str) -> dict:
                return metadata.interpreted(
                    root, path, metadata.read(text), policy, site
                )

            current = interpret(
                "---\ndoc_owner: docs/plans/28-work.md\ndoc_retention: while-dependent\n---\n"
            )
            self.assertEqual(current["doc_owner"], "docs/plans/28-work.md")
            self.assertEqual(current["doc_topics"], ["work"])
            self.assertEqual(
                interpret("# No owner\n")["doc_retention"], "protected-unknown"
            )
            self.assertEqual(
                interpret("---\ndisposition_owner: 28-work.md\n---\n")["doc_owner"],
                "docs/plans/28-work.md",
            )
            self.assertEqual(
                interpret("---\ndisposition_owner: docs/plans/28-work.md\n---\n")[
                    "doc_owner"
                ],
                "docs/plans/28-work.md",
            )
            policy["exceptions"] = [
                {"path": "docs/plans/28-work.md", "values": {"doc_role": "review"}}
            ]
            with self.assertRaisesRegex(ValueError, "conflicting explicit"):
                interpret("---\ndoc_role: plan\n---\n")
            policy["exceptions"] = []
            policy["bundles"].append(policy["bundles"][0])
            with self.assertRaisesRegex(ValueError, "ambiguous bundle"):
                interpret("# Body\n")

    def test_historical_owner_never_becomes_action_authority(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / "review.md"
            path.write_text("# Review\n")
            document = metadata.read(
                "---\ndisposition_owner: git:"
                + "a" * 40
                + ":docs/plans/27-old.md\n---\n"
            )
            result = metadata.interpreted(root, path, document, {}, {"collections": []})
            self.assertNotIn("doc_owner", result)
            self.assertEqual(result["doc_retention"], "protected-unknown")
            self.assertIn("historical_owner", result)


class NativeScopeTests(unittest.TestCase):
    def test_mixed_disproof_and_open_scope_is_open(self) -> None:
        policy = metadata.load_policy(metadata.ROOT)
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / "docs/plans/28-work.md"
            path.parent.mkdir(parents=True)
            path.write_text(
                "## Findings\n\n| Finding | Status |\n|---|---|\n| F1 | disproved as thrash; open as cost |\n"
            )
            policy["scope_tables"] = [
                {
                    "path": "docs/plans/28-work.md",
                    "name": "findings",
                    "heading": "Findings",
                    "headers": ["Finding", "Status"],
                    "id_column": 0,
                    "status_column": 1,
                    "status_profile": "finding",
                    "kind": "finding",
                }
            ]
            policy["relationships"] = []
            result = lifecycle.aggregate(root, policy)
            self.assertEqual(result[0]["state"], "open")
            self.assertEqual(
                result[0]["raw_status"], "disproved as thrash; open as cost"
            )

    def test_validation_checks_scope_bindings_and_unrelated_tables_are_not_schema(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / "docs/plans/28-work.md"
            path.parent.mkdir(parents=True)
            path.write_text(
                "## Explanation\n\n| A | B |\n|---|---|\n| valid abbreviated GFM row |\n\n## Work\n\n| Packet | Status |\n|---|---|\n| P1 | done |\n"
            )
            binding = {
                "path": "docs/plans/28-work.md",
                "name": "packets",
                "heading": "Work",
                "headers": ["Packet", "Status"],
                "id_column": 0,
                "kind": "packet",
            }
            policy = {"scope_tables": [binding]}
            self.assertEqual(len(lifecycle.aggregate(root, policy)), 1)
            binding["heading"] = "Missing"
            with patch.object(lifecycle, "inventory", return_value=([], [])):
                result = lifecycle.snapshot(root, "validate", policy)
            self.assertTrue(result["errors"])

    def test_native_tokens_preserve_escaped_cells_and_exclude_fences(self) -> None:
        body = "# Work\n\n```md\n| Fake | Status |\n|---|---|\n| X | done |\n```\n\n## Packets\n\n| Packet | Meaning | Status |\n|---|---|---|\n| P1 | `left\\|right` and [link](a.md) | Implemented; qualification pending |\n"
        result = lifecycle.tables(body)
        self.assertEqual(len(result), 1)
        self.assertEqual(result[0].headers, ("Packet", "Meaning", "Status"))
        self.assertIn("left|right", result[0].rows[0][1])
        self.assertEqual(result[0].heading, "Packets")

    def test_malformed_arity_fails_instead_of_dropping_scope(self) -> None:
        with self.assertRaisesRegex(ValueError, "arity"):
            lifecycle.tables(
                "## Work\n\n| Packet | Status |\n|---|---|\n| P1 | `a|b` | done |\n"
            )

    def test_concurrent_plans_and_raw_status_have_independent_owners(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            plans = root / "docs/plans"
            plans.mkdir(parents=True)
            policy = {
                "status_profiles": {
                    "native": {
                        "order": ["open", "complete"],
                        "open": ["pending"],
                        "complete": ["^done$"],
                    }
                },
                "scope_tables": [],
            }
            for number, status in [
                (28, "Implemented; qualification pending"),
                (32, "done"),
            ]:
                relative = f"docs/plans/{number}-work.md"
                (root / relative).write_text(
                    f"---\nstatus: in-progress\n---\n\n## Work\n\n| Packet | Status |\n|---|---|\n| P1 | {status} |\n"
                )
                policy["scope_tables"].append(
                    {
                        "path": relative,
                        "name": "packets",
                        "heading": "Work",
                        "headers": ["Packet", "Status"],
                        "id_column": 0,
                        "status_column": 1,
                        "status_profile": "native",
                        "kind": "packet",
                    }
                )
            policy["relationships"] = [
                {
                    "from": "docs/plans/32-work.md::packets::P1",
                    "kind": "depends-on",
                    "to": "docs/plans/28-work.md::packets::P1",
                }
            ]
            result = lifecycle.aggregate(root, policy)
            self.assertEqual([item["state"] for item in result], ["open", "complete"])
            self.assertEqual(
                result[0]["raw_status"], "Implemented; qualification pending"
            )
            self.assertNotEqual(result[0]["owner"], result[1]["owner"])
            self.assertEqual(result[1]["relationships"][0]["target"], result[0]["id"])
            policy["relationships"][0]["kind"] = "status-owned-by"
            owned = lifecycle.aggregate(root, policy)[1]
            self.assertEqual(owned["native_state"], "complete")
            self.assertEqual(owned["state"], "open")
            self.assertEqual(owned["status_owner"], result[0]["id"])
            policy["relationships"][0]["kind"] = "depends-on"
            policy["scope_tables"][1].pop("status_column")
            self.assertEqual(lifecycle.aggregate(root, policy)[1]["state"], "unknown")
            (plans / "33-unbound.md").write_text("# New owner\n")
            self.assertEqual(lifecycle.aggregate(root, policy)[2]["state"], "unknown")

    def test_ambiguous_table_selector_and_duplicate_ids_fail(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            path = root / "docs/plans/28-work.md"
            path.parent.mkdir(parents=True)
            table = "## Work\n\n| Packet | Status |\n|---|---|\n| P1 | done |\n\n"
            path.write_text(table + table)
            binding = {
                "path": "docs/plans/28-work.md",
                "name": "packets",
                "heading": "Work",
                "headers": ["Packet", "Status"],
                "id_column": 0,
                "kind": "packet",
            }
            with self.assertRaisesRegex(ValueError, "exactly one"):
                lifecycle.aggregate(root, {"scope_tables": [binding]})
            binding["occurrence"] = 1
            self.assertEqual(
                len(lifecycle.aggregate(root, {"scope_tables": [binding]})), 1
            )
            path.write_text(
                table.replace("| P1 | done |", "| P1 | done |\n| P1 | done |")
            )
            with self.assertRaisesRegex(ValueError, "duplicate native"):
                lifecycle.aggregate(root, {"scope_tables": [binding]})


if __name__ == "__main__":
    unittest.main()

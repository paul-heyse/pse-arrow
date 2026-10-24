# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Knowledge projection keeps original sources, selected coverage and freshness explicit."""

# ruff: noqa: PT009, PT027 -- stdlib setup controls
from __future__ import annotations

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import knowledge_store, workspace_sources


class KnowledgeStoreTests(unittest.TestCase):
    def test_source_only_corpus_and_selected_later_range_survive_index_loss(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            path = root / "owner.md"
            path.write_text("\n".join(f"line {n}" for n in range(1, 401)))
            with patch.object(
                knowledge_store.evidence_store,
                "run_native",
                side_effect=AssertionError("source-only operation contacted service"),
            ):
                default = knowledge_store.corpus(("owner.md",), root=root)
                selected = knowledge_store.corpus(
                    ("owner.md",), root=root, ranges={"owner.md": (340, 380)}
                )
            self.assertEqual(selected["version"], 2)
            self.assertEqual(selected["subjects"][0]["kind"], "document")
            self.assertTrue(
                all(
                    item["subject_id"] == "document:owner.md"
                    for item in selected["excerpts"]
                )
            )
            self.assertNotIn(
                "line 359", "\n".join(item["text"] for item in default["excerpts"])
            )
            self.assertIn(
                "line 359", "\n".join(item["text"] for item in selected["excerpts"])
            )
            self.assertIn("340-380/400", selected["sources"][0]["title"])
            self.assertEqual(
                default["sources"][0]["version_digest"],
                selected["sources"][0]["version_digest"],
            )
            self.assertEqual(
                knowledge_store.freshness(
                    {"sources": selected["sources"], "excerpts": []}, root=root
                )[0]["state"],
                "current",
            )
            path.write_text(path.read_text() + "\nnewly matching lexical term")
            # A native no-hit page still carries the full generation membership.
            self.assertEqual(
                knowledge_store.freshness(
                    {"sources": selected["sources"], "excerpts": []}, root=root
                )[0]["state"],
                "stale",
            )
            path.unlink()
            self.assertEqual(
                knowledge_store.freshness({"sources": selected["sources"]}, root=root)[
                    0
                ]["state"],
                "missing-or-unavailable",
            )

    def test_current_to_historical_scope_change_invalidates_same_file_bytes(
        self,
    ) -> None:
        original = {
            "id": "owner.md",
            "kind": "source-file",
            "locator": "repo:owner.md",
            "version_digest": "a" * 64,
            "status": "current",
        }
        with patch.object(
            workspace_sources,
            "source",
            return_value={"digest": "a" * 64, "scope": "History"},
        ):
            result = knowledge_store.freshness({"sources": [original]})
        self.assertEqual(result[0]["state"], "stale")

    def test_external_and_historical_absolute_locators_are_not_reopened(self) -> None:
        selected = {
            "id": "experiment",
            "kind": "experiment",
            "locator": "/foreign/private/source",
            "version_digest": "b" * 64,
            "status": "historical",
        }
        with patch.object(
            workspace_sources,
            "source",
            side_effect=AssertionError("external historical source opened"),
        ):
            result = knowledge_store.freshness({"sources": [selected]})
        self.assertEqual(result[0]["state"], "not-read")

    def test_native_dispatch_reuses_existing_activity_boundary_and_captures_json(
        self,
    ) -> None:
        def run(
            state: Path, tool: Path, arguments: list[str], **options: object
        ) -> int:
            options["stdout"].write('{"generation":"selected"}')
            self.assertEqual(arguments, ["knowledge-status"])
            return 0

        with patch.object(
            knowledge_store.evidence_store, "run_native", side_effect=run
        ):
            result = knowledge_store.native(
                Path("/selected"), Path("/built/tool"), ["knowledge-status"], deadline=1
            )
        self.assertEqual(result, {"generation": "selected"})

    def test_partial_domain_projection_is_refused_before_native_effects(self) -> None:
        with (
            patch.object(
                knowledge_store.workspace_domains,
                "project",
                return_value={
                    "complete": False,
                    "omissions": ["subject bound exceeded"],
                },
            ),
            self.assertRaisesRegex(ValueError, "incomplete owner-native projection"),
        ):
            knowledge_store.corpus(("owner.md",))

    def test_shared_source_does_not_attribute_whole_document_to_section(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "owner.md").write_text(
                "first section\nsecret elsewhere\nselected section\ncorrect clause\n"
            )
            fact = workspace_sources.source("owner.md", root=root)
            subject = {
                "id": "contract:20.5",
                "kind": "contract",
                "native_version": fact["digest"],
                "sources": [{"source_id": "owner.md", "role": "metadata"}],
            }
            projection = {
                "complete": True,
                "source_facts": [fact],
                "subjects": [subject],
                "edges": [],
                "productions": [],
                "section_ranges": [
                    {
                        "subject_id": "contract:20.5",
                        "source_id": "owner.md",
                        "start_line": 3,
                        "end_line": 4,
                    }
                ],
            }
            with patch.object(
                knowledge_store.workspace_domains, "project", return_value=projection
            ):
                result = knowledge_store.corpus(("owner.md",), root=root)
            attributed = [
                item["text"]
                for item in result["excerpts"]
                if item["subject_id"] == "contract:20.5"
            ]
            self.assertEqual(attributed, ["selected section\ncorrect clause"])
            self.assertTrue(
                any(item["subject_id"] is None for item in result["excerpts"])
            )

    def test_exact_section_corpus_selects_its_actual_later_owner_range(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "owner.md").write_text(
                "\n".join(f"line {n}" for n in range(1, 401))
            )
            fact = workspace_sources.source("owner.md", root=root)
            subject = {
                "id": "contract:20.5",
                "kind": "contract",
                "native_version": fact["digest"],
                "sources": [{"source_id": "owner.md", "role": "metadata"}],
            }
            projection = {
                "complete": True,
                "source_facts": [
                    {key: value for key, value in fact.items() if key != "text"}
                ],
                "subjects": [subject],
                "items": [subject],
                "edges": [],
                "productions": [],
                "section_ranges": [
                    {
                        "subject_id": "contract:20.5",
                        "source_id": "owner.md",
                        "start_line": 340,
                        "end_line": 380,
                    }
                ],
            }
            with patch.object(
                knowledge_store.workspace_domains, "query", return_value=projection
            ) as query:
                result = knowledge_store.corpus(
                    concept="contract-section", identity="20.5", root=root
                )
            self.assertEqual(query.call_args.args, ("contract-section", "20.5"))
            self.assertIn(
                "line 359", "\n".join(item["text"] for item in result["excerpts"])
            )
            self.assertNotIn(
                "line 239", "\n".join(item["text"] for item in result["excerpts"])
            )


if __name__ == "__main__":
    unittest.main()

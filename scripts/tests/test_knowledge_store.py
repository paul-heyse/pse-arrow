# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Knowledge projection keeps original sources, selected coverage and freshness explicit."""

# ruff: noqa: PT009, PT027 -- stdlib setup controls
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from typing import TextIO
from unittest.mock import patch

from scripts import knowledge_store, workspace_sources


class KnowledgeStoreTests(unittest.TestCase):
    def test_indexed_read_keeps_native_document_and_separate_freshness(self) -> None:
        request = {
            "version": 1,
            "mode": "productions",
            "artifact_id": "artifact:one",
            "limit": 1,
        }
        original = {
            "version": 1,
            "generation": "a" * 64,
            "sources": [],
            "next": {"after": "exact"},
            "subjects": [{"label": "x" * 17000}],
        }

        def read(_state: Path, arguments: list[str], *, deadline: float) -> dict:
            self.assertGreater(deadline, 0)
            self.assertEqual(arguments[0], "knowledge-query")
            self.assertEqual(json.loads(Path(arguments[2]).read_bytes()), request)
            return original

        with patch.object(knowledge_store, "native", side_effect=read):
            result = knowledge_store.indexed(request, state=Path("/selected"))
        self.assertIs(result["document"], original)
        self.assertEqual(result["source_observations"], [])
        self.assertEqual(result["document"]["subjects"][0]["label"], "x" * 17000)

    def test_indexed_refusal_is_not_replaced_by_source_corpus(self) -> None:
        refusal = {"outcome": "refused", "code": "pse.knowledge.cursor_context"}
        with (
            patch.object(knowledge_store, "native", return_value=refusal),
            patch.object(
                knowledge_store, "corpus", side_effect=AssertionError("fallback")
            ),
        ):
            result = knowledge_store.indexed({"version": 1}, state=Path("/state"))
        self.assertEqual(result, refusal)

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
            _state: Path,
            arguments: list[str],
            *,
            deadline: float,
            stdout: TextIO | None = None,
        ) -> int:
            del deadline
            if stdout is None:
                self.fail("Expected the original output stream")
            stdout.write('{"generation":"selected"}')
            self.assertEqual(arguments, ["knowledge-status"])
            return 0

        with patch.object(
            knowledge_store.evidence_store, "run_native", side_effect=run
        ):
            result = knowledge_store.native(
                Path("/selected"), ["knowledge-status"], deadline=1
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

    def test_actual_native_stub_producers_share_one_attributed_governing_extent(
        self,
    ) -> None:
        root = Path(__file__).resolve().parents[2]
        artifact = "generated_artifact:python/pse/_native.pyi"
        projection = knowledge_store.workspace_domains.query(
            knowledge_store.workspace_domain_contracts.parse_query(
                {
                    "concept": "generated-output",
                    "identity": "python/pse/_native.pyi",
                }
            ),
            root=root,
        )
        assert projection["complete"], projection["omissions"]
        expected = {
            (
                f"declaration:{kind}:_native",
                "generator:xtask/src/codegen/python_stubs.rs#run",
                artifact,
            )
            for kind in (
                "pyo3_exception_supplement",
                "pyo3_metadata",
                "pyo3_module",
            )
        }
        assert {
            (row["declaration_id"], row["generator_id"], row["artifact_id"])
            for row in projection["productions"]
        } == expected
        extents = [
            row
            for row in projection["section_ranges"]
            if row["subject_id"] == "contract:21.5"
        ]
        assert len(extents) == 1
        extent = extents[0]
        selected_end = min(extent["end_line"], extent["start_line"] + 239)
        corpus = knowledge_store.corpus(
            concept="generated-output",
            identity="python/pse/_native.pyi",
            root=root,
            ranges={extent["source_id"]: (extent["start_line"], selected_end)},
        )
        attributed = [
            row for row in corpus["excerpts"] if row["subject_id"] == "contract:21.5"
        ]
        assert attributed
        assert len({row["id"] for row in corpus["excerpts"]}) == len(corpus["excerpts"])
        assert len(attributed) == (selected_end - extent["start_line"]) // 40 + 1
        assert all(row["source_id"] == extent["source_id"] for row in attributed)
        assert {
            (row["declaration_id"], row["generator_id"], row["artifact_id"])
            for row in corpus["productions"]
        } == expected

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
            self.assertEqual(len(query.call_args.args), 1)
            request = query.call_args.args[0]
            self.assertEqual(request.concept, "contract-section")
            self.assertEqual(request.identity, "20.5")
            self.assertEqual(request.limit, 100)
            self.assertIn(
                "line 359", "\n".join(item["text"] for item in result["excerpts"])
            )
            self.assertNotIn(
                "line 239", "\n".join(item["text"] for item in result["excerpts"])
            )


if __name__ == "__main__":
    unittest.main()

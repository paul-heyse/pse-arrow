# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Source/model identity and attribution controls for optional retrieval."""
# ruff: noqa: PT009 -- stdlib setup controls
from __future__ import annotations

import hashlib
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import semantic_retrieval as retrieval


class SemanticRetrievalTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory(prefix="pse-retrieval-test-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.text = "pub fn current_contract() {}\n"
        self.path = self.root / "current.rs"
        self.path.write_text(self.text)
        self.source = {"id": "current.rs", "kind": "source-file", "locator": "repo:current.rs", "version_digest": hashlib.sha256(self.text.encode()).hexdigest(), "basis": "declared", "title": "Current contract", "status": "current"}
        self.excerpt = {"id": "current.rs#L1", "source_id": "current.rs", "text": self.text.strip(), "locator": "repo:current.rs#L1", "basis": "text"}
        self.corpus = {"version": 1, "expected_active": None, "sources": [self.source], "excerpts": [self.excerpt], "edges": []}

    def test_source_drift_blocks_inference_without_implicit_refresh(self) -> None:
        self.path.write_text("changed\n")
        with patch.object(retrieval, "vectors") as inference, self.assertRaisesRegex(ValueError, "stale"):
            retrieval.compare(self.corpus, [{"id": "question", "query": "contract"}], root=self.root)
        inference.assert_not_called()

    def test_wrong_excerpt_and_deleted_source_are_refused(self) -> None:
        self.excerpt["text"] = "not in the original source"
        with self.assertRaisesRegex(ValueError, "differs"):
            retrieval.validate_corpus(self.corpus, root=self.root)
        self.path.unlink()
        with self.assertRaisesRegex(ValueError, "missing"):
            retrieval.validate_corpus(self.corpus, root=self.root)

    def test_historical_similarity_is_visible_without_replacing_current_authority(self) -> None:
        historical = self.root / "historical.rs"
        historical.write_text("pub fn old_contract() {}\n")
        self.corpus["sources"].append({**self.source, "id": "historical.rs", "locator": "repo:historical.rs", "version_digest": hashlib.sha256(historical.read_bytes()).hexdigest(), "status": "historical"})
        self.corpus["excerpts"].append({**self.excerpt, "id": "historical.rs#L1", "source_id": "historical.rs", "text": "pub fn old_contract() {}", "locator": "repo:historical.rs#L1"})
        # Mechanism test only: real usefulness is established separately by native model probes.
        observed = {"vectors": [[0.0, 1.0], [1.0, 0.0], [1.0, 0.0]], "runtime": {"runtime_members": {"binary": "fixture"}}, "elapsed_seconds": 0}
        with patch.object(retrieval, "vectors", return_value=observed), patch.object(retrieval.knowledge_store, "freshness", return_value=[{"state": "current"}, {"state": "current"}]), patch.object(retrieval.workspace_sources, "search", return_value={"items": [], "complete": True}):
            result = retrieval.compare(self.corpus, [{"id": "question", "query": "contract"}], root=self.root)
        self.assertEqual(result["questions"][0]["raw_similarity"][0]["status"], "historical")
        self.assertEqual(result["questions"][0]["semantic"][0]["status"], "current")
        self.assertEqual(result["embeddings"][0]["source_digest"], self.source["version_digest"])

    def test_invalid_inputs_fail_before_optional_runtime_lookup(self) -> None:
        for texts in ([], [""], ["x"] * 257, ["<|pse-owned-embedding-boundary|>"]):
            with self.subTest(texts=len(texts)), patch.object(retrieval, "runtime") as runtime, self.assertRaises(ValueError):
                retrieval.vectors(texts)
            runtime.assert_not_called()

    def test_source_change_during_comparison_blocks_mixed_cut_result(self) -> None:
        observed = {"vectors": [[1.0], [1.0]], "runtime": {"runtime_members": {}}, "elapsed_seconds": 0}
        def changed(*args: object, **kwargs: object) -> dict:
            self.path.write_text("changed during lexical read\n")
            return {"items": [], "complete": True}
        with patch.object(retrieval, "vectors", return_value=observed), patch.object(retrieval.workspace_sources, "search", side_effect=changed), self.assertRaisesRegex(ValueError, "stale"):
            retrieval.compare(self.corpus, [{"id": "question", "query": "contract"}], root=self.root)


if __name__ == "__main__":
    unittest.main()

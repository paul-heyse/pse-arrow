# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Targeted source-cut, reconciliation and import-aware migration behavior."""
# ruff: noqa: PT009 -- stdlib setup controls
from __future__ import annotations

import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import semantic_edits as edits
from scripts import semantic_navigation, structural_migrations


class SemanticEditTests(unittest.TestCase):
    def setUp(self) -> None:
        self.directory = tempfile.TemporaryDirectory(prefix="pse-edit-test-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "a.py").write_text("name = 1\n")
        (self.root / "b.py").write_text("print(name)\n")

    def selected(self) -> dict:
        cut = edits.capture(self.root, ["a.py", "b.py"])
        return edits.replacements(self.root, cut, {name: value.replace("name", "result") for name, value in cut.items()}, producer="owned fixture")

    def test_apply_and_reconcile_all_owned_files(self) -> None:
        selected = self.selected()
        result = edits.apply(selected, self.root / "receipt.json")
        self.assertTrue(result["complete"])
        self.assertEqual([f["edits"] for f in result["files"]], [1, 1])
        self.assertEqual((self.root / "a.py").read_text(), "result = 1\n")

    def test_stale_preview_blocks_all_mutation(self) -> None:
        selected = self.selected()
        (self.root / "b.py").write_text("changed\n")
        with self.assertRaisesRegex(ValueError, "stale preview"):
            edits.apply(selected, self.root / "receipt.json")
        self.assertEqual((self.root / "a.py").read_text(), "name = 1\n")

    def test_interruption_records_partial_application(self) -> None:
        selected = self.selected()
        original = edits._write
        count = 0
        def interrupt(path: Path, content: bytes) -> None:
            nonlocal count
            count += 1
            if count == 2:
                raise KeyboardInterrupt
            original(path, content)
        receipt = self.root / "receipt.json"
        with patch.object(edits, "_write", side_effect=interrupt), self.assertRaises(KeyboardInterrupt):
            edits.apply(selected, receipt)
        result = json.loads(receipt.read_bytes())
        self.assertFalse(result["complete"])
        self.assertEqual([f["state"] for f in result["files"]], ["applied", "unchanged"])
        self.assertEqual(edits.reconcile(selected), result)

    def test_drop_truncation_degraded_and_unowned_are_refused(self) -> None:
        cut = edits.capture(self.root, ["a.py"])
        change = {"uri": (self.root / "b.py").as_uri(), "edits": [{"range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 0}}, "new_text": "x"}]}
        for incomplete in ({"dropped": {"out_of_workspace": 1}}, {"truncated": True}, {"positions_degraded": "response"}, {"nested": {"dropped": {"exceeds_item_cap": 1}}}):
            with self.subTest(incomplete=incomplete), self.assertRaises(ValueError):
                edits.preview(self.root, cut, {"changes": [change], **incomplete}, producer="fixture")
        with self.assertRaisesRegex(ValueError, "absent from"):
            edits.preview(self.root, cut, {"changes": [change]}, producer="fixture")

    def test_generated_outside_symlink_and_receipt_source_are_refused(self) -> None:
        for name in ("python/pse/contracts/model.py", "crates/pse-operations-queries/lib.rs", "crates/example/src/generated/data.rs", "python/pse/_native.pyi", "crates/pse-ipopt-sys/src/bindings.rs"):
            path = self.root / name
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text("source")
            with self.subTest(name=name), self.assertRaises(ValueError):
                edits.capture(self.root, [name])
        (self.root / "linked.py").symlink_to(self.root / "a.py")
        for name in ("linked.py", "/etc/passwd", "../outside"):
            with self.subTest(name=name), self.assertRaises(ValueError):
                edits.capture(self.root, [name])
        with self.assertRaises(ValueError):
            edits.apply(self.selected(), self.root / "a.py")

    def test_preview_edit_tampering_and_divergence(self) -> None:
        selected = self.selected()
        selected["files"][0]["edits"][0]["text"] = "unapproved"
        with self.assertRaisesRegex(ValueError, "differ"):
            edits.reconcile(selected)
        selected = self.selected()
        (self.root / "a.py").write_text("other\n")
        self.assertEqual(edits.reconcile(selected)["files"][0]["state"], "diverged")

    def test_missing_partial_output_and_dependency_drift_remain_incomplete(self) -> None:
        selected = self.selected()
        edits.apply(selected, self.root / "receipt.json")
        (self.root / "a.py").unlink()
        state = edits.reconcile(selected)
        self.assertFalse(state["complete"])
        self.assertEqual([item["state"] for item in state["files"]], ["missing", "applied"])
        (self.root / "a.py").write_text("name = 1\n")
        cut = edits.capture(self.root, ["a.py", "b.py"])
        selected = edits.replacements(self.root, cut, {"a.py": "result = 1\n"}, producer="fixture")
        edits.apply(selected, self.root / "receipt.json")
        (self.root / "b.py").write_text("changed dependency\n")
        state = edits.reconcile(selected)
        self.assertFalse(state["complete"])
        self.assertEqual(state["basis"][0]["state"], "diverged")

    def test_one_based_utf16_and_non_ascii(self) -> None:
        (self.root / "a.py").write_text('s = "😀"; name = 1\n')
        raw = {"changes": [{"uri": (self.root / "a.py").as_uri(), "edits": [{"range": {"start": {"line": 1, "character": 11}, "end": {"line": 1, "character": 15}}, "new_text": "result"}]}]}
        selected = edits.preview(self.root, edits.capture(self.root, ["a.py"]), semantic_navigation.edit_result(raw), producer="mcpls fixture")
        self.assertEqual(selected["files"][0]["after"], 's = "😀"; result = 1\n')

    def test_source_basis_drift_and_overlapping_edits(self) -> None:
        cut = edits.capture(self.root, ["a.py", "b.py"])
        selected = edits.replacements(self.root, cut, {"a.py": "result = 1\n"}, producer="fixture")
        (self.root / "b.py").write_text("unrelated dependency drift\n")
        with self.assertRaisesRegex(ValueError, "stale preview basis"):
            edits.apply(selected, self.root / "receipt.json")
        with self.assertRaisesRegex(ValueError, "source changed"):
            edits.replacements(self.root, cut, {"a.py": "result = 1\n"}, producer="fixture")
        cut = edits.capture(self.root, ["a.py"])
        edit = {"range": {"start": {"line": 0, "character": 0}, "end": {"line": 0, "character": 4}}, "new_text": "result"}
        with self.assertRaisesRegex(ValueError, "overlapping"):
            edits.preview(self.root, cut, {"changes": [{"uri": (self.root / "a.py").as_uri(), "edits": [edit, edit]}]}, producer="fixture")

    def test_real_code_action_shape_with_insertion_and_replacement(self) -> None:
        # Pyrefly extraction returns an insertion and replacement at the same start.
        raw = {"changes": [{"uri": (self.root / "a.py").as_uri(), "edits": [
            {"range": {"start": {"line": 1, "character": 1}, "end": {"line": 1, "character": 1}}, "new_text": "# extracted\n"},
            {"range": {"start": {"line": 1, "character": 1}, "end": {"line": 1, "character": 5}}, "new_text": "result"},
        ]}]}
        selected = edits.preview(self.root, edits.capture(self.root, ["a.py"]), semantic_navigation.edit_result(raw), producer="mcpls action fixture")
        self.assertEqual(selected["files"][0]["after"], "# extracted\nresult = 1\n")
        self.assertTrue(edits.apply(selected, self.root / "receipt.json")["complete"])

    def test_structural_positive_near_miss_comments_and_python314(self) -> None:
        fixture = Path(__file__).parent / "fixtures/semantic-edits"
        after, observed = structural_migrations.json_source_bytes((fixture / "json-source.before.txt").read_text())
        self.assertEqual(after, (fixture / "json-source.after.txt").read_text())
        self.assertEqual(len(observed["resolved_edits"]), 2)
        self.assertGreater(observed["ast_grep_candidates"], 2)
        after_again, _ = structural_migrations.json_source_bytes(after)
        self.assertEqual(after_again, after)
        compile(after, "python314-migration-fixture", "exec")

    def test_structural_migration_executes_bytes_json_consumer(self) -> None:
        data = self.root / "data.json"
        data.write_bytes('{"name": "日本語"}'.encode("utf-8"))
        source = 'from json import loads as decode\nfrom pathlib import Path\nresult = decode(Path(' + repr(str(data)) + ').read_text())  # source\n'
        (self.root / "consumer.py").write_text(source)
        selected = structural_migrations.migration(self.root, ["consumer.py"])
        self.assertTrue(edits.apply(selected, self.root / "receipt.json")["complete"])
        namespace = {}
        exec(compile((self.root / "consumer.py").read_text(), "migrated consumer", "exec"), namespace)
        self.assertEqual(namespace["result"], {"name": "日本語"})
        self.assertIn("# source", (self.root / "consumer.py").read_text())


if __name__ == "__main__":
    unittest.main()

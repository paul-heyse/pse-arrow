# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Discovery and structured reach preserve declared grammar and exclusions."""
# ruff: noqa: PT009, PT027 -- stdlib setup controls

from __future__ import annotations

import contextlib
import io
import json
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import affected, workspace_context


class WorkspaceContextTests(unittest.TestCase):
    def test_recipe_grammar_is_not_evaluated_and_private_commands_stay_private(self) -> None:
        recipe = {"doc": "inspect a run", "parameters": [{"name": "run", "default": "literal $HOME"}],
                  "body": [["touch should-never-exist"]]}
        result = workspace_context.recipe_context({"result": recipe, "hidden": {**recipe, "private": True}}, "inspect run")
        self.assertEqual([item["name"] for item in result], ["result"])
        self.assertEqual(result[0]["parameters"][0]["default"], "literal $HOME")
        self.assertNotIn("body", result[0])

    def test_boundary_leads_include_native_owner_and_preserve_truncation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            catalogue = root / 'crates/pse-runtime/src/boundary_documents.rs'
            catalogue.parent.mkdir(parents=True)
            catalogue.write_text('(SelectedRequest, owner, "selected", Both, Native),\n')
            generated = root / 'python/pse/contracts/selected.py'
            generated.parent.mkdir(parents=True)
            generated.write_text('class SelectedRequest: ...\n')
            result = workspace_context.boundary_leads('SelectedRequest', root=root)
            self.assertEqual(len(result['items']), 2)
            self.assertEqual(result['items'][0]['meaning'], 'catalogue declaration')
            self.assertIn('generator', result['items'][1]['edit_owner'])
            self.assertTrue(workspace_context.boundary_leads('SelectedRequest', root=root, limit=1)['truncated'])
            with self.assertRaises(ValueError):
                workspace_context.boundary_leads('x; shell', root=root)

    def test_structured_affected_keeps_reach_and_explicit_omissions(self) -> None:
        roots = {"owner": Path("crates/owner"), "consumer": Path("crates/consumer")}
        with patch.object(affected, "changed_files", return_value=[("M", "crates/owner/src/lib.rs"), ("?", "python/pse/example.py")]), \
             patch.object(affected, "workspace_packages", return_value=(roots, {"owner": {"consumer"}, "consumer": set()})), \
             patch.object(affected, "compose", side_effect=lambda value: value), \
             contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(affected.main(["--json"]), 0)
        result = json.loads(output.getvalue())
        self.assertEqual(result["selected_packages"], ["consumer", "owner"])
        self.assertIn("rdeps(owner)", result["command"])
        self.assertEqual(next(item for item in result["exclusions"] if item["kind"] == "python")["files"], ["python/pse/example.py"])

    def test_empty_structured_selection_does_not_supply_a_workspace_command(self) -> None:
        with patch.object(affected, "changed_files", return_value=[]), \
             patch.object(affected, "workspace_packages", return_value=({}, {})), \
             contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(affected.main(["--json", "--run"]), 0)
        self.assertIsNone(json.loads(output.getvalue())["command"])


if __name__ == "__main__":
    unittest.main()

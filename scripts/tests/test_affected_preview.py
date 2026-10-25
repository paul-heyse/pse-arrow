# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected change previews preserve the original reach owner without effects."""
# ruff: noqa: PT009, PT027 -- standard-library source-owner controls

from __future__ import annotations

import tempfile
import unittest
import contextlib
import io
from pathlib import Path
from unittest.mock import patch

from scripts import affected, scientific_inputs


class AffectedPreviewTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        selected = self.root / "tests/fixtures/scientific-inputs.toml"
        selected.parent.mkdir(parents=True)
        selected.write_text('version=1\npackage_inventory="packages/reference"\n[[families]]\nname="scientific"\nconsumers=["consumer"]\n')
        self.roots = {
            "owner": Path("crates/owner"),
            "consumer": Path("crates/consumer"),
            "unrelated": Path("crates/unrelated"),
        }

    def preview(self, changes: list[tuple[str, str]]) -> dict:
        with (
            patch.object(affected, "workspace_packages", return_value=(self.roots, {"owner": {"consumer"}})),
            patch.object(affected, "changed_files", side_effect=AssertionError("whole worktree discovery")),
            patch.object(affected, "compose", side_effect=AssertionError("live Arrow graph preparation")),
            patch.object(scientific_inputs, "capture", side_effect=AssertionError("scientific compiler/capture")),
        ):
            return affected.preview(changes, root=self.root, base="explicit-source-cut", prepare=False)

    def test_selected_path_reaches_dependents_without_other_dirty_paths(self) -> None:
        result = self.preview([("M", "crates/owner/src/lib.rs")])
        self.assertEqual(result["changed_packages"], ["owner"])
        self.assertEqual(result["selected_packages"], ["consumer", "owner"])
        self.assertEqual(result["base"], "explicit-source-cut")
        self.assertIn("rdeps(owner)", result["command"])
        self.assertEqual(result["command"][:5], ["scripts/pse-env", "--", ".venv/bin/python", "-m", "scripts.arrow_validation"])

    def test_configuration_widens_and_scientific_sources_are_conservative(self) -> None:
        wide = self.preview([("M", ".cargo/config.toml")])
        self.assertEqual(wide["selected_packages"], ["consumer", "owner", "unrelated"])
        self.assertIn("--workspace", wide["command"])
        scientific = self.preview([("D", "packages/reference/domain/removed.pse")])
        self.assertEqual(scientific["selected_packages"], ["consumer"])
        self.assertEqual(scientific["scientific_inputs"]["unknown"], ["packages/reference/domain/removed.pse"])
        self.assertIn("explicitly prepared Rust capture", scientific["scientific_inputs"]["reason"])

    def test_python_generator_and_deleted_paths_keep_their_separate_routes(self) -> None:
        result = self.preview([("?", "python/pse/new.py"), ("M", "crates/pse-schema/src/model.rs"), ("D", "crates/owner/src/old.rs")])
        exclusions = {row["kind"]: row for row in result["exclusions"]}
        self.assertEqual(exclusions["python"]["files"], ["python/pse/new.py"])
        self.assertEqual(exclusions["generator"]["files"], ["crates/pse-schema/src/model.rs"])
        self.assertEqual(result["selected_packages"], ["consumer", "owner"])

    def test_empty_or_escaping_selection_never_becomes_workspace_execution(self) -> None:
        with patch.object(affected, "workspace_packages", side_effect=AssertionError("empty selection needs no Cargo")):
            self.assertIsNone(affected.preview([], root=self.root, prepare=False)["command"])
        for name in ("../other/src.rs", "/other/src.rs"):
            with self.assertRaises(ValueError):
                self.preview([("M", name)])

    def test_unavailable_graph_has_no_fabricated_reverse_closure_or_command(self) -> None:
        manifest = self.root / "crates/owner/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[package]\nname="owner"\nversion="0.0.1"\n')
        with patch.object(affected, "workspace_packages", side_effect=OSError("Cargo unavailable")):
            result = affected.preview([("M", "crates/owner/src/lib.rs")], root=self.root, prepare=False)
        self.assertEqual(result["graph_status"], "unavailable")
        self.assertEqual(result["selected_packages"], ["owner"])
        self.assertIsNone(result["command"])
        self.assertIn("workspace-graph", [row["kind"] for row in result["exclusions"]])
        with patch.object(affected, "preview", return_value=result), contextlib.redirect_stdout(io.StringIO()) as output:
            self.assertEqual(affected.main(["--passive", "--changes-json", '[["M","crates/owner/src/lib.rs"]]']), 0)
        self.assertIn("selected graph is unavailable", output.getvalue())


if __name__ == "__main__":
    unittest.main()

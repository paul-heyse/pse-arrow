# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Selected change previews preserve the original reach owner without effects."""

from __future__ import annotations

import contextlib
import io
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import pytest

from scripts import affected, scientific_inputs


class AffectedPreviewTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        selected = self.root / "tests/fixtures/scientific-inputs.toml"
        selected.parent.mkdir(parents=True)
        selected.write_text(
            'version=1\npackage_inventory="packages/reference"\n[[families]]\nname="scientific"\nconsumers=["consumer"]\n'
        )
        self.roots = {
            "owner": Path("crates/owner"),
            "consumer": Path("crates/consumer"),
            "unrelated": Path("crates/unrelated"),
        }

    def preview(self, changes: list[tuple[str, str]]) -> dict:
        with (
            patch.object(
                affected,
                "workspace_packages",
                return_value=(self.roots, {"owner": {"consumer"}}),
            ),
            patch.object(
                affected,
                "changed_files",
                side_effect=AssertionError("whole worktree discovery"),
            ),
            patch.object(
                affected,
                "compose",
                side_effect=AssertionError("live Arrow graph preparation"),
            ),
            patch.object(
                scientific_inputs,
                "capture",
                side_effect=AssertionError("scientific compiler/capture"),
            ),
        ):
            return affected.preview(
                changes, root=self.root, base="explicit-source-cut", prepare=False
            )

    def test_selected_path_reaches_dependents_without_other_dirty_paths(self) -> None:
        result = self.preview([("M", "crates/owner/src/lib.rs")])
        assert result["changed_packages"] == ["owner"]
        assert result["selected_packages"] == ["consumer", "owner"]
        assert result["base"] == "explicit-source-cut"
        assert "rdeps(owner)" in result["command"]
        assert result["command"][:5] == [
            "scripts/pse-env",
            "--",
            ".venv/bin/python",
            "-m",
            "scripts.arrow_validation",
        ]

    def test_configuration_widens_and_scientific_sources_are_conservative(self) -> None:
        wide = self.preview([("M", ".cargo/config.toml")])
        assert wide["selected_packages"] == ["consumer", "owner", "unrelated"]
        assert "--workspace" in wide["command"]
        scientific = self.preview([("D", "packages/reference/domain/removed.pse")])
        assert scientific["selected_packages"] == ["consumer"]
        assert scientific["scientific_inputs"]["unknown"] == [
            "packages/reference/domain/removed.pse"
        ]
        assert (
            "explicitly prepared Rust capture"
            in scientific["scientific_inputs"]["reason"]
        )

    def test_python_generator_and_deleted_paths_keep_their_separate_routes(
        self,
    ) -> None:
        result = self.preview(
            [
                ("?", "python/pse/new.py"),
                ("M", "crates/pse-schema/src/model.rs"),
                ("D", "crates/owner/src/old.rs"),
            ]
        )
        exclusions = {row["kind"]: row for row in result["exclusions"]}
        assert exclusions["python"]["files"] == ["python/pse/new.py"]
        assert exclusions["generator"]["files"] == ["crates/pse-schema/src/model.rs"]
        assert result["selected_packages"] == ["consumer", "owner"]

    def test_empty_or_escaping_selection_never_becomes_workspace_execution(
        self,
    ) -> None:
        with patch.object(
            affected,
            "workspace_packages",
            side_effect=AssertionError("empty selection needs no Cargo"),
        ):
            assert (
                affected.preview([], root=self.root, prepare=False)["command"] is None
            )
        for name in ("../other/src.rs", "/other/src.rs"):
            with pytest.raises(ValueError, match="inside the selected checkout"):
                self.preview([("M", name)])

    def test_unavailable_graph_has_no_fabricated_reverse_closure_or_command(
        self,
    ) -> None:
        manifest = self.root / "crates/owner/Cargo.toml"
        manifest.parent.mkdir(parents=True)
        manifest.write_text('[package]\nname="owner"\nversion="0.0.1"\n')
        with patch.object(
            affected, "workspace_packages", side_effect=OSError("Cargo unavailable")
        ):
            result = affected.preview(
                [("M", "crates/owner/src/lib.rs")], root=self.root, prepare=False
            )
        assert result["graph_status"] == "unavailable"
        assert result["selected_packages"] == ["owner"]
        assert result["command"] is None
        assert "workspace-graph" in [row["kind"] for row in result["exclusions"]]
        with (
            patch.object(affected, "preview", return_value=result),
            contextlib.redirect_stdout(io.StringIO()) as output,
        ):
            assert (
                affected.main(
                    ["--passive", "--changes-json", '[["M","crates/owner/src/lib.rs"]]']
                )
                == 0
            )
        assert "selected graph is unavailable" in output.getvalue()


if __name__ == "__main__":
    unittest.main()

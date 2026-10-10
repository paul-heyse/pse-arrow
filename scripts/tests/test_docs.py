# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Publication behavior in disposable repositories, without the product environment."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import docs
from scripts.adr import section_owners


class DocumentationFixture(unittest.TestCase):
    def setUp(self) -> None:
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.source = self.root / "docs"
        self.source.mkdir()
        self.write("README.md", "# Welcome\n\nHello architectural readers.\n")
        self.write(
            "authoritative_design/blueprint.md",
            "# Blueprint\n\n## 0.1 Contract\n\nSemantic authority.\n",
        )
        self.write(
            "plans/01-current.md",
            "---\ntitle: Current plan\nstatus: in-progress\n---\n\n# Work\n",
        )
        self.write("plans/00-old.md", "---\nstatus: in-progress\n---\n\n# Old work\n")
        versions = docs.tool_versions()
        self.write(
            "site.toml",
            f"""[tools]
mdbook = "{versions["mdbook"]}"
pagefind = "{versions["pagefind"]}"
[publication]
exclude = ["generated/rustdoc/**"]
current_work = ["plans/01-*.md"]
entrypoints = ["README.md"]
[[collections]]
title = "Start"
root = "."
recursive = false
scope = "Current"
[[collections]]
title = "Architecture"
root = "authoritative_design"
scope = "Reference"
[[collections]]
title = "Work"
root = "plans"
scope = "History"
""",
        )
        shutil.copy2(docs.ROOT / "docs/book.toml", self.source / "book.toml")
        shutil.copytree(docs.ROOT / "docs/theme", self.source / "theme")

    def write(self, path: str, text: str) -> None:
        target = self.source / path
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_text(text, encoding="utf-8")

    def pages(self) -> list[docs.Page]:
        return docs.discover(self.root, docs.configuration(self.root))


class DocumentationTests(DocumentationFixture):
    def test_tool_discovery_is_importable_before_docs_dependencies(self) -> None:
        result = subprocess.run(
            [sys.executable, "-S", "-m", "scripts.docs", "tool-specs"],
            cwd=docs.ROOT,
            check=True,
            capture_output=True,
            text=True,
        )
        self.assertIn("mdbook@", result.stdout)
        with patch.object(
            docs.subprocess,
            "run",
            side_effect=AssertionError(
                "ordinary reads must not synchronize or execute tools"
            ),
        ):
            self.pages()

    def test_discovery_needs_no_navigation_edit_and_ignores_stale_status(self) -> None:
        self.write("plans/02-added.md", "# Newly discovered\n")
        pages = {p.path.as_posix(): p for p in self.pages()}
        self.assertEqual(pages["plans/02-added.md"].title, "Newly discovered")
        self.assertEqual(pages["plans/00-old.md"].scope, "History")
        self.assertEqual(pages["plans/01-current.md"].scope, "Current")
        self.assertEqual(pages["plans/01-current.md"].title, "Current plan")

    def test_titles_ignore_fences_and_empty_metadata_falls_back(self) -> None:
        self.write(
            "example.md",
            "---\ntitle:\nstatus: proposed\n---\n\n```md\n# Example only\n```\n\n# Actual title\n",
        )
        page = next(p for p in self.pages() if p.path.name == "example.md")
        self.assertEqual(page.title, "Actual title")
        self.write("example.md", "```md\n# Example only\n```\n")
        with self.assertRaisesRegex(ValueError, "scalar title or first H1"):
            self.pages()

    def test_tools_must_exist_at_the_declared_version(self) -> None:
        with (
            patch.object(docs.shutil, "which", return_value=None),
            self.assertRaisesRegex(ValueError, "missing mdbook"),
        ):
            docs.check_tools(self.root)
        expected = docs.tool_versions(self.root)["mdbook"]
        output = subprocess.CompletedProcess([], 0, stdout=f"mdbook v{expected}0\n")
        with (
            patch.object(docs.shutil, "which", return_value="mdbook"),
            patch.object(docs.subprocess, "run", return_value=output),
            self.assertRaisesRegex(ValueError, f"expected mdbook {expected}"),
        ):
            docs.check_tools(self.root)

    def test_invalid_publication_inputs_fail(self) -> None:
        config = docs.configuration(self.root)
        config["collections"].append(config["collections"][0])
        with self.assertRaisesRegex(ValueError, "duplicate publication"):
            docs.discover(self.root, config)
        config = docs.configuration(self.root)
        config["publication"]["entrypoints"].append("missing.md")
        with self.assertRaisesRegex(ValueError, "entrypoint"):
            docs.discover(self.root, config)
        self.write("untitled.md", "No title here.\n")
        with self.assertRaisesRegex(ValueError, "scalar title or first H1"):
            self.pages()

    def test_collection_cannot_escape_document_root(self) -> None:
        config = docs.configuration(self.root)
        config["collections"][0]["root"] = "../outside"
        with self.assertRaisesRegex(ValueError, "escapes docs"):
            docs.discover(self.root, config)

    def test_moved_section_has_one_owner_and_fences_are_not_sections(self) -> None:
        self.write(
            "authoritative_design/blueprint.md",
            '# Blueprint\n\n<a id="old-contract"></a>\n\n[Moved](sections/contract.md#01-contract)\n\n```markdown\n## 0.1 Example\n```\n',
        )
        self.write(
            "authoritative_design/sections/contract.md",
            "# Focused contract\n\n## 0.1 Contract\n",
        )
        owners = section_owners(self.source / "authoritative_design/blueprint.md")
        self.assertEqual(owners["0.1"][0].name, "contract.md")
        self.write(
            "authoritative_design/sections/duplicate.md",
            "# Duplicate\n\n## 0.1 Duplicate\n",
        )
        with self.assertRaisesRegex(ValueError, "duplicate section"):
            section_owners(self.source / "authoritative_design/blueprint.md")

    def test_staging_delivers_only_explicit_assets_and_preserves_sources(self) -> None:
        self.write("plans/evidence.json", '{"observation": 7}\n')
        self.write("plans/private.json", '{"retained": true}\n')
        before = {
            p.relative_to(self.source): p.read_bytes()
            for p in self.source.rglob("*")
            if p.is_file()
        }
        self.write(
            "authoritative_design/blueprint.md",
            "# Blueprint\n\n```md\n## 0.1 Contract\n```\n\n## 0.1 Contract\n",
        )
        before[Path("authoritative_design/blueprint.md")] = (
            self.source / "authoritative_design/blueprint.md"
        ).read_bytes()
        staged = self.root / "stage"
        config = docs.configuration(self.root)
        config["publication"]["assets"] = ["plans/evidence.json"]
        pages = docs.stage(self.root, staged, self.pages(), config)
        self.assertFalse((staged / "plans/private.json").exists())
        self.assertEqual(
            (staged / "plans/evidence.json").read_bytes(),
            before[Path("plans/evidence.json")],
        )
        self.assertIn("architecture-sections.md", (staged / "SUMMARY.md").read_text())
        blueprint = (staged / "authoritative_design/blueprint.md").read_text()
        self.assertIn("```md\n## 0.1 Contract\n```", blueprint)
        self.assertIn('<a id="section-0-1"></a>\n\n## 0.1 Contract', blueprint)
        self.assertEqual(blueprint.count('id="section-0-1"'), 1)
        self.assertFalse((staged / "plans/01-current.md").read_text().startswith("---"))
        self.assertEqual(len(pages), len(self.pages()) + 1)
        after = {
            p.relative_to(self.source): p.read_bytes()
            for p in self.source.rglob("*")
            if p.is_file()
        }
        self.assertEqual(before, after)

    def test_attachment_selection_and_bundle_boundaries_are_explicit(self) -> None:
        self.write("plans/chart.svg", "<svg/>\n")
        self.write("plans/01-current.md", "# Work\n\n![Chart](chart.svg)\n")
        config = docs.configuration(self.root)
        with self.assertRaisesRegex(ValueError, "not explicitly selected"):
            docs.stage(self.root, self.root / "missing", self.pages(), config)
        config["asset_bundles"] = [
            {"name": "chart", "root": "plans", "files": ["chart.svg"]}
        ]
        config["publication"]["asset_bundles"] = ["chart"]
        docs.stage(self.root, self.root / "selected", self.pages(), config)
        self.assertTrue((self.root / "selected/plans/chart.svg").exists())
        config["asset_bundles"][0]["files"] = ["../../outside"]
        with self.assertRaisesRegex(ValueError, "escapes bundle"):
            docs.selected_assets(self.root, config)
        config["asset_bundles"][0]["files"] = ["absent.json"]
        with self.assertRaisesRegex(ValueError, "missing"):
            docs.selected_assets(self.root, config)
        config["publication"]["asset_bundles"] = ["unknown"]
        with self.assertRaisesRegex(ValueError, "unknown publication"):
            docs.selected_assets(self.root, config)

    def test_selected_asset_cannot_escape_staging_with_an_in_docs_source(self) -> None:
        self.write("plans/evidence.json", "{}")
        config = docs.configuration(self.root)
        for relative in [
            "../docs/plans/evidence.json",
            str(self.source / "plans/evidence.json"),
        ]:
            config["publication"]["assets"] = [relative]
            with (
                self.subTest(relative=relative),
                self.assertRaisesRegex(ValueError, "noncanonical"),
            ):
                docs.selected_assets(self.root, config)
        with (
            patch.object(
                docs,
                "selected_assets",
                return_value=[Path("../docs/plans/evidence.json")],
            ),
            self.assertRaisesRegex(ValueError, "destination escapes"),
        ):
            docs.stage(self.root, self.root / "scratch/source", self.pages(), config)
        self.assertFalse((self.root / "scratch/docs").exists())


if __name__ == "__main__":
    unittest.main()

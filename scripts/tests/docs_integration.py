# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Real documentation tool integration, separate from bare setup discovery."""

# ruff: noqa: PT009, PT027
from __future__ import annotations

import subprocess
from unittest.mock import patch

from scripts import docs
from scripts.tests.test_docs import DocumentationFixture


class PublisherIntegration(DocumentationFixture):
    def test_real_build_replacement_scopes_and_failure_recovery(self) -> None:
        output = docs.build(self.root)
        current = (output / "plans/01-current.html").read_text()
        historical = (output / "plans/00-old.html").read_text()
        self.assertIn('data-pagefind-filter="scope:Current"', current)
        self.assertIn('data-pagefind-filter="scope:History"', historical)
        self.assertIn("../pagefind/pagefind-component-ui.css", current)
        self.assertIn("/edit/main/docs/./plans/01-current.md", current)
        self.assertNotIn("data-pagefind-body", (output / "index.html").read_text())
        self.assertFalse((output / "searchindex.js").exists())
        self.assertTrue((output / "pagefind/pagefind.js").is_file())
        before = (output / "README.html").read_bytes()
        real_run = subprocess.run

        def fail_index(
            args: list[str],
            *,
            check: bool = False,
            text: bool = False,
            capture_output: bool = False,
        ) -> subprocess.CompletedProcess[str] | subprocess.CompletedProcess[bytes]:
            if args[0] == "pagefind" and "--site" in args:
                raise subprocess.CalledProcessError(1, args)
            return real_run(args, check=check, text=text, capture_output=capture_output)

        with (
            patch.object(docs.subprocess, "run", side_effect=fail_index),
            self.assertRaises(subprocess.CalledProcessError),
        ):
            docs.build(self.root)
        self.assertEqual((output / "README.html").read_bytes(), before)
        (self.source / "plans/00-old.md").unlink()
        self.write(
            "generated/rustdoc/index.html", "<html><body>Optional API</body></html>"
        )
        docs.build(self.root)
        self.assertFalse((output / "plans/00-old.html").exists())
        self.assertTrue((output / "generated/rustdoc/index.html").exists())
        self.assertFalse((self.source / "SUMMARY.md").exists())

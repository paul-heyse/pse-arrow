# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Exercise ADR recipe argument transport in an isolated directory."""

# ruff: noqa: PT009

from __future__ import annotations

import json
import re
import subprocess
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]


class RecipeArgumentTests(unittest.TestCase):
    def test_adr_arguments_reach_script_unchanged(self) -> None:
        source = (ROOT / "justfile").read_text()
        title = 'Scope with "quotes", $HOME, `pwd`, and $(touch injected)'
        for recipe, arguments, expected in [
            (
                "adr-new",
                ["test-scope", "--title", title],
                ["new", "test-scope", "--title", title],
            ),
            (
                "adr-supersede",
                ["0050", "0051"],
                ["supersede", "0050", "0051"],
            ),
        ]:
            with (
                self.subTest(recipe=recipe),
                tempfile.TemporaryDirectory() as directory,
            ):
                match = re.search(
                    rf"(?m)^\[positional-arguments\]\n{recipe}[^\n]*:\n(?:[ \t]+[^\n]*\n)+",
                    source,
                )
                self.assertIsNotNone(match)
                if match is None:
                    raise AssertionError(f"missing positional recipe {recipe}")
                root = Path(directory)
                (root / "justfile").write_text(match.group())
                (root / "scripts").mkdir()
                (root / "scripts/adr.py").write_text(
                    "import json, pathlib, sys\n"
                    "pathlib.Path('arguments.json').write_text(json.dumps(sys.argv[1:]))\n"
                )
                subprocess.run(
                    ["just", recipe, *arguments],
                    cwd=root,
                    check=True,
                    capture_output=True,
                    text=True,
                )
                self.assertEqual(
                    json.loads((root / "arguments.json").read_text()), expected
                )
                self.assertFalse((root / "injected").exists())


if __name__ == "__main__":
    unittest.main()

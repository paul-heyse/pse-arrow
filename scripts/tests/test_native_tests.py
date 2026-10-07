# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Pytest owns default versus explicit native Python test selection."""
# ruff: noqa: PT009 -- stdlib controls exercise the installed pytest CLI

from __future__ import annotations

import os
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

from scripts import native_tests


class PythonSelectionTests(unittest.TestCase):
    def setUp(self) -> None:
        self.scratch = tempfile.TemporaryDirectory()
        self.addCleanup(self.scratch.cleanup)
        self.root = Path(self.scratch.name)
        (self.root / "pytest.ini").write_text(
            "[pytest]\ntestpaths = python/pse/tests python/pse/parity\n"
        )
        for directory, name in (
            ("tests", "first"),
            ("tests", "second"),
            ("parity", "parity"),
        ):
            target = self.root / f"python/pse/{directory}/test_{name}.py"
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(f"def test_{name}(): pass\n")

    def selected(self, *extra: str) -> str:
        environment = dict(os.environ)
        environment.pop("PYTEST_ADDOPTS", None)
        result = subprocess.run(
            [
                sys.executable,
                "-m",
                "pytest",
                *native_tests.PYTHON_DEFAULT_SELECTION,
                "--collect-only",
                "-q",
                *extra,
            ],
            cwd=self.root,
            env=environment,
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        return result.stdout

    def test_option_values_keep_product_defaults_without_parity(self) -> None:
        selected = self.selected(
            "--log-file",
            "collection.log",
            "--assert",
            "plain",
            "--junitprefix",
            "selection",
        )
        self.assertIn("test_first.py::test_first", selected)
        self.assertIn("test_second.py::test_second", selected)
        self.assertNotIn("test_parity.py", selected)

    def test_explicit_file_directory_and_node_remain_exact(self) -> None:
        for target in (
            "python/pse/tests/test_first.py",
            "python/pse/tests/test_first.py::test_first",
            "python/pse/parity",
        ):
            with self.subTest(target=target):
                selected = self.selected("--ignore", "absent.py", target)
                self.assertNotIn("test_second.py", selected)
                self.assertIn("1 test collected", selected)


if __name__ == "__main__":
    unittest.main()

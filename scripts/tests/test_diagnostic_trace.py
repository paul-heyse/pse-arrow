# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Owned tool/finite output controls; no scientific target or external tool required."""

# These command-tool controls also run through the repository's unittest harness.
# ruff: noqa: PT009, PT027

import sys
import tempfile
import unittest
from pathlib import Path

from scripts import diagnostic_trace as trace


class DiagnosticTraceTests(unittest.TestCase):
    def fixture(self, directory: Path, body: str) -> tuple[Path, Path]:
        binary = directory / "owned tool"
        binary.write_text(f"#!{sys.executable}\n{body}\n")
        binary.chmod(0o755)
        source = directory / "raw $(not shell).pftrace"
        source.write_bytes(b"raw retained source")
        return binary, source

    def test_declared_query_passes_paths_as_data_and_retains_raw(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            binary, source = self.fixture(
                root, "import sys\nprint(','.join(sys.argv[1:]))"
            )
            output = root / "view"
            self.assertEqual(
                trace.query(source, "timeline", output, 5, 4096, binary), 0
            )
            self.assertIn(str(source), (output / "rows.csv").read_text())
            self.assertEqual(source.read_bytes(), b"raw retained source")

    def test_timeout_joins_only_owned_child_and_keeps_prefix(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            binary, source = self.fixture(
                root, "import time\nprint('prefix',flush=True)\ntime.sleep(10)"
            )
            output = root / "view"
            self.assertEqual(
                trace.query(source, "coverage", output, 0.1, 4096, binary), 124
            )
            self.assertIn("prefix", (output / "rows.csv").read_text())
            self.assertTrue(source.is_file())

    def test_output_exhaustion_is_failure_with_finite_retained_prefix(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            binary, source = self.fixture(
                root, "import os\nos.write(1,b'x'*100000)\nos.write(1,b'y')"
            )
            output = root / "view"
            self.assertNotEqual(
                trace.query(source, "numerical", output, 5, 4096, binary), 0
            )
            self.assertLessEqual(sum(p.stat().st_size for p in output.iterdir()), 4096)
            self.assertTrue(source.is_file())

    def test_absent_tool_or_unknown_question_has_no_output_effect(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            binary, source = self.fixture(root, "pass")
            for tool, question in [
                (root / "absent", "timeline"),
                (binary, "not-a-query"),
            ]:
                output = root / "view"
                with self.assertRaises(ValueError):
                    trace.query(source, question, output, 5, 4096, tool)
                self.assertFalse(output.exists())

    def test_invalid_clock_or_budget_refuses_before_launch(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            binary, source = self.fixture(root, "raise Exception('must not run')")
            for clock, budget in [
                (0, 4096),
                (float("nan"), 4096),
                (float("inf"), 4096),
                (5, 1),
            ]:
                with self.assertRaises(ValueError):
                    trace.query(
                        source, "timeline", root / "view", clock, budget, binary
                    )
            self.assertFalse((root / "view").exists())

    def test_existing_output_is_preserved(self) -> None:
        with tempfile.TemporaryDirectory() as name:
            root = Path(name)
            binary, source = self.fixture(root, "pass")
            output = root / "view"
            output.mkdir()
            sentinel = output / "existing"
            sentinel.write_text("preserve")
            with self.assertRaises(FileExistsError):
                trace.query(source, "timeline", output, 5, 4096, binary)
            self.assertEqual(sentinel.read_text(), "preserve")


if __name__ == "__main__":
    unittest.main()

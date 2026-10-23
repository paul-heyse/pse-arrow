# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Probe ownership and copy controls; no compiler or selected database."""
# ruff: noqa: PT009, PT027 -- stdlib probe controls

from __future__ import annotations

import subprocess
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.tests import surreal_scan_budget_probe as probe


class ScanProbePreparationTests(unittest.TestCase):
    def test_zero_selected_tests_cannot_claim_probe_success(self) -> None:
        self.assertFalse(probe.selected_test_passed("running 0 tests\ntest result: ok. 0 passed; 0 failed;"))
        self.assertTrue(probe.selected_test_passed(
            f"running 1 test\ntest {probe.TEST} ... ok\ntest result: ok. 1 passed; 0 failed;"
        ))

    def test_git_environment_cannot_redirect_probe_operations(self) -> None:
        with patch.dict("os.environ", {"GIT_DIR": "/unrelated/git", "GIT_WORK_TREE": "/unrelated", "GIT_INDEX_FILE": "/unrelated/index"}):
            environment = probe.git_environment()
        self.assertNotIn("GIT_DIR", environment)
        self.assertNotIn("GIT_WORK_TREE", environment)
        self.assertNotIn("GIT_INDEX_FILE", environment)

    def test_missing_instrumentation_refuses_before_compiler(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            copied = Path(temporary) / "source"
            btree = copied / "surrealdb/core/src/exec/index/iterator/btree.rs"
            btree.parent.mkdir(parents=True)
            btree.write_text("no probe here")
            with patch.object(probe.subprocess, "run") as compiler:
                with self.assertRaisesRegex(ValueError, "lacks the exact test-only"):
                    probe.run(Path(temporary), {})
            compiler.assert_not_called()

    def test_source_and_scratch_must_be_disjoint(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            with self.assertRaisesRegex(ValueError, "disjoint"):
                probe.prepare(source, source / "scratch")
            self.assertFalse((source / "scratch").exists())

    def test_copies_inputs_without_target_or_hard_links_and_preserves_source(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            base = Path(temporary)
            subprocess.run(["git", "init", "--quiet", "--template=", str(base)],
                           env=probe.git_environment(), check=True)
            (base / "fixture").write_text("ancestor must stay unchanged\n")
            source = base / "selected"
            source.mkdir()
            (source / "Cargo.toml").write_text('surrealmx = "=1.0.0"\n')
            (source / "Cargo.lock").write_text('name = "surrealmx"\nversion = "1.0.0"\n')
            (source / ".pse-deps").mkdir()
            (source / ".pse-deps/input").write_text("owned dependency")
            (source / "target").mkdir()
            (source / "target/artifact").write_text("must not copy")
            (source / "fixture").write_text("original\n")
            fixture_patch = base / "test.patch"
            fixture_patch.write_text("--- a/fixture\n+++ b/fixture\n@@ -1 +1 @@\n-original\n+instrumented\n")

            def git(_source: Path, *arguments: str) -> bytes:
                if arguments[0] == "rev-parse":
                    return probe.REVISION.encode()
                if arguments[0] == "ls-files":
                    return b"Cargo.toml\0Cargo.lock\0fixture\0"
                return b""

            with patch.object(probe, "git", side_effect=git), patch.object(probe, "PATCH", fixture_patch):
                receipt = probe.prepare(source, base / "scratch")
            copied = Path(str(receipt["copy"]))
            self.assertEqual((base / "fixture").read_text(), "ancestor must stay unchanged\n")
            self.assertTrue((copied / ".git").is_dir())
            self.assertEqual((source / "fixture").read_text(), "original\n")
            self.assertEqual((copied / "fixture").read_text(), "instrumented\n")
            self.assertNotEqual((source / "Cargo.lock").stat().st_ino, (copied / "Cargo.lock").stat().st_ino)
            self.assertFalse((copied / "target").exists())
            self.assertEqual((copied / ".pse-deps/input").read_text(), "owned dependency")

    def test_changed_native_source_refuses_before_creating_scratch(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            source = Path(temporary)
            (source / "Cargo.toml").write_text('surrealmx = "=1.0.0"\n')
            (source / "Cargo.lock").write_text('name = "surrealmx"\nversion = "1.0.0"\n')
            with patch.object(probe, "git", side_effect=[probe.REVISION.encode(), b"native drift"]):
                with self.assertRaisesRegex(ValueError, "native planner/index/scanner"):
                    probe.prepare(source, source.parent / "uncreated-tool08-scratch")


if __name__ == "__main__":
    unittest.main()

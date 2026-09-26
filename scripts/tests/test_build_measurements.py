# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Isolated snapshot/protocol controls; never run a compiler or benchmark."""
# ruff: noqa: PT009, PT027 -- stdlib setup runner must work before pytest is installed

from __future__ import annotations

import json
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import build_measurements, validation


class BuildMeasurementTests(unittest.TestCase):
    def test_profile_candidate_is_available_to_nested_cargo_commands(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory)
            (source / ".cargo").mkdir()
            config = source / ".cargo/config.toml"
            config.write_text('[target.x86_64-unknown-linux-gnu]\nlinker = "clang"\n')
            build_measurements.profile_override(source, 1, True)
            parsed = tomllib.loads(config.read_text())
            self.assertEqual(
                parsed["target"]["x86_64-unknown-linux-gnu"]["linker"], "clang"
            )
            self.assertEqual(parsed["profile"]["dev"]["package"]["*"], {"opt-level": 1})
            self.assertEqual(
                parsed["profile"]["dev"]["package"]["deltalake-core"],
                {"incremental": False},
            )
            with self.assertRaisesRegex(ValueError, "profile policy"):
                build_measurements.profile_override(source, 2, False)

    def test_snapshot_keeps_dirty_deleted_untracked_ignored_tracked_modes_and_symlinks(
        self,
    ) -> None:
        with (
            tempfile.TemporaryDirectory() as directory,
            patch.object(validation, "SOURCE_PATHS", (".",)),
        ):
            root = Path(directory) / "original"
            root.mkdir()
            subprocess.run(["git", "init", "--quiet"], cwd=root, check=True)
            (root / ".gitignore").write_text("ignored\n")
            (root / "tracked").write_text("before\n")
            (root / "deleted").write_text("gone\n")
            (root / "ignored").write_text("tracked despite ignore\n")
            subprocess.run(["git", "add", "--force", "--all"], cwd=root, check=True)
            index = (root / ".git/index").read_bytes()
            (root / "deleted").unlink()
            (root / "tracked").write_text("after\n")
            (root / "tracked").chmod(0o755)
            (root / "new").write_text("new\n")
            (root / "link").symlink_to("tracked")
            before = validation.sources(root)
            output = Path(directory) / "output"
            output.mkdir()
            copied = build_measurements.snapshot(root, output)
            self.assertEqual((copied / "tracked").read_bytes(), b"after\n")
            self.assertEqual((copied / "tracked").stat().st_mode & 0o777, 0o755)
            self.assertEqual((copied / "link").readlink(), Path("tracked"))
            self.assertEqual(
                (copied / "ignored").read_bytes(), b"tracked despite ignore\n"
            )
            self.assertFalse((copied / "deleted").exists())
            self.assertEqual(validation.sources(root), before)
            self.assertEqual((root / ".git/index").read_bytes(), index)
            (copied / "tracked").write_text("private edit\n")
            self.assertEqual((root / "tracked").read_bytes(), b"after\n")

    def test_supported_artifact_messages_distinguish_fresh_rebuilt_and_incomplete_builds(
        self,
    ) -> None:
        artifact = {
            "reason": "compiler-artifact",
            "package_id": "registry+example#crate@1",
            "target": {"name": "crate", "kind": ["lib"]},
            "profile": {"test": False},
            "features": ["a"],
            "fresh": False,
        }
        finished = {"reason": "build-finished", "success": True}
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "cargo.jsonl"
            path.write_text(
                "\n".join(
                    map(json.dumps, [artifact, {**artifact, "fresh": True}, finished])
                )
            )
            result = build_measurements.artifacts(path)
            self.assertEqual(result["rebuilt_artifacts"], 1)
            self.assertEqual(result["fresh_artifacts"], 1)
            for messages in (
                [artifact],
                [artifact, {**finished, "success": False}],
                [finished],
                [artifact, finished, finished],
            ):
                path.write_text("\n".join(map(json.dumps, messages)))
                with self.assertRaises(ValueError):
                    build_measurements.artifacts(path)

    def test_new_cache_counters_are_counted_from_zero(self) -> None:
        self.assertEqual(
            build_measurements.counter_delta({"hits": {}}, {"hits": {"Rust": 2}}),
            {"hits": {"Rust": 2}},
        )

    def test_actual_unified_feature_mode_must_match_the_build_label(self) -> None:
        unit = {"target": {"name": "arrow_data"}, "features": ["force_validate"]}
        build_measurements.require_validation_mode([unit], True)
        build_measurements.require_validation_mode([{**unit, "features": []}], False)
        for units, expected in [
            ([unit], False),
            ([], True),
            ([unit, {**unit, "features": []}], True),
        ]:
            with self.assertRaises(ValueError):
                build_measurements.require_validation_mode(units, expected)


if __name__ == "__main__":
    unittest.main()

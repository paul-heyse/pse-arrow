# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
# ruff: noqa: PT009
from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import build_storage


class BuildStorageTests(unittest.TestCase):
    def test_inode_counts_exclusions_and_no_follow(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            data = root / "data"
            data.mkdir()
            payload = data / "payload"
            payload.write_bytes(b"x" * 4096)
            os.link(payload, data / "hardlink")
            outside = root / "outside"
            outside.mkdir()
            (outside / "secret").write_bytes(b"x" * 8192)
            (data / "symlink").symlink_to(outside, target_is_directory=True)
            excluded = data / ".venv-docs"
            excluded.mkdir()
            (excluded / "ignored").write_bytes(b"x" * 8192)
            result = build_storage.measure(data)
            self.assertEqual(result["status"], "observed")
            self.assertEqual(result["unique_entries"], 3)
            self.assertEqual(result["excluded_entries"], 1)
            self.assertEqual(
                result["apparent_bytes"],
                data.stat().st_size
                + payload.stat().st_size
                + (data / "symlink").lstat().st_size,
            )

    def test_alias_and_persistent_state_are_not_traversed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            alias = root / "alias"
            alias.symlink_to(root, target_is_directory=True)
            with patch.object(
                build_storage,
                "measure",
                side_effect=AssertionError("unexpected traversal"),
            ):
                records = build_storage.observe(
                    [
                        ("state", root, "persistent-semantic-owner", False),
                        ("alias", alias, "shared", True),
                        ("missing", root / "missing", "unknown", True),
                    ]
                )
            self.assertEqual(
                [row["status"] for row in records],
                ["metadata-only", "alias", "unavailable"],
            )
            self.assertIsNone(records[0]["allocated_bytes"])
            self.assertEqual(records[1]["alias_of"], "state")

    def test_directory_replaced_by_symlink_is_not_followed(self) -> None:
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            data = root / "data"
            data.mkdir()
            child = data / "child"
            child.mkdir()
            outside = root / "outside"
            outside.mkdir()
            (outside / "private").write_text("must not enumerate")
            original_open = os.open

            def replace(
                path: str | Path, flags: int, *, dir_fd: int | None = None
            ) -> int:
                if path == "child":
                    child.rmdir()
                    child.symlink_to(outside, target_is_directory=True)
                return original_open(path, flags, dir_fd=dir_fd)

            with patch("os.open", side_effect=replace):
                result = build_storage.measure(data)
            self.assertEqual(result["status"], "partial")
            self.assertEqual(result["unique_entries"], 2)
            self.assertEqual(result["error_count"], 1)

    def test_failed_measurement_is_partial(self) -> None:
        with patch("os.scandir", side_effect=PermissionError("denied")):
            result = build_storage.measure(Path(__file__).parent)
        self.assertEqual(result["status"], "partial")
        self.assertEqual(result["error_count"], 1)
        self.assertIn("denied", result["errors"][0])


if __name__ == "__main__":
    unittest.main()

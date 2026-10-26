# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Stopped metadata snapshots publish complete private inodes and preserve unknowns."""

from __future__ import annotations

import os
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import backup_image as image


class BackupImageTests(unittest.TestCase):
    def setUp(self) -> None:
        self.root = Path(self.enterContext(tempfile.TemporaryDirectory()))
        self.source = self.root / "source"
        self.source.write_bytes(b"original metadata image" * 100)
        self.source.chmod(0o600)
        self.target = self.root / "destination/image"
        self.expected = image.digest(self.source)

    def test_copy_and_exact_resume_preserve_the_published_inode(self) -> None:
        image.copy(self.source, self.target, self.expected)
        inode = self.target.stat().st_ino
        image.copy(self.source, self.target, self.expected)
        self.assertEqual(self.target.stat().st_ino, inode)
        self.assertEqual(image.digest(self.target), self.expected)
        self.assertEqual(self.target.stat().st_mode & 0o777, 0o600)
        self.target.write_bytes(b"unexpected")
        with self.assertRaises(image.ImageError):
            image.copy(self.source, self.target, self.expected)
        self.assertEqual(self.target.read_bytes(), b"unexpected")

    def test_nested_sibling_copies_create_every_ancestor_privately_for_real_umasks(self) -> None:
        for mask in (0o022, 0o002):
            destination = self.root / f"nested-{mask:o}"
            previous = os.umask(mask)
            try:
                targets = [destination / ".generations/selected/scripts/owner.py",
                           destination / ".generations/selected/.config/selection.toml",
                           destination / ".generations/selected/generation.json"]
                for target in targets:
                    image.copy(self.source, target, self.expected)
                for path in [destination, *destination.rglob("*")]:
                    if path.is_dir():
                        self.assertEqual(path.stat().st_mode & 0o777, 0o700)
                for target in targets:
                    original = target.stat().st_ino
                    image.copy(self.source, target, self.expected)
                    self.assertEqual(target.stat().st_ino, original)
                    self.assertEqual(image.digest(target), self.expected)
            finally:
                os.umask(previous)

    def test_preexisting_nonprivate_ancestor_is_refused_without_chmod_or_new_names(self) -> None:
        for mode in (0o755, 0o775):
            destination = self.root / f"unknown-{mode:o}"
            destination.mkdir(mode=mode)
            destination.chmod(mode)
            unknown = destination / "unrelated"
            unknown.write_bytes(b"retain this original directory and data")
            before = destination.stat()
            with self.assertRaisesRegex(image.ImageError, "private and owned"):
                image.copy(self.source, destination / "nested/image", self.expected)
            self.assertEqual(destination.stat().st_ino, before.st_ino)
            self.assertEqual(destination.stat().st_mode & 0o777, mode)
            self.assertEqual(set(destination.iterdir()), {unknown})
            self.assertEqual(unknown.read_bytes(), b"retain this original directory and data")

    def test_interruption_before_publication_leaves_no_named_partial(self) -> None:
        with (patch.object(image.os, "link", side_effect=InterruptedError("injected interruption")),
              self.assertRaises(InterruptedError)):
            image.copy(self.source, self.target, self.expected)
        self.assertEqual(list(self.target.parent.iterdir()), [])
        image.copy(self.source, self.target, self.expected)
        self.assertEqual(image.digest(self.target), self.expected)

    def test_unknown_entries_aliases_and_changed_source_are_preserved(self) -> None:
        self.target.parent.mkdir(mode=0o700)
        unknown = self.target.parent / "unknown"
        unknown.write_bytes(b"retain")
        unknown.chmod(0o600)
        with self.assertRaisesRegex(image.ImageError, "Unknown"):
            image.check_destination(self.target.parent, {"image": self.expected}, "a" * 64)
        self.assertEqual(unknown.read_bytes(), b"retain")
        os.symlink(self.source, self.target)
        with self.assertRaises(image.ImageError):
            image.copy(self.source, self.target, self.expected)
        self.target.unlink()
        self.source.write_bytes(b"changed source")
        with self.assertRaisesRegex(image.ImageError, "Stopped backup source"):
            image.copy(self.source, self.target, self.expected)
        self.assertFalse(self.target.exists())

    def test_historical_engine_mode_is_preserved_and_live_source_alias_is_refused(self) -> None:
        self.source.chmod(0o644)
        image.copy(self.source, self.target, self.expected)
        self.assertEqual(self.target.stat().st_mode & 0o777, 0o644)
        alias = self.target.parent / "alias"
        os.link(self.source, alias)
        with self.assertRaises(image.ImageError):
            image.copy(self.source, alias, self.expected)
        self.assertEqual(alias.stat().st_ino, self.source.stat().st_ino)


if __name__ == "__main__":
    unittest.main()

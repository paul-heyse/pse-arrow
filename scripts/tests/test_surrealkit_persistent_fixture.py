# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Focused construction/refusal controls for the actual C2/C3 journey."""
# ruff: noqa: PT009, PT027
from __future__ import annotations

import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts.tests import surrealkit_persistent_fixture_check as fixture


class PersistentFixtureTests(unittest.TestCase):
    def test_predecessor_only_removes_nonunique_access_indexes(self) -> None:
        schema = (
            b'DEFINE TABLE item SCHEMAFULL;\n'
            b'DEFINE INDEX identity ON item FIELDS key UNIQUE;\n'
            b'DEFINE INDEX access_one ON item FIELDS first;\n'
            b'DEFINE INDEX access_two ON item FIELDS second;\n'
        )
        baseline, middle, indexes = fixture.predecessors(schema)
        self.assertEqual(indexes, [("access_one", "item"), ("access_two", "item")])
        self.assertIn(b'FIELDS key UNIQUE;', baseline)
        self.assertNotIn(b'access_one', baseline)
        self.assertNotIn(b'access_two', baseline)
        self.assertIn(b'access_one', middle)
        self.assertNotIn(b'access_two', middle)

    def test_missing_fixture_indexes_fail_before_effects(self) -> None:
        with self.assertRaisesRegex(RuntimeError, "Two generated nonunique"):
            fixture.predecessors(b'DEFINE INDEX identity ON item FIELDS key UNIQUE;\n')

    def test_existing_destination_refused_before_registry_or_binary_effects(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            arguments = ["fixture", "--directory", directory, "--binary", "/missing-beta", "--canonical-binary", "/missing-xtask"]
            with patch.object(sys, "argv", arguments), patch.object(fixture.resources, "register_report") as register:
                with self.assertRaisesRegex(SystemExit, "new nonsymlinked"):
                    fixture.main()
                register.assert_not_called()

    def test_missing_native_owner_refused_before_creating_any_owned_output(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            destination = root / "build/fresh"
            arguments = ["fixture", "--directory", str(destination), "--binary", "/missing-beta", "--canonical-binary", "/missing-xtask"]
            with patch.object(fixture, "ROOT", root), patch.object(sys, "argv", arguments), patch.object(fixture.host_admission, "inherit", return_value=None), patch.object(fixture.resources, "register_report") as register:
                with self.assertRaisesRegex(SystemExit, "exclusive native operation"):
                    fixture.main()
                register.assert_not_called()
                self.assertFalse(destination.exists())

    def test_destination_outside_owned_build_refused(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            arguments = ["fixture", "--directory", str(Path(directory) / "fresh"), "--binary", "/missing-beta", "--canonical-binary", "/missing-xtask"]
            with patch.object(sys, "argv", arguments), patch.object(fixture.resources, "register_report") as register:
                with self.assertRaisesRegex(SystemExit, "owned build"):
                    fixture.main()
                register.assert_not_called()


if __name__ == "__main__":
    unittest.main()

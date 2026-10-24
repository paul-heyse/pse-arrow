# SPDX-License-Identifier: MIT OR Apache-2.0
# Copyright (c) 2026 Paul Heyse
"""Protocol pins are shared selections, not an implicit tool upgrade."""
# ruff: noqa: PT009, PT027 -- stdlib unittest controls

import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

from scripts import cargo_tools


class CargoToolsTests(unittest.TestCase):
    def test_real_tool_path_and_declared_pin_are_part_of_selected_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            declaration = root / ".config/diagnostic-tools.toml"
            declaration.parent.mkdir()
            declaration.write_text('[cargo-tools]\ncargo-nextest="0.9.146"\n')
            first = root / "first/cargo-nextest"
            second = root / "second/cargo-nextest"
            for executable in (first, second):
                executable.parent.mkdir()
                executable.write_text("#!/bin/sh\nprintf 'cargo-nextest 0.9.146\\n'\n")
                executable.chmod(0o700)
            with patch.object(cargo_tools, "ROOT", root):
                with patch.object(cargo_tools.shutil, "which", return_value=str(first)):
                    original = cargo_tools.selected_identity("cargo-nextest")
                with patch.object(cargo_tools.shutil, "which", return_value=str(second)):
                    relocated = cargo_tools.selected_identity("cargo-nextest")
                    self.assertEqual(original["sha256"], relocated["sha256"])
                    self.assertNotEqual(original, relocated)
                    declaration.write_text('[cargo-tools]\ncargo-nextest="0.9.147"\n')
                    with self.assertRaisesRegex(ValueError, "version differs from 0.9.147"):
                        cargo_tools.selected_identity("cargo-nextest")

    def test_pins_reject_override_but_leave_other_explicit_versions(self) -> None:
        with patch.object(
            cargo_tools, "versions", return_value={"cargo-hack": "0.6.45"}
        ):
            self.assertEqual(
                cargo_tools.specifications(["cargo-hack", "cargo-nextest@0.9.0"]),
                ["cargo-hack@0.6.45", "cargo-nextest@0.9.0"],
            )
            with self.assertRaisesRegex(ValueError, "declared version"):
                cargo_tools.specifications(["cargo-hack@0.6.44"])
            with self.assertRaisesRegex(ValueError, "invalid Cargo"):
                cargo_tools.specifications(["cargo-hack;false"])
